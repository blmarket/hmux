use crate::src::options::options_owner_ptr;
use crate::src::arguments::{
    args_count, args_flag_values, args_get, args_has, args_string, args_to_vector,
};
use crate::src::cmd::find::{cmd_find_from_pane, cmd_find_from_winlink_pane};
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_state_owned, cmdq_get_event, cmdq_get_target, cmdq_get_target_client,
    cmdq_insert_hook, cmdq_print,
};
use crate::src::cmd::{cmd_get_args_mut, cmd_get_entry};
use crate::src::environ::{environ_create, environ_put};
use crate::src::events::events_fire;
use crate::src::events_payload::{
    event_payload_create, event_payload_set_pane, event_payload_set_string,
    event_payload_set_target, event_payload_set_window,
};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{format_single_cstring, format_single_from_target_cstring};
use crate::src::layout::{
    layout_close_pane, layout_fix_panes, layout_get_floating_cell, layout_get_tiled_cell,
    layout_set_size,
};
use crate::src::options::{
    options_find_choice, options_search, options_set_number, options_set_string,
};
use crate::src::screen::screen_set_title;
use crate::src::server_client::server_client_remove_pane;
use crate::src::server_fn::{
    server_redraw_session, server_redraw_window, server_redraw_window_borders,
};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse, args_value};
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::environment::environ;
use crate::src::shared::events::event_payload;
use crate::src::shared::key::key_event;
use crate::src::shared::layout::layout_cell;
use crate::src::shared::layout::*;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options_table_entry;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{
    PANE_CAPTUREALLKEYS, PANE_CLOSEONCANCEL, PANE_CLOSEONCLICK, PANE_MINIMUM, PANE_REDRAW,
    PANE_STYLECHANGED, PANE_THEMECHANGED,
};
use crate::src::shared::session::session;
use crate::src::shared::spawn::spawn_context;
use crate::src::shared::spawn::{
    SPAWN_BEFORE, SPAWN_DETACHED, SPAWN_EMPTY, SPAWN_FLOATING, SPAWN_FLOATOVERZOOM, SPAWN_FULLSIZE,
    SPAWN_HORIZONTAL, SPAWN_MODAL, SPAWN_SPLIT, SPAWN_ZOOM,
};
use crate::src::shared::window::WINDOW_ZOOMED;
use crate::src::shared::window::{window, winlink};
use crate::src::spawn::spawn_pane;
use crate::src::window::{
    window_active_pane_is_over_zoom, window_get_pane_lines, window_pane_find_by_id,
    window_pane_get_pane_lines, window_pane_is_floating, window_pane_is_visible,
    window_pane_start_input, window_pop_zoom, window_remove_pane, window_unzoom,
};

