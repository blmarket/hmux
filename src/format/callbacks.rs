// Built-in callbacks return owned bytes or copied timestamps. The sorted
// immutable table is shared by lookup and enumeration; external user callbacks
// retain their separate C ABI.
use super::*;
use crate::src::format::bytes::xformat;
use crate::src::server_client::server_client_set_user;
use crate::src::window::{window_pane_stack_first, window_winlinks_first, window_winlinks_next};
use std::ffi::{CStr, CString};
use std::fmt::Write as _;

unsafe fn format_cb_host(_ft: *mut format_tree) -> Option<CString> {
    let mut host: [::core::ffi::c_char; 65] = [0; 65];
    if gethostname(
        &raw mut host as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 65]>() as size_t,
    ) != 0 as ::core::ffi::c_int
    {
        return Some(c"".to_owned());
    }
    return Some(CStr::from_ptr(&raw mut host as *mut ::core::ffi::c_char).to_owned());
}
unsafe fn format_cb_host_short(_ft: *mut format_tree) -> Option<CString> {
    let mut host: [::core::ffi::c_char; 65] = [0; 65];
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if gethostname(
        &raw mut host as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 65]>() as size_t,
    ) != 0 as ::core::ffi::c_int
    {
        return Some(c"".to_owned());
    }
    cp = strchr(&raw mut host as *mut ::core::ffi::c_char, '.' as i32);
    if !cp.is_null() {
        *cp = '\0' as i32 as ::core::ffi::c_char;
    }
    return Some(CStr::from_ptr(&raw mut host as *mut ::core::ffi::c_char).to_owned());
}
unsafe fn format_cb_pid(_ft: *mut format_tree) -> Option<CString> {
    let mut value = None;
    value = Some(
        CString::new(format!(
            "{}",
            (getpid() as ::core::ffi::c_long) as ::core::ffi::c_long
        ))
        .expect("formatted numbers contain no NUL"),
    );
    return value;
}
unsafe fn format_cb_session_attached_list(mut ft: *mut format_tree) -> Option<CString> {
    let mut s: *mut session = (*ft).s;
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    if s.is_null() {
        return None;
    }
    let mut names = Vec::<u8>::new();
    loop_0 = clients.first();
    while !loop_0.is_null() {
        if (*loop_0).session == s {
            if !names.is_empty() {
                names.push(b',');
            }
            names.extend_from_slice(
                std::ffi::CStr::from_ptr(
                    ((*loop_0).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                )
                .to_bytes(),
            );
        }
        loop_0 = clients.next(loop_0);
    }
    if names.is_empty() {
        return None;
    }
    Some(CString::new(names).expect("callback bytes contain no NUL"))
}
unsafe fn format_cb_session_alert(mut ft: *mut format_tree) -> Option<CString> {
    let mut s: *mut session = (*ft).s;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut alerts: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut alerted: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if s.is_null() {
        return None;
    }
    *(&raw mut alerts as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
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
        wl = winlinks_next(&*wl);
    }
    return Some(CStr::from_ptr(&raw mut alerts as *mut ::core::ffi::c_char).to_owned());
}
unsafe fn format_cb_session_alerts(mut ft: *mut format_tree) -> Option<CString> {
    let mut s: *mut session = (*ft).s;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut alerts: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut tmp: [::core::ffi::c_char; 16] = [0; 16];
    if s.is_null() {
        return None;
    }
    *(&raw mut alerts as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
    while !wl.is_null() {
        if !((*wl).flags & WINLINK_ALERTFLAGS == 0 as ::core::ffi::c_int) {
            xformat(&mut tmp, format_args!("{}", ((*wl).idx) as u32));
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
        wl = winlinks_next(&*wl);
    }
    return Some(CStr::from_ptr(&raw mut alerts as *mut ::core::ffi::c_char).to_owned());
}
unsafe fn format_cb_session_stack(mut ft: *mut format_tree) -> Option<CString> {
    let mut s: *mut session = (*ft).s;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut result: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut tmp: [::core::ffi::c_char; 16] = [0; 16];
    if s.is_null() {
        return None;
    }
    xformat(&mut result, format_args!("{}", ((*(*s).curw).idx) as u32));
    wl = crate::src::window::winlink_stack_first(&(*s).lastw);
    while !wl.is_null() {
        xformat(&mut tmp, format_args!("{}", ((*wl).idx) as u32));
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
        wl = crate::src::window::winlink_stack_next(&(*s).lastw, wl);
    }
    return Some(CStr::from_ptr(&raw mut result as *mut ::core::ffi::c_char).to_owned());
}
unsafe fn format_cb_window_stack_index(mut ft: *mut format_tree) -> Option<CString> {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut idx: u_int = 0;
    let mut value = None;
    if (*ft).wl.is_null() {
        return None;
    }
    s = (*(*ft).wl).session;
    idx = 0 as u_int;
    wl = crate::src::window::winlink_stack_first(&(*s).lastw);
    while !wl.is_null() {
        idx = idx.wrapping_add(1);
        if wl == (*ft).wl {
            break;
        }
        wl = crate::src::window::winlink_stack_next(&(*s).lastw, wl);
    }
    if wl.is_null() {
        return Some(c"0".to_owned());
    }
    value =
        Some(CString::new(format!("{}", (idx) as u32)).expect("formatted numbers contain no NUL"));
    return value;
}
unsafe fn format_cb_window_linked_sessions_list(mut ft: *mut format_tree) -> Option<CString> {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut names = Vec::new();
    if (*ft).wl.is_null() {
        return None;
    }
    w = (*(*ft).wl).window;
    wl = window_winlinks_first(w);
    while !wl.is_null() {
        if !names.is_empty() {
            names.push(b',');
        }
        names.extend_from_slice((*(*wl).session).name.as_bytes());
        wl = window_winlinks_next(w, wl);
    }
    if names.is_empty() {
        return None;
    }
    return Some(CString::new(names).expect("callback bytes contain no NUL"));
}
unsafe fn format_cb_window_active_sessions(mut ft: *mut format_tree) -> Option<CString> {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut n: u_int = 0 as u_int;
    let mut value = None;
    if (*ft).wl.is_null() {
        return None;
    }
    w = (*(*ft).wl).window;
    wl = window_winlinks_first(w);
    while !wl.is_null() {
        if (*(*wl).session).curw == wl {
            n = n.wrapping_add(1);
        }
        wl = window_winlinks_next(w, wl);
    }
    value =
        Some(CString::new(format!("{}", (n) as u32)).expect("formatted numbers contain no NUL"));
    return value;
}
unsafe fn format_cb_window_active_sessions_list(mut ft: *mut format_tree) -> Option<CString> {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*ft).wl.is_null() {
        return None;
    }
    w = (*(*ft).wl).window;
    let mut names = Vec::<u8>::new();
    wl = window_winlinks_first(w);
    while !wl.is_null() {
        if (*(*wl).session).curw == wl {
            if !names.is_empty() {
                names.push(b',');
            }
            names.extend_from_slice((*(*wl).session).name.as_bytes());
        }
        wl = window_winlinks_next(w, wl);
    }
    if names.is_empty() {
        return None;
    }
    Some(CString::new(names).expect("callback bytes contain no NUL"))
}
unsafe fn format_cb_window_active_clients(mut ft: *mut format_tree) -> Option<CString> {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut client_session: *mut session = ::core::ptr::null_mut::<session>();
    let mut n: u_int = 0 as u_int;
    let mut value = None;
    if (*ft).wl.is_null() {
        return None;
    }
    w = (*(*ft).wl).window;
    loop_0 = clients.first();
    while !loop_0.is_null() {
        client_session = (*loop_0).session;
        if !client_session.is_null() {
            if w == (*(*client_session).curw).window {
                n = n.wrapping_add(1);
            }
        }
        loop_0 = clients.next(loop_0);
    }
    value =
        Some(CString::new(format!("{}", (n) as u32)).expect("formatted numbers contain no NUL"));
    return value;
}
unsafe fn format_cb_window_active_clients_list(mut ft: *mut format_tree) -> Option<CString> {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut client_session: *mut session = ::core::ptr::null_mut::<session>();
    if (*ft).wl.is_null() {
        return None;
    }
    w = (*(*ft).wl).window;
    let mut names = Vec::<u8>::new();
    loop_0 = clients.first();
    while !loop_0.is_null() {
        client_session = (*loop_0).session;
        if !client_session.is_null() {
            if w == (*(*client_session).curw).window {
                if !names.is_empty() {
                    names.push(b',');
                }
                names.extend_from_slice(
                    std::ffi::CStr::from_ptr(
                        ((*loop_0).name)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    )
                    .to_bytes(),
                );
            }
        }
        loop_0 = clients.next(loop_0);
    }
    if names.is_empty() {
        return None;
    }
    Some(CString::new(names).expect("callback bytes contain no NUL"))
}
unsafe fn format_cb_window_layout(mut ft: *mut format_tree) -> Option<CString> {
    let mut c: *mut client = (*ft).client;
    let mut w: *mut window = (*ft).w;
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if w.is_null() {
        return None;
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
    return layout_dump_owned(lcroot, flags);
}
unsafe fn format_cb_window_visible_layout(mut ft: *mut format_tree) -> Option<CString> {
    let mut c: *mut client = (*ft).client;
    let mut w: *mut window = (*ft).w;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if w.is_null() {
        return None;
    }
    if !c.is_null()
        && (*c).flags & CLIENT_CONTROL as uint64_t != 0
        && !(*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_NEWLAYOUTS != 0
    {
        flags |= LAYOUT_CUSTOM_OLD_FORMAT;
    }
    return layout_dump_owned((*w).layout_root, flags);
}
unsafe fn format_cb_start_command(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if wp.is_null() {
        return None;
    }
    return cmd_stringify_argv_cstring(&(*wp).argv);
}
unsafe fn format_cb_start_command_list(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if wp.is_null() {
        return None;
    }
    if (*wp).argv.is_empty() {
        return Some(c"".to_owned());
    }
    let mut command = Vec::<u8>::new();
    for (i, arg) in (*wp).argv.iter().enumerate() {
        let quoted = format_quote_shell_single(arg.as_c_str());
        if i != 0 {
            command.push(b' ');
        }
        command.extend_from_slice(quoted.as_bytes());
    }
    Some(CString::new(command).expect("callback bytes contain no NUL"))
}
unsafe fn format_cb_start_path(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if wp.is_null() {
        return None;
    }
    if (*wp).cwd.is_none() {
        return Some(c"".to_owned());
    }
    return (*wp).cwd.clone();
}
unsafe fn format_cb_current_command(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if wp.is_null() || (*wp).shell.is_none() {
        return None;
    }
    if let Some(cmd) = osdep_get_name_cstring((*wp).fd) {
        let value = parse_window_name_cstring(cmd.as_c_str());
        return Some(value);
    }
    let argv = cmd_stringify_argv_cstring(&(*wp).argv);
    let source = argv
        .as_ref()
        .filter(|text| !text.as_bytes().is_empty())
        .map_or((*wp).shell.as_deref().unwrap(), |text| text.as_c_str());
    let value = parse_window_name_cstring(source);
    Some(value)
}
unsafe fn format_cb_current_path(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return None;
    }
    cwd = osdep_get_cwd((*wp).fd);
    if cwd.is_null() {
        return None;
    }
    return Some(CStr::from_ptr(cwd).to_owned());
}
unsafe fn format_cb_history_bytes(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut size: size_t = 0 as size_t;
    let mut i: u_int = 0;
    let mut value = None;
    if wp.is_null() {
        return None;
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
    value = Some(
        CString::new(format!("{}", (size) as usize)).expect("formatted numbers contain no NUL"),
    );
    return value;
}
unsafe fn format_cb_history_all_bytes(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut i: u_int = 0;
    let mut lines: u_int = 0;
    let mut cells: u_int = 0 as u_int;
    let mut extended_cells: u_int = 0 as u_int;
    let mut value = None;
    if wp.is_null() {
        return None;
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
    value = Some(
        CString::new(format!(
            "{},{},{},{},{},{}",
            (lines) as u32,
            ((lines as usize).wrapping_mul(::core::mem::size_of::<grid_line>() as usize)) as usize,
            (cells) as u32,
            ((cells as usize).wrapping_mul(::core::mem::size_of::<grid_cell_entry>() as usize))
                as usize,
            (extended_cells) as u32,
            ((extended_cells as usize)
                .wrapping_mul(::core::mem::size_of::<grid_extd_entry>() as usize))
                as usize
        ))
        .expect("formatted numbers contain no NUL"),
    );
    return value;
}
unsafe fn format_cb_pane_tabs(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut i: u_int = 0;
    if wp.is_null() {
        return None;
    }
    let mut tabs = String::new();
    i = 0 as u_int;
    while i < (*(*wp).base.grid).sx {
        if crate::src::screen::screen_has_tab(&(*wp).base, i) {
            if !tabs.is_empty() {
                tabs.push(',');
            }
            write!(&mut tabs, "{i}").expect("writing to a String cannot fail");
        }
        i = i.wrapping_add(1);
    }
    if tabs.is_empty() {
        return None;
    }
    Some(CString::new(tabs).expect("callback bytes contain no NUL"))
}
unsafe fn format_cb_pane_fg(mut ft: *mut format_tree) -> Option<CString> {
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
        return None;
    }
    tty_default_colours(&raw mut gc, wp, ::core::ptr::null_mut::<u_int>());
    return Some(colour_format(gc.fg));
}
unsafe fn format_cb_pane_flags(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(CStr::from_ptr(window_pane_printable_flags((*ft).wp)).to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_floating_flag(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if window_pane_is_floating(wp) != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_modal_flag(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if wp == (*(*wp).window).modal {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_bg(mut ft: *mut format_tree) -> Option<CString> {
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
        return None;
    }
    tty_default_colours(&raw mut gc, wp, ::core::ptr::null_mut::<u_int>());
    return Some(colour_format(gc.bg));
}
unsafe fn format_cb_session_group_list(mut ft: *mut format_tree) -> Option<CString> {
    let mut s: *mut session = (*ft).s;
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if s.is_null() {
        return None;
    }
    sg = session_group_contains(s);
    if sg.is_null() {
        return None;
    }
    let mut names = Vec::<u8>::new();
    for loop_0 in crate::src::session::session_group_members(sg) {
        if !names.is_empty() {
            names.push(b',');
        }
        names.extend_from_slice((*loop_0).name.as_bytes());
    }
    if names.is_empty() {
        return None;
    }
    Some(CString::new(names).expect("callback bytes contain no NUL"))
}
unsafe fn format_cb_session_group_attached_list(mut ft: *mut format_tree) -> Option<CString> {
    let mut s: *mut session = (*ft).s;
    let mut client_session: *mut session = ::core::ptr::null_mut::<session>();
    let _session_loop: *mut session = ::core::ptr::null_mut::<session>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    if s.is_null() {
        return None;
    }
    sg = session_group_contains(s);
    if sg.is_null() {
        return None;
    }
    let mut names = Vec::<u8>::new();
    loop_0 = clients.first();
    while !loop_0.is_null() {
        client_session = (*loop_0).session;
        if !client_session.is_null() {
            for session_loop in crate::src::session::session_group_members(sg) {
                if session_loop == client_session {
                    if !names.is_empty() {
                        names.push(b',');
                    }
                    names.extend_from_slice(
                        std::ffi::CStr::from_ptr(
                            ((*loop_0).name)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        )
                        .to_bytes(),
                    );
                }
            }
        }
        loop_0 = clients.next(loop_0);
    }
    if names.is_empty() {
        return None;
    }
    Some(CString::new(names).expect("callback bytes contain no NUL"))
}
unsafe fn format_cb_pane_in_mode(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut n: u_int = 0 as u_int;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut value = None;
    if wp.is_null() {
        return None;
    }
    wme = (*wp).modes.active;
    while !wme.is_null() {
        n = n.wrapping_add(1);
        wme = crate::src::window::window_pane_mode_next(wme);
    }
    value =
        Some(CString::new(format!("{}", (n) as u32)).expect("formatted numbers contain no NUL"));
    return value;
}
unsafe fn format_cb_pane_at_top(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut status: ::core::ffi::c_int = 0;
    let mut flag: ::core::ffi::c_int = 0;
    let mut value = None;
    if wp.is_null() {
        return None;
    }
    status = window_pane_get_pane_status(wp);
    if status == PANE_STATUS_TOP {
        flag = ((*wp).yoff == 1 as ::core::ffi::c_int) as ::core::ffi::c_int;
    } else {
        flag = ((*wp).yoff == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    value =
        Some(CString::new(format!("{}", (flag) as i32)).expect("formatted numbers contain no NUL"));
    return value;
}
unsafe fn format_cb_pane_at_bottom(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut status: ::core::ffi::c_int = 0;
    let mut flag: ::core::ffi::c_int = 0;
    let mut value = None;
    if wp.is_null() {
        return None;
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
    value =
        Some(CString::new(format!("{}", (flag) as i32)).expect("formatted numbers contain no NUL"));
    return value;
}
unsafe fn format_cb_cursor_character(mut ft: *mut format_tree) -> Option<CString> {
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
    let mut value = None;
    if wp.is_null() {
        return None;
    }
    grid_view_get_cell(&*(*wp).base.grid, (*wp).base.cx, (*wp).base.cy, &mut gc);
    if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
        value = Some(
            CString::new(std::slice::from_raw_parts(
                (&raw mut gc.data.data as *mut u_char).cast::<u8>(),
                libc::strnlen(
                    (&raw mut gc.data.data as *mut u_char).cast(),
                    gc.data.size as ::core::ffi::c_int as usize,
                ),
            ))
            .expect("bounded character contains no NUL"),
        );
    }
    return value;
}
unsafe fn format_cb_cursor_colour(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if wp.is_null() || (*wp).screen.is_null() {
        return None;
    }
    if (*(*wp).screen).ccolour != -(1 as ::core::ffi::c_int) {
        return Some(colour_format((*(*wp).screen).ccolour));
    }
    return Some(colour_format((*(*wp).screen).default_ccolour));
}
unsafe fn format_cb_mouse_word(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
    }
    wp = cmd_mouse_pane(
        &raw mut (*ft).m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() {
        return None;
    }
    if cmd_mouse_at(
        wp,
        &raw mut (*ft).m,
        &raw mut x,
        &raw mut y,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        return None;
    }
    if !(*wp).modes.active.is_null() {
        if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
            return window_copy_get_word_cstring(wp, x, y);
        }
        return None;
    }
    gd = (*wp).base.grid;
    return format_grid_word_cstring(gd, x, (*gd).hsize.wrapping_add(y));
}
unsafe fn format_cb_mouse_hyperlink(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
    }
    wp = cmd_mouse_pane(
        &raw mut (*ft).m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() {
        return None;
    }
    if cmd_mouse_at(
        wp,
        &raw mut (*ft).m,
        &raw mut x,
        &raw mut y,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        return None;
    }
    if !(*wp).modes.active.is_null() {
        if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
            return window_copy_get_hyperlink_cstring(wp, x, y);
        }
        return None;
    }
    gd = (*wp).base.grid;
    return format_grid_hyperlink_cstring(gd, x, (*gd).hsize.wrapping_add(y), (*wp).screen);
}
unsafe fn format_cb_mouse_line(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
    }
    wp = cmd_mouse_pane(
        &raw mut (*ft).m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() {
        return None;
    }
    if cmd_mouse_at(
        wp,
        &raw mut (*ft).m,
        &raw mut x,
        &raw mut y,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        return None;
    }
    if !(*wp).modes.active.is_null() {
        if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
            return window_copy_get_line_cstring(wp, y);
        }
        return None;
    }
    gd = (*wp).base.grid;
    return format_grid_line_cstring(gd, (*gd).hsize.wrapping_add(y));
}
unsafe fn format_cb_mouse_status_line(mut ft: *mut format_tree) -> Option<CString> {
    let mut value = None;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
    }
    if (*ft).c.is_null() || !(*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return None;
    }
    if (*ft).m.statusat == 0 as ::core::ffi::c_int && (*ft).m.y < (*ft).m.statuslines {
        y = (*ft).m.y;
    } else if (*ft).m.statusat > 0 as ::core::ffi::c_int && (*ft).m.y >= (*ft).m.statusat as u_int {
        y = (*ft).m.y.wrapping_sub((*ft).m.statusat as u_int);
    } else {
        return None;
    }
    value =
        Some(CString::new(format!("{}", (y) as u32)).expect("formatted numbers contain no NUL"));
    return value;
}
unsafe fn format_cb_mouse_status_range(mut ft: *mut format_tree) -> Option<CString> {
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
    }
    if (*ft).c.is_null() || !(*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return None;
    }
    if (*ft).m.statusat == 0 as ::core::ffi::c_int && (*ft).m.y < (*ft).m.statuslines {
        x = (*ft).m.x;
        y = (*ft).m.y;
    } else if (*ft).m.statusat > 0 as ::core::ffi::c_int && (*ft).m.y >= (*ft).m.statusat as u_int {
        x = (*ft).m.x;
        y = (*ft).m.y.wrapping_sub((*ft).m.statusat as u_int);
    } else {
        return None;
    }
    sr = status_get_range((*ft).c, x, y);
    if sr.is_null() {
        return None;
    }
    match (*sr).type_0 as ::core::ffi::c_uint {
        0 => return None,
        1 => {
            return Some(c"left".to_owned());
        }
        2 => {
            return Some(c"right".to_owned());
        }
        3 => {
            return Some(c"pane".to_owned());
        }
        4 => {
            return Some(c"window".to_owned());
        }
        5 => {
            return Some(c"session".to_owned());
        }
        6 => {
            return Some(
                CStr::from_ptr(&raw mut (*sr).string as *mut ::core::ffi::c_char).to_owned(),
            );
        }
        7 => {
            return Some(c"control".to_owned());
        }
        _ => {}
    }
    return None;
}
unsafe fn format_cb_alternate_on(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if !(*(*ft).wp).base.saved_grid.is_null() {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_alternate_saved_x(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).wp).base.saved_cx) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_alternate_saved_y(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).wp).base.saved_cy) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_bracket_paste_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        if (*(*(*ft).wp).screen).mode & MODE_BRACKETPASTE != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_buffer_name(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).pb.is_null() {
        return Some(CStr::from_ptr(paste_buffer_name((*ft).pb)).to_owned());
    }
    return None;
}
unsafe fn format_cb_buffer_sample(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).pb.is_null() {
        return Some(paste_make_sample_cstring(&*(*ft).pb));
    }
    return None;
}
unsafe fn format_cb_buffer_full(mut ft: *mut format_tree) -> Option<CString> {
    let mut size: size_t = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !(*ft).pb.is_null() {
        s = paste_buffer_data((*ft).pb, &raw mut size);
        if !s.is_null() {
            return Some(
                CString::new(std::slice::from_raw_parts(
                    s.cast::<u8>(),
                    libc::strnlen(s, size),
                ))
                .expect("bounded buffer contains no NUL"),
            );
        }
    }
    return None;
}
unsafe fn format_cb_buffer_size(mut ft: *mut format_tree) -> Option<CString> {
    let mut size: size_t = 0;
    if !(*ft).pb.is_null() {
        paste_buffer_data((*ft).pb, &raw mut size);
        return Some(
            CString::new(format!("{}", (size) as usize)).expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_client_cell_height(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return Some(
            CString::new(format!("{}", ((*(*ft).c).tty.ypixel) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_client_cell_width(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return Some(
            CString::new(format!("{}", ((*(*ft).c).tty.xpixel) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_client_colours(mut ft: *mut format_tree) -> Option<CString> {
    let mut term: *mut tty_term = ::core::ptr::null_mut::<tty_term>();
    let mut colours: u_int = 0;
    if (*ft).c.is_null() || !(*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return None;
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
    return Some(
        CString::new(format!("{}", (colours) as u32)).expect("formatted numbers contain no NUL"),
    );
}
unsafe fn format_cb_client_control_mode(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        if (*(*ft).c).flags & CLIENT_CONTROL as uint64_t != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_client_discarded(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).c).discarded) as usize))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_client_flags(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        return Some(CStr::from_ptr(server_client_get_flags((*ft).c)).to_owned());
    }
    return None;
}
unsafe fn format_cb_client_height(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return Some(
            CString::new(format!("{}", ((*(*ft).c).tty.sy) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_client_key_table(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        return Some((*(*(*ft).c).keytable).name.clone());
    }
    return None;
}
unsafe fn format_cb_client_last_session(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null()
        && !(*(*ft).c).last_session.is_null()
        && session_alive((*(*ft).c).last_session) != 0
    {
        return Some((*(*(*ft).c).last_session).name.clone());
    }
    return None;
}
unsafe fn format_cb_client_name(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        return Some(
            CStr::from_ptr(
                ((*(*ft).c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            )
            .to_owned(),
        );
    }
    return None;
}
unsafe fn format_cb_client_pid(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        return Some(
            CString::new(format!(
                "{}",
                ((*(*ft).c).pid as ::core::ffi::c_long) as ::core::ffi::c_long
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_client_prefix(mut ft: *mut format_tree) -> Option<CString> {
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !(*ft).c.is_null() {
        name = server_client_get_key_table((*ft).c);
        if strcmp(((*(*(*ft).c).keytable).name).as_ptr().cast_mut(), name)
            == 0 as ::core::ffi::c_int
        {
            return Some(c"0".to_owned());
        }
        return Some(c"1".to_owned());
    }
    return None;
}
unsafe fn format_cb_client_readonly(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        if (*(*ft).c).flags & CLIENT_READONLY as uint64_t != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_client_session(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() && !(*(*ft).c).session.is_null() {
        return Some((*(*(*ft).c).session).name.clone());
    }
    return None;
}
unsafe fn format_cb_client_termfeatures(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        return Some(CStr::from_ptr(tty_get_features((*(*ft).c).term_features)).to_owned());
    }
    return None;
}
unsafe fn format_cb_client_termname(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        return Some(
            CStr::from_ptr(
                ((*(*ft).c).term_name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            )
            .to_owned(),
        );
    }
    return None;
}
unsafe fn format_cb_client_termtype(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        if (*(*ft).c).term_type.is_none() {
            return Some(c"".to_owned());
        }
        return Some(
            CStr::from_ptr(
                ((*(*ft).c).term_type)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            )
            .to_owned(),
        );
    }
    return None;
}
unsafe fn format_cb_client_tty(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        return Some(
            CStr::from_ptr(
                ((*(*ft).c).ttyname)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            )
            .to_owned(),
        );
    }
    return None;
}
unsafe fn format_cb_client_uid(mut ft: *mut format_tree) -> Option<CString> {
    let mut uid: uid_t = 0;
    if !(*ft).c.is_null() {
        uid = proc_get_peer_uid((*(*ft).c).peer);
        if uid != -(1 as ::core::ffi::c_int) as uid_t {
            return Some(
                CString::new(format!(
                    "{}",
                    (uid as ::core::ffi::c_long) as ::core::ffi::c_long
                ))
                .expect("formatted numbers contain no NUL"),
            );
        }
    }
    return None;
}
unsafe fn format_cb_client_user(mut ft: *mut format_tree) -> Option<CString> {
    let mut uid: uid_t = 0;
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    if !(*ft).c.is_null() {
        if !(*(*ft).c).user.is_none() {
            return Some(
                CStr::from_ptr(
                    ((*(*ft).c).user)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                )
                .to_owned(),
            );
        }
        uid = proc_get_peer_uid((*(*ft).c).peer);
        if uid != -(1 as ::core::ffi::c_int) as uid_t && {
            pw = getpwuid(uid as __uid_t);
            !pw.is_null()
        } {
            server_client_set_user(
                &mut *(*ft).c,
                Some(std::ffi::CStr::from_ptr((*pw).pw_name).to_owned()),
            );
            return Some(
                CStr::from_ptr(
                    ((*(*ft).c).user)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                )
                .to_owned(),
            );
        }
    }
    return None;
}
unsafe fn format_cb_client_utf8(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        if (*(*ft).c).flags & CLIENT_UTF8 as uint64_t != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_client_width(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).c).tty.sx) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_client_written(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).c).written) as usize))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_client_theme(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).c.is_null() {
        match (*(*ft).c).theme as ::core::ffi::c_uint {
            2 => {
                return Some(c"dark".to_owned());
            }
            1 => {
                return Some(c"light".to_owned());
            }
            0 => return None,
            _ => {}
        }
    }
    return None;
}
unsafe fn format_cb_config_files(_ft: *mut format_tree) -> Option<CString> {
    let mut paths = Vec::<u8>::new();
    for (index, path) in cfg_files().iter().enumerate() {
        if index != 0 {
            paths.push(b',');
        }
        paths.extend_from_slice(path.as_bytes());
    }
    Some(CString::new(paths).expect("callback bytes contain no NUL"))
}
unsafe fn format_cb_cursor_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_CURSOR != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_cursor_shape(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        match (*(*(*ft).wp).screen).cstyle as ::core::ffi::c_uint {
            1 => {
                return Some(c"block".to_owned());
            }
            2 => {
                return Some(c"underline".to_owned());
            }
            3 => {
                return Some(c"bar".to_owned());
            }
            _ => {
                return Some(c"default".to_owned());
            }
        }
    }
    return None;
}
unsafe fn format_cb_cursor_very_visible(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        if (*(*(*ft).wp).screen).mode & MODE_CURSOR_VERY_VISIBLE != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_cursor_x(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).wp).base.cx) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_cursor_y(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).wp).base.cy) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_cursor_blinking(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        if (*(*(*ft).wp).screen).mode & MODE_CURSOR_BLINKING != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_history_added(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!(
                "{}",
                ((*(*(*ft).wp).base.grid).scroll_added) as u32
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_history_collected(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*wp).base.grid).scroll_collected) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_history_generation(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*wp).base.grid).scroll_generation) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_history_limit(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*(*ft).wp).base.grid).hlimit) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_history_size(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*(*ft).wp).base.grid).hsize) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_insert_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_INSERT != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_keypad_cursor_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_KCURSOR != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_keypad_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_KKEYPAD != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_all_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_ALL != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_any_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & ALL_MOUSE_MODES != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_button_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_BUTTON != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_pane(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (*ft).m.valid != 0 {
        wp = cmd_mouse_pane(
            &raw mut (*ft).m,
            ::core::ptr::null_mut::<*mut session>(),
            ::core::ptr::null_mut::<*mut winlink>(),
        );
        if !wp.is_null() {
            return Some(
                CString::new(format!("%{}", ((*wp).id) as u32))
                    .expect("formatted numbers contain no NUL"),
            );
        }
        return None;
    }
    return None;
}
unsafe fn format_cb_mouse_sgr_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_SGR != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_standard_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_STANDARD != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_utf8_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_UTF8 != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_x(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
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
        return Some(
            CString::new(format!("{}", (x) as u32)).expect("formatted numbers contain no NUL"),
        );
    }
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        if (*ft).m.statusat == 0 as ::core::ffi::c_int && (*ft).m.y < (*ft).m.statuslines {
            return Some(
                CString::new(format!("{}", ((*ft).m.x) as u32))
                    .expect("formatted numbers contain no NUL"),
            );
        }
        if (*ft).m.statusat > 0 as ::core::ffi::c_int && (*ft).m.y >= (*ft).m.statusat as u_int {
            return Some(
                CString::new(format!("{}", ((*ft).m.x) as u32))
                    .expect("formatted numbers contain no NUL"),
            );
        }
    }
    return None;
}
unsafe fn format_cb_mouse_y(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
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
        return Some(
            CString::new(format!("{}", (y) as u32)).expect("formatted numbers contain no NUL"),
        );
    }
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        if (*ft).m.statusat == 0 as ::core::ffi::c_int && (*ft).m.y < (*ft).m.statuslines {
            return Some(
                CString::new(format!("{}", ((*ft).m.y) as u32))
                    .expect("formatted numbers contain no NUL"),
            );
        }
        if (*ft).m.statusat > 0 as ::core::ffi::c_int && (*ft).m.y >= (*ft).m.statusat as u_int {
            return Some(
                CString::new(format!(
                    "{}",
                    ((*ft).m.y.wrapping_sub((*ft).m.statusat as u_int)) as u32
                ))
                .expect("formatted numbers contain no NUL"),
            );
        }
    }
    return None;
}
unsafe fn format_cb_next_session_id(_ft: *mut format_tree) -> Option<CString> {
    return Some(
        CString::new(format!("${}", (next_session_id) as u32))
            .expect("formatted numbers contain no NUL"),
    );
}
unsafe fn format_cb_origin_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_ORIGIN != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_synchronized_output_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_SYNC != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_private_modes(mut ft: *mut format_tree) -> Option<CString> {
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
    let mut value = String::new();
    let mut i: u_int = 0;
    if (*ft).wp.is_null() {
        return None;
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
                if !value.is_empty() {
                    value.push(',');
                }
                write!(&mut value, "{}", table[i as usize].number)
                    .expect("writing to a String cannot fail");
            }
        }
        i = i.wrapping_add(1);
    }
    let value = std::ffi::CString::new(value).expect("mode numbers have no NUL bytes");
    return Some(value);
}
unsafe fn format_cb_pane_active(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*ft).wp == (*(*(*ft).wp).window).active {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_at_left(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).xoff == 0 as ::core::ffi::c_int {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_at_right(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).xoff + (*(*ft).wp).sx as ::core::ffi::c_int
            == (*(*(*ft).wp).window).sx as ::core::ffi::c_int
        {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_bottom(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return Some(
            CString::new(format!(
                "{}",
                ((*wp).yoff + (*wp).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as i32
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_dead(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if (*wp).fd == -(1 as ::core::ffi::c_int) && (*wp).flags & PANE_STATUSREADY != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_dead_signal(mut ft: *mut format_tree) -> Option<CString> {
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
            return Some(CStr::from_ptr(name).to_owned());
        }
        return None;
    }
    return None;
}
unsafe fn format_cb_pane_dead_status(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if (*wp).flags & PANE_STATUSREADY != 0
            && (*wp).status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        {
            return Some(
                CString::new(format!(
                    "{}",
                    (((*wp).status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int)
                        as i32
                ))
                .expect("formatted numbers contain no NUL"),
            );
        }
        return None;
    }
    return None;
}
unsafe fn format_cb_pane_dead_time(mut ft: *mut format_tree) -> Option<time_t> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if (*wp).flags & PANE_STATUSDRAWN != 0 {
            return Some(((*wp).dead_time).tv_sec as time_t);
        }
        return None;
    }
    return None;
}
unsafe fn format_cb_pane_last_output_time(mut ft: *mut format_tree) -> Option<time_t> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() && (*wp).last_output_time != 0 as time_t {
        return Some((*wp).last_output_time as __time_t as time_t);
    }
    return None;
}
unsafe fn format_cb_pane_output_generation(mut ft: *mut format_tree) -> Option<CString> {
    let mut value: ::core::ffi::c_ulonglong = 0;
    if !(*ft).wp.is_null() {
        value = (*(*ft).wp).output_generation as ::core::ffi::c_ulonglong;
        return Some(
            CString::new(format!("{}", (value) as u64)).expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_last_prompt_time(mut ft: *mut format_tree) -> Option<time_t> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() && (*wp).last_prompt_time != 0 as time_t {
        return Some((*wp).last_prompt_time as __time_t as time_t);
    }
    return None;
}
unsafe fn format_cb_pane_command_start_time(mut ft: *mut format_tree) -> Option<time_t> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() && (*wp).cmd_start_time != 0 as time_t {
        return Some((*wp).cmd_start_time as __time_t as time_t);
    }
    return None;
}
unsafe fn format_cb_pane_command_end_time(mut ft: *mut format_tree) -> Option<time_t> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() && (*wp).cmd_end_time != 0 as time_t {
        return Some((*wp).cmd_end_time as __time_t as time_t);
    }
    return None;
}
unsafe fn format_cb_pane_command_running(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return Some(
            CString::new(format!(
                "{}",
                (((*wp).flags & PANE_CMDRUNNING != 0) as ::core::ffi::c_int) as i32
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_command_duration(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut end: time_t = 0;
    if wp.is_null() || (*wp).cmd_start_time == 0 as time_t {
        return None;
    }
    if (*wp).flags & PANE_CMDRUNNING != 0 {
        end = time(::core::ptr::null_mut::<time_t>());
    } else {
        end = (*wp).cmd_end_time;
    }
    if end < (*wp).cmd_start_time {
        end = (*wp).cmd_start_time;
    }
    return Some(
        CString::new(format!(
            "{}",
            ((end - (*wp).cmd_start_time) as ::core::ffi::c_longlong) as i64
        ))
        .expect("formatted numbers contain no NUL"),
    );
}
unsafe fn format_cb_pane_command_status(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() && (*wp).cmd_status != -(1 as ::core::ffi::c_int) {
        return Some(
            CString::new(format!("{}", ((*wp).cmd_status) as i32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_format(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).type_0 as ::core::ffi::c_uint
        == FORMAT_TYPE_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return Some(c"1".to_owned());
    }
    return Some(c"0".to_owned());
}
unsafe fn format_cb_pane_height(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).wp).sy) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_id(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("%{}", ((*(*ft).wp).id) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_index(mut ft: *mut format_tree) -> Option<CString> {
    let mut idx: u_int = 0;
    if !(*ft).wp.is_null() && window_pane_index((*ft).wp, &raw mut idx) == 0 as ::core::ffi::c_int {
        return Some(
            CString::new(format!("{}", (idx) as u32)).expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_input_off(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).flags & PANE_INPUTOFF != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_unseen_changes(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).flags & PANE_UNSEENCHANGES != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_key_mode(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        match (*(*(*ft).wp).screen).mode & EXTENDED_KEY_MODES {
            MODE_KEYS_EXTENDED => {
                return Some(c"Ext 1".to_owned());
            }
            MODE_KEYS_EXTENDED_2 => {
                return Some(c"Ext 2".to_owned());
            }
            _ => {
                return Some(c"VT10x".to_owned());
            }
        }
    }
    return None;
}
unsafe fn format_cb_pane_last(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*ft).wp == window_pane_stack_first((*(*ft).wp).window) {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_left(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).wp).xoff) as i32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_marked(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if server_check_marked() != 0 && marked_pane.wp == (*ft).wp {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_marked_set(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if server_check_marked() != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_mode(mut ft: *mut format_tree) -> Option<CString> {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if !(*ft).wp.is_null() {
        wme = (*(*ft).wp).modes.active;
        if !wme.is_null() {
            return Some(CStr::from_ptr((*(*wme).mode).name.as_ptr()).to_owned());
        }
        return None;
    }
    return None;
}
unsafe fn format_cb_pane_path(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            (*(*ft).wp)
                .base
                .path
                .clone()
                .unwrap_or_else(|| c"".to_owned()),
        );
    }
    return None;
}
unsafe fn format_cb_pane_pid(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() && (*(*ft).wp).fd != -(1 as ::core::ffi::c_int) {
        return Some(
            CString::new(format!(
                "{}",
                ((*(*ft).wp).pid as ::core::ffi::c_long) as ::core::ffi::c_long
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_pipe(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).pipe_fd != -(1 as ::core::ffi::c_int) {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_pipe_pid(mut ft: *mut format_tree) -> Option<CString> {
    let mut value = None;
    if !(*ft).wp.is_null() && (*(*ft).wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        value = Some(
            CString::new(format!(
                "{}",
                ((*(*ft).wp).pipe_pid as ::core::ffi::c_long) as ::core::ffi::c_long
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return value;
}
unsafe fn format_cb_pane_pb_progress(mut ft: *mut format_tree) -> Option<CString> {
    let mut value = None;
    if !(*ft).wp.is_null() {
        value = Some(
            CString::new(format!(
                "{}",
                ((*(*ft).wp).base.progress_bar.progress) as i32
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return value;
}
unsafe fn format_cb_pane_pb_state(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        match (*(*ft).wp).base.progress_bar.state as ::core::ffi::c_uint {
            0 => {
                return Some(c"hidden".to_owned());
            }
            1 => {
                return Some(c"normal".to_owned());
            }
            2 => {
                return Some(c"error".to_owned());
            }
            3 => {
                return Some(c"indeterminate".to_owned());
            }
            4 => {
                return Some(c"paused".to_owned());
            }
            _ => {}
        }
    }
    return None;
}
unsafe fn format_cb_pane_right(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return Some(
            CString::new(format!(
                "{}",
                ((*wp).xoff + (*wp).sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as i32
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_search_string(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).searchstr.is_none() {
            return Some(c"".to_owned());
        }
        return (*(*ft).wp).searchstr.clone();
    }
    return None;
}
unsafe fn format_cb_pane_synchronized(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if options_get_number(
            (*(*ft).wp).options,
            b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_title(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some((*(*ft).wp).base.title.clone());
    }
    return None;
}
unsafe fn format_cb_pane_top(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).wp).yoff) as i32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_tty(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CStr::from_ptr(&raw mut (*(*ft).wp).tty as *mut ::core::ffi::c_char).to_owned(),
        );
    }
    return None;
}
unsafe fn format_cb_pane_unzoomed_height(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut root: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut status: ::core::ffi::c_int = 0;
    let mut floating: ::core::ffi::c_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return None;
    }
    w = (*wp).window as *mut window;
    lc = (*wp).saved_layout_cell;
    if lc.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    if lc.is_null() {
        return None;
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
    return Some(
        CString::new(format!("{}", (sy) as u32)).expect("formatted numbers contain no NUL"),
    );
}
unsafe fn format_cb_pane_unzoomed_width(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut saved: ::core::ffi::c_int = 0;
    let mut sb_w: ::core::ffi::c_int = 0;
    let mut sb_pad: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    if wp.is_null() {
        return None;
    }
    lc = (*wp).saved_layout_cell;
    saved = (lc != NULL_0 as *mut layout_cell) as ::core::ffi::c_int;
    if lc.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    if lc.is_null() {
        return None;
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
    return Some(
        CString::new(format!("{}", (sx) as u32)).expect("formatted numbers contain no NUL"),
    );
}
unsafe fn format_cb_pane_width(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).wp).sx) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_x(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).wp).xoff) as i32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_y(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).wp).yoff) as i32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_z(mut ft: *mut format_tree) -> Option<CString> {
    let mut idx: u_int = 0;
    if !(*ft).wp.is_null() && window_pane_zindex((*ft).wp, &raw mut idx) == 0 as ::core::ffi::c_int
    {
        return Some(
            CString::new(format!("{}", (idx) as u32)).expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_zoomed_flag(mut ft: *mut format_tree) -> Option<CString> {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if (*wp).flags & PANE_ZOOMED != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_scroll_region_lower(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).wp).base.rlower) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_scroll_region_upper(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).wp).base.rupper) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_server_sessions(_ft: *mut format_tree) -> Option<CString> {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut n: u_int = 0 as u_int;
    s = sessions_minmax(&*std::ptr::addr_of!(sessions));
    while !s.is_null() {
        n = n.wrapping_add(1);
        s = sessions_next(&*s);
    }
    return Some(
        CString::new(format!("{}", (n) as u32)).expect("formatted numbers contain no NUL"),
    );
}
unsafe fn format_cb_session_active(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).s.is_null() || (*ft).c.is_null() {
        return None;
    }
    if (*(*ft).c).session == (*ft).s {
        return Some(c"1".to_owned());
    }
    return Some(c"0".to_owned());
}
unsafe fn format_cb_session_activity_flag(mut ft: *mut format_tree) -> Option<CString> {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*ft).s.is_null() {
        wl = winlinks_minmax(&(*(*ft).s).windows, RB_NEGINF);
        if !wl.is_null() {
            if (*(*ft).wl).flags & WINLINK_ACTIVITY != 0 {
                return Some(c"1".to_owned());
            }
            return Some(c"0".to_owned());
        }
    }
    return None;
}
unsafe fn format_cb_session_bell_flag(mut ft: *mut format_tree) -> Option<CString> {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*ft).s.is_null() {
        wl = winlinks_minmax(&(*(*ft).s).windows, RB_NEGINF);
        if !wl.is_null() {
            if (*wl).flags & WINLINK_BELL != 0 {
                return Some(c"1".to_owned());
            }
            return Some(c"0".to_owned());
        }
    }
    return None;
}
unsafe fn format_cb_session_silence_flag(mut ft: *mut format_tree) -> Option<CString> {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*ft).s.is_null() {
        wl = winlinks_minmax(&(*(*ft).s).windows, RB_NEGINF);
        if !wl.is_null() {
            if (*(*ft).wl).flags & WINLINK_SILENCE != 0 {
                return Some(c"1".to_owned());
            }
            return Some(c"0".to_owned());
        }
    }
    return None;
}
unsafe fn format_cb_session_attached(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).s.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).s).attached) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_session_format(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).type_0 as ::core::ffi::c_uint
        == FORMAT_TYPE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return Some(c"1".to_owned());
    }
    return Some(c"0".to_owned());
}
unsafe fn format_cb_session_group(mut ft: *mut format_tree) -> Option<CString> {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if !(*ft).s.is_null() && {
        sg = session_group_contains((*ft).s);
        !sg.is_null()
    } {
        return Some((*sg).name.clone());
    }
    return None;
}
unsafe fn format_cb_session_group_attached(mut ft: *mut format_tree) -> Option<CString> {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if !(*ft).s.is_null() && {
        sg = session_group_contains((*ft).s);
        !sg.is_null()
    } {
        return Some(
            CString::new(format!("{}", (session_group_attached_count(sg)) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_session_group_many_attached(mut ft: *mut format_tree) -> Option<CString> {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if !(*ft).s.is_null() && {
        sg = session_group_contains((*ft).s);
        !sg.is_null()
    } {
        if session_group_attached_count(sg) > 1 as u_int {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_session_group_size(mut ft: *mut format_tree) -> Option<CString> {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if !(*ft).s.is_null() && {
        sg = session_group_contains((*ft).s);
        !sg.is_null()
    } {
        return Some(
            CString::new(format!("{}", (session_group_count(sg)) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_session_grouped(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).s.is_null() {
        if !session_group_contains((*ft).s).is_null() {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_session_id(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).s.is_null() {
        return Some(
            CString::new(format!("${}", ((*(*ft).s).id) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_session_many_attached(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).s.is_null() {
        if (*(*ft).s).attached > 1 as u_int {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_session_marked(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).s.is_null() {
        if server_check_marked() != 0 && marked_pane.s == (*ft).s {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_session_name(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).s.is_null() {
        return Some((*(*ft).s).name.clone());
    }
    return None;
}
unsafe fn format_cb_session_path(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).s.is_null() {
        return Some(
            CStr::from_ptr(
                ((*(*ft).s).cwd)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            )
            .to_owned(),
        );
    }
    return None;
}
unsafe fn format_cb_session_windows(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).s.is_null() {
        return Some(
            CString::new(format!(
                "{}",
                (winlink_count(&raw mut (*(*ft).s).windows)) as u32
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_socket_path(_ft: *mut format_tree) -> Option<CString> {
    return Some(CStr::from_ptr(socket_path).to_owned());
}
unsafe fn format_cb_version(_ft: *mut format_tree) -> Option<CString> {
    return Some(CStr::from_ptr(getversion()).to_owned());
}
unsafe fn format_cb_sixel_support(_ft: *mut format_tree) -> Option<CString> {
    return Some(c"0".to_owned());
}
unsafe fn format_cb_active_window_index(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).s.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*(*ft).s).curw).idx) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_last_window_index(mut ft: *mut format_tree) -> Option<CString> {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*ft).s.is_null() {
        wl = winlinks_minmax(&(*(*ft).s).windows, RB_INF);
        return Some(
            CString::new(format!("{}", ((*wl).idx) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_window_active(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wl.is_null() {
        if (*ft).wl == (*(*(*ft).wl).session).curw {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_activity_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wl.is_null() {
        if (*(*ft).wl).flags & WINLINK_ACTIVITY != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_bell_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wl.is_null() {
        if (*(*ft).wl).flags & WINLINK_BELL != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_bigger(mut ft: *mut format_tree) -> Option<CString> {
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
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_cell_height(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).w.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).w).ypixel) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_window_cell_width(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).w.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).w).xpixel) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_window_end_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wl.is_null() {
        if (*ft).wl == winlinks_minmax(&(*(*(*ft).wl).session).windows, RB_INF) {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_flags(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wl.is_null() {
        return Some(
            CStr::from_ptr(window_printable_flags((*ft).wl, 1 as ::core::ffi::c_int)).to_owned(),
        );
    }
    return None;
}
unsafe fn format_cb_window_format(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).type_0 as ::core::ffi::c_uint
        == FORMAT_TYPE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return Some(c"1".to_owned());
    }
    return Some(c"0".to_owned());
}
unsafe fn format_cb_window_height(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).w.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).w).sy) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_window_manual_height(mut ft: *mut format_tree) -> Option<CString> {
    let mut w: *mut window = (*ft).w;
    if w.is_null() {
        return None;
    }
    if options_get_number(
        (*w).options,
        b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
    ) != WINDOW_SIZE_MANUAL as ::core::ffi::c_longlong
    {
        return Some(c"".to_owned());
    }
    return Some(
        CString::new(format!("{}", ((*w).manual_sy) as u32))
            .expect("formatted numbers contain no NUL"),
    );
}
unsafe fn format_cb_window_id(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).w.is_null() {
        return Some(
            CString::new(format!("@{}", ((*(*ft).w).id) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_window_index(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wl.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).wl).idx) as i32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_window_last_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wl.is_null() {
        if (*ft).wl == crate::src::window::winlink_stack_first(&(*(*(*ft).wl).session).lastw) {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_linked(mut ft: *mut format_tree) -> Option<CString> {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !(*ft).wl.is_null() {
        s = sessions_minmax(&*std::ptr::addr_of!(sessions));
        while !s.is_null() {
            wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
            while !wl.is_null() {
                if (*wl).window == (*(*ft).wl).window {
                    if found != 0 {
                        return Some(c"1".to_owned());
                    }
                    found = 1 as ::core::ffi::c_int;
                }
                wl = winlinks_next(&*wl);
            }
            s = sessions_next(&*s);
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_linked_sessions(mut ft: *mut format_tree) -> Option<CString> {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut n: u_int = 0 as u_int;
    if (*ft).wl.is_null() {
        return None;
    }
    w = (*(*ft).wl).window;
    sg = session_groups_minmax(&*std::ptr::addr_of!(session_groups));
    while !sg.is_null() {
        s = crate::src::session::session_group_members(sg)
            .first()
            .copied()
            .unwrap_or(std::ptr::null_mut());
        if !s.is_null() && !winlink_find_by_window(&raw mut (*s).windows, w).is_null() {
            n = n.wrapping_add(1);
        }
        sg = session_groups_next(&*sg);
    }
    s = sessions_minmax(&*std::ptr::addr_of!(sessions));
    while !s.is_null() {
        if session_group_contains(s).is_null() {
            if !winlink_find_by_window(&raw mut (*s).windows, w).is_null() {
                n = n.wrapping_add(1);
            }
        }
        s = sessions_next(&*s);
    }
    return Some(
        CString::new(format!("{}", (n) as u32)).expect("formatted numbers contain no NUL"),
    );
}
unsafe fn format_cb_window_marked_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wl.is_null() {
        if server_check_marked() != 0 && marked_pane.wl == (*ft).wl {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_modal_pane(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).w.is_null() && !(*(*ft).w).modal.is_null() {
        return Some(
            CString::new(format!("%{}", ((*(*(*ft).w).modal).id) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_window_name(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).w.is_null() {
        return Some((*(*ft).w).name.clone());
    }
    return None;
}
unsafe fn format_cb_window_offset_x(mut ft: *mut format_tree) -> Option<CString> {
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
            return Some(
                CString::new(format!("{}", (ox) as u32)).expect("formatted numbers contain no NUL"),
            );
        }
        return None;
    }
    return None;
}
unsafe fn format_cb_window_offset_y(mut ft: *mut format_tree) -> Option<CString> {
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
            return Some(
                CString::new(format!("{}", (oy) as u32)).expect("formatted numbers contain no NUL"),
            );
        }
        return None;
    }
    return None;
}
unsafe fn format_cb_window_panes(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).w.is_null() {
        return Some(
            CString::new(format!(
                "{}",
                (window_count_panes((*ft).w, 1 as ::core::ffi::c_int)) as u32
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_window_raw_flags(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wl.is_null() {
        return Some(
            CStr::from_ptr(window_printable_flags((*ft).wl, 0 as ::core::ffi::c_int)).to_owned(),
        );
    }
    return None;
}
unsafe fn format_cb_window_silence_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wl.is_null() {
        if (*(*ft).wl).flags & WINLINK_SILENCE != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_start_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wl.is_null() {
        if (*ft).wl == winlinks_minmax(&(*(*(*ft).wl).session).windows, RB_NEGINF) {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_width(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).w.is_null() {
        return Some(
            CString::new(format!("{}", ((*(*ft).w).sx) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_window_manual_width(mut ft: *mut format_tree) -> Option<CString> {
    let mut w: *mut window = (*ft).w;
    if w.is_null() {
        return None;
    }
    if options_get_number(
        (*w).options,
        b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
    ) != WINDOW_SIZE_MANUAL as ::core::ffi::c_longlong
    {
        return Some(c"".to_owned());
    }
    return Some(
        CString::new(format!("{}", ((*w).manual_sx) as u32))
            .expect("formatted numbers contain no NUL"),
    );
}
unsafe fn format_cb_window_zoomed_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).w.is_null() {
        if (*(*ft).w).flags & WINDOW_ZOOMED != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_wrap_flag(mut ft: *mut format_tree) -> Option<CString> {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_WRAP != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_buffer_created(mut ft: *mut format_tree) -> Option<time_t> {
    if !(*ft).pb.is_null() {
        return Some(paste_buffer_created((*ft).pb) as __time_t as time_t);
    }
    return None;
}
unsafe fn format_cb_client_activity(mut ft: *mut format_tree) -> Option<time_t> {
    if !(*ft).c.is_null() {
        return Some(((*(*ft).c).activity_time).tv_sec as time_t);
    }
    return None;
}
unsafe fn format_cb_client_created(mut ft: *mut format_tree) -> Option<time_t> {
    if !(*ft).c.is_null() {
        return Some(((*(*ft).c).creation_time).tv_sec as time_t);
    }
    return None;
}
unsafe fn format_cb_session_activity(mut ft: *mut format_tree) -> Option<time_t> {
    if !(*ft).s.is_null() {
        return Some(((*(*ft).s).activity_time).tv_sec as time_t);
    }
    return None;
}
unsafe fn format_cb_session_created(mut ft: *mut format_tree) -> Option<time_t> {
    if !(*ft).s.is_null() {
        return Some(((*(*ft).s).creation_time).tv_sec as time_t);
    }
    return None;
}
unsafe fn format_cb_session_last_attached(mut ft: *mut format_tree) -> Option<time_t> {
    if !(*ft).s.is_null() {
        return Some(((*(*ft).s).last_attached_time).tv_sec as time_t);
    }
    return None;
}
unsafe fn format_cb_start_time(_ft: *mut format_tree) -> Option<time_t> {
    return Some((start_time).tv_sec as time_t);
}
unsafe fn format_cb_window_activity(mut ft: *mut format_tree) -> Option<time_t> {
    if !(*ft).w.is_null() {
        return Some(((*(*ft).w).activity_time).tv_sec as time_t);
    }
    return None;
}
unsafe fn format_cb_buffer_mode_format(_ft: *mut format_tree) -> Option<CString> {
    return Some(CStr::from_ptr(window_buffer_mode.default_format).to_owned());
}
unsafe fn format_cb_client_mode_format(_ft: *mut format_tree) -> Option<CString> {
    return Some(CStr::from_ptr(window_client_mode.default_format).to_owned());
}
unsafe fn format_cb_tree_mode_format(_ft: *mut format_tree) -> Option<CString> {
    return Some(CStr::from_ptr(window_tree_mode.default_format).to_owned());
}
unsafe fn format_cb_uid(_ft: *mut format_tree) -> Option<CString> {
    return Some(
        CString::new(format!(
            "{}",
            (getuid() as ::core::ffi::c_long) as ::core::ffi::c_long
        ))
        .expect("formatted numbers contain no NUL"),
    );
}
unsafe fn format_cb_user(_ft: *mut format_tree) -> Option<CString> {
    // Preserve retry-on-failure and process-lifetime caching without a leaked
    // libc allocation. No borrow survives the libc lookup or a callback.
    static CACHED: std::sync::OnceLock<CString> = std::sync::OnceLock::new();
    if let Some(name) = CACHED.get() {
        return Some(name.clone());
    }
    let pw = getpwuid(getuid());
    if pw.is_null() {
        return None;
    }
    let name = CStr::from_ptr((*pw).pw_name).to_owned();
    let _ = CACHED.set(name);
    CACHED.get().cloned()
}
pub(super) enum FormatValue {
    String(CString),
    Time(time_t),
}
pub(super) enum FormatCallback {
    String(unsafe fn(*mut format_tree) -> Option<CString>),
    Time(unsafe fn(*mut format_tree) -> Option<time_t>),
}
pub(super) struct FormatTableEntry {
    pub key: &'static CStr,
    callback: FormatCallback,
}
impl FormatTableEntry {
    pub unsafe fn get(&self, ft: *mut format_tree) -> Option<FormatValue> {
        match self.callback {
            FormatCallback::String(cb) => cb(ft).map(FormatValue::String),
            FormatCallback::Time(cb) => cb(ft).map(FormatValue::Time),
        }
    }
}
pub(super) static FORMAT_TABLE: [FormatTableEntry; 214] = [
    FormatTableEntry {
        key: c"active_window_index",
        callback: FormatCallback::String(format_cb_active_window_index),
    },
    FormatTableEntry {
        key: c"alternate_on",
        callback: FormatCallback::String(format_cb_alternate_on),
    },
    FormatTableEntry {
        key: c"alternate_saved_x",
        callback: FormatCallback::String(format_cb_alternate_saved_x),
    },
    FormatTableEntry {
        key: c"alternate_saved_y",
        callback: FormatCallback::String(format_cb_alternate_saved_y),
    },
    FormatTableEntry {
        key: c"bracket_paste_flag",
        callback: FormatCallback::String(format_cb_bracket_paste_flag),
    },
    FormatTableEntry {
        key: c"buffer_created",
        callback: FormatCallback::Time(format_cb_buffer_created),
    },
    FormatTableEntry {
        key: c"buffer_full",
        callback: FormatCallback::String(format_cb_buffer_full),
    },
    FormatTableEntry {
        key: c"buffer_mode_format",
        callback: FormatCallback::String(format_cb_buffer_mode_format),
    },
    FormatTableEntry {
        key: c"buffer_name",
        callback: FormatCallback::String(format_cb_buffer_name),
    },
    FormatTableEntry {
        key: c"buffer_sample",
        callback: FormatCallback::String(format_cb_buffer_sample),
    },
    FormatTableEntry {
        key: c"buffer_size",
        callback: FormatCallback::String(format_cb_buffer_size),
    },
    FormatTableEntry {
        key: c"client_activity",
        callback: FormatCallback::Time(format_cb_client_activity),
    },
    FormatTableEntry {
        key: c"client_cell_height",
        callback: FormatCallback::String(format_cb_client_cell_height),
    },
    FormatTableEntry {
        key: c"client_cell_width",
        callback: FormatCallback::String(format_cb_client_cell_width),
    },
    FormatTableEntry {
        key: c"client_colours",
        callback: FormatCallback::String(format_cb_client_colours),
    },
    FormatTableEntry {
        key: c"client_control_mode",
        callback: FormatCallback::String(format_cb_client_control_mode),
    },
    FormatTableEntry {
        key: c"client_created",
        callback: FormatCallback::Time(format_cb_client_created),
    },
    FormatTableEntry {
        key: c"client_discarded",
        callback: FormatCallback::String(format_cb_client_discarded),
    },
    FormatTableEntry {
        key: c"client_flags",
        callback: FormatCallback::String(format_cb_client_flags),
    },
    FormatTableEntry {
        key: c"client_height",
        callback: FormatCallback::String(format_cb_client_height),
    },
    FormatTableEntry {
        key: c"client_key_table",
        callback: FormatCallback::String(format_cb_client_key_table),
    },
    FormatTableEntry {
        key: c"client_last_session",
        callback: FormatCallback::String(format_cb_client_last_session),
    },
    FormatTableEntry {
        key: c"client_mode_format",
        callback: FormatCallback::String(format_cb_client_mode_format),
    },
    FormatTableEntry {
        key: c"client_name",
        callback: FormatCallback::String(format_cb_client_name),
    },
    FormatTableEntry {
        key: c"client_pid",
        callback: FormatCallback::String(format_cb_client_pid),
    },
    FormatTableEntry {
        key: c"client_prefix",
        callback: FormatCallback::String(format_cb_client_prefix),
    },
    FormatTableEntry {
        key: c"client_readonly",
        callback: FormatCallback::String(format_cb_client_readonly),
    },
    FormatTableEntry {
        key: c"client_session",
        callback: FormatCallback::String(format_cb_client_session),
    },
    FormatTableEntry {
        key: c"client_termfeatures",
        callback: FormatCallback::String(format_cb_client_termfeatures),
    },
    FormatTableEntry {
        key: c"client_termname",
        callback: FormatCallback::String(format_cb_client_termname),
    },
    FormatTableEntry {
        key: c"client_termtype",
        callback: FormatCallback::String(format_cb_client_termtype),
    },
    FormatTableEntry {
        key: c"client_theme",
        callback: FormatCallback::String(format_cb_client_theme),
    },
    FormatTableEntry {
        key: c"client_tty",
        callback: FormatCallback::String(format_cb_client_tty),
    },
    FormatTableEntry {
        key: c"client_uid",
        callback: FormatCallback::String(format_cb_client_uid),
    },
    FormatTableEntry {
        key: c"client_user",
        callback: FormatCallback::String(format_cb_client_user),
    },
    FormatTableEntry {
        key: c"client_utf8",
        callback: FormatCallback::String(format_cb_client_utf8),
    },
    FormatTableEntry {
        key: c"client_width",
        callback: FormatCallback::String(format_cb_client_width),
    },
    FormatTableEntry {
        key: c"client_written",
        callback: FormatCallback::String(format_cb_client_written),
    },
    FormatTableEntry {
        key: c"config_files",
        callback: FormatCallback::String(format_cb_config_files),
    },
    FormatTableEntry {
        key: c"cursor_blinking",
        callback: FormatCallback::String(format_cb_cursor_blinking),
    },
    FormatTableEntry {
        key: c"cursor_character",
        callback: FormatCallback::String(format_cb_cursor_character),
    },
    FormatTableEntry {
        key: c"cursor_colour",
        callback: FormatCallback::String(format_cb_cursor_colour),
    },
    FormatTableEntry {
        key: c"cursor_flag",
        callback: FormatCallback::String(format_cb_cursor_flag),
    },
    FormatTableEntry {
        key: c"cursor_shape",
        callback: FormatCallback::String(format_cb_cursor_shape),
    },
    FormatTableEntry {
        key: c"cursor_very_visible",
        callback: FormatCallback::String(format_cb_cursor_very_visible),
    },
    FormatTableEntry {
        key: c"cursor_x",
        callback: FormatCallback::String(format_cb_cursor_x),
    },
    FormatTableEntry {
        key: c"cursor_y",
        callback: FormatCallback::String(format_cb_cursor_y),
    },
    FormatTableEntry {
        key: c"history_added",
        callback: FormatCallback::String(format_cb_history_added),
    },
    FormatTableEntry {
        key: c"history_all_bytes",
        callback: FormatCallback::String(format_cb_history_all_bytes),
    },
    FormatTableEntry {
        key: c"history_bytes",
        callback: FormatCallback::String(format_cb_history_bytes),
    },
    FormatTableEntry {
        key: c"history_collected",
        callback: FormatCallback::String(format_cb_history_collected),
    },
    FormatTableEntry {
        key: c"history_generation",
        callback: FormatCallback::String(format_cb_history_generation),
    },
    FormatTableEntry {
        key: c"history_limit",
        callback: FormatCallback::String(format_cb_history_limit),
    },
    FormatTableEntry {
        key: c"history_size",
        callback: FormatCallback::String(format_cb_history_size),
    },
    FormatTableEntry {
        key: c"host",
        callback: FormatCallback::String(format_cb_host),
    },
    FormatTableEntry {
        key: c"host_short",
        callback: FormatCallback::String(format_cb_host_short),
    },
    FormatTableEntry {
        key: c"insert_flag",
        callback: FormatCallback::String(format_cb_insert_flag),
    },
    FormatTableEntry {
        key: c"keypad_cursor_flag",
        callback: FormatCallback::String(format_cb_keypad_cursor_flag),
    },
    FormatTableEntry {
        key: c"keypad_flag",
        callback: FormatCallback::String(format_cb_keypad_flag),
    },
    FormatTableEntry {
        key: c"last_window_index",
        callback: FormatCallback::String(format_cb_last_window_index),
    },
    FormatTableEntry {
        key: c"mouse_all_flag",
        callback: FormatCallback::String(format_cb_mouse_all_flag),
    },
    FormatTableEntry {
        key: c"mouse_any_flag",
        callback: FormatCallback::String(format_cb_mouse_any_flag),
    },
    FormatTableEntry {
        key: c"mouse_button_flag",
        callback: FormatCallback::String(format_cb_mouse_button_flag),
    },
    FormatTableEntry {
        key: c"mouse_hyperlink",
        callback: FormatCallback::String(format_cb_mouse_hyperlink),
    },
    FormatTableEntry {
        key: c"mouse_line",
        callback: FormatCallback::String(format_cb_mouse_line),
    },
    FormatTableEntry {
        key: c"mouse_pane",
        callback: FormatCallback::String(format_cb_mouse_pane),
    },
    FormatTableEntry {
        key: c"mouse_sgr_flag",
        callback: FormatCallback::String(format_cb_mouse_sgr_flag),
    },
    FormatTableEntry {
        key: c"mouse_standard_flag",
        callback: FormatCallback::String(format_cb_mouse_standard_flag),
    },
    FormatTableEntry {
        key: c"mouse_status_line",
        callback: FormatCallback::String(format_cb_mouse_status_line),
    },
    FormatTableEntry {
        key: c"mouse_status_range",
        callback: FormatCallback::String(format_cb_mouse_status_range),
    },
    FormatTableEntry {
        key: c"mouse_utf8_flag",
        callback: FormatCallback::String(format_cb_mouse_utf8_flag),
    },
    FormatTableEntry {
        key: c"mouse_word",
        callback: FormatCallback::String(format_cb_mouse_word),
    },
    FormatTableEntry {
        key: c"mouse_x",
        callback: FormatCallback::String(format_cb_mouse_x),
    },
    FormatTableEntry {
        key: c"mouse_y",
        callback: FormatCallback::String(format_cb_mouse_y),
    },
    FormatTableEntry {
        key: c"next_session_id",
        callback: FormatCallback::String(format_cb_next_session_id),
    },
    FormatTableEntry {
        key: c"origin_flag",
        callback: FormatCallback::String(format_cb_origin_flag),
    },
    FormatTableEntry {
        key: c"pane_active",
        callback: FormatCallback::String(format_cb_pane_active),
    },
    FormatTableEntry {
        key: c"pane_at_bottom",
        callback: FormatCallback::String(format_cb_pane_at_bottom),
    },
    FormatTableEntry {
        key: c"pane_at_left",
        callback: FormatCallback::String(format_cb_pane_at_left),
    },
    FormatTableEntry {
        key: c"pane_at_right",
        callback: FormatCallback::String(format_cb_pane_at_right),
    },
    FormatTableEntry {
        key: c"pane_at_top",
        callback: FormatCallback::String(format_cb_pane_at_top),
    },
    FormatTableEntry {
        key: c"pane_bg",
        callback: FormatCallback::String(format_cb_pane_bg),
    },
    FormatTableEntry {
        key: c"pane_bottom",
        callback: FormatCallback::String(format_cb_pane_bottom),
    },
    FormatTableEntry {
        key: c"pane_command_duration",
        callback: FormatCallback::String(format_cb_pane_command_duration),
    },
    FormatTableEntry {
        key: c"pane_command_end_time",
        callback: FormatCallback::Time(format_cb_pane_command_end_time),
    },
    FormatTableEntry {
        key: c"pane_command_running",
        callback: FormatCallback::String(format_cb_pane_command_running),
    },
    FormatTableEntry {
        key: c"pane_command_start_time",
        callback: FormatCallback::Time(format_cb_pane_command_start_time),
    },
    FormatTableEntry {
        key: c"pane_command_status",
        callback: FormatCallback::String(format_cb_pane_command_status),
    },
    FormatTableEntry {
        key: c"pane_current_command",
        callback: FormatCallback::String(format_cb_current_command),
    },
    FormatTableEntry {
        key: c"pane_current_path",
        callback: FormatCallback::String(format_cb_current_path),
    },
    FormatTableEntry {
        key: c"pane_dead",
        callback: FormatCallback::String(format_cb_pane_dead),
    },
    FormatTableEntry {
        key: c"pane_dead_signal",
        callback: FormatCallback::String(format_cb_pane_dead_signal),
    },
    FormatTableEntry {
        key: c"pane_dead_status",
        callback: FormatCallback::String(format_cb_pane_dead_status),
    },
    FormatTableEntry {
        key: c"pane_dead_time",
        callback: FormatCallback::Time(format_cb_pane_dead_time),
    },
    FormatTableEntry {
        key: c"pane_fg",
        callback: FormatCallback::String(format_cb_pane_fg),
    },
    FormatTableEntry {
        key: c"pane_flags",
        callback: FormatCallback::String(format_cb_pane_flags),
    },
    FormatTableEntry {
        key: c"pane_floating_flag",
        callback: FormatCallback::String(format_cb_pane_floating_flag),
    },
    FormatTableEntry {
        key: c"pane_format",
        callback: FormatCallback::String(format_cb_pane_format),
    },
    FormatTableEntry {
        key: c"pane_height",
        callback: FormatCallback::String(format_cb_pane_height),
    },
    FormatTableEntry {
        key: c"pane_id",
        callback: FormatCallback::String(format_cb_pane_id),
    },
    FormatTableEntry {
        key: c"pane_in_mode",
        callback: FormatCallback::String(format_cb_pane_in_mode),
    },
    FormatTableEntry {
        key: c"pane_index",
        callback: FormatCallback::String(format_cb_pane_index),
    },
    FormatTableEntry {
        key: c"pane_input_off",
        callback: FormatCallback::String(format_cb_pane_input_off),
    },
    FormatTableEntry {
        key: c"pane_key_mode",
        callback: FormatCallback::String(format_cb_pane_key_mode),
    },
    FormatTableEntry {
        key: c"pane_last",
        callback: FormatCallback::String(format_cb_pane_last),
    },
    FormatTableEntry {
        key: c"pane_last_output_time",
        callback: FormatCallback::Time(format_cb_pane_last_output_time),
    },
    FormatTableEntry {
        key: c"pane_last_prompt_time",
        callback: FormatCallback::Time(format_cb_pane_last_prompt_time),
    },
    FormatTableEntry {
        key: c"pane_left",
        callback: FormatCallback::String(format_cb_pane_left),
    },
    FormatTableEntry {
        key: c"pane_marked",
        callback: FormatCallback::String(format_cb_pane_marked),
    },
    FormatTableEntry {
        key: c"pane_marked_set",
        callback: FormatCallback::String(format_cb_pane_marked_set),
    },
    FormatTableEntry {
        key: c"pane_modal_flag",
        callback: FormatCallback::String(format_cb_pane_modal_flag),
    },
    FormatTableEntry {
        key: c"pane_mode",
        callback: FormatCallback::String(format_cb_pane_mode),
    },
    FormatTableEntry {
        key: c"pane_output_generation",
        callback: FormatCallback::String(format_cb_pane_output_generation),
    },
    FormatTableEntry {
        key: c"pane_path",
        callback: FormatCallback::String(format_cb_pane_path),
    },
    FormatTableEntry {
        key: c"pane_pb_progress",
        callback: FormatCallback::String(format_cb_pane_pb_progress),
    },
    FormatTableEntry {
        key: c"pane_pb_state",
        callback: FormatCallback::String(format_cb_pane_pb_state),
    },
    FormatTableEntry {
        key: c"pane_pid",
        callback: FormatCallback::String(format_cb_pane_pid),
    },
    FormatTableEntry {
        key: c"pane_pipe",
        callback: FormatCallback::String(format_cb_pane_pipe),
    },
    FormatTableEntry {
        key: c"pane_pipe_pid",
        callback: FormatCallback::String(format_cb_pane_pipe_pid),
    },
    FormatTableEntry {
        key: c"pane_private_modes",
        callback: FormatCallback::String(format_cb_pane_private_modes),
    },
    FormatTableEntry {
        key: c"pane_right",
        callback: FormatCallback::String(format_cb_pane_right),
    },
    FormatTableEntry {
        key: c"pane_search_string",
        callback: FormatCallback::String(format_cb_pane_search_string),
    },
    FormatTableEntry {
        key: c"pane_start_command",
        callback: FormatCallback::String(format_cb_start_command),
    },
    FormatTableEntry {
        key: c"pane_start_command_list",
        callback: FormatCallback::String(format_cb_start_command_list),
    },
    FormatTableEntry {
        key: c"pane_start_path",
        callback: FormatCallback::String(format_cb_start_path),
    },
    FormatTableEntry {
        key: c"pane_synchronized",
        callback: FormatCallback::String(format_cb_pane_synchronized),
    },
    FormatTableEntry {
        key: c"pane_tabs",
        callback: FormatCallback::String(format_cb_pane_tabs),
    },
    FormatTableEntry {
        key: c"pane_title",
        callback: FormatCallback::String(format_cb_pane_title),
    },
    FormatTableEntry {
        key: c"pane_top",
        callback: FormatCallback::String(format_cb_pane_top),
    },
    FormatTableEntry {
        key: c"pane_tty",
        callback: FormatCallback::String(format_cb_pane_tty),
    },
    FormatTableEntry {
        key: c"pane_unseen_changes",
        callback: FormatCallback::String(format_cb_pane_unseen_changes),
    },
    FormatTableEntry {
        key: c"pane_unzoomed_height",
        callback: FormatCallback::String(format_cb_pane_unzoomed_height),
    },
    FormatTableEntry {
        key: c"pane_unzoomed_width",
        callback: FormatCallback::String(format_cb_pane_unzoomed_width),
    },
    FormatTableEntry {
        key: c"pane_width",
        callback: FormatCallback::String(format_cb_pane_width),
    },
    FormatTableEntry {
        key: c"pane_x",
        callback: FormatCallback::String(format_cb_pane_x),
    },
    FormatTableEntry {
        key: c"pane_y",
        callback: FormatCallback::String(format_cb_pane_y),
    },
    FormatTableEntry {
        key: c"pane_z",
        callback: FormatCallback::String(format_cb_pane_z),
    },
    FormatTableEntry {
        key: c"pane_zoomed_flag",
        callback: FormatCallback::String(format_cb_pane_zoomed_flag),
    },
    FormatTableEntry {
        key: c"pid",
        callback: FormatCallback::String(format_cb_pid),
    },
    FormatTableEntry {
        key: c"scroll_region_lower",
        callback: FormatCallback::String(format_cb_scroll_region_lower),
    },
    FormatTableEntry {
        key: c"scroll_region_upper",
        callback: FormatCallback::String(format_cb_scroll_region_upper),
    },
    FormatTableEntry {
        key: c"server_sessions",
        callback: FormatCallback::String(format_cb_server_sessions),
    },
    FormatTableEntry {
        key: c"session_active",
        callback: FormatCallback::String(format_cb_session_active),
    },
    FormatTableEntry {
        key: c"session_activity",
        callback: FormatCallback::Time(format_cb_session_activity),
    },
    FormatTableEntry {
        key: c"session_activity_flag",
        callback: FormatCallback::String(format_cb_session_activity_flag),
    },
    FormatTableEntry {
        key: c"session_alert",
        callback: FormatCallback::String(format_cb_session_alert),
    },
    FormatTableEntry {
        key: c"session_alerts",
        callback: FormatCallback::String(format_cb_session_alerts),
    },
    FormatTableEntry {
        key: c"session_attached",
        callback: FormatCallback::String(format_cb_session_attached),
    },
    FormatTableEntry {
        key: c"session_attached_list",
        callback: FormatCallback::String(format_cb_session_attached_list),
    },
    FormatTableEntry {
        key: c"session_bell_flag",
        callback: FormatCallback::String(format_cb_session_bell_flag),
    },
    FormatTableEntry {
        key: c"session_created",
        callback: FormatCallback::Time(format_cb_session_created),
    },
    FormatTableEntry {
        key: c"session_format",
        callback: FormatCallback::String(format_cb_session_format),
    },
    FormatTableEntry {
        key: c"session_group",
        callback: FormatCallback::String(format_cb_session_group),
    },
    FormatTableEntry {
        key: c"session_group_attached",
        callback: FormatCallback::String(format_cb_session_group_attached),
    },
    FormatTableEntry {
        key: c"session_group_attached_list",
        callback: FormatCallback::String(format_cb_session_group_attached_list),
    },
    FormatTableEntry {
        key: c"session_group_list",
        callback: FormatCallback::String(format_cb_session_group_list),
    },
    FormatTableEntry {
        key: c"session_group_many_attached",
        callback: FormatCallback::String(format_cb_session_group_many_attached),
    },
    FormatTableEntry {
        key: c"session_group_size",
        callback: FormatCallback::String(format_cb_session_group_size),
    },
    FormatTableEntry {
        key: c"session_grouped",
        callback: FormatCallback::String(format_cb_session_grouped),
    },
    FormatTableEntry {
        key: c"session_id",
        callback: FormatCallback::String(format_cb_session_id),
    },
    FormatTableEntry {
        key: c"session_last_attached",
        callback: FormatCallback::Time(format_cb_session_last_attached),
    },
    FormatTableEntry {
        key: c"session_many_attached",
        callback: FormatCallback::String(format_cb_session_many_attached),
    },
    FormatTableEntry {
        key: c"session_marked",
        callback: FormatCallback::String(format_cb_session_marked),
    },
    FormatTableEntry {
        key: c"session_name",
        callback: FormatCallback::String(format_cb_session_name),
    },
    FormatTableEntry {
        key: c"session_path",
        callback: FormatCallback::String(format_cb_session_path),
    },
    FormatTableEntry {
        key: c"session_silence_flag",
        callback: FormatCallback::String(format_cb_session_silence_flag),
    },
    FormatTableEntry {
        key: c"session_stack",
        callback: FormatCallback::String(format_cb_session_stack),
    },
    FormatTableEntry {
        key: c"session_windows",
        callback: FormatCallback::String(format_cb_session_windows),
    },
    FormatTableEntry {
        key: c"sixel_support",
        callback: FormatCallback::String(format_cb_sixel_support),
    },
    FormatTableEntry {
        key: c"socket_path",
        callback: FormatCallback::String(format_cb_socket_path),
    },
    FormatTableEntry {
        key: c"start_time",
        callback: FormatCallback::Time(format_cb_start_time),
    },
    FormatTableEntry {
        key: c"synchronized_output_flag",
        callback: FormatCallback::String(format_cb_synchronized_output_flag),
    },
    FormatTableEntry {
        key: c"tree_mode_format",
        callback: FormatCallback::String(format_cb_tree_mode_format),
    },
    FormatTableEntry {
        key: c"uid",
        callback: FormatCallback::String(format_cb_uid),
    },
    FormatTableEntry {
        key: c"user",
        callback: FormatCallback::String(format_cb_user),
    },
    FormatTableEntry {
        key: c"version",
        callback: FormatCallback::String(format_cb_version),
    },
    FormatTableEntry {
        key: c"window_active",
        callback: FormatCallback::String(format_cb_window_active),
    },
    FormatTableEntry {
        key: c"window_active_clients",
        callback: FormatCallback::String(format_cb_window_active_clients),
    },
    FormatTableEntry {
        key: c"window_active_clients_list",
        callback: FormatCallback::String(format_cb_window_active_clients_list),
    },
    FormatTableEntry {
        key: c"window_active_sessions",
        callback: FormatCallback::String(format_cb_window_active_sessions),
    },
    FormatTableEntry {
        key: c"window_active_sessions_list",
        callback: FormatCallback::String(format_cb_window_active_sessions_list),
    },
    FormatTableEntry {
        key: c"window_activity",
        callback: FormatCallback::Time(format_cb_window_activity),
    },
    FormatTableEntry {
        key: c"window_activity_flag",
        callback: FormatCallback::String(format_cb_window_activity_flag),
    },
    FormatTableEntry {
        key: c"window_bell_flag",
        callback: FormatCallback::String(format_cb_window_bell_flag),
    },
    FormatTableEntry {
        key: c"window_bigger",
        callback: FormatCallback::String(format_cb_window_bigger),
    },
    FormatTableEntry {
        key: c"window_cell_height",
        callback: FormatCallback::String(format_cb_window_cell_height),
    },
    FormatTableEntry {
        key: c"window_cell_width",
        callback: FormatCallback::String(format_cb_window_cell_width),
    },
    FormatTableEntry {
        key: c"window_end_flag",
        callback: FormatCallback::String(format_cb_window_end_flag),
    },
    FormatTableEntry {
        key: c"window_flags",
        callback: FormatCallback::String(format_cb_window_flags),
    },
    FormatTableEntry {
        key: c"window_format",
        callback: FormatCallback::String(format_cb_window_format),
    },
    FormatTableEntry {
        key: c"window_height",
        callback: FormatCallback::String(format_cb_window_height),
    },
    FormatTableEntry {
        key: c"window_id",
        callback: FormatCallback::String(format_cb_window_id),
    },
    FormatTableEntry {
        key: c"window_index",
        callback: FormatCallback::String(format_cb_window_index),
    },
    FormatTableEntry {
        key: c"window_last_flag",
        callback: FormatCallback::String(format_cb_window_last_flag),
    },
    FormatTableEntry {
        key: c"window_layout",
        callback: FormatCallback::String(format_cb_window_layout),
    },
    FormatTableEntry {
        key: c"window_linked",
        callback: FormatCallback::String(format_cb_window_linked),
    },
    FormatTableEntry {
        key: c"window_linked_sessions",
        callback: FormatCallback::String(format_cb_window_linked_sessions),
    },
    FormatTableEntry {
        key: c"window_linked_sessions_list",
        callback: FormatCallback::String(format_cb_window_linked_sessions_list),
    },
    FormatTableEntry {
        key: c"window_manual_height",
        callback: FormatCallback::String(format_cb_window_manual_height),
    },
    FormatTableEntry {
        key: c"window_manual_width",
        callback: FormatCallback::String(format_cb_window_manual_width),
    },
    FormatTableEntry {
        key: c"window_marked_flag",
        callback: FormatCallback::String(format_cb_window_marked_flag),
    },
    FormatTableEntry {
        key: c"window_modal_pane",
        callback: FormatCallback::String(format_cb_window_modal_pane),
    },
    FormatTableEntry {
        key: c"window_name",
        callback: FormatCallback::String(format_cb_window_name),
    },
    FormatTableEntry {
        key: c"window_offset_x",
        callback: FormatCallback::String(format_cb_window_offset_x),
    },
    FormatTableEntry {
        key: c"window_offset_y",
        callback: FormatCallback::String(format_cb_window_offset_y),
    },
    FormatTableEntry {
        key: c"window_panes",
        callback: FormatCallback::String(format_cb_window_panes),
    },
    FormatTableEntry {
        key: c"window_raw_flags",
        callback: FormatCallback::String(format_cb_window_raw_flags),
    },
    FormatTableEntry {
        key: c"window_silence_flag",
        callback: FormatCallback::String(format_cb_window_silence_flag),
    },
    FormatTableEntry {
        key: c"window_stack_index",
        callback: FormatCallback::String(format_cb_window_stack_index),
    },
    FormatTableEntry {
        key: c"window_start_flag",
        callback: FormatCallback::String(format_cb_window_start_flag),
    },
    FormatTableEntry {
        key: c"window_visible_layout",
        callback: FormatCallback::String(format_cb_window_visible_layout),
    },
    FormatTableEntry {
        key: c"window_width",
        callback: FormatCallback::String(format_cb_window_width),
    },
    FormatTableEntry {
        key: c"window_zoomed_flag",
        callback: FormatCallback::String(format_cb_window_zoomed_flag),
    },
    FormatTableEntry {
        key: c"wrap_flag",
        callback: FormatCallback::String(format_cb_wrap_flag),
    },
];
pub(super) fn format_table_get(key: &CStr) -> Option<&'static FormatTableEntry> {
    FORMAT_TABLE
        .binary_search_by(|entry| entry.key.cmp(key))
        .ok()
        .map(|index| &FORMAT_TABLE[index])
}

#[cfg(test)]
mod owned_callback_tests {
    use super::*;

    #[test]
    fn builtin_lookup_preserves_sorted_keys_and_missing_results() {
        unsafe {
            assert!(FORMAT_TABLE
                .windows(2)
                .all(|pair| pair[0].key < pair[1].key));
            for entry in &FORMAT_TABLE {
                assert!(std::ptr::eq(format_table_get(entry.key).unwrap(), entry));
            }
            assert!(format_table_get(c"not_a_builtin").is_none());
            let ft = format_create(std::ptr::null_mut(), std::ptr::null_mut(), 0, 0);
            for key in [
                c"buffer_full",
                c"client_created",
                c"session_created",
                c"pane_dead_time",
            ] {
                assert!(format_table_get(key).unwrap().get(ft).is_none());
            }
            format_free(ft);
        }
    }

    #[test]
    fn callback_strings_and_timestamps_outlive_the_buffer_and_tree() {
        use crate::src::paste::{paste_free, paste_get_name, paste_set_owned};
        unsafe {
            let mut cause: Option<std::ffi::CString> = None;
            let name = c"callback-owned-results";
            assert_eq!(
                paste_set_owned(
                    b"A\xff\0B".to_vec().into_boxed_slice(),
                    name.as_ptr(),
                    Some(&mut cause)
                ),
                0
            );
            let pb = paste_get_name(name.as_ptr());
            let ft = format_create(std::ptr::null_mut(), std::ptr::null_mut(), 0, 0);
            (*ft).pb = pb;
            let full = format_cb_buffer_full(ft).unwrap();
            let sample = format_cb_buffer_sample(ft).unwrap();
            let created = format_cb_buffer_created(ft).unwrap();
            let old_created = (*pb).created;
            (*pb).created = 0;
            assert_eq!(format_cb_buffer_created(ft), Some(0));
            (*ft).pb = std::ptr::null_mut();
            paste_free(pb);
            format_free(ft);
            assert_eq!(full.as_bytes(), b"A\xff");
            assert!(!sample.as_bytes().is_empty());
            assert_eq!(created, old_created);
        }
    }
}
