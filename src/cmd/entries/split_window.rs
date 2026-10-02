use crate::src::arguments::{
    args_count, args_flag_values, args_get, args_has, args_string, args_to_vector,
};
use crate::src::cmd::find::{cmd_find_from_pane, cmd_find_from_winlink_pane};
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_event, cmdq_get_state_owned, cmdq_get_target, cmdq_get_target_client,
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
use crate::src::layout::{layout_close_pane, layout_fix_panes, layout_set_size};
use crate::src::options::{
    options_find_choice, options_search, options_set_number, options_set_string,
};

use crate::src::server_client::Client as _;
use crate::src::server_fn::{
    server_redraw_session, server_redraw_window, server_redraw_window_borders,
};
use crate::src::session::Session;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse, args_value};
use crate::src::shared::client::client;
use crate::src::shared::client::ClientRef;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::environment::environ;
use crate::src::shared::events::event_payload;
use crate::src::shared::key::key_event;
use crate::src::shared::layout::*;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options_table_entry;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_MINIMUM;
use crate::src::shared::session::session;
use crate::src::shared::session::SessionRef;
use crate::src::shared::spawn::spawn_context;
use crate::src::shared::spawn::{
    SPAWN_BEFORE, SPAWN_DETACHED, SPAWN_EMPTY, SPAWN_FLOATING, SPAWN_FLOATOVERZOOM, SPAWN_FULLSIZE,
    SPAWN_HORIZONTAL, SPAWN_MODAL, SPAWN_SPLIT, SPAWN_ZOOM,
};
use crate::src::shared::window::winlink;
use crate::src::window::Window;

use crate::src::window_pane::WindowPane as _;