pub const SPLIT_WINDOW_TEMPLATE: [::core::ffi::c_char; 46] = unsafe {
    ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
        *b"#{session_name}:#{window_index}.#{pane_index}\0",
    )
};
pub static cmd_new_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"new-pane",
        alias: Some(c"newp"),
        args: args_parse {
            template: c"AbB:Cc:Dde:EfF:hIkl:KLMm:Op:PR:s:S:t:T:vWx:X:y:Y:Z",
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: c"[-AbCDefhIkKLMOPvWZ] [-B border-lines] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-T title] [-x width] [-y height] [-X x-position] [-Y y-position] [-t target-pane] [shell-command [argument ...]]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_split_window_exec
                as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
pub static cmd_split_window_entry: cmd_entry = {
    cmd_entry {
        name: c"split-window",
        alias: Some(c"splitw"),
        args: args_parse {
            template: c"bB:c:de:EfF:hIkl:m:p:PR:s:S:t:T:vWZ",
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: c"[-bdefhIklPvWZ] [-B border-lines] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-T title] [-t target-pane] [shell-command [argument ...]]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_split_window_exec
                as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe fn cmd_split_window_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let current = cmdq_get_state_owned(item);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut sc: spawn_context = spawn_context {
        item: ::core::ptr::null_mut::<cmdq_item>(),
        s: None,
        wl: ::core::ptr::null_mut::<winlink>(),
        tc: None,
        wp0: None,
        lc: ::core::ptr::null_mut::<layout_cell>(),
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        argv: Vec::new(),
        environ: None,
        idx: 0,
        cwd: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
    };
    let mut argv_owner = Vec::new();
    let tc_owner = cmdq_get_target_client(item);
    let mut tc: *mut client = tc_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut s: *mut session = (*target).s_ptr();
    let mut wl: *mut winlink = (*target).wl_ptr();
    let mut w: *mut window = (*wl).window_ptr();
    let mut wp: *mut window_pane = (*target).wp_ptr();
    let mut new_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut event_snapshot = cmdq_get_event(item);
    let event: *mut key_event = &mut event_snapshot;
    let mut input: ::core::ffi::c_int = 0;
    let mut empty: ::core::ffi::c_int = 0;
    let mut is_floating: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut restore_zoom: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut style: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cause: Option<std::ffi::CString> = None;
    let mut choice_cause: Option<std::ffi::CString> = None;
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut lines: pane_lines = PANE_LINES_SINGLE;
    let mut count: u_int = args_count(args);
    if window_active_pane_is_over_zoom(w) != 0 {
        restore_zoom = 1 as ::core::ffi::c_int;
    }
    if std::ptr::eq(cmd_get_entry(&*self_0), &cmd_new_pane_entry) {
        is_floating = (args_has(args, 'L' as i32 as u_char) == 0) as ::core::ffi::c_int;
    } else {
        if window_pane_is_visible(wp) == 0 {
            restore_zoom = 0 as ::core::ffi::c_int;
        }
        if restore_zoom == 0 {
            window_unzoom(w, 1 as ::core::ffi::c_int);
        }
        is_floating = window_pane_is_floating(&*wp);
        flags |= SPAWN_SPLIT;
    }
    if args_has(args, 'O' as i32 as u_char) != 0 {
        if is_floating == 0 {
            cmdq_error(item, |out| out.write_all(b"modal pane must be floating"));
            return CMD_RETURN_ERROR;
        }
        if (*w).modal.upgrade().is_some() {
            cmdq_error(item, |out| {
                out.write_all(b"window already has a modal pane")
            });
            return CMD_RETURN_ERROR;
        }
    }
    if args_has(args, 'M' as i32 as u_char) != 0 && is_floating != 0 {
        if event.is_null() || (*event).m.valid == 0 || tc.is_null() {
            return CMD_RETURN_NORMAL;
        }
    }
    if is_floating != 0 {
        flags |= SPAWN_FLOATING;
    }
    if args_has(args, 'h' as i32 as u_char) != 0 {
        flags |= SPAWN_HORIZONTAL;
    }
    if args_has(args, 'b' as i32 as u_char) != 0 {
        flags |= SPAWN_BEFORE;
    }
    if args_has(args, 'f' as i32 as u_char) != 0 {
        flags |= SPAWN_FULLSIZE;
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        flags |= SPAWN_DETACHED;
    }
    if args_has(args, 'Z' as i32 as u_char) != 0 {
        flags |= SPAWN_ZOOM;
    }
    if args_has(args, 'O' as i32 as u_char) != 0 {
        flags |= SPAWN_MODAL | SPAWN_FLOATOVERZOOM;
    }
    if is_floating != 0 && args_has(args, 'A' as i32 as u_char) != 0 {
        flags |= SPAWN_FLOATOVERZOOM;
    }
    if (*w).flags & WINDOW_ZOOMED != 0 && flags & SPAWN_FLOATOVERZOOM != 0 {
        restore_zoom = 1 as ::core::ffi::c_int;
    }
    input = args_has(args, 'I' as i32 as u_char);
    if input != 0
        || count == 1 as u_int
            && *args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr()) as ::core::ffi::c_int == '\0' as i32
    {
        empty = 1 as ::core::ffi::c_int;
    } else {
        empty = args_has(args, 'E' as i32 as u_char);
    }
    if empty != 0
        && count != 0 as u_int
        && (count != 1 as u_int
            || *args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr()) as ::core::ffi::c_int != '\0' as i32)
    {
        cmdq_error(item, |out| {
            out.write_all(b"command cannot be given for empty pane")
        });
        return CMD_RETURN_ERROR;
    }
    if empty != 0 {
        flags |= SPAWN_EMPTY;
    }
    value = args_get(&*(args), 'B' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    if value.is_null() {
        lines = window_get_pane_lines(w);
    } else {
        oe = options_search(b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char).map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry);
        lines = options_find_choice(oe, value, &raw mut choice_cause) as pane_lines;
        if let Some(cause) = choice_cause.as_ref() {
            cmdq_error(item, |out| {
                out.write_all(b"pane-border-lines ")?;
                write_cstr(out, cause.as_ptr())
            });
            return CMD_RETURN_ERROR;
        }
    }
    if flags & SPAWN_FLOATING != 0 {
        lc = match layout_get_floating_cell(item, args, lines, w, wp, flags) {
            Ok(cell) => cell,
            Err(error) => {
                cmdq_error(item, |out| write_cstr(out, error.as_ptr()));
                if restore_zoom != 0 {
                    window_pop_zoom(w);
                }
                return CMD_RETURN_ERROR;
            }
        };
    } else {
        lc = match layout_get_tiled_cell(item, args, w, wp, flags) {
            Ok(cell) => cell,
            Err(error) => {
                cmdq_error(item, |out| write_cstr(out, error.as_ptr()));
                if restore_zoom != 0 {
                    window_pop_zoom(w);
                }
                return CMD_RETURN_ERROR;
            }
        };
    }
    sc.item = item;
    sc.s = (*s).observer.upgrade();
    sc.wl = wl;
    sc.wp0 = (*wp).observer.upgrade();
    sc.lc = lc;
    argv_owner = args_to_vector(&*args);
    sc.argv = argv_owner;
    sc.environ = Some(environ_create());
    for av in args_flag_values(&*args, 'e' as i32 as u_char) {
        environ_put(
            sc.environ.as_deref_mut().expect("environment"),
            av.string_ptr(),
            0 as ::core::ffi::c_int,
        );
    }
    sc.idx = -(1 as ::core::ffi::c_int);
    sc.cwd = args_get(&*(args), 'c' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    sc.flags = flags;
    new_wp = spawn_pane(&raw mut sc, &raw mut cause);
    if new_wp.is_null() {
        cmdq_error(item, |out| {
            out.write_all(b"create pane failed: ")?;
            write_cstr(
                out,
                cause
                    .as_ref()
                    .map_or(::core::ptr::null(), |value| value.as_ptr()),
            )
        });
    } else {
        if args_has(args, 'K' as i32 as u_char) != 0 && args_has(args, 'O' as i32 as u_char) != 0 {
            (*new_wp).flags |= PANE_CAPTUREALLKEYS;
        }
        if args_has(args, 'C' as i32 as u_char) != 0 && args_has(args, 'O' as i32 as u_char) != 0 {
            (*new_wp).flags |= PANE_CLOSEONCLICK;
        }
        if args_has(args, 'D' as i32 as u_char) != 0 && args_has(args, 'O' as i32 as u_char) != 0 {
            (*new_wp).flags |= PANE_CLOSEONCANCEL;
        }
        style = args_get(&*(args), 's' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
        if !style.is_null() {
            if options_set_string(
                options_owner_ptr(&mut (*new_wp).options).map_or(std::ptr::null_mut(), |options| options),
                b"window-style\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
                |out| write_cstr(out, style),
            )
            .is_null()
            {
                cmdq_error(item, |out| {
                    out.write_all(b"bad style: ")?;
                    write_cstr(out, style)
                });
                current_block = 9814746494299271243;
            } else {
                options_set_string(
                    options_owner_ptr(&mut (*new_wp).options).map_or(std::ptr::null_mut(), |options| options),
                    b"window-active-style\0" as *const u8 as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    |out| write_cstr(out, style),
                );
                (*new_wp).flags |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
                current_block = 14329534724295951598;
            }
        } else {
            current_block = 14329534724295951598;
        }
        match current_block {
            9814746494299271243 => {}
            _ => {
                style = args_get(&*(args), 'S' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
                if !style.is_null() {
                    if options_set_string(
                        options_owner_ptr(&mut (*new_wp).options).map_or(std::ptr::null_mut(), |options| options),
                        b"pane-active-border-style\0" as *const u8 as *const ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                        |out| write_cstr(out, style),
                    )
                    .is_null()
                    {
                        cmdq_error(item, |out| {
                            out.write_all(b"bad active border style: ")?;
                            write_cstr(out, style)
                        });
                        current_block = 9814746494299271243;
                    } else {
                        current_block = 12070711452894729854;
                    }
                } else {
                    current_block = 12070711452894729854;
                }
                match current_block {
                    9814746494299271243 => {}
                    _ => {
                        style = args_get(&*(args), 'R' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
                        if !style.is_null() {
                            if options_set_string(
                                options_owner_ptr(&mut (*new_wp).options).map_or(std::ptr::null_mut(), |options| options),
                                b"pane-border-style\0" as *const u8 as *const ::core::ffi::c_char,
                                0 as ::core::ffi::c_int,
                                |out| write_cstr(out, style),
                            )
                            .is_null()
                            {
                                cmdq_error(item, |out| {
                                    out.write_all(b"bad inactive border style: ")?;
                                    write_cstr(out, style)
                                });
                                current_block = 9814746494299271243;
                            } else {
                                current_block = 16313536926714486912;
                            }
                        } else {
                            current_block = 16313536926714486912;
                        }
                        match current_block {
                            9814746494299271243 => {}
                            _ => {
                                if args_has(args, 'B' as i32 as u_char) != 0 {
                                    options_set_number(
                                        options_owner_ptr(&mut (*new_wp).options).map_or(std::ptr::null_mut(), |options| options),
                                        b"pane-border-lines\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        lines as ::core::ffi::c_longlong,
                                    );
                                }
                                if args_has(args, 'k' as i32 as u_char) != 0
                                    || args_has(args, 'm' as i32 as u_char) != 0
                                {
                                    options_set_number(
                                        options_owner_ptr(&mut (*new_wp).options).map_or(std::ptr::null_mut(), |options| options),
                                        b"remain-on-exit\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        3 as ::core::ffi::c_longlong,
                                    );
                                    if args_has(args, 'm' as i32 as u_char) != 0 {
                                        options_set_string(
                                            options_owner_ptr(&mut (*new_wp).options).map_or(std::ptr::null_mut(), |options| options),
                                            b"remain-on-exit-format\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            0 as ::core::ffi::c_int,
                                            |out| {
                                                write_cstr(
                                                    out,
                                                    args_get(&*(args), 'm' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()),
                                                )
                                            },
                                        );
                                    }
                                }
                                if args_has(args, 'T' as i32 as u_char) != 0 {
                                    let title = format_single_from_target_cstring(
                                        item,
                                        args_get(&*(args), 'T' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()),
                                    );
                                    screen_set_title(
                                        &mut (*new_wp).base,
                                        &title,
                                        0 as ::core::ffi::c_int,
                                    );
                                    let mut ep = event_payload_create();
                                    cmd_find_from_pane(
                                        &raw mut fs,
                                        new_wp,
                                        0 as ::core::ffi::c_int,
                                    );
                                    event_payload_set_target(&mut *ep, &fs);
                                    event_payload_set_pane(
                                        &mut *ep,
                                        b"pane\0" as *const u8 as *const ::core::ffi::c_char,
                                        new_wp,
                                    );
                                    event_payload_set_window(
                                        &mut *ep,
                                        b"window\0" as *const u8 as *const ::core::ffi::c_char,
                                        (*new_wp).window as *mut window,
                                    );
                                    event_payload_set_string(
                                        &mut *ep,
                                        b"new_title\0" as *const u8 as *const ::core::ffi::c_char,
                                        |out| write_cstr(out, title.as_ptr()),
                                    );
                                    events_fire(
                                        b"pane-title-changed\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        ep,
                                    );
                                }
                                if input != 0 {
                                    match window_pane_start_input(new_wp, item) {
                                        Err(error) => {
                                            cmdq_error(item, |out| write_cstr(out, error.as_ptr()));
                                            current_block = 9814746494299271243;
                                        }
                                        Ok(1) => {
                                            input = 0;
                                            current_block = 12543410360505780601;
                                        }
                                        Ok(_) => current_block = 12543410360505780601,
                                    }
                                } else {
                                    current_block = 12543410360505780601;
                                }
                                match current_block {
                                    9814746494299271243 => {}
                                    _ => {
                                        if !flags & SPAWN_DETACHED != 0 {
                                            cmd_find_from_winlink_pane(
                                                &mut *current.current.borrow_mut(),
                                                wl,
                                                new_wp,
                                                0 as ::core::ffi::c_int,
                                            );
                                        }
                                        if restore_zoom != 0 {
                                            window_pop_zoom((*wp).window as *mut window);
                                            server_redraw_window((*wp).window as *mut window);
                                        } else if !flags & SPAWN_FLOATING != 0
                                            && args_has(args, 'O' as i32 as u_char) == 0
                                        {
                                            window_pop_zoom((*wp).window as *mut window);
                                            server_redraw_window((*wp).window as *mut window);
                                        }
                                        server_redraw_session(s);
                                        if args_has(args, 'M' as i32 as u_char) != 0
                                            && is_floating != 0
                                        {
                                            (*tc).tty.mouse_last_pane =
                                                (*new_wp).id as ::core::ffi::c_int;
                                            let drag_client = std::ptr::NonNull::new(tc)
                                                .expect("live drag client");
                                            (*tc).tty.mouse_drag_update =
                                                Some(Box::new(move |m| unsafe {
                                                    cmd_split_window_mouse_resize(
                                                        drag_client.as_ptr(),
                                                        m as *mut mouse_event,
                                                    )
                                                }));
                                            cmd_split_window_mouse_resize(tc, &raw mut (*event).m);
                                        }
                                        if args_has(args, 'P' as i32 as u_char) != 0 {
                                            template = args_get(&*(args), 'F' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
                                            if template.is_null() {
                                                template = SPLIT_WINDOW_TEMPLATE.as_ptr();
                                            }
                                            let cp = format_single_cstring(
                                                item, template, tc, s, wl, new_wp,
                                            );
                                            cmdq_print(item, |out| write_cstr(out, cp.as_ptr()));
                                        }
                                        cmd_find_from_winlink_pane(
                                            &raw mut fs,
                                            wl,
                                            new_wp,
                                            0 as ::core::ffi::c_int,
                                        );
                                        cmdq_insert_hook(s, item, &raw mut fs, |out| {
                                            out.write_all(b"after-split-window")
                                        });
                                        drop(sc.environ.take());
                                        if input != 0 {
                                            return CMD_RETURN_WAIT;
                                        }
                                        if args_has(args, 'W' as i32 as u_char) != 0 {
                                            (*new_wp).wait_item = item;
                                            return CMD_RETURN_WAIT;
                                        }
                                        return CMD_RETURN_NORMAL;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !new_wp.is_null() {
        server_client_remove_pane(new_wp);
        if is_floating == 0 {
            layout_close_pane(new_wp);
        }
        window_remove_pane((*wp).window as *mut window, new_wp);
    }
    if restore_zoom != 0 || !flags & SPAWN_FLOATING != 0 {
        window_pop_zoom((*wp).window as *mut window);
    }
    drop(sc.environ.take());
    return CMD_RETURN_ERROR;
}
unsafe fn cmd_split_window_mouse_resize(mut c: *mut client, mut m: *mut mouse_event) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lines: pane_lines = PANE_LINES_SINGLE;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut border: ::core::ffi::c_int = 0;
    if (*c).tty.mouse_last_pane == -(1 as ::core::ffi::c_int) {
        return;
    }
    let lookup_wp_owner = window_pane_find_by_id((*c).tty.mouse_last_pane as u_int);
    wp = lookup_wp_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() || window_pane_is_floating(&*wp) == 0 {
        (*c).tty.mouse_drag_update = None;
        return;
    }
    w = (*wp).window as *mut window;
    lc = (*wp).layout_cell as *mut layout_cell;
    x = (*m).x.wrapping_add((*m).ox) as ::core::ffi::c_int;
    y = (*m).y.wrapping_add((*m).oy) as ::core::ffi::c_int;
    if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines as ::core::ffi::c_int {
        y = (y as u_int).wrapping_sub((*m).statuslines) as ::core::ffi::c_int as ::core::ffi::c_int;
    } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat {
        y = (*m).statusat - 1 as ::core::ffi::c_int;
    }
    lines = window_pane_get_pane_lines(wp);
    border = (lines as ::core::ffi::c_uint
        != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    if x >= (*c).tty.mouse_drag_x as ::core::ffi::c_int {
        xoff = (*c).tty.mouse_drag_x.wrapping_add(border as u_int) as ::core::ffi::c_int;
        sx = (x as u_int)
            .wrapping_sub((*c).tty.mouse_drag_x)
            .wrapping_add(1 as u_int);
    } else {
        sx = (*c)
            .tty
            .mouse_drag_x
            .wrapping_sub(x as u_int)
            .wrapping_add(1 as u_int);
        xoff = (*c)
            .tty
            .mouse_drag_x
            .wrapping_sub(sx)
            .wrapping_add(1 as u_int) as ::core::ffi::c_int;
        if border != 0 {
            xoff += 1;
        }
    }
    if y >= (*c).tty.mouse_drag_y as ::core::ffi::c_int {
        yoff = (*c).tty.mouse_drag_y.wrapping_add(border as u_int) as ::core::ffi::c_int;
        sy = (y as u_int)
            .wrapping_sub((*c).tty.mouse_drag_y)
            .wrapping_add(1 as u_int);
    } else {
        sy = (*c)
            .tty
            .mouse_drag_y
            .wrapping_sub(y as u_int)
            .wrapping_add(1 as u_int);
        yoff = (*c)
            .tty
            .mouse_drag_y
            .wrapping_sub(sy)
            .wrapping_add(1 as u_int) as ::core::ffi::c_int;
        if border != 0 {
            yoff += 1;
        }
    }
    if border != 0 {
        if sx <= 2 as u_int {
            sx = PANE_MINIMUM as u_int;
        } else {
            sx = sx.wrapping_sub(2 as u_int);
        }
        if sy <= 2 as u_int {
            sy = PANE_MINIMUM as u_int;
        } else {
            sy = sy.wrapping_sub(2 as u_int);
        }
    }
    if sx < PANE_MINIMUM as u_int {
        sx = PANE_MINIMUM as u_int;
    }
    if sy < PANE_MINIMUM as u_int {
        sy = PANE_MINIMUM as u_int;
    }
    layout_set_size(lc, sx, sy, xoff, yoff);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    server_redraw_window(w);
    server_redraw_window_borders(w);
}
