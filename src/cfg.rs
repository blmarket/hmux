use crate::src::cmd::cmd_list_free;
use crate::src::cmd::parse::{cmd_parse_from_buffer, cmd_parse_from_file};
use crate::src::cmd::queue::{
    cmdq_add_format, cmdq_append, cmdq_continue, cmdq_copy_state, cmdq_free_state,
    cmdq_get_callback1, cmdq_get_client, cmdq_get_command, cmdq_get_state, cmdq_insert_after,
    cmdq_new_state, cmdq_print,
};
use crate::src::compat::stdio::CFile;
use crate::src::control::control_notify_write;
use crate::src::ffi::libc::{__errno_location, fopen, free, memset, strerror};
use crate::src::log::log_debug;
use crate::src::prompt_history::prompt_load_history;
use crate::src::server::clients;
use crate::src::session::sessions;
use crate::src::session::sessions_minmax;
use crate::src::shared::abi::*;
use crate::src::shared::abi::{__off64_t, __off_t};
use crate::src::shared::arguments::args;
use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::client::*;
use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::command::*;
use crate::src::shared::command::{
    cmd_find_state, cmd_list, cmdq_cb, cmdq_item, cmdq_list, cmdq_state, cmds,
};
use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
use crate::src::shared::command::{CMD_PARSE_PARSEONLY, CMD_PARSE_QUIET};
use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::environment::environ;
use crate::src::shared::errno::ENOENT;
use crate::src::shared::event::*;
use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
use crate::src::shared::hyperlinks::hyperlinks;
use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options;
use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
use crate::src::shared::pane::{window_pane_offset, window_pane_resize, window_pane_resizes};
use crate::src::shared::process::tmuxpeer;
use crate::src::shared::prompt::prompt;
use crate::src::shared::redraw::redraw_scene;
use crate::src::shared::screen::{screen, screen_sel, screen_titles};
use crate::src::shared::screen_write::screen_write_cline;
use crate::src::shared::session::{session, session_entry};
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::status::status_line;
use crate::src::shared::stdio::{
    _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, _IO_FILE, FILE,
};
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
use crate::src::shared::variadic::{__builtin_va_list, __gnuc_va_list, __va_list_tag, va_list};
use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};
use crate::src::window::window_pane_set_mode;
use crate::src::window_copy::{window_copy_add, window_view_mode};
use crate::src::xmalloc::xvasprintf_cstring;
use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::sync::{Mutex, OnceLock};

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut cfg_client: *mut client = ::core::ptr::null::<client>() as *mut client;
#[no_mangle]
pub static mut cfg_finished: ::core::ffi::c_int = 0;
static CFG_CAUSES: Mutex<VecDeque<CString>> = Mutex::new(VecDeque::new());
#[cfg(test)]
pub(crate) static CFG_TEST_LOCK: Mutex<()> = Mutex::new(());
static mut cfg_item: *mut cmdq_item = ::core::ptr::null::<cmdq_item>() as *mut cmdq_item;
#[no_mangle]
pub static mut cfg_quiet: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
// Startup publishes the list before client_main can start a server. C readers
// borrow the stable CString storage; the list is never mutated after publication.
static CFG_FILES: OnceLock<Vec<CString>> = OnceLock::new();

pub(crate) fn cfg_set_files(files: Vec<CString>) {
    assert!(
        CFG_FILES.set(files).is_ok(),
        "configuration paths initialized twice"
    );
}

