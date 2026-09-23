use crate::src::arguments::{args_get, args_has, args_string};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd_queue::{cmdq_continue, cmdq_error, cmdq_get_client, cmdq_get_target_client};
use crate::src::ffi::libc::{free, memcpy, strerror};
use crate::src::file::file_read_with_cleanup;
use crate::src::format::format_single_from_target;
use crate::src::paste::paste_set;
use crate::src::reactor::{evbuffer_get_length, evbuffer_pullup};
use crate::src::server_client::server_client_unref;
pub use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
pub use crate::src::shared::client::CLIENT_DEAD;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
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
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::tty::tty_set_selection;
use crate::src::xmalloc::xmalloc;
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[repr(C)]
pub struct cmd_load_buffer_data {
    pub client: *mut client,
    pub item: *mut cmdq_item,
    pub name: Option<CString>,
}

unsafe fn cmd_load_buffer_release(cdata: Box<cmd_load_buffer_data>) {
    if !cdata.client.is_null() {
        server_client_unref(cdata.client);
    }
}

unsafe extern "C" fn cmd_load_buffer_cancelled(data: *mut ::core::ffi::c_void) {
    cmd_load_buffer_release(Box::from_raw(data.cast::<cmd_load_buffer_data>()));
}
#[no_mangle]
pub static mut cmd_load_buffer_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"load-buffer\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"loadb\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"b:t:w\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-b buffer-name] [-t target-client] path\0" as *const u8
            as *const ::core::ffi::c_char,
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
            cmd_load_buffer_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_load_buffer_done(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
    mut buffer: *mut evbuffer,
    mut data: *mut ::core::ffi::c_void,
) {
    // Progress notifications do not need contiguous storage. Coalesce only
    // once, after the complete file has arrived.
    if closed == 0 {
        return;
    }
    let cdata = Box::from_raw(data.cast::<cmd_load_buffer_data>());
    let mut tc: *mut client = cdata.client;
    let mut item: *mut cmdq_item = cdata.item;
    let mut bdata: *mut ::core::ffi::c_void =
        evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_void;
    let mut bsize: size_t = evbuffer_get_length(buffer);
    let mut copy: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if error != 0 as ::core::ffi::c_int {
        cmdq_error(
            item,
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(error),
            path,
        );
    } else if bsize != 0 as size_t {
        copy = xmalloc(bsize);
        memcpy(copy, bdata, bsize);
        if paste_set(
            copy as *mut ::core::ffi::c_char,
            bsize,
            cdata
                .name
                .as_ref()
                .map_or(::core::ptr::null(), |name| name.as_ptr()),
            &raw mut cause,
        ) != 0 as ::core::ffi::c_int
        {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            free(copy);
        } else if !tc.is_null()
            && !(*tc).session.is_null()
            && !(*tc).flags & CLIENT_DEAD as uint64_t != 0
        {
            tty_set_selection(
                &raw mut (*tc).tty,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                copy as *const ::core::ffi::c_char,
                bsize,
            );
        }
    }
    cmd_load_buffer_release(cdata);
    cmdq_continue(item);
}
unsafe extern "C" fn cmd_load_buffer_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut cdata = Box::new(cmd_load_buffer_data {
        client: ::core::ptr::null_mut(),
        item,
        name: None,
    });
    let mut bufname: *const ::core::ffi::c_char = args_get(args, 'b' as i32 as u_char);
    let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !bufname.is_null() {
        cdata.name = Some(CStr::from_ptr(bufname).to_owned());
    }
    if args_has(args, 'w' as i32 as u_char) != 0 && !tc.is_null() {
        cdata.client = tc;
        (*tc).references += 1;
    }
    path = format_single_from_target(item, args_string(args, 0 as u_int));
    file_read_with_cleanup(
        cmdq_get_client(item),
        path,
        Some(
            cmd_load_buffer_done
                as unsafe extern "C" fn(
                    *mut client,
                    *const ::core::ffi::c_char,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                    *mut evbuffer,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        Box::into_raw(cdata).cast(),
        Some(cmd_load_buffer_cancelled),
    );
    free(path as *mut ::core::ffi::c_void);
    return CMD_RETURN_WAIT;
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmux_buffer::{Buf, BufMut, Buffer, SegmentedBuf};

    #[test]
    fn file_progress_keeps_segments_until_completion() {
        let mut data = cmd_load_buffer_data {
            client: std::ptr::null_mut(),
            item: std::ptr::null_mut(),
            name: None,
        };
        let mut buffer = SegmentedBuf::from(vec![1; 4096]);
        let first = buffer.chunk().as_ptr();
        for count in 2..=16 {
            buffer.put(SegmentedBuf::from(vec![2; 4096]));
            unsafe {
                cmd_load_buffer_done(
                    std::ptr::null_mut(),
                    c"input".as_ptr(),
                    0,
                    0,
                    &mut buffer,
                    (&mut data as *mut cmd_load_buffer_data).cast(),
                );
            }
            assert_eq!(buffer.chunks().count(), count);
            assert_eq!(buffer.chunk().as_ptr(), first);
            assert_eq!(buffer.remaining(), count * 4096);
        }
    }
}
