use crate::src::arguments::args_has;
use crate::src::cmd::cmd_get_args;
use crate::src::cmd_queue::{cmdq_get_target_client, cmdq_print};
use crate::src::ffi::libc::free;
use crate::src::format::{
    format_add, format_add_tv, format_create_from_target, format_expand, format_free,
};
use crate::src::job::job_print_summary;
use crate::src::server::message_log;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::command::{CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_TFLAG};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{format_job_tree, format_tree};
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
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
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
pub use crate::src::shared::status::{message_entry, message_list};
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tty::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::tty_term::tty_terms;
use crate::src::tty_term::{tty_term_describe, tty_term_ncodes};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub const SHOW_MESSAGES_TEMPLATE: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"#{t/p:message_time}: #{message_text}\0",
    )
};
#[no_mangle]
pub static mut cmd_show_messages_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"show-messages\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"showmsgs\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"JTt:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-JT] [-t target-client]\0" as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        flags: CMD_AFTERHOOK | CMD_CLIENT_TFLAG | CMD_CLIENT_CANFAIL,
        exec: Some(
            cmd_show_messages_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_show_messages_terminals(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
    mut blank: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut term: *mut tty_term = ::core::ptr::null_mut::<tty_term>();
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    n = 0 as u_int;
    term = tty_terms.lh_first;
    while !term.is_null() {
        if !(args_has(args, 't' as i32 as u_char) != 0 && !tc.is_null() && term != (*tc).tty.term) {
            if blank != 0 {
                cmdq_print(
                    item,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                );
                blank = 0 as ::core::ffi::c_int;
            }
            cmdq_print(
                item,
                b"Terminal %u: %s for %s, flags=0x%x:\0" as *const u8 as *const ::core::ffi::c_char,
                n,
                (*term).name,
                (*(*(*term).tty).client).name,
                (*term).flags,
            );
            n = n.wrapping_add(1);
            i = 0 as u_int;
            while i < tty_term_ncodes() {
                cmdq_print(
                    item,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    tty_term_describe(term, i as tty_code_code),
                );
                i = i.wrapping_add(1);
            }
        }
        term = (*term).entry.le_next;
    }
    return (n != 0 as u_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn cmd_show_messages_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut done: ::core::ffi::c_int = 0;
    let mut blank: ::core::ffi::c_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    blank = 0 as ::core::ffi::c_int;
    done = blank;
    if args_has(args, 'T' as i32 as u_char) != 0 {
        blank = cmd_show_messages_terminals(self_0, item, blank);
        done = 1 as ::core::ffi::c_int;
    }
    if args_has(args, 'J' as i32 as u_char) != 0 {
        job_print_summary(item, blank);
        done = 1 as ::core::ffi::c_int;
    }
    if done != 0 {
        return CMD_RETURN_NORMAL;
    }
    ft = format_create_from_target(item);
    for msg in message_log.iter_rev() {
        format_add(
            ft,
            b"message_text\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            msg.msg.as_ptr(),
        );
        format_add(
            ft,
            b"message_number\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            msg.msg_num,
        );
        let mut msg_time = msg.msg_time;
        format_add_tv(
            ft,
            b"message_time\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut msg_time,
        );
        s = format_expand(ft, SHOW_MESSAGES_TEMPLATE.as_ptr());
        cmdq_print(item, b"%s\0" as *const u8 as *const ::core::ffi::c_char, s);
        free(s as *mut ::core::ffi::c_void);
    }
    format_free(ft);
    return CMD_RETURN_NORMAL;
}
