// Private default-callback implementation.  The callback bodies and their
// sorted 214-entry lookup table stay together so callback order, static cache
// lifetime, and allocation/free ownership remain unchanged.  This module
// consumes the facade's generated model types and FFI helpers and exposes only
// the table lookup/storage needed by the expression and tree groups.
use super::*;

unsafe extern "C" fn format_printf(
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> *mut ::core::ffi::c_char {
    let mut ap: ::core::ffi::VaList;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    xvasprintf(&raw mut s, fmt, ap);
    return s;
}
unsafe extern "C" fn format_cb_host(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut host: [::core::ffi::c_char; 65] = [0; 65];
    if gethostname(
        &raw mut host as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 65]>() as size_t,
    ) != 0 as ::core::ffi::c_int
    {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return xstrdup(&raw mut host as *mut ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_host_short(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut host: [::core::ffi::c_char; 65] = [0; 65];
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if gethostname(
        &raw mut host as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 65]>() as size_t,
    ) != 0 as ::core::ffi::c_int
    {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    cp = strchr(&raw mut host as *mut ::core::ffi::c_char, '.' as i32);
    if !cp.is_null() {
        *cp = '\0' as i32 as ::core::ffi::c_char;
    }
    return xstrdup(&raw mut host as *mut ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pid(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    xasprintf(
        &raw mut value,
        b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
        getpid() as ::core::ffi::c_long,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_attached_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = (*ft).s;
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if (*loop_0).session == s {
            if evbuffer_get_length(buffer) > 0 as size_t {
                evbuffer_add(
                    buffer,
                    b",\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    1 as size_t,
                );
            }
            evbuffer_add_printf(
                buffer,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*loop_0).name,
            );
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_alert(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = (*ft).s;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut alerts: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut alerted: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    *(&raw mut alerts as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        if !((*wl).flags & WINLINK_ALERTFLAGS == 0 as ::core::ffi::c_int) {
            if !alerted & (*wl).flags & WINLINK_ACTIVITY != 0 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b"#\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
                alerted |= WINLINK_ACTIVITY;
            }
            if !alerted & (*wl).flags & WINLINK_BELL != 0 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b"!\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
                alerted |= WINLINK_BELL;
            }
            if !alerted & (*wl).flags & WINLINK_SILENCE != 0 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b"~\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
                alerted |= WINLINK_SILENCE;
            }
        }
        wl = winlinks_RB_NEXT(wl);
    }
    return xstrdup(&raw mut alerts as *mut ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_alerts(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = (*ft).s;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut alerts: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut tmp: [::core::ffi::c_char; 16] = [0; 16];
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    *(&raw mut alerts as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        if !((*wl).flags & WINLINK_ALERTFLAGS == 0 as ::core::ffi::c_int) {
            xsnprintf(
                &raw mut tmp as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*wl).idx,
            );
            if *(&raw mut alerts as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b",\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
            }
            strlcat(
                &raw mut alerts as *mut ::core::ffi::c_char,
                &raw mut tmp as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
            );
            if (*wl).flags & WINLINK_ACTIVITY != 0 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b"#\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
            }
            if (*wl).flags & WINLINK_BELL != 0 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b"!\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
            }
            if (*wl).flags & WINLINK_SILENCE != 0 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b"~\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
            }
        }
        wl = winlinks_RB_NEXT(wl);
    }
    return xstrdup(&raw mut alerts as *mut ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_stack(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = (*ft).s;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut result: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut tmp: [::core::ffi::c_char; 16] = [0; 16];
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    xsnprintf(
        &raw mut result as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*s).curw).idx,
    );
    wl = (*s).lastw.tqh_first;
    while !wl.is_null() {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).idx,
        );
        if *(&raw mut result as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
            strlcat(
                &raw mut result as *mut ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
            );
        }
        strlcat(
            &raw mut result as *mut ::core::ffi::c_char,
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
        wl = (*wl).sentry.tqe_next;
    }
    return xstrdup(&raw mut result as *mut ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_stack_index(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut idx: u_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    s = (*(*ft).wl).session;
    idx = 0 as u_int;
    wl = (*s).lastw.tqh_first;
    while !wl.is_null() {
        idx = idx.wrapping_add(1);
        if wl == (*ft).wl {
            break;
        }
        wl = (*wl).sentry.tqe_next;
    }
    if wl.is_null() {
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    xasprintf(
        &raw mut value,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        idx,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_linked_sessions_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*(*ft).wl).window;
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if evbuffer_get_length(buffer) > 0 as size_t {
            evbuffer_add(
                buffer,
                b",\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
        }
        evbuffer_add_printf(
            buffer,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*wl).session).name,
        );
        wl = (*wl).wentry.tqe_next;
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_active_sessions(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut n: u_int = 0 as u_int;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*(*ft).wl).window;
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if (*(*wl).session).curw == wl {
            n = n.wrapping_add(1);
        }
        wl = (*wl).wentry.tqe_next;
    }
    xasprintf(
        &raw mut value,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        n,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_active_sessions_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*(*ft).wl).window;
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if (*(*wl).session).curw == wl {
            if evbuffer_get_length(buffer) > 0 as size_t {
                evbuffer_add(
                    buffer,
                    b",\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    1 as size_t,
                );
            }
            evbuffer_add_printf(
                buffer,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*wl).session).name,
            );
        }
        wl = (*wl).wentry.tqe_next;
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_active_clients(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut client_session: *mut session = ::core::ptr::null_mut::<session>();
    let mut n: u_int = 0 as u_int;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*(*ft).wl).window;
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        client_session = (*loop_0).session;
        if !client_session.is_null() {
            if w == (*(*client_session).curw).window {
                n = n.wrapping_add(1);
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    xasprintf(
        &raw mut value,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        n,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_active_clients_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut client_session: *mut session = ::core::ptr::null_mut::<session>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*(*ft).wl).window;
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        client_session = (*loop_0).session;
        if !client_session.is_null() {
            if w == (*(*client_session).curw).window {
                if evbuffer_get_length(buffer) > 0 as size_t {
                    evbuffer_add(
                        buffer,
                        b",\0" as *const u8 as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        1 as size_t,
                    );
                }
                evbuffer_add_printf(
                    buffer,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*loop_0).name,
                );
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_layout(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut c: *mut client = (*ft).client;
    let mut w: *mut window = (*ft).w;
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if w.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if !(*w).saved_layout_root.is_null() {
        lcroot = (*w).saved_layout_root;
    } else {
        lcroot = (*w).layout_root;
    }
    if !c.is_null()
        && (*c).flags & CLIENT_CONTROL as uint64_t != 0
        && !(*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_NEWLAYOUTS != 0
    {
        flags |= LAYOUT_CUSTOM_OLD_FORMAT;
    }
    return layout_dump(w, lcroot, flags) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_visible_layout(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut c: *mut client = (*ft).client;
    let mut w: *mut window = (*ft).w;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if w.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if !c.is_null()
        && (*c).flags & CLIENT_CONTROL as uint64_t != 0
        && !(*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_NEWLAYOUTS != 0
    {
        flags |= LAYOUT_CUSTOM_OLD_FORMAT;
    }
    return layout_dump(w, (*w).layout_root, flags) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_start_command(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return cmd_stringify_argv((*wp).argc, (*wp).argv) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_start_command_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0 as size_t;
    let mut i: ::core::ffi::c_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*wp).argc == 0 as ::core::ffi::c_int {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    i = 0 as ::core::ffi::c_int;
    while i < (*wp).argc {
        s = format_quote_shell_single(*(*wp).argv.offset(i as isize));
        len = len.wrapping_add(strlen(s).wrapping_add(1 as size_t));
        buf = xrealloc(buf as *mut ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
        if i == 0 as ::core::ffi::c_int {
            *buf = '\0' as i32 as ::core::ffi::c_char;
        } else {
            strlcat(buf, b" \0" as *const u8 as *const ::core::ffi::c_char, len);
        }
        strlcat(buf, s, len);
        free(s as *mut ::core::ffi::c_void);
        i += 1;
    }
    return buf as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_start_path(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*wp).cwd.is_null() {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return xstrdup((*wp).cwd) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_current_command(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() || (*wp).shell.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    cmd = osdep_get_name((*wp).fd, &raw mut (*wp).tty as *mut ::core::ffi::c_char);
    if cmd.is_null() || *cmd as ::core::ffi::c_int == '\0' as i32 {
        free(cmd as *mut ::core::ffi::c_void);
        cmd = cmd_stringify_argv((*wp).argc, (*wp).argv);
        if cmd.is_null() || *cmd as ::core::ffi::c_int == '\0' as i32 {
            free(cmd as *mut ::core::ffi::c_void);
            cmd = xstrdup((*wp).shell);
        }
    }
    value = parse_window_name(cmd);
    free(cmd as *mut ::core::ffi::c_void);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_current_path(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    cwd = osdep_get_cwd((*wp).fd);
    if cwd.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return xstrdup(cwd) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_history_bytes(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut size: size_t = 0 as size_t;
    let mut i: u_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    gd = (*wp).base.grid;
    i = 0 as u_int;
    while i < (*gd).hsize.wrapping_add((*gd).sy) {
        gl = grid_get_line(gd, i);
        size = (size as ::core::ffi::c_ulong).wrapping_add(
            ((*gl).cellsize as usize)
                .wrapping_mul(::core::mem::size_of::<grid_cell_entry>() as usize)
                as ::core::ffi::c_ulong,
        ) as size_t as size_t;
        size = (size as ::core::ffi::c_ulong).wrapping_add(
            ((*gl).extdsize as usize)
                .wrapping_mul(::core::mem::size_of::<grid_extd_entry>() as usize)
                as ::core::ffi::c_ulong,
        ) as size_t as size_t;
        i = i.wrapping_add(1);
    }
    size = (size as ::core::ffi::c_ulong).wrapping_add(
        ((*gd).hsize.wrapping_add((*gd).sy) as usize)
            .wrapping_mul(::core::mem::size_of::<grid_line>() as usize)
            as ::core::ffi::c_ulong,
    ) as size_t as size_t;
    xasprintf(
        &raw mut value,
        b"%zu\0" as *const u8 as *const ::core::ffi::c_char,
        size,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_history_all_bytes(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut i: u_int = 0;
    let mut lines: u_int = 0;
    let mut cells: u_int = 0 as u_int;
    let mut extended_cells: u_int = 0 as u_int;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    gd = (*wp).base.grid;
    lines = (*gd).hsize.wrapping_add((*gd).sy);
    i = 0 as u_int;
    while i < lines {
        gl = grid_get_line(gd, i);
        cells = cells.wrapping_add((*gl).cellsize as u_int);
        extended_cells = extended_cells.wrapping_add((*gl).extdsize);
        i = i.wrapping_add(1);
    }
    xasprintf(
        &raw mut value,
        b"%u,%zu,%u,%zu,%u,%zu\0" as *const u8 as *const ::core::ffi::c_char,
        lines,
        (lines as usize).wrapping_mul(::core::mem::size_of::<grid_line>() as usize),
        cells,
        (cells as usize).wrapping_mul(::core::mem::size_of::<grid_cell_entry>() as usize),
        extended_cells,
        (extended_cells as usize).wrapping_mul(::core::mem::size_of::<grid_extd_entry>() as usize),
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_tabs(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut i: u_int = 0;
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    i = 0 as u_int;
    while i < (*(*wp).base.grid).sx {
        if !(*(*wp)
            .base
            .tabs
            .offset((i >> 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << (i & 0x7 as u_int)
            == 0)
        {
            if evbuffer_get_length(buffer) > 0 as size_t {
                evbuffer_add(
                    buffer,
                    b",\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    1 as size_t,
                );
            }
            evbuffer_add_printf(
                buffer,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
        }
        i = i.wrapping_add(1);
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_fg(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    tty_default_colours(&raw mut gc, wp, ::core::ptr::null_mut::<u_int>());
    return xstrdup(colour_format(gc.fg).as_ptr()) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_flags(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return xstrdup(window_pane_printable_flags((*ft).wp)) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_floating_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if window_pane_is_floating(wp) != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_modal_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if wp == (*(*wp).window).modal {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_bg(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    tty_default_colours(&raw mut gc, wp, ::core::ptr::null_mut::<u_int>());
    return xstrdup(colour_format(gc.bg).as_ptr()) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_group_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = (*ft).s;
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut loop_0: *mut session = ::core::ptr::null_mut::<session>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    sg = session_group_contains(s);
    if sg.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    loop_0 = (*sg).sessions.tqh_first;
    while !loop_0.is_null() {
        if evbuffer_get_length(buffer) > 0 as size_t {
            evbuffer_add(
                buffer,
                b",\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
        }
        evbuffer_add_printf(
            buffer,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*loop_0).name,
        );
        loop_0 = (*loop_0).gentry.tqe_next;
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_group_attached_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = (*ft).s;
    let mut client_session: *mut session = ::core::ptr::null_mut::<session>();
    let mut session_loop: *mut session = ::core::ptr::null_mut::<session>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    sg = session_group_contains(s);
    if sg.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        client_session = (*loop_0).session;
        if !client_session.is_null() {
            session_loop = (*sg).sessions.tqh_first;
            while !session_loop.is_null() {
                if session_loop == client_session {
                    if evbuffer_get_length(buffer) > 0 as size_t {
                        evbuffer_add(
                            buffer,
                            b",\0" as *const u8 as *const ::core::ffi::c_char
                                as *const ::core::ffi::c_void,
                            1 as size_t,
                        );
                    }
                    evbuffer_add_printf(
                        buffer,
                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                        (*loop_0).name,
                    );
                }
                session_loop = (*session_loop).gentry.tqe_next;
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_in_mode(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut n: u_int = 0 as u_int;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    wme = (*wp).modes.tqh_first;
    while !wme.is_null() {
        n = n.wrapping_add(1);
        wme = (*wme).entry.tqe_next;
    }
    xasprintf(
        &raw mut value,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        n,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_at_top(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut status: ::core::ffi::c_int = 0;
    let mut flag: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    status = window_pane_get_pane_status(wp);
    if status == PANE_STATUS_TOP {
        flag = ((*wp).yoff == 1 as ::core::ffi::c_int) as ::core::ffi::c_int;
    } else {
        flag = ((*wp).yoff == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    xasprintf(
        &raw mut value,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        flag,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_at_bottom(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut status: ::core::ffi::c_int = 0;
    let mut flag: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*wp).window as *mut window;
    status = window_pane_get_pane_status(wp);
    if status == PANE_STATUS_BOTTOM {
        flag = ((*wp).yoff + (*wp).sy as ::core::ffi::c_int
            == (*w).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
            as ::core::ffi::c_int;
    } else {
        flag = ((*wp).yoff + (*wp).sy as ::core::ffi::c_int == (*w).sy as ::core::ffi::c_int)
            as ::core::ffi::c_int;
    }
    xasprintf(
        &raw mut value,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        flag,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_cursor_character(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    grid_view_get_cell((*wp).base.grid, (*wp).base.cx, (*wp).base.cy, &raw mut gc);
    if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
        xasprintf(
            &raw mut value,
            b"%.*s\0" as *const u8 as *const ::core::ffi::c_char,
            gc.data.size as ::core::ffi::c_int,
            &raw mut gc.data.data as *mut u_char,
        );
    }
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_cursor_colour(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if wp.is_null() || (*wp).screen.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*(*wp).screen).ccolour != -(1 as ::core::ffi::c_int) {
        return xstrdup(colour_format((*(*wp).screen).ccolour).as_ptr())
            as *mut ::core::ffi::c_void;
    }
    return xstrdup(colour_format((*(*wp).screen).default_ccolour).as_ptr())
        as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_mouse_word(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    wp = cmd_mouse_pane(
        &raw mut (*ft).m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if cmd_mouse_at(
        wp,
        &raw mut (*ft).m,
        &raw mut x,
        &raw mut y,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if !(*wp).modes.tqh_first.is_null() {
        if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
            return window_copy_get_word(wp, x, y) as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    gd = (*wp).base.grid;
    return format_grid_word(gd, x, (*gd).hsize.wrapping_add(y)) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_mouse_hyperlink(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    wp = cmd_mouse_pane(
        &raw mut (*ft).m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if cmd_mouse_at(
        wp,
        &raw mut (*ft).m,
        &raw mut x,
        &raw mut y,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if !(*wp).modes.tqh_first.is_null() {
        if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
            return window_copy_get_hyperlink(wp, x, y) as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    gd = (*wp).base.grid;
    return format_grid_hyperlink(gd, x, (*gd).hsize.wrapping_add(y), (*wp).screen)
        as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_mouse_line(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    wp = cmd_mouse_pane(
        &raw mut (*ft).m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if cmd_mouse_at(
        wp,
        &raw mut (*ft).m,
        &raw mut x,
        &raw mut y,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if !(*wp).modes.tqh_first.is_null() {
        if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
            return window_copy_get_line(wp, y) as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    gd = (*wp).base.grid;
    return format_grid_line(gd, (*gd).hsize.wrapping_add(y)) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_mouse_status_line(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*ft).c.is_null() || !(*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*ft).m.statusat == 0 as ::core::ffi::c_int && (*ft).m.y < (*ft).m.statuslines {
        y = (*ft).m.y;
    } else if (*ft).m.statusat > 0 as ::core::ffi::c_int && (*ft).m.y >= (*ft).m.statusat as u_int {
        y = (*ft).m.y.wrapping_sub((*ft).m.statusat as u_int);
    } else {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    xasprintf(
        &raw mut value,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        y,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_mouse_status_range(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*ft).c.is_null() || !(*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*ft).m.statusat == 0 as ::core::ffi::c_int && (*ft).m.y < (*ft).m.statuslines {
        x = (*ft).m.x;
        y = (*ft).m.y;
    } else if (*ft).m.statusat > 0 as ::core::ffi::c_int && (*ft).m.y >= (*ft).m.statusat as u_int {
        x = (*ft).m.x;
        y = (*ft).m.y.wrapping_sub((*ft).m.statusat as u_int);
    } else {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    sr = status_get_range((*ft).c, x, y);
    if sr.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    match (*sr).type_0 as ::core::ffi::c_uint {
        0 => return ::core::ptr::null_mut::<::core::ffi::c_void>(),
        1 => {
            return xstrdup(b"left\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        2 => {
            return xstrdup(b"right\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        3 => {
            return xstrdup(b"pane\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        4 => {
            return xstrdup(b"window\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        5 => {
            return xstrdup(b"session\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        6 => {
            return xstrdup(&raw mut (*sr).string as *mut ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        7 => {
            return xstrdup(b"control\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        _ => {}
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_alternate_on(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if !(*(*ft).wp).base.saved_grid.is_null() {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_alternate_saved_x(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.saved_cx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_alternate_saved_y(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.saved_cy,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_bracket_paste_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        if (*(*(*ft).wp).screen).mode & MODE_BRACKETPASTE != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_buffer_name(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).pb.is_null() {
        return xstrdup(paste_buffer_name((*ft).pb)) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_buffer_sample(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).pb.is_null() {
        return paste_make_sample((*ft).pb) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_buffer_full(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut size: size_t = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !(*ft).pb.is_null() {
        s = paste_buffer_data((*ft).pb, &raw mut size);
        if !s.is_null() {
            return xstrndup(s, size) as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_buffer_size(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut size: size_t = 0;
    if !(*ft).pb.is_null() {
        paste_buffer_data((*ft).pb, &raw mut size);
        return format_printf(b"%zu\0" as *const u8 as *const ::core::ffi::c_char, size)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_cell_height(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).tty.ypixel,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_cell_width(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).tty.xpixel,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_colours(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut term: *mut tty_term = ::core::ptr::null_mut::<tty_term>();
    let mut colours: u_int = 0;
    if (*ft).c.is_null() || !(*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    term = (*(*ft).c).tty.term;
    if (*term).flags & TERM_RGBCOLOURS != 0 {
        colours = 16777216 as ::core::ffi::c_int as u_int;
    } else if (*term).flags & TERM_256COLOURS != 0 {
        colours = 256 as u_int;
    } else {
        colours = tty_term_number(term, TTYC_COLORS) as u_int;
        if colours < 8 as u_int {
            colours = 2 as u_int;
        } else if colours < 16 as u_int {
            colours = 8 as u_int;
        } else {
            colours = 16 as u_int;
        }
    }
    return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, colours)
        as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_client_control_mode(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        if (*(*ft).c).flags & CLIENT_CONTROL as uint64_t != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_discarded(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return format_printf(
            b"%zu\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).discarded,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_flags(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return xstrdup(server_client_get_flags((*ft).c)) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_height(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).tty.sy,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_key_table(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return xstrdup((*(*(*ft).c).keytable).name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_last_session(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null()
        && !(*(*ft).c).last_session.is_null()
        && session_alive((*(*ft).c).last_session) != 0
    {
        return xstrdup((*(*(*ft).c).last_session).name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_name(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return xstrdup((*(*ft).c).name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_pid(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return format_printf(
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).pid as ::core::ffi::c_long,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_prefix(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !(*ft).c.is_null() {
        name = server_client_get_key_table((*ft).c);
        if strcmp((*(*(*ft).c).keytable).name, name) == 0 as ::core::ffi::c_int {
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_readonly(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        if (*(*ft).c).flags & CLIENT_READONLY as uint64_t != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_session(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() && !(*(*ft).c).session.is_null() {
        return xstrdup((*(*(*ft).c).session).name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_termfeatures(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return xstrdup(tty_get_features((*(*ft).c).term_features)) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_termname(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return xstrdup((*(*ft).c).term_name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_termtype(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        if (*(*ft).c).term_type.is_null() {
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup((*(*ft).c).term_type) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_tty(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return xstrdup((*(*ft).c).ttyname) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_uid(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut uid: uid_t = 0;
    if !(*ft).c.is_null() {
        uid = proc_get_peer_uid((*(*ft).c).peer);
        if uid != -(1 as ::core::ffi::c_int) as uid_t {
            return format_printf(
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                uid as ::core::ffi::c_long,
            ) as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_user(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut uid: uid_t = 0;
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    if !(*ft).c.is_null() {
        if !(*(*ft).c).user.is_null() {
            return xstrdup((*(*ft).c).user) as *mut ::core::ffi::c_void;
        }
        uid = proc_get_peer_uid((*(*ft).c).peer);
        if uid != -(1 as ::core::ffi::c_int) as uid_t && {
            pw = getpwuid(uid as __uid_t);
            !pw.is_null()
        } {
            (*(*ft).c).user = xstrdup((*pw).pw_name);
            return xstrdup((*(*ft).c).user) as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_utf8(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        if (*(*ft).c).flags & CLIENT_UTF8 as uint64_t != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_width(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).tty.sx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_written(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return format_printf(
            b"%zu\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).written,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_theme(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        match (*(*ft).c).theme as ::core::ffi::c_uint {
            2 => {
                return xstrdup(b"dark\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            1 => {
                return xstrdup(b"light\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            0 => return ::core::ptr::null_mut::<::core::ffi::c_void>(),
            _ => {}
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_config_files(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut slen: size_t = 0 as size_t;
    let mut i: u_int = 0;
    let mut n: size_t = 0;
    i = 0 as u_int;
    while i < cfg_nfiles {
        n = strlen(*cfg_files.offset(i as isize)).wrapping_add(1 as size_t);
        s = xrealloc(
            s as *mut ::core::ffi::c_void,
            slen.wrapping_add(n).wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        slen = slen.wrapping_add(xsnprintf(
            s.offset(slen as isize),
            n.wrapping_add(1 as size_t),
            b"%s,\0" as *const u8 as *const ::core::ffi::c_char,
            *cfg_files.offset(i as isize),
        ) as size_t);
        i = i.wrapping_add(1);
    }
    if s.is_null() {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    *s.offset(slen.wrapping_sub(1 as size_t) as isize) = '\0' as i32 as ::core::ffi::c_char;
    return s as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_cursor_flag(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_CURSOR != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_cursor_shape(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        match (*(*(*ft).wp).screen).cstyle as ::core::ffi::c_uint {
            1 => {
                return xstrdup(b"block\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            2 => {
                return xstrdup(b"underline\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            3 => {
                return xstrdup(b"bar\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            _ => {
                return xstrdup(b"default\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_cursor_very_visible(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        if (*(*(*ft).wp).screen).mode & MODE_CURSOR_VERY_VISIBLE != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_cursor_x(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.cx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_cursor_y(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.cy,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_cursor_blinking(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        if (*(*(*ft).wp).screen).mode & MODE_CURSOR_BLINKING != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_history_added(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ft).wp).base.grid).scroll_added,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_history_collected(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*wp).base.grid).scroll_collected,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_history_generation(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*wp).base.grid).scroll_generation,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_history_limit(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ft).wp).base.grid).hlimit,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_history_size(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ft).wp).base.grid).hsize,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_insert_flag(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_INSERT != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_keypad_cursor_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_KCURSOR != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_keypad_flag(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_KKEYPAD != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_all_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_ALL != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_any_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & ALL_MOUSE_MODES != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_button_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_BUTTON != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_pane(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (*ft).m.valid != 0 {
        wp = cmd_mouse_pane(
            &raw mut (*ft).m,
            ::core::ptr::null_mut::<*mut session>(),
            ::core::ptr::null_mut::<*mut winlink>(),
        );
        if !wp.is_null() {
            return format_printf(
                b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            ) as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_sgr_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_SGR != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_standard_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_STANDARD != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_utf8_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_UTF8 != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_x(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    wp = cmd_mouse_pane(
        &raw mut (*ft).m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if !wp.is_null()
        && cmd_mouse_at(
            wp,
            &raw mut (*ft).m,
            &raw mut x,
            &raw mut y,
            0 as ::core::ffi::c_int,
        ) == 0 as ::core::ffi::c_int
    {
        return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, x)
            as *mut ::core::ffi::c_void;
    }
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        if (*ft).m.statusat == 0 as ::core::ffi::c_int && (*ft).m.y < (*ft).m.statuslines {
            return format_printf(
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*ft).m.x,
            ) as *mut ::core::ffi::c_void;
        }
        if (*ft).m.statusat > 0 as ::core::ffi::c_int && (*ft).m.y >= (*ft).m.statusat as u_int {
            return format_printf(
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*ft).m.x,
            ) as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_y(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    wp = cmd_mouse_pane(
        &raw mut (*ft).m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if !wp.is_null()
        && cmd_mouse_at(
            wp,
            &raw mut (*ft).m,
            &raw mut x,
            &raw mut y,
            0 as ::core::ffi::c_int,
        ) == 0 as ::core::ffi::c_int
    {
        return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, y)
            as *mut ::core::ffi::c_void;
    }
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        if (*ft).m.statusat == 0 as ::core::ffi::c_int && (*ft).m.y < (*ft).m.statuslines {
            return format_printf(
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*ft).m.y,
            ) as *mut ::core::ffi::c_void;
        }
        if (*ft).m.statusat > 0 as ::core::ffi::c_int && (*ft).m.y >= (*ft).m.statusat as u_int {
            return format_printf(
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*ft).m.y.wrapping_sub((*ft).m.statusat as u_int),
            ) as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_next_session_id(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    return format_printf(
        b"$%u\0" as *const u8 as *const ::core::ffi::c_char,
        next_session_id,
    ) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_origin_flag(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_ORIGIN != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_synchronized_output_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_SYNC != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_private_modes(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    static mut table: [C2RustUnnamed_43; 14] = [
        C2RustUnnamed_43 {
            mode: MODE_KCURSOR,
            number: 1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_ORIGIN,
            number: 6 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_WRAP,
            number: 7 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_CURSOR_BLINKING,
            number: 12 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_CURSOR,
            number: 25 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_MOUSE_STANDARD,
            number: 1000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_MOUSE_BUTTON,
            number: 1002 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_MOUSE_ALL,
            number: 1003 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_FOCUSON,
            number: 1004 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_MOUSE_UTF8,
            number: 1005 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_MOUSE_SGR,
            number: 1006 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_BRACKETPASTE,
            number: 2004 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_SYNC,
            number: 2026 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_THEME_UPDATES,
            number: 2031 as ::core::ffi::c_int,
        },
    ];
    let mut mode: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    if (*ft).wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    mode = (*(*ft).wp).base.mode;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_43; 14]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_43>() as usize)
    {
        if !(!mode & table[i as usize].mode != 0) {
            if !(table[i as usize].mode == MODE_CURSOR_BLINKING
                && !mode & MODE_CURSOR_BLINKING_SET != 0)
            {
                if value.is_null() {
                    xasprintf(
                        &raw mut value,
                        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                        table[i as usize].number,
                    );
                } else {
                    xasprintf(
                        &raw mut tmp,
                        b"%s,%d\0" as *const u8 as *const ::core::ffi::c_char,
                        value,
                        table[i as usize].number,
                    );
                    free(value as *mut ::core::ffi::c_void);
                    value = tmp;
                }
            }
        }
        i = i.wrapping_add(1);
    }
    if value.is_null() {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_active(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*ft).wp == (*(*(*ft).wp).window).active {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_at_left(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).xoff == 0 as ::core::ffi::c_int {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_at_right(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).xoff + (*(*ft).wp).sx as ::core::ffi::c_int
            == (*(*(*ft).wp).window).sx as ::core::ffi::c_int
        {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_bottom(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).yoff + (*wp).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_dead(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if (*wp).fd == -(1 as ::core::ffi::c_int) && (*wp).flags & PANE_STATUSREADY != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_dead_signal(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !wp.is_null() {
        if (*wp).flags & PANE_STATUSREADY != 0
            && (((*wp).status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                as ::core::ffi::c_schar as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int
                > 0 as ::core::ffi::c_int
        {
            name = sig2name((*wp).status & 0x7f as ::core::ffi::c_int);
            return format_printf(b"%s\0" as *const u8 as *const ::core::ffi::c_char, name)
                as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_dead_status(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if (*wp).flags & PANE_STATUSREADY != 0
            && (*wp).status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        {
            return format_printf(
                b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                ((*wp).status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int,
            ) as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_dead_time(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if (*wp).flags & PANE_STATUSDRAWN != 0 {
            return &raw mut (*wp).dead_time as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_last_output_time(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    static mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if !wp.is_null() && (*wp).last_output_time != 0 as time_t {
        tv.tv_sec = (*wp).last_output_time as __time_t;
        tv.tv_usec = 0 as __suseconds_t;
        return &raw mut tv as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_output_generation(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut value: ::core::ffi::c_ulonglong = 0;
    if !(*ft).wp.is_null() {
        value = (*(*ft).wp).output_generation as ::core::ffi::c_ulonglong;
        return format_printf(b"%llu\0" as *const u8 as *const ::core::ffi::c_char, value)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_last_prompt_time(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    static mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if !wp.is_null() && (*wp).last_prompt_time != 0 as time_t {
        tv.tv_sec = (*wp).last_prompt_time as __time_t;
        tv.tv_usec = 0 as __suseconds_t;
        return &raw mut tv as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_command_start_time(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    static mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if !wp.is_null() && (*wp).cmd_start_time != 0 as time_t {
        tv.tv_sec = (*wp).cmd_start_time as __time_t;
        tv.tv_usec = 0 as __suseconds_t;
        return &raw mut tv as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_command_end_time(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    static mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if !wp.is_null() && (*wp).cmd_end_time != 0 as time_t {
        tv.tv_sec = (*wp).cmd_end_time as __time_t;
        tv.tv_usec = 0 as __suseconds_t;
        return &raw mut tv as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_command_running(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            ((*wp).flags & PANE_CMDRUNNING != 0) as ::core::ffi::c_int,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_command_duration(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut end: time_t = 0;
    if wp.is_null() || (*wp).cmd_start_time == 0 as time_t {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*wp).flags & PANE_CMDRUNNING != 0 {
        end = time(::core::ptr::null_mut::<time_t>());
    } else {
        end = (*wp).cmd_end_time;
    }
    if end < (*wp).cmd_start_time {
        end = (*wp).cmd_start_time;
    }
    return format_printf(
        b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
        (end - (*wp).cmd_start_time) as ::core::ffi::c_longlong,
    ) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_command_status(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() && (*wp).cmd_status != -(1 as ::core::ffi::c_int) {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).cmd_status,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_format(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if (*ft).type_0 as ::core::ffi::c_uint
        == FORMAT_TYPE_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_height(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).sy,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_id(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).id,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_index(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut idx: u_int = 0;
    if !(*ft).wp.is_null() && window_pane_index((*ft).wp, &raw mut idx) == 0 as ::core::ffi::c_int {
        return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, idx)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_input_off(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).flags & PANE_INPUTOFF != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_unseen_changes(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).flags & PANE_UNSEENCHANGES != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_key_mode(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        match (*(*(*ft).wp).screen).mode & EXTENDED_KEY_MODES {
            MODE_KEYS_EXTENDED => {
                return xstrdup(b"Ext 1\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            MODE_KEYS_EXTENDED_2 => {
                return xstrdup(b"Ext 2\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            _ => {
                return xstrdup(b"VT10x\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_last(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*ft).wp == (*(*(*ft).wp).window).last_panes.tqh_first {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_left(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).xoff,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_marked(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if server_check_marked() != 0 && marked_pane.wp == (*ft).wp {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_marked_set(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if server_check_marked() != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_mode(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if !(*ft).wp.is_null() {
        wme = (*(*ft).wp).modes.tqh_first;
        if !wme.is_null() {
            return xstrdup((*(*wme).mode).name) as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_path(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.path.is_null() {
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup((*(*ft).wp).base.path) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_pid(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() && (*(*ft).wp).fd != -(1 as ::core::ffi::c_int) {
        return format_printf(
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).pid as ::core::ffi::c_long,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_pipe(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).pipe_fd != -(1 as ::core::ffi::c_int) {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_pipe_pid(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*ft).wp.is_null() && (*(*ft).wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        xasprintf(
            &raw mut value,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).pipe_pid as ::core::ffi::c_long,
        );
    }
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_pb_progress(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*ft).wp.is_null() {
        xasprintf(
            &raw mut value,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.progress_bar.progress,
        );
    }
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_pb_state(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        match (*(*ft).wp).base.progress_bar.state as ::core::ffi::c_uint {
            0 => {
                return xstrdup(b"hidden\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            1 => {
                return xstrdup(b"normal\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            2 => {
                return xstrdup(b"error\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            3 => {
                return xstrdup(b"indeterminate\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            4 => {
                return xstrdup(b"paused\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            _ => {}
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_right(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).xoff + (*wp).sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_search_string(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).searchstr.is_null() {
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup((*(*ft).wp).searchstr) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_synchronized(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if options_get_number(
            (*(*ft).wp).options,
            b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_title(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return xstrdup((*(*ft).wp).base.title) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_top(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).yoff,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_tty(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return xstrdup(&raw mut (*(*ft).wp).tty as *mut ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_unzoomed_height(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut root: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut status: ::core::ffi::c_int = 0;
    let mut floating: ::core::ffi::c_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*wp).window as *mut window;
    lc = (*wp).saved_layout_cell;
    if lc.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    if lc.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    sy = (*lc).g.sy;
    floating = (*lc).flags & LAYOUT_CELL_FLOATING;
    root = (*w).saved_layout_root;
    if root.is_null() {
        root = (*w).layout_root;
    }
    if lc == (*wp).saved_layout_cell && floating == 0 {
        status = window_get_pane_status(w);
    } else {
        status = window_pane_get_pane_status(wp);
    }
    if floating == 0
        && !root.is_null()
        && layout_add_horizontal_border(root, lc, status) != 0
        && sy > 1 as u_int
    {
        sy = sy.wrapping_sub(1);
    }
    return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, sy)
        as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_unzoomed_width(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut saved: ::core::ffi::c_int = 0;
    let mut sb_w: ::core::ffi::c_int = 0;
    let mut sb_pad: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    lc = (*wp).saved_layout_cell;
    saved = (lc != NULL_0 as *mut layout_cell) as ::core::ffi::c_int;
    if lc.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    if lc.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    sx = (*lc).g.sx;
    if saved != 0 && (*wp).base.saved_grid.is_null() && (*(*wp).window).sb == PANE_SCROLLBARS_ALWAYS
        || saved == 0 && window_pane_scrollbar_reserve(wp) != 0
    {
        sb_w = (*wp).scrollbar_style.width;
        sb_pad = (*wp).scrollbar_style.pad;
        if sb_w < 1 as ::core::ffi::c_int {
            sb_w = 1 as ::core::ffi::c_int;
        }
        if sb_pad < 0 as ::core::ffi::c_int {
            sb_pad = 0 as ::core::ffi::c_int;
        }
        if sx as ::core::ffi::c_int - sb_w - sb_pad < PANE_MINIMUM {
            sx = PANE_MINIMUM as u_int;
        } else {
            sx = sx.wrapping_sub((sb_w + sb_pad) as u_int);
        }
    }
    return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, sx)
        as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_width(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).sx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_x(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).xoff,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_y(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).yoff,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_z(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut idx: u_int = 0;
    if !(*ft).wp.is_null() && window_pane_zindex((*ft).wp, &raw mut idx) == 0 as ::core::ffi::c_int
    {
        return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, idx)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_zoomed_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if (*wp).flags & PANE_ZOOMED != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_scroll_region_lower(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.rlower,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_scroll_region_upper(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.rupper,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_server_sessions(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut n: u_int = 0 as u_int;
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        n = n.wrapping_add(1);
        s = sessions_RB_NEXT(s);
    }
    return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, n)
        as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_active(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if (*ft).s.is_null() || (*ft).c.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*(*ft).c).session == (*ft).s {
        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_activity_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*ft).s.is_null() {
        wl = winlinks_RB_MINMAX(&raw mut (*(*ft).s).windows, RB_NEGINF);
        if !wl.is_null() {
            if (*(*ft).wl).flags & WINLINK_ACTIVITY != 0 {
                return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_bell_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*ft).s.is_null() {
        wl = winlinks_RB_MINMAX(&raw mut (*(*ft).s).windows, RB_NEGINF);
        if !wl.is_null() {
            if (*wl).flags & WINLINK_BELL != 0 {
                return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_silence_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*ft).s.is_null() {
        wl = winlinks_RB_MINMAX(&raw mut (*(*ft).s).windows, RB_NEGINF);
        if !wl.is_null() {
            if (*(*ft).wl).flags & WINLINK_SILENCE != 0 {
                return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_attached(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).s).attached,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_format(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if (*ft).type_0 as ::core::ffi::c_uint
        == FORMAT_TYPE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_group(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if !(*ft).s.is_null() && {
        sg = session_group_contains((*ft).s);
        !sg.is_null()
    } {
        return xstrdup((*sg).name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_group_attached(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if !(*ft).s.is_null() && {
        sg = session_group_contains((*ft).s);
        !sg.is_null()
    } {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            session_group_attached_count(sg),
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_group_many_attached(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if !(*ft).s.is_null() && {
        sg = session_group_contains((*ft).s);
        !sg.is_null()
    } {
        if session_group_attached_count(sg) > 1 as u_int {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_group_size(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if !(*ft).s.is_null() && {
        sg = session_group_contains((*ft).s);
        !sg.is_null()
    } {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            session_group_count(sg),
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_grouped(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        if !session_group_contains((*ft).s).is_null() {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_id(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return format_printf(
            b"$%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).s).id,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_many_attached(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        if (*(*ft).s).attached > 1 as u_int {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_marked(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        if server_check_marked() != 0 && marked_pane.s == (*ft).s {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_name(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return xstrdup((*(*ft).s).name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_path(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return xstrdup((*(*ft).s).cwd) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_windows(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            winlink_count(&raw mut (*(*ft).s).windows),
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_socket_path(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    return xstrdup(socket_path) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_version(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    return xstrdup(getversion()) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_sixel_support(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_active_window_index(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ft).s).curw).idx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_last_window_index(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*ft).s.is_null() {
        wl = winlinks_RB_MINMAX(&raw mut (*(*ft).s).windows, RB_INF);
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).idx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_active(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*ft).wl == (*(*(*ft).wl).session).curw {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_activity_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*(*ft).wl).flags & WINLINK_ACTIVITY != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_bell_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*(*ft).wl).flags & WINLINK_BELL != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_bigger(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if !(*ft).c.is_null() {
        if tty_window_offset(
            &raw mut (*(*ft).c).tty,
            &raw mut ox,
            &raw mut oy,
            &raw mut sx,
            &raw mut sy,
        ) != 0
        {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_cell_height(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).w).ypixel,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_cell_width(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).w).xpixel,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_end_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*ft).wl == winlinks_RB_MINMAX(&raw mut (*(*(*ft).wl).session).windows, RB_INF) {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_flags(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        return xstrdup(window_printable_flags((*ft).wl, 1 as ::core::ffi::c_int))
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_format(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if (*ft).type_0 as ::core::ffi::c_uint
        == FORMAT_TYPE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_height(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).w).sy,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_manual_height(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = (*ft).w;
    if w.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if options_get_number(
        (*w).options,
        b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
    ) != WINDOW_SIZE_MANUAL as ::core::ffi::c_longlong
    {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return format_printf(
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).manual_sy,
    ) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_id(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"@%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).w).id,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_index(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wl).idx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_last_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*ft).wl == (*(*(*ft).wl).session).lastw.tqh_first {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_linked(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !(*ft).wl.is_null() {
        s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
        while !s.is_null() {
            wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
            while !wl.is_null() {
                if (*wl).window == (*(*ft).wl).window {
                    if found != 0 {
                        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                            as *mut ::core::ffi::c_void;
                    }
                    found = 1 as ::core::ffi::c_int;
                }
                wl = winlinks_RB_NEXT(wl);
            }
            s = sessions_RB_NEXT(s);
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_linked_sessions(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut n: u_int = 0 as u_int;
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*(*ft).wl).window;
    sg = session_groups_minmax(&raw mut session_groups, RB_NEGINF);
    while !sg.is_null() {
        s = (*sg).sessions.tqh_first;
        if !winlink_find_by_window(&raw mut (*s).windows, w).is_null() {
            n = n.wrapping_add(1);
        }
        sg = session_groups_next(sg);
    }
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        if session_group_contains(s).is_null() {
            if !winlink_find_by_window(&raw mut (*s).windows, w).is_null() {
                n = n.wrapping_add(1);
            }
        }
        s = sessions_RB_NEXT(s);
    }
    return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, n)
        as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_marked_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if server_check_marked() != 0 && marked_pane.wl == (*ft).wl {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_modal_pane(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() && !(*(*ft).w).modal.is_null() {
        return format_printf(
            b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ft).w).modal).id,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_name(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).w).name,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_offset_x(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if !(*ft).c.is_null() {
        if tty_window_offset(
            &raw mut (*(*ft).c).tty,
            &raw mut ox,
            &raw mut oy,
            &raw mut sx,
            &raw mut sy,
        ) != 0
        {
            return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, ox)
                as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_offset_y(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if !(*ft).c.is_null() {
        if tty_window_offset(
            &raw mut (*(*ft).c).tty,
            &raw mut ox,
            &raw mut oy,
            &raw mut sx,
            &raw mut sy,
        ) != 0
        {
            return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, oy)
                as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_panes(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            window_count_panes((*ft).w, 1 as ::core::ffi::c_int),
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_raw_flags(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        return xstrdup(window_printable_flags((*ft).wl, 0 as ::core::ffi::c_int))
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_silence_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*(*ft).wl).flags & WINLINK_SILENCE != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_start_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*ft).wl == winlinks_RB_MINMAX(&raw mut (*(*(*ft).wl).session).windows, RB_NEGINF) {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_width(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).w).sx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_manual_width(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = (*ft).w;
    if w.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if options_get_number(
        (*w).options,
        b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
    ) != WINDOW_SIZE_MANUAL as ::core::ffi::c_longlong
    {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return format_printf(
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).manual_sx,
    ) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_zoomed_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        if (*(*ft).w).flags & WINDOW_ZOOMED != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_wrap_flag(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_WRAP != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_buffer_created(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    static mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if !(*ft).pb.is_null() {
        tv.tv_usec = 0 as __suseconds_t;
        tv.tv_sec = tv.tv_usec as __time_t;
        tv.tv_sec = paste_buffer_created((*ft).pb) as __time_t;
        return &raw mut tv as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_activity(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return &raw mut (*(*ft).c).activity_time as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_created(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return &raw mut (*(*ft).c).creation_time as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_activity(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return &raw mut (*(*ft).s).activity_time as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_created(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return &raw mut (*(*ft).s).creation_time as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_last_attached(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return &raw mut (*(*ft).s).last_attached_time as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_start_time(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    return &raw mut start_time as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_activity(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return &raw mut (*(*ft).w).activity_time as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_buffer_mode_format(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    return xstrdup(window_buffer_mode.default_format) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_client_mode_format(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    return xstrdup(window_client_mode.default_format) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_tree_mode_format(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    return xstrdup(window_tree_mode.default_format) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_uid(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    return format_printf(
        b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
        getuid() as ::core::ffi::c_long,
    ) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_user(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    static mut cached: *mut ::core::ffi::c_char =
        ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char;
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    if cached.is_null() && {
        pw = getpwuid(getuid());
        !pw.is_null()
    } {
        cached = xstrdup((*pw).pw_name);
    }
    if !cached.is_null() {
        return xstrdup(cached) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
pub(super) static mut format_table: [format_table_entry; 214] = unsafe {
    [
        format_table_entry {
            key: b"active_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_active_window_index
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"alternate_on\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_alternate_on
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"alternate_saved_x\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_alternate_saved_x
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"alternate_saved_y\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_alternate_saved_y
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"bracket_paste_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_bracket_paste_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"buffer_created\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_buffer_created
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"buffer_full\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_buffer_full
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"buffer_mode_format\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_buffer_mode_format
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"buffer_name\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_buffer_name
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"buffer_sample\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_buffer_sample
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"buffer_size\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_buffer_size
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_activity\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_client_activity
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_cell_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_cell_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_cell_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_cell_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_colours\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_colours
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_control_mode\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_control_mode
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_created\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_client_created
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_discarded\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_discarded
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_flags\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_flags
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_key_table\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_key_table
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_last_session\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_last_session
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_mode_format\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_mode_format
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_name\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_name
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_pid\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_pid
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_prefix\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_prefix
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_readonly\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_readonly
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_session\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_session
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_termfeatures\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_termfeatures
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_termname\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_termname
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_termtype\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_termtype
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_theme\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_theme
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_tty\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_tty
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_uid\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_uid
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_user\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_user
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_utf8\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_utf8
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_written\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_written
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"config_files\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_config_files
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_blinking\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_blinking
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_character\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_character
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_colour\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_colour
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_shape\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_shape
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_very_visible\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_very_visible
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_x\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_x
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_y\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_y
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_added\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_added
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_all_bytes\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_all_bytes
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_bytes\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_bytes
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_collected\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_collected
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_generation\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_generation
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_limit\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_limit
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_size\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_size
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"host\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_host
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"host_short\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_host_short
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"insert_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_insert_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"keypad_cursor_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_keypad_cursor_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"keypad_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_keypad_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"last_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_last_window_index
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_all_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_all_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_any_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_any_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_button_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_button_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_hyperlink\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_hyperlink
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_line\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_line
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_pane\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_pane
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_sgr_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_sgr_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_standard_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_standard_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_status_line\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_status_line
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_status_range\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_status_range
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_utf8_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_utf8_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_word\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_word
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_x\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_x
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_y\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_y
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"next_session_id\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_next_session_id
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"origin_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_origin_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_active\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_active
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_at_bottom\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_at_bottom
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_at_left\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_at_left
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_at_right\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_at_right
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_at_top\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_at_top
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_bg\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_bg
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_bottom\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_bottom
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_command_duration\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_command_duration
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_command_end_time\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_pane_command_end_time
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_command_running\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_command_running
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_command_start_time\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_pane_command_start_time
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_command_status\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_command_status
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_current_command\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_current_command
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_current_path\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_current_path
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_dead\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_dead
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_dead_signal\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_dead_signal
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_dead_status\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_dead_status
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_dead_time\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_pane_dead_time
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_fg\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_fg
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_flags\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_flags
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_floating_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_floating_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_format\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_format
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_id\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_id
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_in_mode\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_in_mode
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_index\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_index
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_input_off\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_input_off
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_key_mode\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_key_mode
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_last\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_last
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_last_output_time\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_pane_last_output_time
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_last_prompt_time\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_pane_last_prompt_time
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_left\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_left
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_marked\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_marked
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_marked_set\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_marked_set
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_modal_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_modal_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_mode\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_mode
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_output_generation\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_output_generation
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_path\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_path
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_pb_progress\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_pb_progress
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_pb_state\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_pb_state
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_pid\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_pid
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_pipe\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_pipe
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_pipe_pid\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_pipe_pid
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_private_modes\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_private_modes
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_right
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_search_string\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_search_string
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_start_command\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_start_command
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_start_command_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_start_command_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_start_path\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_start_path
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_synchronized\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_synchronized
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_tabs\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_tabs
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_title\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_title
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_top
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_tty\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_tty
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_unseen_changes\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_unseen_changes
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_unzoomed_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_unzoomed_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_unzoomed_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_unzoomed_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_x\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_x
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_y\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_y
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_z\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_z
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_zoomed_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_zoomed_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pid\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pid as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"scroll_region_lower\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_scroll_region_lower
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"scroll_region_upper\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_scroll_region_upper
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"server_sessions\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_server_sessions
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_active\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_active
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_activity\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_session_activity
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_activity_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_activity_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_alert\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_alert
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_alerts\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_alerts
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_attached\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_attached
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_attached_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_attached_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_bell_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_bell_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_created\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_session_created
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_format\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_format
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_group\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_group
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_group_attached\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_group_attached
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_group_attached_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_group_attached_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_group_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_group_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_group_many_attached\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_group_many_attached
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_group_size\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_group_size
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_grouped\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_grouped
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_id\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_id
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_last_attached\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_session_last_attached
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_many_attached\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_many_attached
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_marked\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_marked
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_name\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_name
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_path\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_path
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_silence_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_silence_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_stack\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_stack
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_windows\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_windows
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"sixel_support\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_sixel_support
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"socket_path\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_socket_path
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"start_time\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_start_time
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"synchronized_output_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_synchronized_output_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"tree_mode_format\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_tree_mode_format
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"uid\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_uid as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"user\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_user
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"version\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_version
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_active\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_active
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_active_clients\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_active_clients
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_active_clients_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_active_clients_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_active_sessions\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_active_sessions
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_active_sessions_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_active_sessions_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_activity\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_window_activity
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_activity_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_activity_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_bell_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_bell_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_bigger\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_bigger
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_cell_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_cell_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_cell_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_cell_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_end_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_end_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_flags\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_flags
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_format\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_format
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_id\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_id
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_index
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_last_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_layout\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_layout
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_linked\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_linked
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_linked_sessions\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_linked_sessions
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_linked_sessions_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_linked_sessions_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_manual_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_manual_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_manual_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_manual_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_marked_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_marked_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_modal_pane\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_modal_pane
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_name\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_name
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_offset_x\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_offset_x
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_offset_y\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_offset_y
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_panes\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_panes
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_raw_flags\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_raw_flags
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_silence_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_silence_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_stack_index\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_stack_index
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_start_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_start_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_visible_layout\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_visible_layout
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_zoomed_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_zoomed_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"wrap_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_wrap_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
    ]
};
pub(super) unsafe extern "C" fn format_table_compare(
    mut key0: *const ::core::ffi::c_void,
    mut entry0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut key: *const ::core::ffi::c_char = key0 as *const ::core::ffi::c_char;
    let mut entry: *const format_table_entry = entry0 as *const format_table_entry;
    return strcmp(key, (*entry).key);
}
pub(super) unsafe extern "C" fn format_table_get(
    mut key: *const ::core::ffi::c_char,
) -> *const format_table_entry {
    return bsearch(
        key as *const ::core::ffi::c_void,
        &raw const format_table as *const format_table_entry as *const ::core::ffi::c_void,
        (::core::mem::size_of::<[format_table_entry; 214]>() as size_t)
            .wrapping_div(::core::mem::size_of::<format_table_entry>() as size_t),
        ::core::mem::size_of::<format_table_entry>() as size_t,
        Some(
            format_table_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    ) as *const format_table_entry;
}
