use crate::src::cmd::parse::{cmd_parse_from_buffer, cmd_parse_from_file};
use crate::src::cmd::queue::{
    cmdq_add_format, cmdq_append, cmdq_continue, cmdq_copy_state,
    cmdq_get_callback_owned, cmdq_get_client, cmdq_get_command, cmdq_get_state, cmdq_insert_after,
    cmdq_new_state, cmdq_print,
};
use crate::src::compat::stdio::CFile;
use crate::src::control::control_notify_write;
use crate::src::ffi::libc::{__errno_location, fopen, strerror};
use crate::src::format::bytes::format_message_with;
use crate::src::format::bytes::write_cstr;
use crate::src::log::{log_cstr, log_debug};
use crate::src::prompt_history::prompt_load_history;
use crate::src::server::clients;
use crate::src::session::sessions;
use crate::src::session::sessions_minmax;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd_find_state, cmdq_item, cmdq_state};
use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
use crate::src::shared::command::{CMD_PARSE_PARSEONLY, CMD_PARSE_QUIET};
use crate::src::shared::errno::ENOENT;
use crate::src::shared::key::key_event;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::stdio::FILE;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::window::{window, window_mode_entry, winlink};
use crate::src::window::window_pane_set_mode;
use crate::src::window_copy::{window_copy_add, window_view_mode};
use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::sync::{Mutex, OnceLock};
pub static mut cfg_client: std::rc::Weak<std::cell::UnsafeCell<client>> = std::rc::Weak::new();
pub static mut cfg_finished: ::core::ffi::c_int = 0;
static CFG_CAUSES: Mutex<VecDeque<CString>> = Mutex::new(VecDeque::new());
#[cfg(test)]
pub(crate) static CFG_TEST_LOCK: Mutex<()> = Mutex::new(());
static mut cfg_item: *mut cmdq_item = ::core::ptr::null::<cmdq_item>() as *mut cmdq_item;
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
unsafe fn cfg_client_done() -> cmd_retval {
    if cfg_finished == 0 {
        return CMD_RETURN_WAIT;
    }
    return CMD_RETURN_NORMAL;
}
unsafe fn cfg_done() -> cmd_retval {
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
pub unsafe fn start_cfg() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut registry_c_owner = clients.first();
    c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    cfg_client = registry_c_owner.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
    if !c.is_null() {
        cfg_item = cmdq_get_callback_owned(
            b"cfg_client_done\0" as *const u8 as *const ::core::ffi::c_char,
            Some(Box::new(|_| unsafe { cfg_client_done() })),
        );
        cmdq_append(c, cfg_item);
    }
    if cfg_quiet != 0 {
        flags = CMD_PARSE_QUIET;
    }
    for path in cfg_files() {
        load_cfg(path.as_ptr(), registry_c_owner.as_ref(), flags);
    }
    cmdq_append(
        ::core::ptr::null_mut::<client>(),
        cmdq_get_callback_owned(
            b"cfg_done\0" as *const u8 as *const ::core::ffi::c_char,
            Some(Box::new(|_| unsafe { cfg_done() })),
        ),
    );
}
pub unsafe fn load_cfg(
    mut path: *const ::core::ffi::c_char,
    c: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();

    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut pi: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: None,
        line: 0,
        item: ::core::ptr::null_mut::<cmdq_item>(),
        c: Default::default(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: Default::default(),
            wl: Default::default(),
            w: Default::default(),
            wp: Default::default(),
            idx: 0,
        },
    };
    let mut pr: cmd_parse_result = cmd_parse_result::empty();
    let mut new_item0: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let state;

    log_debug(format_args!("loading {}", log_cstr((path) as *const _)));
    f = fopen(path, b"rb\0" as *const u8 as *const ::core::ffi::c_char) as *mut FILE;
    if f.is_null() {
        if *__errno_location() == ENOENT && flags & CMD_PARSE_QUIET != 0 {
            return 0 as ::core::ffi::c_int;
        }
        cfg_add_cause(|out| {
            write_cstr(out, path)?;
            out.write_all(b": ")?;
            write_cstr(out, strerror(*__errno_location()))
        });
        return -(1 as ::core::ffi::c_int);
    }
    let stream = CFile::from_raw(f).expect("fopen returned a non-null stream");
    pi.flags = flags;
    pi.file = Some(CStr::from_ptr(path).to_owned());
    pi.line = 1 as u_int;
    pi.item = item;
    pi.c = c.map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
    pr = cmd_parse_from_file(stream.as_ptr(), &raw mut pi);
    drop(stream);
    if pr.status as ::core::ffi::c_uint
        == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cfg_add_cause(|out| {
            write_cstr(
                out,
                pr.error
                    .as_ref()
                    .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
            )
        });
        return -(1 as ::core::ffi::c_int);
    }
    if flags & CMD_PARSE_PARSEONLY != 0 {
        drop(pr.cmdlist.take());
        return 0 as ::core::ffi::c_int;
    }

    state = cmdq_new_state(
        ::core::ptr::null_mut::<cmd_find_state>(),
        ::core::ptr::null_mut::<key_event>(),
        0 as ::core::ffi::c_int,
    );

    cmdq_add_format(
        &state,
        c"current_file",
        // Preserve libc's former %s rendering when the filename is absent.
        pi.file.as_deref().unwrap_or(c"(null)"),
    );
    new_item0 = cmdq_get_command(pr.cmdlist.as_ref().expect("successful command parse"), Some(&state));

    new_item0 = cmdq_append(::core::ptr::null_mut::<client>(), new_item0);

    drop(pr.cmdlist.take());


    return 0 as ::core::ffi::c_int;
}
pub unsafe fn load_cfg_from_buffer(
    mut buf: *const ::core::ffi::c_void,
    mut len: size_t,
    mut path: *const ::core::ffi::c_char,
    c: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
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
        c: Default::default(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: Default::default(),
            wl: Default::default(),
            w: Default::default(),
            wp: Default::default(),
            idx: 0,
        },
    };
    let mut pr: cmd_parse_result = cmd_parse_result::empty();
    let mut new_item0: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let state;
    if !new_item.is_null() {
        *new_item = ::core::ptr::null_mut::<cmdq_item>();
    }
    log_debug(format_args!("loading {}", log_cstr((path) as *const _)));
    pi.flags = flags;
    pi.file = Some(CStr::from_ptr(path).to_owned());
    pi.line = 1 as u_int;
    pi.item = item;
    pi.c = c.map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
    pr = cmd_parse_from_buffer(buf, len, &raw mut pi);
    if pr.status as ::core::ffi::c_uint
        == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cfg_add_cause(|out| {
            write_cstr(
                out,
                pr.error
                    .as_ref()
                    .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
            )
        });
        return -(1 as ::core::ffi::c_int);
    }
    if flags & CMD_PARSE_PARSEONLY != 0 {
        drop(pr.cmdlist.take());
        return 0 as ::core::ffi::c_int;
    }
    if !item.is_null() {
        state = cmdq_copy_state(cmdq_get_state(&*item).expect("command queue state"), current);
    } else {
        state = cmdq_new_state(
            ::core::ptr::null_mut::<cmd_find_state>(),
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
    }
    cmdq_add_format(
        &state,
        c"current_file",
        // Preserve libc's former %s rendering when the filename is absent.
        pi.file.as_deref().unwrap_or(c"(null)"),
    );
    new_item0 = cmdq_get_command(pr.cmdlist.as_ref().expect("successful command parse"), Some(&state));
    if !item.is_null() {
        new_item0 = cmdq_insert_after(item, new_item0);
    } else {
        new_item0 = cmdq_append(::core::ptr::null_mut::<client>(), new_item0);
    }
    drop(pr.cmdlist.take());

    if !new_item.is_null() {
        *new_item = new_item0;
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn cfg_add_cause(write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>) {
    let msg = format_message_with(write);
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
pub unsafe fn cfg_print_causes(mut item: *mut cmdq_item) {
    let c_owner = cmdq_get_client(item);
    let mut c: *mut client = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    cfg_drain_causes(|cause| {
        if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_notify_write(c, |out| {
                out.write_all(b"%config-error ")?;
                write_cstr(out, cause.as_ptr())
            });
        } else {
            cmdq_print(item, |out| write_cstr(out, cause.as_ptr()));
        }
    });
}
pub unsafe fn cfg_show_causes(mut s: *mut session) {
    let mut registry_c_owner = clients.first();
    let mut c: *mut client = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if CFG_CAUSES.lock().unwrap().is_empty() {
        return;
    }
    if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        cfg_drain_causes(|cause| {
            control_notify_write(c, |out| {
                out.write_all(b"%config-error ")?;
                write_cstr(out, cause.as_ptr())
            });
        });
    } else {
        if s.is_null() {
            if !c.is_null() && !(*c).session.is_null() {
                s = (*c).session;
            } else {
                let mut s_owner = sessions_minmax(&*std::ptr::addr_of!(sessions));
                s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
            }
        }
        if s.is_null() || (*s).attached == 0 as u_int {
            return;
        }
        wp = (*(*(*s).curw).window_ptr()).active;
        wme = (*wp).modes.active;
        if wme.is_null() || !std::ptr::eq((*wme).mode, &window_view_mode) {
            window_pane_set_mode(
                wp,
                ::core::ptr::null_mut::<window_pane>(),
                &window_view_mode,
                ::core::ptr::null_mut::<cmdq_item>(),
                ::core::ptr::null_mut::<cmd_find_state>(),
                ::core::ptr::null_mut::<args>(),
            );
        }
        cfg_drain_causes(|cause| {
            window_copy_add(wp, 0 as ::core::ffi::c_int, |out| {
                write_cstr(out, cause.as_ptr())
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{cfg_add_cause, cfg_drain_causes, cfg_test_take_causes, CFG_TEST_LOCK};
    use crate::src::format::bytes::write_cstr;

    #[test]
    fn causes_keep_c_string_bytes_and_drain_reentrant_additions_in_order() {
        let _guard = CFG_TEST_LOCK.lock().unwrap();
        unsafe {
            let _ = cfg_test_take_causes();
            cfg_add_cause(|out| {
                write_cstr(out, c"first".as_ptr())?;
                write!(out, ":{}", (7u32) as u32)
            });
            cfg_add_cause(|out| {
                write_cstr(out, b"second\xff\0".as_ptr().cast::<::core::ffi::c_char>())
            });
        }

        let mut actual = Vec::new();
        cfg_drain_causes(|cause| {
            actual.push(cause.to_bytes().to_vec());
            if actual.len() == 1 {
                unsafe { cfg_add_cause(|out| write_cstr(out, c"third".as_ptr())) };
            }
        });
        assert_eq!(actual, [b"first:7".as_slice(), b"second\xff", b"third"]);
        assert!(unsafe { cfg_test_take_causes() }.is_empty());
    }
}