pub const SPLIT_WINDOW_TEMPLATE: &std::ffi::CStr = c"#{session_name}:#{window_index}.#{pane_index}";
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
        exec: Some(cmd_split_window_exec),
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
        exec: Some(cmd_split_window_exec),
    }
};
unsafe fn cmd_split_window_exec(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let item = item_handle.get();
    let mut args: *mut args =
        cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let current = cmdq_get_state_owned(&*(item));
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut sc: spawn_context = spawn_context {
        item: std::rc::Weak::new(),
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        tc: std::rc::Weak::new(),
        wp0: std::rc::Weak::new(),
        layout: None,
        name: None,
        argv: Vec::new(),
        environ: None,
        idx: 0,
        cwd: None,
        flags: 0,
    };
    let mut argv_owner = Vec::new();
    let tc_owner = cmdq_get_target_client((item).as_ref());
    let mut tc: Option<ClientRef> = tc_owner.clone();
    let mut s: Option<SessionRef> = (*target).session_handle();
    let mut wl: refbox::Weak<winlink> = (*target).winlink_handle();
    let original_window = wl
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("split target window");
    let result = (|| {
        let mut current_block: u64;
        let original_pane = (*target).pane_handle().expect("split target pane");
        if original_window.is_scrolling() {
            if std::ptr::eq(
                cmd_get_entry(self_0.get_unchecked()),
                &cmd_split_window_entry,
            ) {
                cmdq_error(item_handle, |out| {
                    out.write_all(b"cannot split a scrolling layout; use new-pane -L")
                });
                return CMD_RETURN_ERROR;
            }
            if args_has(args, b'L') != 0
                && b"fhvlpxyXY".iter().any(|flag| args_has(args, *flag) != 0)
            {
                cmdq_error(item_handle, |out| {
                    out.write_all(
                        b"scrolling panes start at half width; use resize-pane -W to toggle width",
                    )
                });
                return CMD_RETURN_ERROR;
            }
        }
        let mut fs: cmd_find_state = cmd_find_state {
            flags: 0,
            s: Default::default(),
            wl: Default::default(),
            w: Default::default(),
            wp: Default::default(),
            idx: 0,
        };
        let mut event_snapshot = cmdq_get_event(&*(item));
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
        if std::rc::Rc::clone((wl.get_unchecked().window_handle().as_ref()).expect("live window"))
            .active_pane_over_zoom()
            != 0
        {
            restore_zoom = 1 as ::core::ffi::c_int;
        }
        if std::ptr::eq(cmd_get_entry(self_0.get_unchecked()), &cmd_new_pane_entry) {
            is_floating = (args_has(args, 'L' as i32 as u_char) == 0) as ::core::ffi::c_int;
        } else {
            if !original_pane.is_visible() {
                restore_zoom = 0 as ::core::ffi::c_int;
            }
            if restore_zoom == 0 {
                std::rc::Rc::clone(
                    (wl.get_unchecked().window_handle().as_ref()).expect("live window"),
                )
                .unzoom(true);
            }
            is_floating = original_pane.is_floating() as i32;
            flags |= SPAWN_SPLIT;
        }
        if args_has(args, 'O' as i32 as u_char) != 0 {
            if is_floating == 0 {
                cmdq_error(item_handle, |out| {
                    out.write_all(b"modal pane must be floating")
                });
                return CMD_RETURN_ERROR;
            }
            if original_window.modal_pane().is_some() {
                cmdq_error(item_handle, |out| {
                    out.write_all(b"window already has a modal pane")
                });
                return CMD_RETURN_ERROR;
            }
        }
        if args_has(args, 'M' as i32 as u_char) != 0
            && is_floating != 0
            && (event.is_null() || (*event).m.valid == 0 || tc.is_none())
        {
            return CMD_RETURN_NORMAL;
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
        if original_window.is_zoomed() && flags & SPAWN_FLOATOVERZOOM != 0 {
            restore_zoom = 1 as ::core::ffi::c_int;
        }
        input = args_has(args, 'I' as i32 as u_char);
        if input != 0
            || count == 1 as u_int
                && *args_string(&mut *(args), 0 as u_int)
                    .map_or(std::ptr::null(), |value| value.as_ptr())
                    as ::core::ffi::c_int
                    == '\0' as i32
        {
            empty = 1 as ::core::ffi::c_int;
        } else {
            empty = args_has(args, 'E' as i32 as u_char);
        }
        if empty != 0
            && count != 0 as u_int
            && (count != 1 as u_int
                || *args_string(&mut *(args), 0 as u_int)
                    .map_or(std::ptr::null(), |value| value.as_ptr())
                    as ::core::ffi::c_int
                    != '\0' as i32)
        {
            cmdq_error(item_handle, |out| {
                out.write_all(b"command cannot be given for empty pane")
            });
            return CMD_RETURN_ERROR;
        }
        if empty != 0 {
            flags |= SPAWN_EMPTY;
        }
        value = args_get(&*(args), 'B' as i32 as u_char)
            .map_or(std::ptr::null(), |value| value.as_ptr());
        if value.is_null() {
            lines = original_window.pane_border_lines();
        } else {
            oe = options_search(c"pane-border-lines").map_or(std::ptr::null(), |entry| {
                entry as *const crate::src::shared::options::options_table_entry
            });
            lines = options_find_choice(oe, value, &raw mut choice_cause) as pane_lines;
            if let Some(cause) = choice_cause.as_ref() {
                cmdq_error(item_handle, |out| {
                    out.write_all(b"pane-border-lines ")?;
                    write_cstr(out, cause.as_ptr())
                });
                return CMD_RETURN_ERROR;
            }
        }
        sc.item = std::rc::Rc::downgrade(item_handle);
        sc.s = std::rc::Rc::downgrade(s.as_ref().expect("live session"));
        sc.set_wl(wl.clone());
        sc.wp0 = std::rc::Rc::downgrade(&original_pane);
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
        sc.cwd = args_get(&*(args), 'c' as i32 as u_char).map(|value| value.to_owned());
        sc.flags = flags;
        let window_owner = wl
            .get_unchecked()
            .window_handle()
            .expect("target window")
            .clone();
        let spawned_pane =
            match window_owner.split_pane(&mut sc, &mut *args, lines, restore_zoom != 0, |error| {
                cmdq_error(item_handle, |out| write_cstr(out, error.as_ptr()))
            }) {
                Ok(pane) => Some(pane),
                Err(_) => {
                    window_owner.release(c"cmd_split_window");
                    return CMD_RETURN_ERROR;
                }
            };
        if spawned_pane.is_none() {
            cmdq_error(item_handle, |out| {
                write_cstr(
                    out,
                    cause
                        .as_ref()
                        .map_or(::core::ptr::null(), |value| value.as_ptr()),
                )
            });
        } else {
            let new_pane = spawned_pane.as_ref().expect("spawned pane");
            let modal = args_has(args, b'O') != 0;
            new_pane.configure_modal(
                modal && args_has(args, b'K') != 0,
                modal && args_has(args, b'C') != 0,
                modal && args_has(args, b'D') != 0,
            );
            style = args_get(&*(args), 's' as i32 as u_char)
                .map_or(std::ptr::null(), |value| value.as_ptr());
            if !style.is_null() {
                if new_pane.with_options_mut(|options| {
                    options_set_string(options, c"window-style", 0 as ::core::ffi::c_int, |out| {
                        write_cstr(out, style)
                    })
                    .is_null()
                }) {
                    cmdq_error(item_handle, |out| {
                        out.write_all(b"bad style: ")?;
                        write_cstr(out, style)
                    });
                    current_block = 9814746494299271243;
                } else {
                    new_pane.with_options_mut(|options| {
                        options_set_string(
                            options,
                            c"window-active-style",
                            0 as ::core::ffi::c_int,
                            |out| write_cstr(out, style),
                        )
                        .is_null()
                    });
                    new_pane.invalidate_style();
                    current_block = 14329534724295951598;
                }
            } else {
                current_block = 14329534724295951598;
            }
            match current_block {
                9814746494299271243 => {}
                _ => {
                    style = args_get(&*(args), 'S' as i32 as u_char)
                        .map_or(std::ptr::null(), |value| value.as_ptr());
                    if !style.is_null() {
                        if new_pane.with_options_mut(|options| {
                            options_set_string(
                                options,
                                c"pane-active-border-style",
                                0 as ::core::ffi::c_int,
                                |out| write_cstr(out, style),
                            )
                            .is_null()
                        }) {
                            cmdq_error(item_handle, |out| {
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
                            style = args_get(&*(args), 'R' as i32 as u_char)
                                .map_or(std::ptr::null(), |value| value.as_ptr());
                            if !style.is_null() {
                                if new_pane.with_options_mut(|options| {
                                    options_set_string(
                                        options,
                                        c"pane-border-style",
                                        0 as ::core::ffi::c_int,
                                        |out| write_cstr(out, style),
                                    )
                                    .is_null()
                                }) {
                                    cmdq_error(item_handle, |out| {
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
                                        new_pane.with_options_mut(|options| {
                                            options_set_number(
                                                options,
                                                c"pane-border-lines",
                                                lines as ::core::ffi::c_longlong,
                                            )
                                        });
                                    }
                                    if args_has(args, 'k' as i32 as u_char) != 0
                                        || args_has(args, 'm' as i32 as u_char) != 0
                                    {
                                        new_pane.with_options_mut(|options| {
                                            options_set_number(
                                                options,
                                                c"remain-on-exit",
                                                3 as ::core::ffi::c_longlong,
                                            )
                                        });
                                        if args_has(args, 'm' as i32 as u_char) != 0 {
                                            new_pane.with_options_mut(|options| {
                                                options_set_string(
                                                    options,
                                                    c"remain-on-exit-format",
                                                    0 as ::core::ffi::c_int,
                                                    |out| {
                                                        write_cstr(
                                                            out,
                                                            args_get(
                                                                &*(args),
                                                                'm' as i32 as u_char,
                                                            )
                                                            .map_or(std::ptr::null(), |value| {
                                                                value.as_ptr()
                                                            }),
                                                        )
                                                    },
                                                )
                                                .is_null()
                                            });
                                        }
                                    }
                                    if args_has(args, 'T' as i32 as u_char) != 0 {
                                        let title = format_single_from_target_cstring(
                                            item_handle,
                                            args_get(&*(args), 'T' as i32 as u_char)
                                                .map_or(std::ptr::null(), |value| value.as_ptr()),
                                        );
                                        new_pane.set_title(&title);
                                        let mut ep = event_payload_create();
                                        cmd_find_from_pane(
                                            &raw mut fs,
                                            new_pane,
                                            0 as ::core::ffi::c_int,
                                        );
                                        event_payload_set_target(&mut ep, &fs);
                                        event_payload_set_pane(
                                            &mut ep,
                                            c"pane".as_ptr(),
                                            std::rc::Rc::clone(new_pane),
                                        );
                                        event_payload_set_window(
                                            &mut ep,
                                            c"window".as_ptr(),
                                            new_pane
                                                .window_observer()
                                                .upgrade()
                                                .expect("spawned pane window"),
                                        );
                                        event_payload_set_string(
                                            &mut ep,
                                            c"new_title".as_ptr(),
                                            |out| write_cstr(out, title.as_ptr()),
                                        );
                                        events_fire(c"pane-title-changed".as_ptr(), ep);
                                    }
                                    if input != 0 {
                                        match new_pane.start_input(item_handle) {
                                            Err(error) => {
                                                cmdq_error(item_handle, |out| {
                                                    write_cstr(out, error.as_ptr())
                                                });
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
                                                    wl.clone(),
                                                    new_pane,
                                                    0 as ::core::ffi::c_int,
                                                );
                                            }
                                            if restore_zoom != 0
                                                || (!flags & SPAWN_FLOATING != 0
                                                    && args_has(args, 'O' as i32 as u_char) == 0)
                                            {
                                                original_window.pop_zoom();
                                                server_redraw_window(&original_window);
                                            }
                                            server_redraw_session(
                                                s.as_ref().expect("live session"),
                                            );
                                            if args_has(args, 'M' as i32 as u_char) != 0
                                                && is_floating != 0
                                            {
                                                tc.as_ref()
                                                    .expect("live client")
                                                    .borrow_terminal_mut()
                                                    .mouse_last_pane =
                                                    new_pane.id() as ::core::ffi::c_int;
                                                tc.as_ref()
                                                    .expect("live client")
                                                    .borrow_terminal_mut()
                                                    .mouse_drag_update = Some(Box::new(
                                                    crate::src::tty::tty_mouse_client_callback(
                                                        tc_owner
                                                            .as_ref()
                                                            .expect("live drag client"),
                                                        cmd_split_window_mouse_resize,
                                                    ),
                                                ));
                                                cmd_split_window_mouse_resize(
                                                    tc_owner.as_ref().expect("live drag client"),
                                                    &raw mut (*event).m,
                                                );
                                            }
                                            if args_has(args, 'P' as i32 as u_char) != 0 {
                                                template = args_get(&*(args), 'F' as i32 as u_char)
                                                    .map_or(std::ptr::null(), |value| {
                                                        value.as_ptr()
                                                    });
                                                if template.is_null() {
                                                    template = SPLIT_WINDOW_TEMPLATE.as_ptr();
                                                }
                                                let cp = format_single_cstring(
                                                    Some(item_handle),
                                                    template,
                                                    tc.as_ref(),
                                                    s.as_ref(),
                                                    wl.clone(),
                                                    Some(new_pane),
                                                );
                                                cmdq_print(item_handle, |out| {
                                                    write_cstr(out, cp.as_ptr())
                                                });
                                            }
                                            cmd_find_from_winlink_pane(
                                                &raw mut fs,
                                                wl.clone(),
                                                new_pane,
                                                0 as ::core::ffi::c_int,
                                            );
                                            cmdq_insert_hook(
                                                s.as_ref(),
                                                item_handle,
                                                &raw mut fs,
                                                |out| out.write_all(b"after-split-window"),
                                            );
                                            drop(sc.environ.take());
                                            if input != 0 {
                                                window_owner.release(c"cmd_split_window");
                                                return CMD_RETURN_WAIT;
                                            }
                                            if args_has(args, 'W' as i32 as u_char) != 0 {
                                                new_pane.wait_until_close(item_handle);
                                                window_owner.release(c"cmd_split_window");
                                                return CMD_RETURN_WAIT;
                                            }
                                            window_owner.release(c"cmd_split_window");
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
        if let Some(new_pane) = spawned_pane.as_ref() {
            ClientRef::forget_pane(new_pane);
            if is_floating == 0 {
                layout_close_pane(new_pane);
            }
            original_window.remove_pane(new_pane);
        }
        if restore_zoom != 0 || !flags & SPAWN_FLOATING != 0 {
            original_window.pop_zoom();
        }
        drop(sc.environ.take());
        window_owner.release(c"cmd_split_window");
        CMD_RETURN_ERROR
    })();
    original_window.release(c"split target window");
    result
}
unsafe fn cmd_split_window_mouse_resize(client_owner: &ClientRef, mut m: *mut mouse_event) {
    let _c: Option<ClientRef> = Some(client_owner.clone());
    let mut lines: pane_lines = PANE_LINES_SINGLE;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut border: ::core::ffi::c_int = 0;
    let (last_pane, drag_x, drag_y) = {
        let tty = client_owner.borrow_terminal();
        (tty.mouse_last_pane, tty.mouse_drag_x, tty.mouse_drag_y)
    };
    if last_pane == -(1 as ::core::ffi::c_int) {
        return;
    }
    let lookup_wp_owner =
        std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::find_by_id(last_pane as u_int);

    let Some(pane_owner) = lookup_wp_owner.filter(|pane| pane.is_floating()) else {
        client_owner.borrow_terminal_mut().mouse_drag_update = None;
        return;
    };
    let window_owner = pane_owner
        .window_observer()
        .upgrade()
        .expect("mouse window");

    let cell_id = pane_owner
        .layout_identity(false)
        .expect("resized pane layout");
    x = (*m).x.wrapping_add((*m).ox) as ::core::ffi::c_int;
    y = (*m).y.wrapping_add((*m).oy) as ::core::ffi::c_int;
    if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines as ::core::ffi::c_int {
        y = (y as u_int).wrapping_sub((*m).statuslines) as ::core::ffi::c_int as ::core::ffi::c_int;
    } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat {
        y = (*m).statusat - 1 as ::core::ffi::c_int;
    }
    lines = pane_owner.pane_lines();
    border = (lines as ::core::ffi::c_uint
        != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    if x >= drag_x as ::core::ffi::c_int {
        xoff = drag_x.wrapping_add(border as u_int) as ::core::ffi::c_int;
        sx = (x as u_int).wrapping_sub(drag_x).wrapping_add(1 as u_int);
    } else {
        sx = drag_x.wrapping_sub(x as u_int).wrapping_add(1 as u_int);
        xoff = drag_x.wrapping_sub(sx).wrapping_add(1 as u_int) as ::core::ffi::c_int;
        if border != 0 {
            xoff += 1;
        }
    }
    if y >= drag_y as ::core::ffi::c_int {
        yoff = drag_y.wrapping_add(border as u_int) as ::core::ffi::c_int;
        sy = (y as u_int).wrapping_sub(drag_y).wrapping_add(1 as u_int);
    } else {
        sy = drag_y.wrapping_sub(y as u_int).wrapping_add(1 as u_int);
        yoff = drag_y.wrapping_sub(sy).wrapping_add(1 as u_int) as ::core::ffi::c_int;
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
    {
        let mut cell = window_owner
            .borrow_layout_cell_mut(cell_id)
            .expect("resized pane belongs to layout");
        layout_set_size(&mut *cell, sx, sy, xoff, yoff);
    }
    layout_fix_panes(&window_owner, None);
    server_redraw_window(&window_owner);
    server_redraw_window_borders(&window_owner);
    window_owner.release(c"cmd_split_window_mouse_resize");
}
