use crate::src::options::options_owner_ptr;
use crate::src::server_client::Client as _;
use crate::src::shared::client::client_handle;
use crate::src::shared::client::ClientRef;
use crate::src::shared::session::SessionRef;
use crate::src::shared::window::WindowRef;
use crate::src::tty_term::tty_term_owner_ptr;
use crate::src::window::Window as _;
use crate::src::window_pane::WindowPane as _;
// Built-in callbacks return owned bytes or copied timestamps. The sorted
// immutable table is shared by lookup and enumeration; external user callbacks
// retain their separate C ABI.
use super::*;
use crate::src::format::bytes::xformat;
use crate::src::session::Session;
use crate::src::shared::rc::same;
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString};
use std::fmt::Write as _;
use std::rc::Rc;

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
unsafe fn format_cb_session_attached_list(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_attached_list", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_alert(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_alert", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_alerts(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_alerts", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_stack(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_stack", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_window_stack_index(ft: *mut format_tree) -> Option<CString> {
    let link = (*ft).winlink_handle();
    let owner = link.try_borrow_mut().ok()?.session.upgrade()?;
    let mut context = format_tree::default();
    context.s = Rc::downgrade(&owner);
    context.wl = link;
    match owner.format_value(c"window_stack_index", &mut context)? {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("window_stack_index is a string builtin"),
    }
}
unsafe fn format_cb_window_linked_sessions_list(ft: *mut format_tree) -> Option<CString> {
    let link = (*ft).winlink_handle();
    if !link.is_alive() {
        return None;
    }
    let window = link.get_unchecked().window_handle().expect("linked window");
    let mut names = Vec::new();
    let mut cursor = window.next_winlink(None);
    while cursor.is_alive() {
        if let Some(session) = cursor.get_unchecked().session.upgrade() {
            if !names.is_empty() {
                names.push(b',');
            }
            names.extend_from_slice(session.name().as_bytes());
        }
        cursor = window.next_winlink(Some(cursor));
    }
    if names.is_empty() {
        return None;
    }
    Some(CString::new(names).expect("callback bytes contain no NUL"))
}

unsafe fn format_cb_window_active_sessions(ft: *mut format_tree) -> Option<CString> {
    let link = (*ft).winlink_handle();
    if !link.is_alive() {
        return None;
    }
    let window = link.get_unchecked().window_handle().expect("linked window");
    let mut count = 0u32;
    let mut cursor = window.next_winlink(None);
    while cursor.is_alive() {
        if let Some(session) = cursor.get_unchecked().session.upgrade() {
            if session.current_winlink() == cursor {
                count = count.wrapping_add(1);
            }
        }
        cursor = window.next_winlink(Some(cursor));
    }
    Some(CString::new(count.to_string()).expect("formatted number contains no NUL"))
}

unsafe fn format_cb_window_active_sessions_list(ft: *mut format_tree) -> Option<CString> {
    let link = (*ft).winlink_handle();
    if !link.is_alive() {
        return None;
    }
    let window = link.get_unchecked().window_handle().expect("linked window");
    let mut names = Vec::new();
    let mut cursor = window.next_winlink(None);
    while cursor.is_alive() {
        if let Some(session) = cursor.get_unchecked().session.upgrade() {
            if session.current_winlink() == cursor {
                if !names.is_empty() {
                    names.push(b',');
                }
                names.extend_from_slice(session.name().as_bytes());
            }
        }
        cursor = window.next_winlink(Some(cursor));
    }
    if names.is_empty() {
        return None;
    }
    Some(CString::new(names).expect("callback bytes contain no NUL"))
}

unsafe fn format_cb_window_active_clients(ft: *mut format_tree) -> Option<CString> {
    let link = (*ft).winlink_handle();
    if !link.is_alive() {
        return None;
    }
    let window = link.get_unchecked().window_handle();
    let mut count = 0u32;
    let mut cursor = clients.first();
    while let Some(client) = cursor {
        if let Some(session) = client.attached_session().upgrade() {
            if same(
                window,
                session.current_winlink().get_unchecked().window_handle(),
            ) {
                count = count.wrapping_add(1);
            }
        }
        cursor = clients.next(&client);
    }
    Some(CString::new(count.to_string()).expect("formatted number contains no NUL"))
}

unsafe fn format_cb_window_active_clients_list(ft: *mut format_tree) -> Option<CString> {
    let link = (*ft).winlink_handle();
    if !link.is_alive() {
        return None;
    }
    let window = link.get_unchecked().window_handle();
    let mut names = Vec::new();
    let mut cursor = clients.first();
    while let Some(client) = cursor {
        if let Some(session) = client.attached_session().upgrade() {
            if same(
                window,
                session.current_winlink().get_unchecked().window_handle(),
            ) {
                if !names.is_empty() {
                    names.push(b',');
                }
                names.extend_from_slice(client.name().as_deref().expect("client name").to_bytes());
            }
        }
        cursor = clients.next(&client);
    }
    if names.is_empty() {
        return None;
    }
    Some(CString::new(names).expect("callback bytes contain no NUL"))
}

unsafe fn format_cb_window_layout(ft: *mut format_tree) -> Option<CString> {
    let window = (*ft).w.upgrade()?;
    let legacy = client_handle(&(*ft).client)
        .as_ref()
        .is_some_and(|client| client.uses_legacy_layout_format());
    let value = window.layout_string(crate::src::window::LayoutView::Unzoomed, legacy);
    window.release(c"format window layout");
    value
}

unsafe fn format_cb_window_visible_layout(ft: *mut format_tree) -> Option<CString> {
    let window = (*ft).w.upgrade()?;
    let legacy = client_handle(&(*ft).client)
        .as_ref()
        .is_some_and(|client| client.uses_legacy_layout_format());
    let value = window.layout_string(crate::src::window::LayoutView::Visible, legacy);
    window.release(c"format visible window layout");
    value
}

unsafe fn format_cb_start_command(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    if wp.is_null() {
        return None;
    }
    return cmd_stringify_argv_cstring(&(*wp).argv);
}
unsafe fn format_cb_start_command_list(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    if wp.is_null() {
        return None;
    }
    if (*wp).cwd.is_none() {
        return Some(c"".to_owned());
    }
    return (*wp).cwd.clone();
}
unsafe fn format_cb_current_command(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut size: size_t = 0 as size_t;
    let mut i: u_int = 0;
    let mut value = None;
    if wp.is_null() {
        return None;
    }
    gd = (*wp).base.grid_mut();
    i = 0 as u_int;
    while i < (*gd).hsize.wrapping_add((*gd).sy) {
        let gl = grid_get_line(&*gd, i);
        size = (size as ::core::ffi::c_ulong).wrapping_add(
            (gl.cellsize as usize).wrapping_mul(::core::mem::size_of::<grid_cell_entry>() as usize)
                as ::core::ffi::c_ulong,
        ) as size_t as size_t;
        size = (size as ::core::ffi::c_ulong).wrapping_add(
            (gl.extdsize as usize).wrapping_mul(::core::mem::size_of::<grid_extd_entry>() as usize)
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut i: u_int = 0;
    let mut lines: u_int = 0;
    let mut cells: u_int = 0 as u_int;
    let mut extended_cells: u_int = 0 as u_int;
    let mut value = None;
    if wp.is_null() {
        return None;
    }
    gd = (*wp).base.grid_mut();
    lines = (*gd).hsize.wrapping_add((*gd).sy);
    i = 0 as u_int;
    while i < lines {
        let gl = grid_get_line(&*gd, i);
        cells = cells.wrapping_add(gl.cellsize as u_int);
        extended_cells = extended_cells.wrapping_add(gl.extdsize);
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    let mut i: u_int = 0;
    if wp.is_null() {
        return None;
    }
    let mut tabs = String::new();
    i = 0 as u_int;
    while i < (*wp).base.grid().sx {
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
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
    gc = tty_default_colours(&(*(wp)).observer.upgrade().expect("live window_pane")).0;
    return Some(colour_format(gc.fg));
}
unsafe fn format_cb_pane_flags(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(window_pane_printable_flags(
            &(*(format_pane))
                .observer
                .upgrade()
                .expect("live window_pane"),
        ));
    }
    return None;
}
unsafe fn format_cb_pane_floating_flag(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    if !wp.is_null() {
        if window_pane_is_floating(&*wp) != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_modal_flag(ft: *mut format_tree) -> Option<CString> {
    let pane = (*ft).wp.upgrade()?;
    let window = pane.window_observer().upgrade().expect("pane window");
    let modal = window
        .modal_pane()
        .is_some_and(|modal| Rc::ptr_eq(&modal, &pane));
    Some(if modal { c"1" } else { c"0" }.to_owned())
}

unsafe fn format_cb_pane_bg(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
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
    gc = tty_default_colours(&(*(wp)).observer.upgrade().expect("live window_pane")).0;
    return Some(colour_format(gc.bg));
}
unsafe fn format_cb_session_group_list(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_group_list", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_group_attached_list(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_group_attached_list", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_pane_in_mode(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let pane = &*format_pane_owner?.get();
    let count = pane.modes.len();
    Some(CString::new(format!("{count}")).expect("formatted numbers contain no NUL"))
}
unsafe fn format_cb_pane_at_top(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    let mut status: ::core::ffi::c_int = 0;
    let mut flag: ::core::ffi::c_int = 0;
    let mut value = None;
    if wp.is_null() {
        return None;
    }
    status = window_pane_get_pane_status(&*wp);
    if status == PANE_STATUS_TOP {
        flag = ((*wp).yoff == 1 as ::core::ffi::c_int) as ::core::ffi::c_int;
    } else {
        flag = ((*wp).yoff == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    value =
        Some(CString::new(format!("{}", (flag) as i32)).expect("formatted numbers contain no NUL"));
    return value;
}
unsafe fn format_cb_pane_at_bottom(ft: *mut format_tree) -> Option<CString> {
    let pane = (*ft).wp.upgrade()?;
    let window = pane.window_observer().upgrade().expect("pane window");
    let status = pane.border_status();
    let (_, height, _, y) = pane.geometry();
    let bottom = window.size().1 as i32 - if status == PANE_STATUS_BOTTOM { 1 } else { 0 };
    Some(
        if y + height as i32 == bottom {
            c"1"
        } else {
            c"0"
        }
        .to_owned(),
    )
}

unsafe fn format_cb_cursor_character(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
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
    grid_view_get_cell((*wp).base.grid(), (*wp).base.cx, (*wp).base.cy, &mut gc);
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    if wp.is_null() || (*wp).screen_ptr().is_null() {
        return None;
    }
    if (*(*wp).screen_ptr()).ccolour != -(1 as ::core::ffi::c_int) {
        return Some(colour_format((*(*wp).screen_ptr()).ccolour));
    }
    return Some(colour_format((*(*wp).screen_ptr()).default_ccolour));
}
unsafe fn format_cb_mouse_word(mut ft: *mut format_tree) -> Option<CString> {
    let mouse_pane_owner;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
    }
    mouse_pane_owner = cmd_mouse_pane(
        &raw mut (*ft).m,
        None,
        ::core::ptr::null_mut::<refbox::Weak<winlink>>(),
    );
    wp = mouse_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() {
        return None;
    }
    if cmd_mouse_at(
        &*(wp),
        &raw mut (*ft).m,
        &raw mut x,
        &raw mut y,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        return None;
    }
    if !(*wp).modes.is_empty() {
        if window_pane_mode(&*wp) != WINDOW_PANE_NO_MODE {
            return window_copy_get_word_cstring(
                &(*(wp)).observer.upgrade().expect("live window_pane"),
                x,
                y,
            );
        }
        return None;
    }
    gd = (*wp).base.grid_mut();
    return format_grid_word_cstring(&*gd, x, (*gd).hsize.wrapping_add(y));
}
unsafe fn format_cb_mouse_hyperlink(mut ft: *mut format_tree) -> Option<CString> {
    let mouse_pane_owner;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
    }
    mouse_pane_owner = cmd_mouse_pane(
        &raw mut (*ft).m,
        None,
        ::core::ptr::null_mut::<refbox::Weak<winlink>>(),
    );
    wp = mouse_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() {
        return None;
    }
    if cmd_mouse_at(
        &*(wp),
        &raw mut (*ft).m,
        &raw mut x,
        &raw mut y,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        return None;
    }
    if !(*wp).modes.is_empty() {
        if window_pane_mode(&*wp) != WINDOW_PANE_NO_MODE {
            return window_copy_get_hyperlink_cstring(
                &(*(wp)).observer.upgrade().expect("live window_pane"),
                x,
                y,
            );
        }
        return None;
    }
    gd = (*wp).base.grid_mut();
    return format_grid_hyperlink_cstring(
        &*gd,
        x,
        (*gd).hsize.wrapping_add(y),
        &*(*wp).screen_ptr(),
    );
}
unsafe fn format_cb_mouse_line(mut ft: *mut format_tree) -> Option<CString> {
    let mouse_pane_owner;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
    }
    mouse_pane_owner = cmd_mouse_pane(
        &raw mut (*ft).m,
        None,
        ::core::ptr::null_mut::<refbox::Weak<winlink>>(),
    );
    wp = mouse_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() {
        return None;
    }
    if cmd_mouse_at(
        &*(wp),
        &raw mut (*ft).m,
        &raw mut x,
        &raw mut y,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        return None;
    }
    if !(*wp).modes.is_empty() {
        if window_pane_mode(&*wp) != WINDOW_PANE_NO_MODE {
            return window_copy_get_line_cstring(
                &(*(wp)).observer.upgrade().expect("live window_pane"),
                y,
            );
        }
        return None;
    }
    gd = (*wp).base.grid_mut();
    return format_grid_line_cstring(&*gd, (*gd).hsize.wrapping_add(y));
}
unsafe fn format_cb_mouse_status_line(mut ft: *mut format_tree) -> Option<CString> {
    let format_client_owner = (*ft).c.upgrade();
    let mut format_client: Option<ClientRef> = format_client_owner.clone();
    let mut value = None;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
    }
    if format_client.is_none()
        || !format_client
            .as_ref()
            .expect("live client")
            .terminal_started()
    {
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
    let format_client_owner = (*ft).c.upgrade();
    let mut format_client: Option<ClientRef> = format_client_owner.clone();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
    }
    if format_client.is_none()
        || !format_client
            .as_ref()
            .expect("live client")
            .terminal_started()
    {
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
    let sr = status_get_range(format_client.as_ref().expect("live client"), x, y)?;
    match sr.type_0 as ::core::ffi::c_uint {
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
            return Some(CStr::from_ptr(sr.string.as_ptr()).to_owned());
        }
        7 => {
            return Some(c"control".to_owned());
        }
        _ => {}
    }
    return None;
}
unsafe fn format_cb_alternate_on(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.saved_grid.is_some() {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_alternate_saved_x(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).base.saved_cx) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_alternate_saved_y(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).base.saved_cy) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_bracket_paste_flag(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() && !(*format_pane).screen_ptr().is_null() {
        if (*(*format_pane).screen_ptr()).mode & MODE_BRACKETPASTE != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_buffer_name(ft: *mut format_tree) -> Option<CString> {
    Some(paste_buffer_name(&*(*ft).pb.as_ref()?.try_borrow()?).to_owned())
}
unsafe fn format_cb_buffer_sample(ft: *mut format_tree) -> Option<CString> {
    Some(paste_make_sample_cstring(
        &*(*ft).pb.as_ref()?.try_borrow()?,
    ))
}
unsafe fn format_cb_buffer_full(ft: *mut format_tree) -> Option<CString> {
    let buffer = (*ft).pb.as_ref()?.try_borrow()?;
    let bytes = paste_buffer_data(&buffer)?;
    let end = bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len());
    Some(CString::new(&bytes[..end]).expect("bounded buffer contains no NUL"))
}
unsafe fn format_cb_buffer_size(ft: *mut format_tree) -> Option<CString> {
    let buffer = (*ft).pb.as_ref()?.try_borrow()?;
    Some(CString::new(buffer.size().to_string()).expect("formatted numbers contain no NUL"))
}
unsafe fn format_cb_client_cell_height(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_cell_height", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_cell_width(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_cell_width", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_colours(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_colours", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_control_mode(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_control_mode", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_discarded(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_discarded", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_flags(ft: *mut format_tree) -> Option<CString> {
    match (*ft).c.upgrade()?.format_value(c"client_flags", &mut *ft)? {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_height(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_height", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_key_table(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_key_table", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_last_session(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_last_session", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_name(ft: *mut format_tree) -> Option<CString> {
    match (*ft).c.upgrade()?.format_value(c"client_name", &mut *ft)? {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_pid(ft: *mut format_tree) -> Option<CString> {
    match (*ft).c.upgrade()?.format_value(c"client_pid", &mut *ft)? {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_prefix(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_prefix", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_readonly(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_readonly", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_session(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_session", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_termfeatures(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_termfeatures", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_termname(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_termname", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_termtype(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_termtype", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_tty(ft: *mut format_tree) -> Option<CString> {
    match (*ft).c.upgrade()?.format_value(c"client_tty", &mut *ft)? {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_uid(ft: *mut format_tree) -> Option<CString> {
    match (*ft).c.upgrade()?.format_value(c"client_uid", &mut *ft)? {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_user(ft: *mut format_tree) -> Option<CString> {
    match (*ft).c.upgrade()?.format_value(c"client_user", &mut *ft)? {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_utf8(ft: *mut format_tree) -> Option<CString> {
    match (*ft).c.upgrade()?.format_value(c"client_utf8", &mut *ft)? {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_width(ft: *mut format_tree) -> Option<CString> {
    match (*ft).c.upgrade()?.format_value(c"client_width", &mut *ft)? {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_written(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_written", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_theme(ft: *mut format_tree) -> Option<CString> {
    match (*ft).c.upgrade()?.format_value(c"client_theme", &mut *ft)? {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.mode & MODE_CURSOR != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_cursor_shape(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() && !(*format_pane).screen_ptr().is_null() {
        match (*(*format_pane).screen_ptr()).cstyle as ::core::ffi::c_uint {
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() && !(*format_pane).screen_ptr().is_null() {
        if (*(*format_pane).screen_ptr()).mode & MODE_CURSOR_VERY_VISIBLE != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_cursor_x(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).base.cx) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_cursor_y(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).base.cy) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_cursor_blinking(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() && !(*format_pane).screen_ptr().is_null() {
        if (*(*format_pane).screen_ptr()).mode & MODE_CURSOR_BLINKING != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_history_added(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!(
                "{}",
                ((*format_pane).base.grid().scroll_added) as u32
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_history_collected(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    if !wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*wp).base.grid().scroll_collected) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_history_generation(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    if !wp.is_null() {
        return Some(
            CString::new(format!("{}", ((*wp).base.grid().scroll_generation) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_history_limit(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).base.grid().hlimit) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_history_size(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).base.grid().hsize) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_insert_flag(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.mode & MODE_INSERT != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_keypad_cursor_flag(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.mode & MODE_KCURSOR != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_keypad_flag(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.mode & MODE_KKEYPAD != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_all_flag(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.mode & MODE_MOUSE_ALL != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_any_flag(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.mode & ALL_MOUSE_MODES != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_button_flag(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.mode & MODE_MOUSE_BUTTON != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_pane(mut ft: *mut format_tree) -> Option<CString> {
    let mouse_pane_owner;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (*ft).m.valid != 0 {
        mouse_pane_owner = cmd_mouse_pane(
            &raw mut (*ft).m,
            None,
            ::core::ptr::null_mut::<refbox::Weak<winlink>>(),
        );
        wp = mouse_pane_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.mode & MODE_MOUSE_SGR != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_standard_flag(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.mode & MODE_MOUSE_STANDARD != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_utf8_flag(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.mode & MODE_MOUSE_UTF8 != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_mouse_x(mut ft: *mut format_tree) -> Option<CString> {
    let format_client_owner = (*ft).c.upgrade();
    let mut format_client: Option<ClientRef> = format_client_owner.clone();
    let mouse_pane_owner;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
    }
    mouse_pane_owner = cmd_mouse_pane(
        &raw mut (*ft).m,
        None,
        ::core::ptr::null_mut::<refbox::Weak<winlink>>(),
    );
    wp = mouse_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !wp.is_null()
        && cmd_mouse_at(
            &*(wp),
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
    if !format_client.is_none()
        && format_client
            .as_ref()
            .expect("live client")
            .terminal_started()
    {
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
    let format_client_owner = (*ft).c.upgrade();
    let mut format_client: Option<ClientRef> = format_client_owner.clone();
    let mouse_pane_owner;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return None;
    }
    mouse_pane_owner = cmd_mouse_pane(
        &raw mut (*ft).m,
        None,
        ::core::ptr::null_mut::<refbox::Weak<winlink>>(),
    );
    wp = mouse_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !wp.is_null()
        && cmd_mouse_at(
            &*(wp),
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
    if !format_client.is_none()
        && format_client
            .as_ref()
            .expect("live client")
            .terminal_started()
    {
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.mode & MODE_ORIGIN != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_synchronized_output_flag(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.mode & MODE_SYNC != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_private_modes(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
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
    if format_pane.is_null() {
        return None;
    }
    mode = (*format_pane).base.mode;
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if format_pane
            == (((*format_pane).window_handle().as_ref()).expect("live window"))
                .active_pane()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get())
        {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_at_left(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).xoff == 0 as ::core::ffi::c_int {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_at_right(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).xoff + (*format_pane).sx as ::core::ffi::c_int
            == (((*format_pane).window_handle().as_ref()).expect("live window"))
                .size()
                .0 as ::core::ffi::c_int
        {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_bottom(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    if !wp.is_null() {
        if (*wp).fd == -(1 as ::core::ffi::c_int) && (*wp).flags & PANE_STATUSREADY != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_dead_signal(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    if !wp.is_null() {
        if (*wp).flags & PANE_STATUSDRAWN != 0 {
            return Some(((*wp).dead_time).tv_sec as time_t);
        }
        return None;
    }
    return None;
}
unsafe fn format_cb_pane_last_output_time(mut ft: *mut format_tree) -> Option<time_t> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    if !wp.is_null() && (*wp).last_output_time != 0 as time_t {
        return Some((*wp).last_output_time as __time_t as time_t);
    }
    return None;
}
unsafe fn format_cb_pane_output_generation(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut value: ::core::ffi::c_ulonglong = 0;
    if !format_pane.is_null() {
        value = (*format_pane).output_generation as ::core::ffi::c_ulonglong;
        return Some(
            CString::new(format!("{}", (value) as u64)).expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_last_prompt_time(mut ft: *mut format_tree) -> Option<time_t> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    if !wp.is_null() && (*wp).last_prompt_time != 0 as time_t {
        return Some((*wp).last_prompt_time as __time_t as time_t);
    }
    return None;
}
unsafe fn format_cb_pane_command_start_time(mut ft: *mut format_tree) -> Option<time_t> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    if !wp.is_null() && (*wp).cmd_start_time != 0 as time_t {
        return Some((*wp).cmd_start_time as __time_t as time_t);
    }
    return None;
}
unsafe fn format_cb_pane_command_end_time(mut ft: *mut format_tree) -> Option<time_t> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    if !wp.is_null() && (*wp).cmd_end_time != 0 as time_t {
        return Some((*wp).cmd_end_time as __time_t as time_t);
    }
    return None;
}
unsafe fn format_cb_pane_command_running(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).sy) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_id(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("%{}", ((*format_pane).id) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_index(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut idx: u_int = 0;
    if !format_pane.is_null()
        && window_pane_index(&*format_pane)
            .map(|value| {
                idx = value;
            })
            .is_some()
    {
        return Some(
            CString::new(format!("{}", (idx) as u32)).expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_input_off(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).flags & PANE_INPUTOFF != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_unseen_changes(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).flags & PANE_UNSEENCHANGES != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_key_mode(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() && !(*format_pane).screen_ptr().is_null() {
        match (*(*format_pane).screen_ptr()).mode & EXTENDED_KEY_MODES {
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
unsafe fn format_cb_pane_last(ft: *mut format_tree) -> Option<CString> {
    let pane = (*ft).wp.upgrade()?;
    let Some(window) = pane.window_observer().upgrade() else {
        return Some(c"0".to_owned());
    };
    let is_last = window
        .last_active_pane()
        .as_ref()
        .is_some_and(|last| Rc::ptr_eq(last, &pane));
    let value = if is_last { c"1" } else { c"0" }.to_owned();
    window.release(c"format last pane");
    Some(value)
}

unsafe fn format_cb_pane_left(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).xoff) as i32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_marked(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if server_check_marked() != 0
            && marked_pane
                .pane_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get())
                == format_pane
        {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_marked_set(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if server_check_marked() != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_mode(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let pane = &*format_pane_owner?.get();
    pane.active_mode().map(|mode| mode.name.to_owned())
}
unsafe fn format_cb_pane_path(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            (*format_pane)
                .base
                .path
                .clone()
                .unwrap_or_else(|| c"".to_owned()),
        );
    }
    return None;
}
unsafe fn format_cb_pane_pid(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() && (*format_pane).fd != -(1 as ::core::ffi::c_int) {
        return Some(
            CString::new(format!(
                "{}",
                ((*format_pane).pid as ::core::ffi::c_long) as ::core::ffi::c_long
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_pipe(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).pipe_fd != -(1 as ::core::ffi::c_int) {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_pane_pipe_pid(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut value = None;
    if !format_pane.is_null() && (*format_pane).pipe_fd != -(1 as ::core::ffi::c_int) {
        value = Some(
            CString::new(format!(
                "{}",
                ((*format_pane).pipe_pid as ::core::ffi::c_long) as ::core::ffi::c_long
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return value;
}
unsafe fn format_cb_pane_pb_progress(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut value = None;
    if !format_pane.is_null() {
        value = Some(
            CString::new(format!(
                "{}",
                ((*format_pane).base.progress_bar.progress) as i32
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return value;
}
unsafe fn format_cb_pane_pb_state(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        match (*format_pane).base.progress_bar.state as ::core::ffi::c_uint {
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).searchstr.is_none() {
            return Some(c"".to_owned());
        }
        return (*format_pane).searchstr.clone();
    }
    return None;
}
unsafe fn format_cb_pane_synchronized(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if options_get_number(
            options_owner_ptr(&mut (*format_pane).options)
                .map_or(std::ptr::null_mut(), |options| options),
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
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some((*format_pane).base.title.clone());
    }
    return None;
}
unsafe fn format_cb_pane_top(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).yoff) as i32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_tty(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CStr::from_ptr(&raw mut (*format_pane).tty as *mut ::core::ffi::c_char).to_owned(),
        );
    }
    return None;
}
unsafe fn format_cb_pane_unzoomed_height(ft: *mut format_tree) -> Option<CString> {
    let size = (*ft).wp.upgrade()?.unzoomed_height()?;
    Some(CString::new(size.to_string()).expect("formatted pane size"))
}

unsafe fn format_cb_pane_unzoomed_width(ft: *mut format_tree) -> Option<CString> {
    let size = (*ft).wp.upgrade()?.unzoomed_width()?;
    Some(CString::new(size.to_string()).expect("formatted pane size"))
}

unsafe fn format_cb_pane_width(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).sx) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_x(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).xoff) as i32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_y(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).yoff) as i32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_z(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut idx: u_int = 0;
    if !format_pane.is_null()
        && window_pane_zindex(&*format_pane)
            .map(|value| {
                idx = value;
            })
            .is_some()
    {
        return Some(
            CString::new(format!("{}", (idx) as u32)).expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_pane_zoomed_flag(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = format_pane;
    if !wp.is_null() {
        if (*wp).flags & PANE_ZOOMED != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_scroll_region_lower(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).base.rlower) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_scroll_region_upper(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        return Some(
            CString::new(format!("{}", ((*format_pane).base.rupper) as u32))
                .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_server_sessions(_ft: *mut format_tree) -> Option<CString> {
    let mut s: Option<SessionRef> = None;
    let mut n: u_int = 0 as u_int;
    let mut s_owner = sessions_minmax(&sessions);
    s = s_owner.clone();
    while !s.is_none() {
        n = n.wrapping_add(1);
        s_owner = s.as_ref().expect("live session").next_session();
        s = s_owner.clone();
    }
    return Some(
        CString::new(format!("{}", (n) as u32)).expect("formatted numbers contain no NUL"),
    );
}
unsafe fn format_cb_session_active(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_active", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_activity_flag(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_activity_flag", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_bell_flag(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_bell_flag", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_silence_flag(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_silence_flag", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_attached(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_attached", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_format(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).type_0 as ::core::ffi::c_uint
        == FORMAT_TYPE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return Some(c"1".to_owned());
    }
    return Some(c"0".to_owned());
}
unsafe fn format_cb_session_group(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_group", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_group_attached(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_group_attached", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_group_many_attached(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_group_many_attached", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_group_size(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_group_size", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_grouped(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_grouped", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_id(ft: *mut format_tree) -> Option<CString> {
    match (*ft).s.upgrade()?.format_value(c"session_id", &mut *ft)? {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_many_attached(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_many_attached", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_marked(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_marked", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_name(ft: *mut format_tree) -> Option<CString> {
    match (*ft).s.upgrade()?.format_value(c"session_name", &mut *ft)? {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_path(ft: *mut format_tree) -> Option<CString> {
    match (*ft).s.upgrade()?.format_value(c"session_path", &mut *ft)? {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_windows(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_windows", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
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
unsafe fn format_cb_active_window_index(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"active_window_index", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_last_window_index(ft: *mut format_tree) -> Option<CString> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"last_window_index", &mut *ft)?
    {
        FormatValue::String(value) => Some(value),
        FormatValue::Time(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_window_active(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).winlink_handle().is_alive() {
        let session_owner = ((*ft).winlink_handle()).get_unchecked().session.upgrade()?;
        if (*ft).winlink_handle() == session_owner.current_winlink() {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_activity_flag(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).winlink_handle().is_alive() {
        if ((*ft).winlink_handle()).get_unchecked().flags & WINLINK_ACTIVITY != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_bell_flag(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).winlink_handle().is_alive() {
        if ((*ft).winlink_handle()).get_unchecked().flags & WINLINK_BELL != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_bigger(ft: *mut format_tree) -> Option<CString> {
    let format_client_owner = (*ft).c.upgrade();
    let mut format_client: Option<ClientRef> = format_client_owner.clone();
    let c = format_client.as_ref()?;
    let view = c.terminal_view();
    Some(if view.bigger { c"1" } else { c"0" }.to_owned())
}
unsafe fn format_cb_window_cell_height(ft: *mut format_tree) -> Option<CString> {
    let window = (*ft).w.upgrade()?;
    let value =
        CString::new(window.cell_size().1.to_string()).expect("formatted number contains no NUL");
    window.release(c"format window_cell_height");
    Some(value)
}

unsafe fn format_cb_window_cell_width(ft: *mut format_tree) -> Option<CString> {
    let window = (*ft).w.upgrade()?;
    let value =
        CString::new(window.cell_size().0.to_string()).expect("formatted number contains no NUL");
    window.release(c"format window_cell_width");
    Some(value)
}

unsafe fn format_cb_window_end_flag(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).winlink_handle().is_alive() {
        let session_owner = ((*ft).winlink_handle()).get_unchecked().session.upgrade()?;
        if (*ft).winlink_handle()
            == session_owner.with_winlinks(|links| winlinks_minmax(links, RB_INF))
        {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_flags(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).winlink_handle().is_alive() {
        return Some(window_printable_flags(
            ((*ft).winlink_handle()).clone(),
            1 as ::core::ffi::c_int,
        ));
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
unsafe fn format_cb_window_height(ft: *mut format_tree) -> Option<CString> {
    let window = (*ft).w.upgrade()?;
    let value =
        CString::new(window.size().1.to_string()).expect("formatted number contains no NUL");
    window.release(c"format window_height");
    Some(value)
}

unsafe fn format_cb_window_manual_height(ft: *mut format_tree) -> Option<CString> {
    let window = (*ft).w.upgrade()?;
    let manual = window.with_options_mut(|options| {
        options_get_number(options, c"window-size".as_ptr()) == WINDOW_SIZE_MANUAL as _
    });
    let value = if manual {
        CString::new(window.manual_size().1.to_string()).expect("formatted number")
    } else {
        c"".to_owned()
    };
    window.release(c"format manual window height");
    Some(value)
}
unsafe fn format_cb_window_id(ft: *mut format_tree) -> Option<CString> {
    let window = (*ft).w.upgrade()?;
    let value = CString::new(format!("@{}", window.id())).expect("formatted ID contains no NUL");
    window.release(c"format window ID");
    Some(value)
}

unsafe fn format_cb_window_index(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).winlink_handle().is_alive() {
        return Some(
            CString::new(format!(
                "{}",
                (((*ft).winlink_handle()).get_unchecked().idx) as i32
            ))
            .expect("formatted numbers contain no NUL"),
        );
    }
    return None;
}
unsafe fn format_cb_window_last_flag(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).winlink_handle().is_alive() {
        let session_owner = ((*ft).winlink_handle()).get_unchecked().session.upgrade()?;
        if (*ft).winlink_handle() == session_owner.last_winlink() {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_linked(mut ft: *mut format_tree) -> Option<CString> {
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut s: Option<SessionRef> = None;
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*ft).winlink_handle().is_alive() {
        let mut s_owner = sessions_minmax(&sessions);
        s = s_owner.clone();
        while !s.is_none() {
            wl = s_owner
                .as_ref()
                .expect("session registry entry")
                .with_winlinks(|links| winlinks_minmax(links, RB_NEGINF));
            while wl.is_alive() {
                if same(
                    wl.get_unchecked().window_handle(),
                    (*ft).winlink_handle().get_unchecked().window_handle(),
                ) {
                    if found != 0 {
                        return Some(c"1".to_owned());
                    }
                    found = 1 as ::core::ffi::c_int;
                }
                wl = winlinks_next(wl.get_unchecked());
            }
            s_owner = s.as_ref().expect("live session").next_session();
            s = s_owner.clone();
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_linked_sessions(mut ft: *mut format_tree) -> Option<CString> {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut s: Option<SessionRef> = None;
    let mut n: u_int = 0 as u_int;
    if !(*ft).winlink_handle().is_alive() {
        return None;
    }
    let window = (*ft)
        .winlink_handle()
        .get_unchecked()
        .window_handle()
        .expect("linked window")
        .clone();
    sg = session_groups_minmax(&session_groups);
    while !sg.is_null() {
        let group_members = crate::src::session::session_group_members(sg);
        s = group_members.first().cloned();
        if group_members.first().is_some_and(|session| {
            session.with_winlinks(|links| winlink_find_by_window(links, &window).is_alive())
        }) {
            n = n.wrapping_add(1);
        }
        sg = session_groups_next(&*sg);
    }
    let mut s_owner = sessions_minmax(&sessions);
    s = s_owner.clone();
    while !s.is_none() {
        if crate::src::session::session_group_for(&Rc::downgrade(
            s_owner.as_ref().expect("registered session"),
        ))
        .is_null()
        {
            if s_owner
                .as_ref()
                .expect("registered session")
                .with_winlinks(|links| winlink_find_by_window(links, &window).is_alive())
            {
                n = n.wrapping_add(1);
            }
        }
        s_owner = s.as_ref().expect("live session").next_session();
        s = s_owner.clone();
    }
    window.release(c"format linked sessions");
    return Some(
        CString::new(format!("{}", (n) as u32)).expect("formatted numbers contain no NUL"),
    );
}
unsafe fn format_cb_window_marked_flag(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).winlink_handle().is_alive() {
        if server_check_marked() != 0 && marked_pane.winlink_handle() == (*ft).winlink_handle() {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_modal_pane(ft: *mut format_tree) -> Option<CString> {
    let window = (*ft).w.upgrade()?;
    let value = window
        .modal_pane()
        .map(|modal| CString::new(format!("%{}", modal.id())).expect("formatted pane ID"));
    window.release(c"format modal window pane");
    value
}

unsafe fn format_cb_window_name(ft: *mut format_tree) -> Option<CString> {
    let window = (*ft).w.upgrade()?;
    let value = window.name();
    window.release(c"format window name");
    Some(value)
}

unsafe fn format_cb_window_offset_x(ft: *mut format_tree) -> Option<CString> {
    let format_client_owner = (*ft).c.upgrade();
    let mut format_client: Option<ClientRef> = format_client_owner.clone();
    let c = format_client.as_ref()?;
    let view = c.terminal_view();
    view.bigger
        .then(|| CString::new(view.ox.to_string()).expect("formatted number contains no NUL"))
}
unsafe fn format_cb_window_offset_y(ft: *mut format_tree) -> Option<CString> {
    let format_client_owner = (*ft).c.upgrade();
    let mut format_client: Option<ClientRef> = format_client_owner.clone();
    let c = format_client.as_ref()?;
    let view = c.terminal_view();
    view.bigger
        .then(|| CString::new(view.oy.to_string()).expect("formatted number contains no NUL"))
}
unsafe fn format_cb_window_panes(ft: *mut format_tree) -> Option<CString> {
    let window = (*ft).w.upgrade()?;
    let value = CString::new(window.pane_count(true).to_string())
        .expect("formatted number contains no NUL");
    window.release(c"format window_panes");
    Some(value)
}

unsafe fn format_cb_window_raw_flags(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).winlink_handle().is_alive() {
        return Some(window_printable_flags(
            ((*ft).winlink_handle()).clone(),
            0 as ::core::ffi::c_int,
        ));
    }
    return None;
}
unsafe fn format_cb_window_silence_flag(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).winlink_handle().is_alive() {
        if ((*ft).winlink_handle()).get_unchecked().flags & WINLINK_SILENCE != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_start_flag(mut ft: *mut format_tree) -> Option<CString> {
    if (*ft).winlink_handle().is_alive() {
        let session_owner = ((*ft).winlink_handle()).get_unchecked().session.upgrade()?;
        if (*ft).winlink_handle()
            == session_owner.with_winlinks(|links| winlinks_minmax(links, RB_NEGINF))
        {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_window_width(ft: *mut format_tree) -> Option<CString> {
    let window = (*ft).w.upgrade()?;
    let value =
        CString::new(window.size().0.to_string()).expect("formatted number contains no NUL");
    window.release(c"format window_width");
    Some(value)
}

unsafe fn format_cb_window_manual_width(ft: *mut format_tree) -> Option<CString> {
    let window = (*ft).w.upgrade()?;
    let manual = window.with_options_mut(|options| {
        options_get_number(options, c"window-size".as_ptr()) == WINDOW_SIZE_MANUAL as _
    });
    let value = if manual {
        CString::new(window.manual_size().0.to_string()).expect("formatted number")
    } else {
        c"".to_owned()
    };
    window.release(c"format manual window width");
    Some(value)
}
unsafe fn format_cb_window_zoomed_flag(ft: *mut format_tree) -> Option<CString> {
    let window = (*ft).w.upgrade()?;
    let value = if window.is_zoomed() { c"1" } else { c"0" }.to_owned();
    window.release(c"format window zoom flag");
    Some(value)
}

unsafe fn format_cb_wrap_flag(mut ft: *mut format_tree) -> Option<CString> {
    let format_pane_owner = (*ft).wp.upgrade();
    let format_pane = format_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !format_pane.is_null() {
        if (*format_pane).base.mode & MODE_WRAP != 0 {
            return Some(c"1".to_owned());
        }
        return Some(c"0".to_owned());
    }
    return None;
}
unsafe fn format_cb_buffer_created(ft: *mut format_tree) -> Option<time_t> {
    Some(paste_buffer_created(&*(*ft).pb.as_ref()?.try_borrow()?) as __time_t as time_t)
}
unsafe fn format_cb_client_activity(ft: *mut format_tree) -> Option<time_t> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_activity", &mut *ft)?
    {
        FormatValue::Time(value) => Some(value),
        FormatValue::String(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_client_created(ft: *mut format_tree) -> Option<time_t> {
    match (*ft)
        .c
        .upgrade()?
        .format_value(c"client_created", &mut *ft)?
    {
        FormatValue::Time(value) => Some(value),
        FormatValue::String(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_activity(ft: *mut format_tree) -> Option<time_t> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_activity", &mut *ft)?
    {
        FormatValue::Time(value) => Some(value),
        FormatValue::String(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_created(ft: *mut format_tree) -> Option<time_t> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_created", &mut *ft)?
    {
        FormatValue::Time(value) => Some(value),
        FormatValue::String(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_session_last_attached(ft: *mut format_tree) -> Option<time_t> {
    match (*ft)
        .s
        .upgrade()?
        .format_value(c"session_last_attached", &mut *ft)?
    {
        FormatValue::Time(value) => Some(value),
        FormatValue::String(_) => unreachable!("builtin value type"),
    }
}
unsafe fn format_cb_start_time(_ft: *mut format_tree) -> Option<time_t> {
    return Some((start_time).tv_sec as time_t);
}
unsafe fn format_cb_window_activity(ft: *mut format_tree) -> Option<time_t> {
    let window = (*ft).w.upgrade()?;
    let value = window.activity_time().tv_sec as time_t;
    window.release(c"format window activity");
    Some(value)
}

unsafe fn format_cb_buffer_mode_format(_ft: *mut format_tree) -> Option<CString> {
    return Some(
        window_buffer_mode
            .default_format
            .expect("mode default format")
            .to_owned(),
    );
}
unsafe fn format_cb_client_mode_format(_ft: *mut format_tree) -> Option<CString> {
    return Some(
        window_client_mode
            .default_format
            .expect("mode default format")
            .to_owned(),
    );
}
unsafe fn format_cb_tree_mode_format(_ft: *mut format_tree) -> Option<CString> {
    return Some(
        window_tree_mode
            .default_format
            .expect("mode default format")
            .to_owned(),
    );
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
pub enum FormatValue {
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
        use crate::src::server_client::Client;
        use crate::src::session::Session;
        use crate::src::window::{Window, WindowPane};
        // Preserve each callback's absent-context behavior. Static mode-format
        // defaults do not belong to a particular holder.
        if model_key(self.key, b"session_") {
            if let Some(owner) = (*ft).s.upgrade() {
                return owner.format_value(self.key, &mut *ft);
            }
        } else if model_key(self.key, b"window_") {
            if let Some(owner) = (*ft).w.upgrade() {
                let result = owner.format_value(self.key, &mut *ft);
                crate::src::window::window_remove_ref(owner, c"format builtin".as_ptr());
                return result;
            }
        } else if model_key(self.key, b"pane_") {
            if let Some(owner) = (*ft).wp.upgrade() {
                return owner.format_value(self.key, &mut *ft);
            }
        } else if model_key(self.key, b"client_") {
            if let Some(owner) = (*ft).c.upgrade() {
                return owner.format_value(self.key, &mut *ft);
            }
        }
        self.evaluate(ft)
    }

    unsafe fn evaluate(&self, ft: *mut format_tree) -> Option<FormatValue> {
        match self.callback {
            FormatCallback::String(cb) => cb(ft).map(FormatValue::String),
            FormatCallback::Time(cb) => cb(ft).map(FormatValue::Time),
        }
    }
}

fn model_key(key: &CStr, prefix: &[u8]) -> bool {
    key.to_bytes().starts_with(prefix) && !key.to_bytes().ends_with(b"_mode_format")
}

// These adapters preserve lazy builtin evaluation. Callback bodies remain the
// legacy implementation while their field accesses migrate to entity owners.

pub(crate) unsafe fn window_format_value(
    owner: &WindowRef,
    key: &CStr,
    context: &mut format_tree,
) -> Option<FormatValue> {
    if !context.w.ptr_eq(&Rc::downgrade(owner)) || !model_key(key, b"window_") {
        return None;
    }
    format_table_get(key)?.evaluate(context)
}

pub(crate) unsafe fn pane_format_value(
    owner: &Rc<UnsafeCell<window_pane>>,
    key: &CStr,
    context: &mut format_tree,
) -> Option<FormatValue> {
    if !context.wp.ptr_eq(&Rc::downgrade(owner)) || !model_key(key, b"pane_") {
        return None;
    }
    format_table_get(key)?.evaluate(context)
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
    fn pane_formats_observe_context_and_getter_retains_selected_pane() {
        unsafe {
            let owner = window_pane::new();
            (*owner.get()).id = 42;
            (*owner.get()).sx = 80;
            let observer = std::rc::Rc::downgrade(&owner);
            let mut ft_owner = format_create(None, None, 0, 0);
            let ft = &raw mut *ft_owner;
            super::super::format_defaults_pane(ft, &owner.clone());
            assert!(observer.ptr_eq(&(*ft).wp));
            assert_eq!(format_cb_pane_id(ft).unwrap().as_c_str(), c"%42");
            assert_eq!(format_cb_pane_width(ft).unwrap().as_c_str(), c"80");
            let selected = super::super::format_get_pane(&*ft).unwrap();
            drop(owner);
            assert!(observer.upgrade().is_some());
            drop(selected);
            assert!(observer.upgrade().is_none());
            assert!(super::super::format_get_pane(&*ft).is_none());
            assert!(format_cb_pane_id(ft).is_none());
            assert!(format_cb_pane_width(ft).is_none());
            format_free(ft_owner);
        }
    }

    #[test]
    fn window_formats_observe_context_window_and_allow_clearing_it() {
        unsafe {
            let owner = window::new();
            owner.initialize_name(c"observed-window".to_owned(), false);
            let observer = std::rc::Rc::downgrade(&owner);
            let mut ft_owner = format_create(None, None, 0, 0);
            let ft = &raw mut *ft_owner;
            super::super::format_defaults_window(ft, Some(&owner));
            assert!(observer.ptr_eq(&(*ft).w));
            assert_eq!(
                format_cb_window_name(ft).unwrap().as_c_str(),
                c"observed-window"
            );
            super::super::format_defaults_window(ft, None);
            assert!(format_cb_window_name(ft).is_none());
            super::super::format_defaults_window(ft, Some(&owner));
            crate::src::window::window_remove_ref(owner, c"test owner".as_ptr());
            assert!(observer.upgrade().is_none());
            assert!(format_cb_window_name(ft).is_none());
            assert!(format_cb_window_id(ft).is_none());
            format_free(ft_owner);
        }
    }

    #[test]
    fn client_formats_observe_context_client_without_retaining_it() {
        unsafe {
            let owner = client::with_names_for_test(Some(c"observed-client"), None);
            let observer = std::rc::Rc::downgrade(&owner);
            let mut ft_owner = format_create(None, None, 0, 0);
            let ft = &raw mut *ft_owner;
            (*ft).c = observer.clone();
            assert_eq!(
                format_cb_client_name(ft).unwrap().as_c_str(),
                c"observed-client"
            );
            drop(owner);
            assert!(observer.upgrade().is_none());
            assert!(format_cb_client_name(ft).is_none());
            assert!(format_cb_client_width(ft).is_none());
            format_free(ft_owner);
        }
    }

    #[test]
    fn buffer_formats_preserve_missing_empty_and_logical_binary_lengths() {
        let owner = refbox::RefBox::new(crate::src::shared::paste::paste_buffer::empty());
        let buffer = crate::src::shared::paste::PasteBufferRef::observe(&owner);
        unsafe {
            let mut ft_owner = format_create(None, None, 0, 0);
            let ft = &raw mut *ft_owner;
            (*ft).pb = Some(buffer.clone());
            assert!(paste_buffer_data(&buffer.borrow()).is_none());
            assert!(format_cb_buffer_full(ft).is_none());
            assert_eq!(format_cb_buffer_size(ft).unwrap().as_c_str(), c"0");

            buffer.borrow_mut().data = Some(Box::default());
            assert_eq!(paste_buffer_data(&buffer.borrow()), Some(&b""[..]));
            assert_eq!(format_cb_buffer_full(ft).unwrap().as_c_str(), c"");

            buffer.borrow_mut().data = Some(b"A\xff\0B".to_vec().into_boxed_slice());
            {
                let value = buffer.borrow();
                let bytes = paste_buffer_data(&value).unwrap();
                assert_eq!(bytes, b"A\xff\0B");
                assert_eq!(bytes.as_ptr(), value.data.as_ref().unwrap().as_ptr());
            }
            assert_eq!(format_cb_buffer_full(ft).unwrap().as_bytes(), b"A\xff");
            assert_eq!(format_cb_buffer_size(ft).unwrap().as_c_str(), c"4");

            buffer.borrow_mut().data = Some(b"A".to_vec().into_boxed_slice());
            assert_eq!(format_cb_buffer_full(ft).unwrap().as_c_str(), c"A");
            assert_eq!(format_cb_buffer_size(ft).unwrap().as_c_str(), c"1");
            (*ft).pb = None;
            format_free(ft_owner);
        }
    }

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
            let mut ft_owner = format_create(None, None, 0, 0);
            let ft = &raw mut *ft_owner;
            for key in [
                c"buffer_full",
                c"client_created",
                c"session_created",
                c"pane_dead_time",
            ] {
                assert!(format_table_get(key).unwrap().get(ft).is_none());
            }
            format_free(ft_owner);
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
                    Some(name),
                    Some(&mut cause)
                ),
                0
            );
            let pb = paste_get_name(name).unwrap();
            let mut ft_owner = format_create(None, None, 0, 0);
            let ft = &raw mut *ft_owner;
            (*ft).pb = Some(pb.clone());
            let full = format_cb_buffer_full(ft).unwrap();
            let sample = format_cb_buffer_sample(ft).unwrap();
            let created = format_cb_buffer_created(ft).unwrap();
            let old_created = pb.borrow().created;
            pb.borrow_mut().created = 0;
            assert_eq!(format_cb_buffer_created(ft), Some(0));
            let weak = pb.clone();
            paste_free(&pb);
            drop(pb);
            assert!(!weak.is_alive(), "format tree only observes the buffer");
            assert!(format_cb_buffer_full(ft).is_none());
            format_free(ft_owner);
            assert!(!weak.is_alive());
            assert_eq!(full.as_bytes(), b"A\xff");
            assert!(!sample.as_bytes().is_empty());
            assert_eq!(created, old_created);
        }
    }
}
