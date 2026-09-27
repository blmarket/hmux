use crate::src::session::session_remove_ref;
use crate::src::shared::client::{ClientOwner, client_owner_ptr};
use crate::src::shared::rc;
use crate::src::window::window_pane_upgrade;
use std::cell::UnsafeCell;
use std::rc::Weak;
use crate::src::options::options_owner_ptr;
use crate::src::cmd::cmd_list_print_cstring;
use crate::src::cmd::find::{
    cmd_find_clear_state, cmd_find_copy_state, cmd_find_empty_state, cmd_find_from_nothing,
    cmd_find_from_pane, cmd_find_from_session, cmd_find_from_winlink, cmd_find_from_winlink_pane,
    cmd_find_valid_state,
};
use crate::src::cmd::parse::cmd_parse_from_string;
use crate::src::cmd::queue::{
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
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_create_owned, format_create_defaults, format_expand_cstring, format_free,
    format_log_debug, format_merge,
};
use crate::src::log::{log_cstr, log_debug, log_get_level};
use crate::src::monitor::{
    monitor_add, monitor_create_session_owned, monitor_get_fire_count,
    monitor_get_fire_time,
};
use crate::src::options::{
    options_array_first, options_array_item_value, options_array_next, options_get,
    options_get_monitor_data, options_get_only, options_get_string, options_hook_fired,
    options_name, options_search, options_set_monitor_data, options_set_string,
};
use crate::src::options_table::options_table;
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::command::CMDQ_STATE_NOHOOKS;
use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_state};
use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
use crate::src::shared::events::{event_payload, events_callback, events_sink};
use crate::src::shared::format::format_tree;
use crate::src::shared::format::{FORMAT_NOJOBS, FORMAT_NONE};
use crate::src::shared::key::key_event;
use crate::src::shared::monitor::monitor_type;
use crate::src::shared::monitor::{monitor_callback, monitor_change, monitor_set};
use crate::src::shared::options::OPTIONS_TABLE_IS_HOOK;
use crate::src::shared::options::{
    options, options_array_item, options_entry, options_table_entry,
};
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::window::{window, winlink};
use crate::src::tmux::global_s_options;
use std::ffi::{CStr, CString};

struct hooks_event {
    // The registry owns this C string for as long as its event is registered.
    name: CString,
}

#[derive(Default)]
struct HooksEvents {
    // Boxes keep each event record stable when the owning vector grows.
    events: Vec<Box<hooks_event>>,
}

impl HooksEvents {
    fn contains(&self, name: &CStr) -> bool {
        self.events
            .iter()
            .any(|event| event.name.as_bytes() == name.to_bytes())
    }

    fn insert(&mut self, name: CString) -> bool {
        if self.contains(name.as_c_str()) {
            return false;
        }
        self.events.push(Box::new(hooks_event { name }));
        true
    }
}
#[repr(C)]
pub struct hooks_data<'a> {
    pub name: &'a CStr,
    pub fs: cmd_find_state,
    pub formats: crate::src::shared::format::FormatTreeOwner,
    pub oo: *mut options,
    pub client: Weak<UnsafeCell<client>>,
    pub expand: ::core::ffi::c_int,
}
#[repr(C)]
pub struct hooks_monitor {
    // options_entry.monitor_data owns the boxed record; callbacks only borrow it.
    pub oo: *mut options,
    pub set: Option<crate::src::shared::monitor::MonitorOwner>,
    pub sink: *mut events_sink,
    pub fs: cmd_find_state,
    pub type_0: monitor_type,
    pub id: ::core::ffi::c_int,
    /// hooks_monitor_get lends this pointer until the monitor is removed.
    pub format: CString,
}

