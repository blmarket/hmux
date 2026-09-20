use crate::src::arguments::{args_count, args_has, args_string};
use crate::src::cfg::{cfg_finished, cfg_print_causes, load_cfg_from_buffer};
use crate::src::cmd::{cmd_get_args, cmd_get_parse_flags};
use crate::src::cmd_queue::{
    cmdq_continue, cmdq_error, cmdq_get_callback1, cmdq_get_client, cmdq_get_target,
    cmdq_insert_after,
};
use crate::src::ffi::libc::{__ctype_b_loc, free, glob, globfree, strcmp, strerror, strlen};
use crate::src::ffi::libevent::{evbuffer_get_length, evbuffer_pullup};
use crate::src::file::file_read;
use crate::src::format::format_single_from_target;
use crate::src::log::log_debug;
use crate::src::server_client::{server_client_get_cwd, server_client_unref};
use crate::src::xmalloc::{xasprintf, xcalloc, xmalloc, xreallocarray, xstrdup};
pub use crate::src::shared::posix_io::{dirent, glob_t, stat};
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_cb, cmdq_item, cmdq_list,
    cmds,
};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{options};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::process::{tmuxpeer};
pub use crate::src::shared::prompt::{prompt};
pub use crate::src::shared::redraw::{redraw_scene};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::ctype::{
    _ISalnum, _ISalpha, _ISblank, _IScntrl, _ISdigit, _ISgraph, _ISlower, _ISprint, _ISpunct,
    _ISspace, _ISupper, _ISxdigit, ctype_code,
};
pub use crate::src::shared::errno::{EINVAL, ENOENT, ENOMEM};
pub use crate::src::shared::abi::{__size_t, ssize_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{
    CMD_FIND_CANFAIL, CMD_PARSE_PARSEONLY, CMD_PARSE_QUIET, CMD_PARSE_VERBOSE,
    CMD_SOURCE_FILE_DEPTH_LIMIT,
};
pub use crate::src::shared::client::{CLIENT_CONTROL};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::arguments::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_14;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_source_file_data {
    pub item: *mut cmdq_item,
    pub client: *mut client,
    pub flags: ::core::ffi::c_int,
    pub after: *mut cmdq_item,
    pub retval: cmd_retval,
    pub current: u_int,
    pub files: *mut *mut ::core::ffi::c_char,
    pub nfiles: u_int,
}

pub const GLOB_NOSPACE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const GLOB_NOMATCH: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
static mut cmd_source_file_depth: u_int = 0;
#[no_mangle]
pub static mut cmd_source_file_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"source-file\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"source\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"t:Fnqv\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: b"[-Fnqv] [-t target-pane] path ...\0" as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_CANFAIL,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_source_file_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_source_file_free_data(mut cdata: *mut cmd_source_file_data) {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < (*cdata).nfiles {
        free(*(*cdata).files.offset(i as isize) as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    free((*cdata).files as *mut ::core::ffi::c_void);
    if !(*cdata).client.is_null() {
        server_client_unref((*cdata).client);
    }
    free(cdata as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmd_source_file_complete_cb(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut cdata: *mut cmd_source_file_data = data as *mut cmd_source_file_data;
    let mut c: *mut client = (*cdata).client;
    if c.is_null() {
        cmd_source_file_depth = cmd_source_file_depth.wrapping_sub(1);
        log_debug(
            b"%s: depth now %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_source_file_complete_cb\0" as *const u8 as *const ::core::ffi::c_char,
            cmd_source_file_depth,
        );
    } else {
        (*c).source_file_depth = (*c).source_file_depth.wrapping_sub(1);
        log_debug(
            b"%s: depth now %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_source_file_complete_cb\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).source_file_depth,
        );
    }
    cfg_print_causes(item);
    cmd_source_file_free_data(cdata);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_source_file_complete(mut cdata: *mut cmd_source_file_data) {
    let mut c: *mut client = (*cdata).client;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    if cfg_finished == 0 {
        cmd_source_file_free_data(cdata);
        return;
    }
    if (*cdata).retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int
        && !c.is_null()
        && (*c).session.is_null()
    {
        (*c).retval = 1 as ::core::ffi::c_int;
    }
    new_item = cmdq_get_callback1(
        b"cmd_source_file_complete_cb\0" as *const u8 as *const ::core::ffi::c_char,
        Some(
            cmd_source_file_complete_cb
                as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
        ),
        cdata as *mut ::core::ffi::c_void,
    );
    cmdq_insert_after((*cdata).after, new_item);
}
unsafe extern "C" fn cmd_source_file_done(
    mut oc: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
    mut buffer: *mut evbuffer,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut cdata: *mut cmd_source_file_data = data as *mut cmd_source_file_data;
    let mut item: *mut cmdq_item = (*cdata).item;
    let mut c: *mut client = (*cdata).client;
    let mut bdata: *mut ::core::ffi::c_void =
        evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_void;
    let mut bsize: size_t = evbuffer_get_length(buffer);
    let mut n: u_int = 0;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    if closed == 0 {
        return;
    }
    if error != 0 as ::core::ffi::c_int {
        cmdq_error(
            item,
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(error),
            path,
        );
    } else if bsize != 0 as size_t {
        if load_cfg_from_buffer(
            bdata,
            bsize,
            path,
            c,
            (*cdata).after,
            target,
            (*cdata).flags,
            &raw mut new_item,
        ) < 0 as ::core::ffi::c_int
        {
            (*cdata).retval = CMD_RETURN_ERROR;
        } else if !new_item.is_null() {
            (*cdata).after = new_item;
        }
    }
    (*cdata).current = (*cdata).current.wrapping_add(1);
    n = (*cdata).current;
    if n < (*cdata).nfiles {
        file_read(
            c,
            *(*cdata).files.offset(n as isize),
            Some(
                cmd_source_file_done
                    as unsafe extern "C" fn(
                        *mut client,
                        *const ::core::ffi::c_char,
                        ::core::ffi::c_int,
                        ::core::ffi::c_int,
                        *mut evbuffer,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            cdata as *mut ::core::ffi::c_void,
        );
    } else {
        cmd_source_file_complete(cdata);
        cmdq_continue(item);
    };
}
unsafe extern "C" fn cmd_source_file_add(
    mut cdata: *mut cmd_source_file_data,
    mut path: *const ::core::ffi::c_char,
) {
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_source_file_add\0" as *const u8 as *const ::core::ffi::c_char,
        path,
    );
    (*cdata).files = xreallocarray(
        (*cdata).files as *mut ::core::ffi::c_void,
        (*cdata).nfiles.wrapping_add(1 as u_int) as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    let fresh0 = (*cdata).nfiles;
    (*cdata).nfiles = (*cdata).nfiles.wrapping_add(1);
    let ref mut fresh1 = *(*cdata).files.offset(fresh0 as isize);
    *fresh1 = xstrdup(path);
}
unsafe extern "C" fn cmd_source_file_quote_for_glob(
    mut path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut quoted: *mut ::core::ffi::c_char = xmalloc(
        (2 as size_t)
            .wrapping_mul(strlen(path))
            .wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    let mut q: *mut ::core::ffi::c_char = quoted;
    let mut p: *const ::core::ffi::c_char = path;
    while *p as ::core::ffi::c_int != '\0' as i32 {
        if (*p as u_char as ::core::ffi::c_int) < 128 as ::core::ffi::c_int
            && *(*__ctype_b_loc()).offset(*p as u_char as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                == 0
            && *p as ::core::ffi::c_int != '/' as i32
        {
            let fresh2 = q;
            q = q.offset(1);
            *fresh2 = '\\' as i32 as ::core::ffi::c_char;
        }
        let fresh3 = p;
        p = p.offset(1);
        let fresh4 = q;
        q = q.offset(1);
        *fresh4 = *fresh3;
    }
    *q = '\0' as i32 as ::core::ffi::c_char;
    return quoted;
}
unsafe extern "C" fn cmd_source_file_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut cdata: *mut cmd_source_file_data = ::core::ptr::null_mut::<cmd_source_file_data>();
    let mut c: *mut client = cmdq_get_client(item);
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let mut pattern: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut g: glob_t = glob_t {
        gl_pathc: 0,
        gl_pathv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        gl_offs: 0,
        gl_flags: 0,
        gl_closedir: None,
        gl_readdir: None,
        gl_opendir: None,
        gl_lstat: None,
        gl_stat: None,
    };
    let mut result: ::core::ffi::c_int = 0;
    let mut parse_flags: ::core::ffi::c_int = 0;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    if c.is_null() {
        if cmd_source_file_depth >= CMD_SOURCE_FILE_DEPTH_LIMIT as u_int {
            cmdq_error(
                item,
                b"too many nested files\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        cmd_source_file_depth = cmd_source_file_depth.wrapping_add(1);
        log_debug(
            b"%s: depth now %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_source_file_exec\0" as *const u8 as *const ::core::ffi::c_char,
            cmd_source_file_depth,
        );
    } else {
        if (*c).source_file_depth >= CMD_SOURCE_FILE_DEPTH_LIMIT as u_int {
            cmdq_error(
                item,
                b"too many nested files\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        (*c).source_file_depth = (*c).source_file_depth.wrapping_add(1);
        log_debug(
            b"%s: depth now %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_source_file_exec\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).source_file_depth,
        );
    }
    cdata = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<cmd_source_file_data>() as size_t,
    ) as *mut cmd_source_file_data;
    (*cdata).item = item;
    (*cdata).client = c;
    if !c.is_null() {
        (*c).references += 1;
    }
    if args_has(args, 'q' as i32 as u_char) != 0 {
        (*cdata).flags |= CMD_PARSE_QUIET;
    }
    if args_has(args, 'n' as i32 as u_char) != 0 {
        (*cdata).flags |= CMD_PARSE_PARSEONLY;
    }
    if c.is_null() || !(*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        parse_flags = cmd_get_parse_flags(self_0);
        if args_has(args, 'v' as i32 as u_char) != 0 || parse_flags & CMD_PARSE_VERBOSE != 0 {
            (*cdata).flags |= CMD_PARSE_VERBOSE;
        }
    }
    cwd = cmd_source_file_quote_for_glob(server_client_get_cwd(
        c,
        ::core::ptr::null_mut::<session>(),
    ));
    i = 0 as u_int;
    while i < args_count(args) {
        path = args_string(args, i);
        if args_has(args, 'F' as i32 as u_char) != 0 {
            free(expanded as *mut ::core::ffi::c_void);
            expanded = format_single_from_target(item, path);
            path = expanded;
        }
        if strcmp(path, b"-\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            cmd_source_file_add(cdata, b"-\0" as *const u8 as *const ::core::ffi::c_char);
        } else {
            if *path as ::core::ffi::c_int == '/' as i32 {
                pattern = xstrdup(path);
            } else {
                xasprintf(
                    &raw mut pattern,
                    b"%s/%s\0" as *const u8 as *const ::core::ffi::c_char,
                    cwd,
                    path,
                );
            }
            log_debug(
                b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                b"cmd_source_file_exec\0" as *const u8 as *const ::core::ffi::c_char,
                pattern,
            );
            result = glob(pattern, 0 as ::core::ffi::c_int, None, &raw mut g);
            if result != 0 as ::core::ffi::c_int {
                if result != GLOB_NOMATCH || !(*cdata).flags & CMD_PARSE_QUIET != 0 {
                    if result == GLOB_NOMATCH {
                        error = strerror(ENOENT);
                    } else if result == GLOB_NOSPACE {
                        error = strerror(ENOMEM);
                    } else {
                        error = strerror(EINVAL);
                    }
                    cmdq_error(
                        item,
                        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        error,
                        path,
                    );
                    retval = CMD_RETURN_ERROR;
                }
                globfree(&raw mut g);
                free(pattern as *mut ::core::ffi::c_void);
            } else {
                free(pattern as *mut ::core::ffi::c_void);
                j = 0 as u_int;
                while (j as __size_t) < g.gl_pathc {
                    cmd_source_file_add(cdata, *g.gl_pathv.offset(j as isize));
                    j = j.wrapping_add(1);
                }
                globfree(&raw mut g);
            }
        }
        i = i.wrapping_add(1);
    }
    free(expanded as *mut ::core::ffi::c_void);
    (*cdata).after = item;
    (*cdata).retval = retval;
    if (*cdata).nfiles != 0 as u_int {
        file_read(
            c,
            *(*cdata).files.offset(0 as ::core::ffi::c_int as isize),
            Some(
                cmd_source_file_done
                    as unsafe extern "C" fn(
                        *mut client,
                        *const ::core::ffi::c_char,
                        ::core::ffi::c_int,
                        ::core::ffi::c_int,
                        *mut evbuffer,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            cdata as *mut ::core::ffi::c_void,
        );
        retval = CMD_RETURN_WAIT;
    } else {
        cmd_source_file_complete(cdata);
    }
    free(cwd as *mut ::core::ffi::c_void);
    return retval;
}
