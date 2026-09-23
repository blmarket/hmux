use crate::src::cmd::cmd_list_print_cstring;
use crate::src::cmd_find::{
    cmd_find_clear_state, cmd_find_copy_state, cmd_find_empty_state, cmd_find_from_nothing,
    cmd_find_from_pane, cmd_find_from_session, cmd_find_from_winlink, cmd_find_from_winlink_pane,
    cmd_find_valid_state,
};
use crate::src::cmd_parse::cmd_parse_from_string;
use crate::src::cmd_queue::{
    cmdq_add_formats, cmdq_append, cmdq_error, cmdq_free_state, cmdq_get_client, cmdq_get_command,
    cmdq_get_event, cmdq_get_flags, cmdq_get_target, cmdq_insert_after, cmdq_new_state,
    cmdq_running,
};
use crate::src::events::{events_add_sink, events_fire, events_remove_sink};
use crate::src::events_payload::{
    event_payload_add_formats, event_payload_create, event_payload_get_client,
    event_payload_get_pointer, event_payload_get_target, event_payload_set_client,
    event_payload_set_int, event_payload_set_pane, event_payload_set_pointer,
    event_payload_set_session, event_payload_set_string, event_payload_set_target,
    event_payload_set_window,
};
use crate::src::ffi::libc::{free, memset, strcmp};
use crate::src::format::{
    format_add, format_create, format_create_defaults, format_expand, format_free,
    format_log_debug, format_merge,
};
use crate::src::log::{log_debug, log_get_level};
use crate::src::monitor::{
    monitor_add, monitor_create_session, monitor_destroy, monitor_get_fire_count,
    monitor_get_fire_time,
};
use crate::src::options::{
    options_array_first, options_array_item_value, options_array_next, options_get,
    options_get_monitor_data, options_get_only, options_get_string, options_hook_fired,
    options_name, options_search, options_set_monitor_data, options_set_string,
};
use crate::src::options_table::options_table;
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::CMDQ_STATE_NOHOOKS;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmdq_state, cmds,
};
pub use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::events::{
    event_payload, event_payload_free_cb, event_payload_print_cb, events_cb, events_sink,
};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::format::{FORMAT_NOJOBS, FORMAT_NONE};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::monitor::{monitor_cb, monitor_change, monitor_set};
pub use crate::src::shared::monitor::{
    monitor_type, MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_PANE, MONITOR_SESSION,
    MONITOR_WINDOW,
};
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::OPTIONS_TABLE_IS_HOOK;
use crate::src::shared::options::*;
pub use crate::src::shared::options::{
    options, options_array, options_array_item, options_entry, options_table_entry, options_value,
};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::tmux::global_s_options;
use crate::src::xmalloc::{xcalloc, xstrdup};
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct hooks_event {
    pub name: *mut ::core::ffi::c_char,
    pub sink: *mut events_sink,
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub tqe_next: *mut hooks_event,
    pub tqe_prev: *mut *mut hooks_event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hooks_events {
    pub tqh_first: *mut hooks_event,
    pub tqh_last: *mut *mut hooks_event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hooks_data {
    pub name: *const ::core::ffi::c_char,
    pub fs: cmd_find_state,
    pub formats: *mut format_tree,
    pub oo: *mut options,
    pub client: *mut client,
    pub expand: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hooks_monitor {
    pub oo: *mut options,
    pub set: *mut monitor_set,
    pub sink: *mut events_sink,
    pub fs: cmd_find_state,
    pub type_0: monitor_type,
    pub id: ::core::ffi::c_int,
    pub format: *mut ::core::ffi::c_char,
}

static mut hooks_events: hooks_events = hooks_events {
    tqh_first: ::core::ptr::null::<hooks_event>() as *mut hooks_event,
    tqh_last: ::core::ptr::null::<*mut hooks_event>() as *mut *mut hooks_event,
};
unsafe extern "C" fn hooks_insert_one(
    mut item: *mut cmdq_item,
    mut hd: *mut hooks_data,
    mut cmdlist: *mut cmd_list,
    mut state: *mut cmdq_state,
) -> *mut cmdq_item {
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    if cmdlist.is_null() {
        return item;
    }
    if log_get_level() != 0 as ::core::ffi::c_int {
        let s = cmd_list_print_cstring(cmdlist, 0 as ::core::ffi::c_int);
        log_debug(
            b"%s: hook %s is: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"hooks_insert_one\0" as *const u8 as *const ::core::ffi::c_char,
            (*hd).name,
            s.as_ptr(),
        );
    }
    new_item = cmdq_get_command(cmdlist, state);
    if !item.is_null() {
        return cmdq_insert_after(item, new_item);
    }
    return cmdq_append(::core::ptr::null_mut::<client>(), new_item);
}
unsafe extern "C" fn hooks_parse(
    mut hd: *mut hooks_data,
    mut fs: *mut cmd_find_state,
    mut value: *const ::core::ffi::c_char,
) -> *mut cmd_parse_result {
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*hd).expand == 0 {
        return cmd_parse_from_string(value, ::core::ptr::null_mut::<cmd_parse_input>());
    }
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        (*hd).client,
        (*fs).s,
        (*fs).wl,
        (*fs).wp,
    );
    if !(*hd).formats.is_null() {
        format_merge(ft, (*hd).formats);
    }
    expanded = format_expand(ft, value);
    format_free(ft);
    pr = cmd_parse_from_string(expanded, ::core::ptr::null_mut::<cmd_parse_input>());
    free(expanded as *mut ::core::ffi::c_void);
    return pr;
}
unsafe extern "C" fn hooks_insert(mut item: *mut cmdq_item, mut hd: *mut hooks_data) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    log_debug(
        b"%s: inserting hook %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"hooks_insert\0" as *const u8 as *const ::core::ffi::c_char,
        (*hd).name,
    );
    cmd_find_clear_state(&raw mut fs, 0 as ::core::ffi::c_int);
    if cmd_find_empty_state(&raw mut (*hd).fs) != 0 || cmd_find_valid_state(&raw mut (*hd).fs) == 0
    {
        cmd_find_from_nothing(&raw mut fs, 0 as ::core::ffi::c_int);
    } else {
        cmd_find_copy_state(&raw mut fs, &raw mut (*hd).fs);
    }
    if !(*hd).oo.is_null() {
        oo = (*hd).oo;
        o = options_get_only(oo, (*hd).name);
    } else {
        if fs.s.is_null() {
            oo = global_s_options;
        } else {
            oo = (*fs.s).options;
        }
        o = options_get(oo, (*hd).name);
        if o.is_null() && !fs.wp.is_null() {
            oo = (*fs.wp).options;
            o = options_get(oo, (*hd).name);
        }
        if o.is_null() && !fs.wl.is_null() {
            oo = (*(*fs.wl).window).options;
            o = options_get(oo, (*hd).name);
        }
    }
    if o.is_null() {
        log_debug(
            b"%s: hook %s not found\0" as *const u8 as *const ::core::ffi::c_char,
            b"hooks_insert\0" as *const u8 as *const ::core::ffi::c_char,
            (*hd).name,
        );
        return;
    }
    options_hook_fired(o);
    if item.is_null() {
        state = cmdq_new_state(
            &raw mut fs,
            ::core::ptr::null_mut::<key_event>(),
            CMDQ_STATE_NOHOOKS,
        );
    } else {
        state = cmdq_new_state(&raw mut fs, cmdq_get_event(item), CMDQ_STATE_NOHOOKS);
    }
    cmdq_add_formats(state, (*hd).formats);
    if *(*hd).name as ::core::ffi::c_int == '@' as i32 {
        value = options_get_string(oo, (*hd).name);
        pr = hooks_parse(hd, &raw mut fs, value);
        match (*pr).status as ::core::ffi::c_uint {
            0 => {
                log_debug(
                    b"%s: can't parse hook %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    b"hooks_insert\0" as *const u8 as *const ::core::ffi::c_char,
                    (*hd).name,
                    (*pr).error,
                );
                free((*pr).error as *mut ::core::ffi::c_void);
            }
            1 => {
                hooks_insert_one(item, hd, (*pr).cmdlist, state);
            }
            _ => {}
        }
    } else {
        a = options_array_first(o);
        while !a.is_null() {
            if (*hd).expand != 0 {
                value = (*options_array_item_value(a)).string;
                pr = hooks_parse(hd, &raw mut fs, value);
                match (*pr).status as ::core::ffi::c_uint {
                    0 => {
                        if !(*pr).error.is_null() {
                            cmdq_error(
                                item,
                                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                (*pr).error,
                            );
                        }
                    }
                    1 => {
                        item = hooks_insert_one(item, hd, (*pr).cmdlist, state);
                    }
                    _ => {}
                }
            } else {
                cmdlist = (*options_array_item_value(a)).cmdlist;
                item = hooks_insert_one(item, hd, cmdlist, state);
            }
            a = options_array_next(a);
        }
    }
    cmdq_free_state(state);
}
unsafe extern "C" fn hooks_insert_event(
    mut item: *mut cmdq_item,
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut oo: *mut options,
    mut expand: ::core::ffi::c_int,
) {
    let mut hd: hooks_data = hooks_data {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: ::core::ptr::null_mut::<session>(),
            wl: ::core::ptr::null_mut::<winlink>(),
            w: ::core::ptr::null_mut::<window>(),
            wp: ::core::ptr::null_mut::<window_pane>(),
            idx: 0,
        },
        formats: ::core::ptr::null_mut::<format_tree>(),
        oo: ::core::ptr::null_mut::<options>(),
        client: ::core::ptr::null_mut::<client>(),
        expand: 0,
    };
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if !item.is_null() && cmdq_get_flags(item) & CMDQ_STATE_NOHOOKS != 0 {
        return;
    }
    c = event_payload_get_client(ep, b"client\0" as *const u8 as *const ::core::ffi::c_char);
    ft = format_create(c, item, FORMAT_NONE, FORMAT_NOJOBS);
    event_payload_add_formats(
        ep,
        ft,
        b"hook_\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        ft,
        b"hook\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    format_log_debug(
        ft,
        b"hooks_insert_event\0" as *const u8 as *const ::core::ffi::c_char,
    );
    memset(
        &raw mut hd as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<hooks_data>() as size_t,
    );
    hd.name = name;
    cmd_find_clear_state(&raw mut hd.fs, 0 as ::core::ffi::c_int);
    event_payload_get_target(ep, &raw mut hd.fs);
    hd.formats = ft;
    hd.oo = oo;
    hd.client = c;
    hd.expand = expand;
    hooks_insert(item, &raw mut hd);
    format_free(ft);
}
unsafe extern "C" fn hooks_event_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    if !event_payload_get_pointer(
        ep,
        b"_hooks_monitor\0" as *const u8 as *const ::core::ffi::c_char,
    )
    .is_null()
    {
        return;
    }
    item = event_payload_get_pointer(
        ep,
        b"_cmdq_item\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut cmdq_item;
    if !item.is_null() {
        hooks_insert_event(
            item,
            name,
            ep,
            ::core::ptr::null_mut::<options>(),
            0 as ::core::ffi::c_int,
        );
        return;
    }
    item = cmdq_running(::core::ptr::null_mut::<client>());
    if item.is_null() || !cmdq_get_flags(item) & CMDQ_STATE_NOHOOKS != 0 {
        hooks_insert_event(
            ::core::ptr::null_mut::<cmdq_item>(),
            name,
            ep,
            ::core::ptr::null_mut::<options>(),
            0 as ::core::ffi::c_int,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn hooks_add_event(mut name: *const ::core::ffi::c_char) {
    let mut he: *mut hooks_event = ::core::ptr::null_mut::<hooks_event>();
    he = hooks_events.tqh_first;
    while !he.is_null() {
        if strcmp((*he).name, name) == 0 as ::core::ffi::c_int {
            return;
        }
        he = (*he).entry.tqe_next;
    }
    he = xcalloc(1 as size_t, ::core::mem::size_of::<hooks_event>() as size_t) as *mut hooks_event;
    (*he).name = xstrdup(name);
    (*he).sink = events_add_sink(
        name,
        Some(
            hooks_event_cb
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_char,
                    *mut event_payload,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        NULL,
    );
    (*he).entry.tqe_next = ::core::ptr::null_mut::<hooks_event>();
    (*he).entry.tqe_prev = hooks_events.tqh_last;
    *hooks_events.tqh_last = he;
    hooks_events.tqh_last = &raw mut (*he).entry.tqe_next;
}
#[no_mangle]
pub unsafe extern "C" fn hooks_is_event(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut he: *mut hooks_event = ::core::ptr::null_mut::<hooks_event>();
    he = hooks_events.tqh_first;
    while !he.is_null() {
        if strcmp((*he).name, name) == 0 as ::core::ffi::c_int {
            return 1 as ::core::ffi::c_int;
        }
        he = (*he).entry.tqe_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn hooks_valid_event_name(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    if *name as ::core::ffi::c_int == '@' as i32 {
        return 1 as ::core::ffi::c_int;
    }
    oe = options_search(name);
    return (!oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn hooks_build_events() {
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    oe = &raw const options_table as *const options_table_entry;
    while !(*oe).name.is_null() {
        if (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0 {
            hooks_add_event((*oe).name);
        }
        oe = oe.offset(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn hooks_run(mut item: *mut cmdq_item, mut name: *const ::core::ffi::c_char) {
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut hd: hooks_data = hooks_data {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: ::core::ptr::null_mut::<session>(),
            wl: ::core::ptr::null_mut::<winlink>(),
            w: ::core::ptr::null_mut::<window>(),
            wp: ::core::ptr::null_mut::<window_pane>(),
            idx: 0,
        },
        formats: ::core::ptr::null_mut::<format_tree>(),
        oo: ::core::ptr::null_mut::<options>(),
        client: ::core::ptr::null_mut::<client>(),
        expand: 0,
    };
    hd.name = name;
    cmd_find_copy_state(&raw mut hd.fs, target);
    hd.client = cmdq_get_client(item);
    hd.formats = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        0 as ::core::ffi::c_int,
        FORMAT_NOJOBS,
    );
    format_add(
        hd.formats,
        b"hook\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    format_log_debug(
        hd.formats,
        b"hooks_run\0" as *const u8 as *const ::core::ffi::c_char,
    );
    hooks_insert(item, &raw mut hd);
    format_free(hd.formats);
}
#[no_mangle]
pub unsafe extern "C" fn hooks_monitor_free(mut data: *mut ::core::ffi::c_void) {
    let mut hm: *mut hooks_monitor = data as *mut hooks_monitor;
    events_remove_sink((*hm).sink);
    monitor_destroy((*hm).set);
    free((*hm).format as *mut ::core::ffi::c_void);
    free(hm as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn hooks_monitor_remove(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut hm: *mut hooks_monitor = ::core::ptr::null_mut::<hooks_monitor>();
    o = options_get_only(oo, name);
    if o.is_null() {
        return;
    }
    hm = options_get_monitor_data(o) as *mut hooks_monitor;
    if !hm.is_null() {
        options_set_monitor_data(o, NULL);
        hooks_monitor_free(hm as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn hooks_monitor_hook_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut hm: *mut hooks_monitor = sink_data as *mut hooks_monitor;
    if event_payload_get_pointer(
        ep,
        b"_hooks_monitor\0" as *const u8 as *const ::core::ffi::c_char,
    ) == hm as *mut ::core::ffi::c_void
    {
        hooks_insert_event(
            cmdq_running(::core::ptr::null_mut::<client>()),
            name,
            ep,
            (*hm).oo,
            1 as ::core::ffi::c_int,
        );
    }
}
unsafe extern "C" fn hooks_monitor_cb(
    mut change: *mut monitor_change,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut hm: *mut hooks_monitor = data as *mut hooks_monitor;
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut wl: *mut winlink = (*change).wl;
    let mut wp: *mut window_pane = (*change).wp;
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    ep = event_payload_create();
    event_payload_set_pointer(
        ep,
        b"_hooks_monitor\0" as *const u8 as *const ::core::ffi::c_char,
        data,
        None,
        None,
    );
    cmd_find_clear_state(&raw mut fs, 0 as ::core::ffi::c_int);
    if !wl.is_null() && !wp.is_null() && (*wp).window == (*wl).window {
        cmd_find_from_winlink_pane(&raw mut fs, wl, wp, 0 as ::core::ffi::c_int);
    } else if !wl.is_null() {
        cmd_find_from_winlink(&raw mut fs, wl, 0 as ::core::ffi::c_int);
    } else if !wp.is_null() {
        cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    } else if !(*change).s.is_null() {
        cmd_find_from_session(&raw mut fs, (*change).s, 0 as ::core::ffi::c_int);
    } else {
        cmd_find_copy_state(&raw mut fs, &raw mut (*hm).fs);
    }
    event_payload_set_target(ep, &raw mut fs);
    if !(*change).value.is_null() {
        event_payload_set_string(
            ep,
            b"value\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*change).value,
        );
    } else {
        event_payload_set_string(
            ep,
            b"value\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !(*change).last.is_null() {
        event_payload_set_string(
            ep,
            b"last\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*change).last,
        );
    } else {
        event_payload_set_string(
            ep,
            b"last\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !(*change).c.is_null() {
        event_payload_set_client(
            ep,
            b"client\0" as *const u8 as *const ::core::ffi::c_char,
            (*change).c,
        );
    }
    if !(*change).s.is_null() {
        event_payload_set_session(
            ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            (*change).s,
        );
    }
    if !wl.is_null() {
        if (*change).s.is_null() {
            event_payload_set_session(
                ep,
                b"session\0" as *const u8 as *const ::core::ffi::c_char,
                (*wl).session,
            );
        }
        event_payload_set_window(
            ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).window,
        );
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).idx,
        );
    }
    if !wp.is_null() {
        event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
        if wl.is_null() {
            event_payload_set_window(
                ep,
                b"window\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).window as *mut window,
            );
        }
    }
    events_fire((*change).name, ep);
}
#[no_mangle]
pub unsafe extern "C" fn hooks_monitor_add(
    mut item: *mut cmdq_item,
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut type_0: monitor_type,
    mut id: ::core::ffi::c_int,
    mut format: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut fs: *mut cmd_find_state,
    mut s: *mut session,
) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut hm: *mut hooks_monitor = ::core::ptr::null_mut::<hooks_monitor>();
    hooks_monitor_remove(oo, name);
    o = options_get_only(oo, name);
    if o.is_null() {
        o = options_set_string(
            oo,
            name,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    hm = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<hooks_monitor>() as size_t,
    ) as *mut hooks_monitor;
    (*hm).oo = oo;
    cmd_find_copy_state(&raw mut (*hm).fs, fs);
    (*hm).type_0 = type_0;
    (*hm).id = id;
    (*hm).format = xstrdup(format);
    (*hm).set = monitor_create_session(
        s,
        Some(
            hooks_monitor_cb
                as unsafe extern "C" fn(*mut monitor_change, *mut ::core::ffi::c_void) -> (),
        ),
        hm as *mut ::core::ffi::c_void,
    );
    (*hm).sink = events_add_sink(
        name,
        Some(
            hooks_monitor_hook_cb
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_char,
                    *mut event_payload,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        hm as *mut ::core::ffi::c_void,
    );
    options_set_monitor_data(o, hm as *mut ::core::ffi::c_void);
    monitor_add((*hm).set, name, type_0, id, format, flags);
}
#[no_mangle]
pub unsafe extern "C" fn hooks_monitor_to_string(
    o: *mut options_entry,
) -> *mut ::core::ffi::c_char {
    hooks_monitor_to_cstring(o).map_or(::core::ptr::null_mut(), |s| xstrdup(s.as_ptr()))
}

pub(crate) unsafe fn hooks_monitor_to_cstring(o: *mut options_entry) -> Option<CString> {
    let hm = options_get_monitor_data(o) as *mut hooks_monitor;
    if hm.is_null() {
        return None;
    }
    let mut bytes = CStr::from_ptr(options_name(o)).to_bytes().to_vec();
    let target = match (*hm).type_0 {
        0 => b"::".as_slice(),
        1 => b":%".as_slice(),
        2 => b":%*:".as_slice(),
        3 => b":@".as_slice(),
        4 => b":@*:".as_slice(),
        _ => return None,
    };
    bytes.extend_from_slice(target);
    if matches!((*hm).type_0, 1 | 3) {
        bytes.extend_from_slice((*hm).id.to_string().as_bytes());
        bytes.push(b':');
    }
    bytes.extend_from_slice(CStr::from_ptr((*hm).format).to_bytes());
    Some(CString::new(bytes).expect("C string parts contain no NUL"))
}
#[no_mangle]
pub unsafe extern "C" fn hooks_monitor_get(
    mut o: *mut options_entry,
    mut type_0: *mut monitor_type,
    mut id: *mut ::core::ffi::c_int,
    mut format: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut hm: *mut hooks_monitor = options_get_monitor_data(o) as *mut hooks_monitor;
    if hm.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    *type_0 = (*hm).type_0;
    *id = (*hm).id;
    *format = (*hm).format;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn hooks_monitor_get_fire_count(mut o: *mut options_entry) -> u_int {
    let mut hm: *mut hooks_monitor = options_get_monitor_data(o) as *mut hooks_monitor;
    if hm.is_null() {
        return 0 as u_int;
    }
    return monitor_get_fire_count((*hm).set, options_name(o));
}
#[no_mangle]
pub unsafe extern "C" fn hooks_monitor_get_fire_time(mut o: *mut options_entry) -> time_t {
    let mut hm: *mut hooks_monitor = options_get_monitor_data(o) as *mut hooks_monitor;
    if hm.is_null() {
        return 0 as time_t;
    }
    return monitor_get_fire_time((*hm).set, options_name(o));
}
unsafe extern "C" fn run_static_initializers() {
    hooks_events = hooks_events {
        tqh_first: ::core::ptr::null_mut::<hooks_event>(),
        tqh_last: &raw mut hooks_events.tqh_first,
    };
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];