pub(crate) fn cfg_files() -> &'static [CString] {
    CFG_FILES.get().map_or(&[], Vec::as_slice)
}
unsafe extern "C" fn cfg_client_done(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    if cfg_finished == 0 {
        return CMD_RETURN_WAIT;
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cfg_done(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    if cfg_finished != 0 {
        return CMD_RETURN_NORMAL;
    }
    cfg_finished = 1 as ::core::ffi::c_int;
    cfg_show_causes(::core::ptr::null_mut::<session>());
    if !cfg_item.is_null() {
        cmdq_continue(cfg_item);
    }
    prompt_load_history();
    return CMD_RETURN_NORMAL;
}
#[no_mangle]
pub unsafe extern "C" fn start_cfg() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    c = clients.first();
    cfg_client = c;
    if !c.is_null() {
        cfg_item = cmdq_get_callback1(
            b"cfg_client_done\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                cfg_client_done
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        );
        cmdq_append(c, cfg_item);
    }
    if cfg_quiet != 0 {
        flags = CMD_PARSE_QUIET;
    }
    for path in cfg_files() {
        load_cfg(
            path.as_ptr(),
            c,
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<cmd_find_state>(),
            flags,
            ::core::ptr::null_mut::<*mut cmdq_item>(),
        );
    }
    cmdq_append(
        ::core::ptr::null_mut::<client>(),
        cmdq_get_callback1(
            b"cfg_done\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                cfg_done
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ),
    );
}
#[no_mangle]
pub unsafe extern "C" fn load_cfg(
    mut path: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut item: *mut cmdq_item,
    mut current: *mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
    mut new_item: *mut *mut cmdq_item,
) -> ::core::ffi::c_int {
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut pi: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: None,
        line: 0,
        item: ::core::ptr::null_mut::<cmdq_item>(),
        c: ::core::ptr::null_mut::<client>(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: ::core::ptr::null_mut::<session>(),
            wl: ::core::ptr::null_mut::<winlink>(),
            w: ::core::ptr::null_mut::<window>(),
            wp: ::core::ptr::null_mut::<window_pane>(),
            idx: 0,
        },
    };
    let mut pr: cmd_parse_result = cmd_parse_result::empty();
    let mut new_item0: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    if !new_item.is_null() {
        *new_item = ::core::ptr::null_mut::<cmdq_item>();
    }
    log_debug(
        b"loading %s\0" as *const u8 as *const ::core::ffi::c_char,
        path,
    );
    f = fopen(path, b"rb\0" as *const u8 as *const ::core::ffi::c_char) as *mut FILE;
    if f.is_null() {
        if *__errno_location() == ENOENT && flags & CMD_PARSE_QUIET != 0 {
            return 0 as ::core::ffi::c_int;
        }
        cfg_add_cause(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            path,
            strerror(*__errno_location()),
        );
        return -(1 as ::core::ffi::c_int);
    }
    let stream = CFile::from_raw(f).expect("fopen returned a non-null stream");
    pi.flags = flags;
    pi.file = Some(CStr::from_ptr(path).to_owned());
    pi.line = 1 as u_int;
    pi.item = item;
    pi.c = c;
    pr = cmd_parse_from_file(stream.as_ptr(), &raw mut pi);
    drop(stream);
    if pr.status as ::core::ffi::c_uint
        == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cfg_add_cause(
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            pr.error
                .as_ref()
                .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
        );
        return -(1 as ::core::ffi::c_int);
    }
    if flags & CMD_PARSE_PARSEONLY != 0 {
        cmd_list_free(pr.cmdlist);
        return 0 as ::core::ffi::c_int;
    }
    if !item.is_null() {
        state = cmdq_copy_state(cmdq_get_state(item), current);
    } else {
        state = cmdq_new_state(
            ::core::ptr::null_mut::<cmd_find_state>(),
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
    }
    cmdq_add_format(
        state,
        b"current_file\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        pi.file
            .as_ref()
            .map_or(::core::ptr::null(), |file| file.as_ptr()),
    );
    new_item0 = cmdq_get_command(pr.cmdlist, state);
    if !item.is_null() {
        new_item0 = cmdq_insert_after(item, new_item0);
    } else {
        new_item0 = cmdq_append(::core::ptr::null_mut::<client>(), new_item0);
    }
    cmd_list_free(pr.cmdlist);
    cmdq_free_state(state);
    if !new_item.is_null() {
        *new_item = new_item0;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn load_cfg_from_buffer(
    mut buf: *const ::core::ffi::c_void,
    mut len: size_t,
    mut path: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut item: *mut cmdq_item,
    mut current: *mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
    mut new_item: *mut *mut cmdq_item,
) -> ::core::ffi::c_int {
    let mut pi: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: None,
        line: 0,
        item: ::core::ptr::null_mut::<cmdq_item>(),
        c: ::core::ptr::null_mut::<client>(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: ::core::ptr::null_mut::<session>(),
            wl: ::core::ptr::null_mut::<winlink>(),
            w: ::core::ptr::null_mut::<window>(),
            wp: ::core::ptr::null_mut::<window_pane>(),
            idx: 0,
        },
    };
    let mut pr: cmd_parse_result = cmd_parse_result::empty();
    let mut new_item0: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    if !new_item.is_null() {
        *new_item = ::core::ptr::null_mut::<cmdq_item>();
    }
    log_debug(
        b"loading %s\0" as *const u8 as *const ::core::ffi::c_char,
        path,
    );
    pi.flags = flags;
    pi.file = Some(CStr::from_ptr(path).to_owned());
    pi.line = 1 as u_int;
    pi.item = item;
    pi.c = c;
    pr = cmd_parse_from_buffer(buf, len, &raw mut pi);
    if pr.status as ::core::ffi::c_uint
        == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cfg_add_cause(
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            pr.error
                .as_ref()
                .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
        );
        return -(1 as ::core::ffi::c_int);
    }
    if flags & CMD_PARSE_PARSEONLY != 0 {
        cmd_list_free(pr.cmdlist);
        return 0 as ::core::ffi::c_int;
    }
    if !item.is_null() {
        state = cmdq_copy_state(cmdq_get_state(item), current);
    } else {
        state = cmdq_new_state(
            ::core::ptr::null_mut::<cmd_find_state>(),
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
    }
    cmdq_add_format(
        state,
        b"current_file\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        pi.file
            .as_ref()
            .map_or(::core::ptr::null(), |file| file.as_ptr()),
    );
    new_item0 = cmdq_get_command(pr.cmdlist, state);
    if !item.is_null() {
        new_item0 = cmdq_insert_after(item, new_item0);
    } else {
        new_item0 = cmdq_append(::core::ptr::null_mut::<client>(), new_item0);
    }
    cmd_list_free(pr.cmdlist);
    cmdq_free_state(state);
    if !new_item.is_null() {
        *new_item = new_item0;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cfg_add_cause(mut fmt: *const ::core::ffi::c_char, mut args: ...) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    let msg = xvasprintf_cstring(fmt, ap);
    CFG_CAUSES.lock().unwrap().push_back(msg);
}

fn cfg_drain_causes(mut deliver: impl FnMut(&CStr)) {
    loop {
        // Release the lock before delivery: output may call cfg_add_cause or
        // another cause consumer through a callback.
        let cause = CFG_CAUSES.lock().unwrap().pop_front();
        let Some(cause) = cause else { break };
        deliver(&cause);
    }
}

#[cfg(test)]
pub(crate) unsafe fn cfg_test_take_causes() -> Vec<Vec<u8>> {
    let mut causes = Vec::new();
    cfg_drain_causes(|cause| causes.push(cause.to_bytes().to_vec()));
    causes
}

#[no_mangle]
pub unsafe extern "C" fn cfg_print_causes(mut item: *mut cmdq_item) {
    let mut c: *mut client = cmdq_get_client(item);
    cfg_drain_causes(|cause| {
        if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_notify_write(
                c,
                b"%%config-error %s\0" as *const u8 as *const ::core::ffi::c_char,
                cause.as_ptr(),
            );
        } else {
            cmdq_print(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause.as_ptr(),
            );
        }
    });
}
#[no_mangle]
pub unsafe extern "C" fn cfg_show_causes(mut s: *mut session) {
    let mut c: *mut client = clients.first();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if CFG_CAUSES.lock().unwrap().is_empty() {
        return;
    }
    if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        cfg_drain_causes(|cause| {
            control_notify_write(
                c,
                b"%%config-error %s\0" as *const u8 as *const ::core::ffi::c_char,
                cause.as_ptr(),
            );
        });
    } else {
        if s.is_null() {
            if !c.is_null() && !(*c).session.is_null() {
                s = (*c).session;
            } else {
                s = sessions_minmax(&*std::ptr::addr_of!(sessions), RB_NEGINF);
            }
        }
        if s.is_null() || (*s).attached == 0 as u_int {
            return;
        }
        wp = (*(*(*s).curw).window).active;
        wme = (*wp).modes.active;
        if wme.is_null() || (*wme).mode != &raw const window_view_mode {
            window_pane_set_mode(
                wp,
                ::core::ptr::null_mut::<window_pane>(),
                &raw const window_view_mode,
                ::core::ptr::null_mut::<cmdq_item>(),
                ::core::ptr::null_mut::<cmd_find_state>(),
                ::core::ptr::null_mut::<args>(),
            );
        }
        cfg_drain_causes(|cause| {
            window_copy_add(
                wp,
                0 as ::core::ffi::c_int,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause.as_ptr(),
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{cfg_add_cause, cfg_drain_causes, cfg_test_take_causes, CFG_TEST_LOCK};

    #[test]
    fn causes_keep_c_string_bytes_and_drain_reentrant_additions_in_order() {
        let _guard = CFG_TEST_LOCK.lock().unwrap();
        unsafe {
            let _ = cfg_test_take_causes();
            cfg_add_cause(c"%s:%u".as_ptr(), c"first".as_ptr(), 7u32);
            cfg_add_cause(
                c"%s".as_ptr(),
                b"second\xff\0".as_ptr().cast::<::core::ffi::c_char>(),
            );
        }

        let mut actual = Vec::new();
        cfg_drain_causes(|cause| {
            actual.push(cause.to_bytes().to_vec());
            if actual.len() == 1 {
                unsafe { cfg_add_cause(c"%s".as_ptr(), c"third".as_ptr()) };
            }
        });
        assert_eq!(actual, [b"first:7".as_slice(), b"second\xff", b"third"]);
        assert!(unsafe { cfg_test_take_causes() }.is_empty());
    }
}
