use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target, cmdq_print};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_create_with_client, format_defaults, format_expand_cstring, format_free,
    format_true,
};
use crate::src::session::sessions;
use crate::src::session::{sessions_minmax, sessions_next};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::window::winlink;
use crate::src::sort::{sort_get_panes_window, sort_order_from_string};
use crate::src::window::{winlinks_minmax, winlinks_next};
pub static cmd_list_panes_entry: cmd_entry = {
    cmd_entry {
        name: c"list-panes",
        alias: Some(c"lsp"),
        args: args_parse {
            template: c"aF:f:O:rst:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-asr] [-F format] [-f filter] [-O order][-t target-window]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_list_panes_exec),
    }
};
unsafe fn cmd_list_panes_exec(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let item = item_handle.get();
    let mut args: *mut args =
        cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut s: *mut session = (*target)
        .session_handle()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wl: refbox::Weak<winlink> = (*target).winlink_handle();
    let mut order: sort_order = SORT_ACTIVITY;
    order = sort_order_from_string(
        args_get(&*(args), 'O' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()),
    );
    if order as ::core::ffi::c_uint == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        && args_has(args, 'O' as i32 as u_char) != 0
    {
        cmdq_error(item_handle, |out| out.write_all(b"invalid sort order"));
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'a' as i32 as u_char) != 0 {
        cmd_list_panes_server(self_0.clone(), item_handle);
    } else if args_has(args, 's' as i32 as u_char) != 0 {
        cmd_list_panes_session(
            self_0.clone(),
            &(*(s)).observer.upgrade().expect("live session"),
            item_handle,
            1 as ::core::ffi::c_int,
        );
    } else {
        cmd_list_panes_window(
            self_0.clone(),
            &(*(s)).observer.upgrade().expect("live session"),
            wl.clone(),
            item_handle,
            0 as ::core::ffi::c_int,
        );
    }
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_list_panes_server(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut s_owner = sessions_minmax(&sessions);
    s = s_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    while !s.is_null() {
        cmd_list_panes_session(
            self_0.clone(),
            &(*(s)).observer.upgrade().expect("live session"),
            item_handle,
            2 as ::core::ffi::c_int,
        );
        s_owner = sessions_next(&*s);
        s = s_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    }
}
unsafe fn cmd_list_panes_session(
    mut self_0: refbox::Weak<cmd>,
    s_owner: &std::rc::Rc<std::cell::UnsafeCell<session>>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut type_0: ::core::ffi::c_int,
) {
    let mut s = s_owner.get();
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
    while wl.is_alive() {
        cmd_list_panes_window(self_0.clone(), s_owner, wl.clone(), item_handle, type_0);
        wl = winlinks_next(wl.get_unchecked());
    }
}
unsafe fn cmd_list_panes_window(
    mut self_0: refbox::Weak<cmd>,
    s_owner: &std::rc::Rc<std::cell::UnsafeCell<session>>,
    mut wl: refbox::Weak<winlink>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut type_0: ::core::ffi::c_int,
) {
    let item = item_handle.get();
    let _s = s_owner.get();
    let queue_client = cmdq_get_client((item).as_ref());
    let mut args: *mut args =
        cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: *mut client = c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: u_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut filter: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut flag: ::core::ffi::c_int = 0;
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_ACTIVITY,
        reversed: 0,
        order_seq: &[],
    };
    template =
        args_get(&*(args), 'F' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    if template.is_null() {
        match type_0 {
            0 => {
                template = b"#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}\0"
                    as *const u8 as *const ::core::ffi::c_char;
            }
            1 => {
                template = b"#{window_index}.#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}\0"
                    as *const u8 as *const ::core::ffi::c_char;
            }
            2 => {
                template = b"#{session_name}:#{window_index}.#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}\0"
                    as *const u8 as *const ::core::ffi::c_char;
            }
            _ => {}
        }
    }
    filter =
        args_get(&*(args), 'f' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    sort_crit.order = sort_order_from_string(
        args_get(&*(args), 'O' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()),
    );
    sort_crit.reversed = args_has(args, 'r' as i32 as u_char);
    let l = sort_get_panes_window(
        &*wl.get_unchecked()
            .window_handle()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get()),
        &sort_crit,
    );
    let n = u_int::try_from(l.len()).expect("too many panes to list");
    i = 0 as u_int;
    while i < n {
        wp = l[i as usize].get();
        let mut ft_owner = format_create_with_client(
            queue_client.as_ref(),
            Some(item_handle),
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
        ft = &raw mut *ft_owner;
        format_add(
            ft,
            b"line\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (n) as u32),
        );
        format_defaults(
            ft,
            (c).as_ref()
                .and_then(|model| model.observer.upgrade())
                .as_ref(),
            Some(s_owner),
            wl.clone(),
            (wp).as_ref()
                .and_then(|model| model.observer.upgrade())
                .as_ref(),
        );
        if !filter.is_null() {
            let expanded = format_expand_cstring(ft, filter);
            flag = format_true(expanded.as_ptr());
        } else {
            flag = 1 as ::core::ffi::c_int;
        }
        if flag != 0 {
            let line = format_expand_cstring(ft, template);
            cmdq_print(item_handle, |out| write_cstr(out, line.as_ptr()));
        }
        format_free(ft_owner);
        i = i.wrapping_add(1);
    }
}