static mut hooks_events: HooksEvents = HooksEvents { events: Vec::new() };
unsafe fn hooks_insert_one(
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
        let s = cmd_list_print_cstring(&*cmdlist, 0 as ::core::ffi::c_int);
        log_debug(format_args!(
            "{}: hook {} is: {}",
            "hooks_insert_one",
            log_cstr((*hd).name.as_ptr()),
            log_cstr((s.as_ptr()) as *const _)
        ));
    }
    new_item = cmdq_get_command(cmdlist, state);
    if !item.is_null() {
        return cmdq_insert_after(item, new_item);
    }
    return cmdq_append(::core::ptr::null_mut::<client>(), new_item);
}
unsafe fn hooks_parse(hd: *mut hooks_data, fs: &cmd_find_state, value: &CStr) -> cmd_parse_result {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    if (*hd).expand == 0 {
        return cmd_parse_from_string(value, ::core::ptr::null_mut::<cmd_parse_input>());
    }
    let client_owner = ClientOwner::upgrade(&(*hd).client);
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        client_owner_ptr(&client_owner),
        fs.s,
        fs.wl,
        fs.wp,
    );
    format_merge(ft, (*hd).formats.as_ptr());
    let expanded = format_expand_cstring(ft, value.as_ptr());
    format_free(ft);
    let pr = cmd_parse_from_string(
        expanded.as_c_str(),
        ::core::ptr::null_mut::<cmd_parse_input>(),
    );
    return pr;
}
unsafe fn hooks_insert(mut item: *mut cmdq_item, mut hd: *mut hooks_data) {
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
    let mut pr: cmd_parse_result = cmd_parse_result::empty();
    log_debug(format_args!(
        "{}: inserting hook {}",
        "hooks_insert",
        log_cstr((*hd).name.as_ptr())
    ));
    cmd_find_clear_state(&raw mut fs, 0 as ::core::ffi::c_int);
    if cmd_find_empty_state(&(*hd).fs) != 0 || cmd_find_valid_state(&(*hd).fs) == 0 {
        cmd_find_from_nothing(&raw mut fs, 0 as ::core::ffi::c_int);
    } else {
        cmd_find_copy_state(&raw mut fs, &raw mut (*hd).fs);
    }
    if !(*hd).oo.is_null() {
        oo = (*hd).oo;
        o = options_get_only(oo, (*hd).name.as_ptr());
    } else {
        if fs.s.is_null() {
            oo = global_s_options;
        } else {
            oo = options_owner_ptr(&mut (*fs.s).options);
        }
        o = options_get(oo, (*hd).name.as_ptr());
        if o.is_null() && !fs.wp.is_null() {
            oo = options_owner_ptr(&mut (*fs.wp).options);
            o = options_get(oo, (*hd).name.as_ptr());
        }
        if o.is_null() && !fs.wl.is_null() {
            oo = options_owner_ptr(&mut (*(*fs.wl).window_ptr()).options);
            o = options_get(oo, (*hd).name.as_ptr());
        }
    }
    if o.is_null() {
        log_debug(format_args!(
            "{}: hook {} not found",
            "hooks_insert",
            log_cstr((*hd).name.as_ptr())
        ));
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
    cmdq_add_formats(state, (*hd).formats.as_ptr());
    if *(*hd).name.as_ptr() as ::core::ffi::c_int == '@' as i32 {
        value = options_get_string(oo, (*hd).name.as_ptr());
        pr = hooks_parse(hd, &fs, CStr::from_ptr(value));
        match pr.status as ::core::ffi::c_uint {
            0 => {
                log_debug(format_args!(
                    "{}: can't parse hook {}: {}",
                    "hooks_insert",
                    log_cstr((*hd).name.as_ptr()),
                    log_cstr(
                        (pr.error
                            .as_ref()
                            .map_or(::core::ptr::null(), |cause| cause.as_ptr()))
                            as *const _
                    )
                ));
            }
            1 => {
                hooks_insert_one(item, hd, pr.cmdlist_ptr(), state);
            }
            _ => {}
        }
    } else {
        a = options_array_first(o);
        while !a.is_null() {
            if (*hd).expand != 0 {
                value = (*options_array_item_value(a)).string_ptr();
                pr = hooks_parse(hd, &fs, CStr::from_ptr(value));
                match pr.status as ::core::ffi::c_uint {
                    0 => {
                        if let Some(error) = pr.error.as_ref() {
                            cmdq_error(item, |out| write_cstr(out, error.as_ptr()));
                        }
                    }
                    1 => {
                        item = hooks_insert_one(item, hd, pr.cmdlist_ptr(), state);
                    }
                    _ => {}
                }
            } else {
                cmdlist = (*options_array_item_value(a)).cmdlist();
                item = hooks_insert_one(item, hd, cmdlist, state);
            }
            a = options_array_next(a);
        }
    }
    cmdq_free_state(state);
}
unsafe fn hooks_insert_event(
    item: *mut cmdq_item,
    name: *const ::core::ffi::c_char,
    ep: &event_payload,
    oo: *mut options,
    expand: ::core::ffi::c_int,
) {
    if !item.is_null() && cmdq_get_flags(item) & CMDQ_STATE_NOHOOKS != 0 {
        return;
    }
    let c = event_payload_get_client(ep);
    let mut formats = format_create_owned(c, item, FORMAT_NONE, FORMAT_NOJOBS);
    let ft = formats.as_ptr();
    event_payload_add_formats(ep, ft, c"hook_".as_ptr());
    format_add(ft, c"hook".as_ptr(), |out| write_cstr(out, name));
    format_log_debug(ft, c"hooks_insert_event".as_ptr());
    let mut hd = hooks_data {
        name: CStr::from_ptr(name),
        fs: cmd_find_state::default(),
        formats,
        oo,
        client: if c.is_null() { Weak::new() } else { rc::downgrade(c) },
        expand,
    };
    event_payload_get_target(ep, &mut hd.fs);
    hooks_insert(item, &raw mut hd);
}
unsafe fn hooks_event_cb(name: &CStr, payload: &mut event_payload) {
    let name = name.as_ptr();
    let ep = &*payload;
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
    item = cmdq_running();
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
pub unsafe fn hooks_add_event(mut name: *const ::core::ffi::c_char) {
    let event_name = CStr::from_ptr(name);
    let events = &raw const hooks_events;
    if (*events).contains(event_name) {
        return;
    }

    events_add_sink(
        event_name,
        events_callback(|name, payload| unsafe { hooks_event_cb(name, payload) }),
    );
    let events = &raw mut hooks_events;
    (*events).insert(event_name.to_owned());
}
pub unsafe fn hooks_is_event(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let events = &raw const hooks_events;
    return (*events).contains(CStr::from_ptr(name)) as ::core::ffi::c_int;
}
pub unsafe fn hooks_valid_event_name(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    if *name as ::core::ffi::c_int == '@' as i32 {
        return 1 as ::core::ffi::c_int;
    }
    oe = options_search(name);
    return (!oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0) as ::core::ffi::c_int;
}
pub unsafe fn hooks_build_events() {
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    oe = &raw const options_table as *const options_table_entry;
    while !(*oe).name_ptr().is_null() {
        if (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0 {
            hooks_add_event((*oe).name_ptr());
        }
        oe = oe.offset(1);
    }
}
pub unsafe fn hooks_run(item: *mut cmdq_item, name: *const ::core::ffi::c_char) {
    let target = cmdq_get_target(item);
    let mut hd = hooks_data {
        name: CStr::from_ptr(name),
        fs: cmd_find_state::default(),
        formats: format_create_owned(
            std::ptr::null_mut(), std::ptr::null_mut(), 0, FORMAT_NOJOBS,
        ),
        oo: std::ptr::null_mut(),
        client: {
            let client = cmdq_get_client(item);
            if client.is_null() { Weak::new() } else { rc::downgrade(client) }
        },
        expand: 0,
    };
    cmd_find_copy_state(&raw mut hd.fs, target);
    format_add(hd.formats.as_ptr(), c"hook".as_ptr(), |out| write_cstr(out, name));
    format_log_debug(hd.formats.as_ptr(), c"hooks_run".as_ptr());
    hooks_insert(item, &raw mut hd);
}
impl Drop for hooks_monitor {
    fn drop(&mut self) {
        unsafe {
            events_remove_sink(self.sink);
            drop(self.set.take());
        }
    }
}
pub unsafe fn hooks_monitor_remove(mut oo: *mut options, mut name: *const ::core::ffi::c_char) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut hm: *mut hooks_monitor = ::core::ptr::null_mut::<hooks_monitor>();
    o = options_get_only(oo, name);
    if o.is_null() {
        return;
    }
    hm = options_get_monitor_data(o) as *mut hooks_monitor;
    if !hm.is_null() {
        options_set_monitor_data(o, None);
    }
}
unsafe fn hooks_monitor_hook_cb(name: &CStr, payload: &mut event_payload, hm: *mut hooks_monitor) {
    let name = name.as_ptr();
    let ep = &*payload;
    if event_payload_get_pointer(
        ep,
        b"_hooks_monitor\0" as *const u8 as *const ::core::ffi::c_char,
    ) == hm as *mut ::core::ffi::c_void
    {
        hooks_insert_event(cmdq_running(), name, ep, (*hm).oo, 1 as ::core::ffi::c_int);
    }
}
unsafe fn hooks_monitor_cb(change: &monitor_change, hm: *mut hooks_monitor) {
    let wl = change.wl;
    let client_owner = change.c.as_ref().and_then(ClientOwner::upgrade);
    let c = client_owner_ptr(&client_owner);
    let session_owner = change.s.as_ref().and_then(Weak::upgrade);
    let s = session_owner.as_ref().map_or(std::ptr::null_mut(), rc::as_ptr);
    let pane_owner = change.wp.as_ref().and_then(|pane| window_pane_upgrade(pane));
    let wp = pane_owner.as_ref().map_or(std::ptr::null_mut(), rc::as_ptr);
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut ep = event_payload_create();
    event_payload_set_pointer(
        &mut *ep,
        b"_hooks_monitor\0" as *const u8 as *const ::core::ffi::c_char,
        crate::src::shared::events::EventPayloadPointer::Raw(hm.cast()),
    );
    cmd_find_clear_state(&raw mut fs, 0 as ::core::ffi::c_int);
    if !wl.is_null() && !wp.is_null() && (*wp).window == (*wl).window_ptr() {
        cmd_find_from_winlink_pane(&raw mut fs, wl, wp, 0 as ::core::ffi::c_int);
    } else if !wl.is_null() {
        cmd_find_from_winlink(&raw mut fs, wl, 0 as ::core::ffi::c_int);
    } else if !wp.is_null() {
        cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    } else if !s.is_null() {
        cmd_find_from_session(&raw mut fs, s, 0 as ::core::ffi::c_int);
    } else {
        cmd_find_copy_state(&raw mut fs, &raw mut (*hm).fs);
    }
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_string(
        &mut *ep,
        c"value".as_ptr(),
        |out| out.write_all(change.value.to_bytes()),
    );
    if let Some(last) = change.last {
        event_payload_set_string(
            &mut *ep,
            b"last\0" as *const u8 as *const ::core::ffi::c_char,
            |out| out.write_all(last.to_bytes()),
        );
    } else {
        event_payload_set_string(
            &mut *ep,
            b"last\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, b"\0" as *const u8 as *const ::core::ffi::c_char),
        );
    }
    if !c.is_null() {
        event_payload_set_client(&mut *ep, c);
    }
    if !s.is_null() {
        event_payload_set_session(
            &mut *ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            s,
        );
    }
    if !wl.is_null() {
        if s.is_null() {
            event_payload_set_session(
                &mut *ep,
                b"session\0" as *const u8 as *const ::core::ffi::c_char,
                (*wl).session,
            );
        }
        event_payload_set_window(
            &mut *ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).window_ptr(),
        );
        event_payload_set_int(
            &mut *ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).idx,
        );
    }
    if !wp.is_null() {
        event_payload_set_pane(
            &mut *ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            wp,
        );
        if wl.is_null() {
            event_payload_set_window(
                &mut *ep,
                b"window\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).window as *mut window,
            );
        }
    }
    events_fire(change.name.as_ptr(), ep);
    if let Some(owner) = session_owner {
        session_remove_ref(owner, c"hooks_monitor_cb");
    }
}
pub unsafe fn hooks_monitor_add(
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
        o = options_set_string(oo, name, 0 as ::core::ffi::c_int, |out| {
            write_cstr(out, b"\0" as *const u8 as *const ::core::ffi::c_char)
        });
    }
    let mut owner = Box::new(hooks_monitor {
        oo,
        set: None,
        sink: ::core::ptr::null_mut(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut(),
            s: ::core::ptr::null_mut(),
            wl: ::core::ptr::null_mut(),
            w: ::core::ptr::null_mut(),
            wp: ::core::ptr::null_mut(),
            idx: 0,
        },
        type_0,
        id,
        format: CStr::from_ptr(format).to_owned(),
    });
    hm = &raw mut *owner;
    cmd_find_copy_state(&raw mut (*hm).fs, fs);
    (*hm).set = Some(monitor_create_session_owned(
        s,
        monitor_callback(move |change| unsafe { hooks_monitor_cb(change, hm) }),
    ));
    (*hm).sink = events_add_sink(
        CStr::from_ptr(name),
        events_callback(move |name, payload| unsafe { hooks_monitor_hook_cb(name, payload, hm) }),
    );
    options_set_monitor_data(o, Some(owner));
    monitor_add((*hm).set.as_mut().expect("hook monitor").as_ptr(), name, type_0, id, format, flags);
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
    bytes.extend_from_slice((*hm).format.as_bytes());
    Some(CString::new(bytes).expect("C string parts contain no NUL"))
}
pub unsafe fn hooks_monitor_get(
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
    *format = (*hm).format.as_ptr();
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn hooks_monitor_get_fire_count(mut o: *mut options_entry) -> u_int {
    let mut hm: *mut hooks_monitor = options_get_monitor_data(o) as *mut hooks_monitor;
    if hm.is_null() {
        return 0 as u_int;
    }
    return monitor_get_fire_count((*hm).set.as_mut().expect("hook monitor").as_ptr(), options_name(o));
}
pub unsafe fn hooks_monitor_get_fire_time(mut o: *mut options_entry) -> time_t {
    let mut hm: *mut hooks_monitor = options_get_monitor_data(o) as *mut hooks_monitor;
    if hm.is_null() {
        return 0 as time_t;
    }
    return monitor_get_fire_time((*hm).set.as_mut().expect("hook monitor").as_ptr(), options_name(o));
}

#[cfg(test)]
mod hooks_events_tests {
    use super::*;

    #[test]
    fn registry_owns_stable_c_names_and_deduplicates_by_bytes() {
        let mut registry = HooksEvents::default();
        let name = CString::new(vec![b'@', 0xff]).unwrap();
        assert!(registry.insert(name));

        let event_address = &*registry.events[0] as *const hooks_event;
        let name_address = registry.events[0].name.as_ptr();

        for i in 0..128 {
            let other = CString::new(format!("event-{i}")).unwrap();
            assert!(registry.insert(other));
        }

        // The first owner and its C string stay at stable addresses as the
        // collection grows, and arbitrary non-UTF-8 event names are preserved.
        assert_eq!(&*registry.events[0] as *const hooks_event, event_address);
        assert_eq!(registry.events[0].name.as_ptr(), name_address);
        assert_eq!(
            unsafe { CStr::from_ptr(name_address) }.to_bytes(),
            [b'@', 0xff]
        );
        assert!(registry.contains(CStr::from_bytes_with_nul(b"@\xff\0").unwrap()));
        assert!(!registry.insert(CString::new(vec![b'@', 0xff]).unwrap()));
    }
}
