use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::cmd_find::{cmd_find_copy_state, cmd_find_valid_state};
use crate::src::ffi::libc::{__ctype_tolower_loc, memcpy, strcasestr, strlen, strstr};
use crate::src::format::{
    format_add, format_create, format_defaults, format_defaults_paste_buffer,
    format_expand_cstring, format_free, format_true,
};
use crate::src::grid::grid_default_cell;
use crate::src::key_string::key_string_parse_cstr;
use crate::src::mode_tree::{
    mode_tree_add, mode_tree_build, mode_tree_down, mode_tree_draw, mode_tree_each_tagged,
    mode_tree_free, mode_tree_get_current, mode_tree_key, mode_tree_resize, mode_tree_run_command,
    mode_tree_start, mode_tree_up, mode_tree_zoom,
};
use crate::src::paste::{
    paste_buffer_data, paste_buffer_name, paste_buffer_order, paste_free, paste_get_name,
    paste_is_empty, paste_replace,
};
use crate::src::screen_write::{
    screen_write_box, screen_write_clearcharacter, screen_write_cursormove, screen_write_nputs,
    screen_write_start, screen_write_stop,
};
use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{__int32_t, ssize_t};
pub use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::FORMAT_NONE;
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
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::menu_data;
pub use crate::src::shared::menu::menu_item;
use crate::src::shared::message::*;
pub use crate::src::shared::mode_tree::{
    mode_tree_build_cb, mode_tree_data, mode_tree_draw_cb, mode_tree_each_cb, mode_tree_height_cb,
    mode_tree_help_cb, mode_tree_item, mode_tree_key_cb, mode_tree_menu_cb, mode_tree_search_cb,
    mode_tree_sort_cb, mode_tree_swap_cb,
};
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
    PANE_REDRAW,
};
pub use crate::src::shared::paste::{
    paste_buffer, paste_buffer_name_entry, paste_buffer_time_entry,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::screen_write::{screen_write_ctx, screen_write_init_ctx_cb};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
pub use crate::src::shared::spawn::{spawn_editor_state, spawn_finish_edit_cb};
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{
    tty, tty_code, tty_ctx, tty_ctx_c2rust_unnamed, tty_ctx_c2rust_unnamed_data,
    tty_ctx_c2rust_unnamed_sel, tty_ctx_redraw_cb, tty_ctx_set_client_cb, tty_key, tty_style_ctx,
    tty_term, tty_term_entry,
};
pub use crate::src::shared::vis::{VIS_CSTYLE, VIS_OCTAL, VIS_TAB};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::sort::sort_get_buffers;
use crate::src::spawn::{spawn_cancel_editor, spawn_editor, spawn_get_editor_pid};
use crate::src::utf8::utf8_strvis;
use crate::src::window::{window_pane_find_by_id, window_pane_reset_mode};
use crate::src::xmalloc::xsnprintf;
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub use crate::src::shared::key::key_code_enum as C2RustUnnamed_38;

#[repr(C)]
pub struct window_buffer_modedata {
    pub wp: *mut window_pane,
    pub fs: cmd_find_state,
    pub data: *mut mode_tree_data,
    pub editor: *mut spawn_editor_state,
    pub edit: *mut window_buffer_editdata,
    pub command: CString,
    pub format: CString,
    pub key_format: CString,
    item_list: Vec<Box<window_buffer_itemdata>>,
}
#[repr(C)]
pub struct window_buffer_itemdata {
    pub name: std::ffi::CString,
    pub order: u_int,
    pub size: size_t,
}

impl window_buffer_itemdata {
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            order: unsafe { ::core::mem::zeroed() },
            size: unsafe { ::core::mem::zeroed() },
        }
    }
}

// The mode tree borrows `item` during callbacks. The box keeps its address
// stable as the list grows, and `name` keeps the C string alive with it.

#[repr(C)]
pub struct window_buffer_editdata {
    pub wp_id: u_int,
    pub name: *mut ::core::ffi::c_char,
    pub pb: *mut paste_buffer,
    pub editor: *mut spawn_editor_state,
    // The callback and mode teardown borrow `name` from this stable owner.
    name_owner: CString,
}

#[inline]
unsafe extern "C" fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}

pub const WINDOW_BUFFER_DEFAULT_COMMAND: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"paste-buffer -p -b '%%'\0")
};
pub const WINDOW_BUFFER_DEFAULT_FORMAT: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"#{t/p:buffer_created}: #{buffer_sample}\0",
    )
};
pub const WINDOW_BUFFER_DEFAULT_KEY_FORMAT: [::core::ffi::c_char; 83] = unsafe {
    ::core::mem::transmute::<[u8; 83], [::core::ffi::c_char; 83]>(
        *b"#{?#{e|<:#{line},10},#{line},#{e|<:#{line},36},M-#{a:#{e|+:97,#{e|-:#{line},10}}}}\0",
    )
};
static mut window_buffer_menu_items: [menu_item; 12] = [
    menu_item {
        name: b"Paste\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'p' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Paste Tagged\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'P' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Tag\0" as *const u8 as *const ::core::ffi::c_char,
        key: 't' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Tag All\0" as *const u8 as *const ::core::ffi::c_char,
        key: '\u{14}' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Tag None\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'T' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Delete\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'd' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Delete Tagged\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'D' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Cancel\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'q' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];
#[no_mangle]
pub static mut window_buffer_mode: window_mode = unsafe {
    window_mode {
        name: b"buffer-mode\0" as *const u8 as *const ::core::ffi::c_char,
        default_format: WINDOW_BUFFER_DEFAULT_FORMAT.as_ptr(),
        flags: 0,
        init: Some(
            window_buffer_init
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_buffer_free as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        resize: Some(
            window_buffer_resize
                as unsafe extern "C" fn(*mut window_mode_entry, u_int, u_int) -> (),
        ),
        update: Some(window_buffer_update as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        style_changed: None,
        key: Some(
            window_buffer_key
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut client,
                    *mut session,
                    *mut winlink,
                    key_code,
                    *mut mouse_event,
                ) -> (),
        ),
        key_table: None,
        command: None,
        formats: None,
        get_screen: None,
    }
};
static mut window_buffer_order_seq: [sort_order; 4] =
    [SORT_CREATION, SORT_NAME, SORT_SIZE, SORT_END];
fn window_buffer_add_item(
    items: &mut Vec<Box<window_buffer_itemdata>>,
    name: &CStr,
) -> *mut window_buffer_itemdata {
    let name = name.to_owned();
    let mut owned = Box::new(window_buffer_itemdata {
        name: name,
        order: 0,
        size: 0,
    });
    let item = &mut *owned as *mut window_buffer_itemdata;
    items.push(owned);
    item
}

fn window_buffer_clear_items(items: &mut Vec<Box<window_buffer_itemdata>>) {
    for item in items.drain(..) {
        drop(item);
    }
}
unsafe extern "C" fn window_buffer_build(
    mut modedata: *mut ::core::ffi::c_void,
    mut sort_crit: *mut sort_criteria,
    mut tag: *mut uint64_t,
    mut filter: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_buffer_modedata = modedata as *mut window_buffer_modedata;
    let mut item: *mut window_buffer_itemdata = ::core::ptr::null_mut::<window_buffer_itemdata>();
    let mut i: u_int = 0;
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    window_buffer_clear_items(&mut (*data).item_list);
    let buffers = sort_get_buffers(sort_crit);
    for pb in buffers {
        let name = CStr::from_ptr(paste_buffer_name(pb));
        item = window_buffer_add_item(&mut (*data).item_list, name);
        paste_buffer_data(pb, &raw mut (*item).size);
        (*item).order = paste_buffer_order(pb);
    }
    if cmd_find_valid_state(&raw mut (*data).fs) != 0 {
        s = (*data).fs.s;
        wl = (*data).fs.wl;
        wp = (*data).fs.wp;
    }
    let mut current_block_32: u64;
    i = 0 as u_int;
    while (i as usize) < (*data).item_list.len() {
        item = {
            let items = &mut (*data).item_list;
            &mut *items[i as usize] as *mut window_buffer_itemdata
        };
        pb = paste_get_name(((*item).name).as_ptr().cast_mut());
        if !pb.is_null() {
            ft = format_create(
                ::core::ptr::null_mut::<client>(),
                ::core::ptr::null_mut::<cmdq_item>(),
                FORMAT_NONE,
                0 as ::core::ffi::c_int,
            );
            format_defaults(ft, ::core::ptr::null_mut::<client>(), s, wl, wp);
            format_defaults_paste_buffer(ft, pb);
            if !filter.is_null() {
                let cp = format_expand_cstring(ft, filter);
                if format_true(cp.as_ptr()) == 0 {
                    format_free(ft);
                    current_block_32 = 5948590327928692120;
                } else {
                    current_block_32 = 1608152415753874203;
                }
            } else {
                current_block_32 = 1608152415753874203;
            }
            match current_block_32 {
                5948590327928692120 => {}
                _ => {
                    let text = format_expand_cstring(ft, (*data).format.as_ptr());
                    mode_tree_add(
                        (*data).data,
                        ::core::ptr::null_mut::<mode_tree_item>(),
                        item as *mut ::core::ffi::c_void,
                        (*item).order as uint64_t,
                        ((*item).name).as_ptr().cast_mut(),
                        text.as_ptr(),
                        -(1 as ::core::ffi::c_int),
                    );
                    format_free(ft);
                }
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn window_buffer_draw(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut item: *mut window_buffer_itemdata = itemdata as *mut window_buffer_itemdata;
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut pdata: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut buf = Vec::<u8>::new();
    let mut psize: size_t = 0;
    let mut i: u_int = 0;
    let mut cx: u_int = (*(*ctx).s).cx;
    let mut cy: u_int = (*(*ctx).s).cy;
    pb = paste_get_name(((*item).name).as_ptr().cast_mut());
    if pb.is_null() {
        return;
    }
    end = paste_buffer_data(pb, &raw mut psize);
    pdata = end;
    i = 0 as u_int;
    while i < sy {
        start = end;
        while end != pdata.offset(psize as isize) && *end as ::core::ffi::c_int != '\n' as i32 {
            end = end.offset(1);
        }
        let line_len = end.offset_from(start) as size_t;
        buf.resize(4 * (line_len + 1), 0);
        utf8_strvis(
            buf.as_mut_ptr().cast(),
            start,
            line_len,
            VIS_OCTAL | VIS_CSTYLE | VIS_TAB,
        );
        if buf[0] != 0 {
            screen_write_cursormove(
                ctx,
                cx as ::core::ffi::c_int,
                cy.wrapping_add(i) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_nputs(
                ctx,
                sx as ssize_t,
                &raw const grid_default_cell,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                buf.as_ptr().cast::<::core::ffi::c_char>(),
            );
        }
        if end == pdata.offset(psize as isize) {
            break;
        }
        end = end.offset(1);
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn window_buffer_find(
    mut data: *const ::core::ffi::c_void,
    mut datalen: size_t,
    mut find: *const ::core::ffi::c_void,
    mut findlen: size_t,
    mut icase: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut udata: *const u_char = data as *const u_char;
    let mut ufind: *const u_char = find as *const u_char;
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    if findlen == 0 as size_t || datalen < findlen {
        return 0 as ::core::ffi::c_int;
    }
    i = 0 as size_t;
    while i.wrapping_add(findlen) <= datalen {
        j = 0 as size_t;
        while j < findlen {
            if icase == 0
                && *udata.offset(i.wrapping_add(j) as isize) as ::core::ffi::c_int
                    != *ufind.offset(j as isize) as ::core::ffi::c_int
            {
                break;
            }
            if icase != 0
                && ({
                    let mut __res: ::core::ffi::c_int = 0;
                    if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                        if 0 != 0 {
                            let mut __c: ::core::ffi::c_int =
                                *udata.offset(i.wrapping_add(j) as isize) as ::core::ffi::c_int;
                            __res = (if __c < -(128 as ::core::ffi::c_int)
                                || __c > 255 as ::core::ffi::c_int
                            {
                                __c as __int32_t
                            } else {
                                *(*__ctype_tolower_loc()).offset(__c as isize)
                            }) as ::core::ffi::c_int;
                        } else {
                            __res = tolower(
                                *udata.offset(i.wrapping_add(j) as isize) as ::core::ffi::c_int
                            );
                        }
                    } else {
                        __res =
                            *(*__ctype_tolower_loc())
                                .offset(*udata.offset(i.wrapping_add(j) as isize)
                                    as ::core::ffi::c_int
                                    as isize) as ::core::ffi::c_int;
                    }
                    __res
                }) != ({
                    let mut __res: ::core::ffi::c_int = 0;
                    if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                        if 0 != 0 {
                            let mut __c: ::core::ffi::c_int =
                                *ufind.offset(j as isize) as ::core::ffi::c_int;
                            __res = (if __c < -(128 as ::core::ffi::c_int)
                                || __c > 255 as ::core::ffi::c_int
                            {
                                __c as __int32_t
                            } else {
                                *(*__ctype_tolower_loc()).offset(__c as isize)
                            }) as ::core::ffi::c_int;
                        } else {
                            __res = tolower(*ufind.offset(j as isize) as ::core::ffi::c_int);
                        }
                    } else {
                        __res = *(*__ctype_tolower_loc())
                            .offset(*ufind.offset(j as isize) as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int;
                    }
                    __res
                })
            {
                break;
            }
            j = j.wrapping_add(1);
        }
        if j == findlen {
            return 1 as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_buffer_search(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut ss: *const ::core::ffi::c_char,
    mut icase: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut item: *mut window_buffer_itemdata = itemdata as *mut window_buffer_itemdata;
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut bufdata: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufsize: size_t = 0;
    pb = paste_get_name(((*item).name).as_ptr().cast_mut());
    if pb.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if icase != 0 {
        if !strcasestr(((*item).name).as_ptr().cast_mut(), ss).is_null() {
            return 1 as ::core::ffi::c_int;
        }
        bufdata = paste_buffer_data(pb, &raw mut bufsize);
        return window_buffer_find(
            bufdata as *const ::core::ffi::c_void,
            bufsize,
            ss as *const ::core::ffi::c_void,
            strlen(ss),
            icase,
        );
    } else {
        if !strstr(((*item).name).as_ptr().cast_mut(), ss).is_null() {
            return 1 as ::core::ffi::c_int;
        }
        bufdata = paste_buffer_data(pb, &raw mut bufsize);
        return window_buffer_find(
            bufdata as *const ::core::ffi::c_void,
            bufsize,
            ss as *const ::core::ffi::c_void,
            strlen(ss),
            icase,
        );
    };
}
unsafe extern "C" fn window_buffer_menu(
    mut modedata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut data: *mut window_buffer_modedata = modedata as *mut window_buffer_modedata;
    let mut wp: *mut window_pane = (*data).wp;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    wme = (*wp).modes.active;
    if wme.is_null() || (*wme).data != modedata {
        return;
    }
    window_buffer_key(
        wme,
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        key,
        ::core::ptr::null_mut::<mouse_event>(),
    );
}
unsafe extern "C" fn window_buffer_get_key(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut line: u_int,
) -> key_code {
    let mut data: *mut window_buffer_modedata = modedata as *mut window_buffer_modedata;
    let mut item: *mut window_buffer_itemdata = itemdata as *mut window_buffer_itemdata;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut key: key_code = 0;
    if cmd_find_valid_state(&raw mut (*data).fs) != 0 {
        s = (*data).fs.s;
        wl = (*data).fs.wl;
        wp = (*data).fs.wp;
    }
    pb = paste_get_name(((*item).name).as_ptr().cast_mut());
    if pb.is_null() {
        return KEYC_NONE as ::core::ffi::c_ulong as key_code;
    }
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    format_defaults(ft, ::core::ptr::null_mut::<client>(), s, wl, wp);
    format_defaults_paste_buffer(ft, pb);
    format_add(
        ft,
        b"line\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        line,
    );
    let expanded = format_expand_cstring(ft, (*data).key_format.as_ptr());
    key = key_string_parse_cstr(expanded.as_c_str()).unwrap_or(KEYC_UNKNOWN);
    format_free(ft);
    return key;
}
unsafe extern "C" fn window_buffer_sort(mut sort_crit: *mut sort_criteria) {
    (*sort_crit).order_seq = &raw mut window_buffer_order_seq as *mut sort_order;
    if (*sort_crit).order as ::core::ffi::c_uint
        == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*sort_crit).order = *(*sort_crit)
            .order_seq
            .offset(0 as ::core::ffi::c_int as isize);
    }
}
static mut window_buffer_help_lines: [*const ::core::ffi::c_char; 8] = [
    b"#[fg=themelightgrey]      Enter #[#{E:tree-mode-border-style},acs]x#[default] Paste selected %1\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          p #[#{E:tree-mode-border-style},acs]x#[default] Paste selected %1\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          P #[#{E:tree-mode-border-style},acs]x#[default] Paste tagged %1s\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          d #[#{E:tree-mode-border-style},acs]x#[default] Delete selected %1\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          D #[#{E:tree-mode-border-style},acs]x#[default] Delete tagged %1s\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          e #[#{E:tree-mode-border-style},acs]x#[default] Open %1 in editor\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          f #[#{E:tree-mode-border-style},acs]x#[default] Enter a filter\0"
        as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
unsafe extern "C" fn window_buffer_help(
    mut width: *mut u_int,
    mut item: *mut *const ::core::ffi::c_char,
) -> *mut *const ::core::ffi::c_char {
    *width = 0 as u_int;
    *item = b"buffer\0" as *const u8 as *const ::core::ffi::c_char;
    return &raw mut window_buffer_help_lines as *mut *const ::core::ffi::c_char;
}
unsafe extern "C" fn window_buffer_init(
    mut wme: *mut window_mode_entry,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_buffer_modedata = ::core::ptr::null_mut::<window_buffer_modedata>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let format = if args.is_null() || args_has(args, 'F' as i32 as u_char) == 0 {
        CStr::from_ptr(WINDOW_BUFFER_DEFAULT_FORMAT.as_ptr()).to_owned()
    } else {
        CStr::from_ptr(args_get(args, 'F' as i32 as u_char)).to_owned()
    };
    let key_format = if args.is_null() || args_has(args, 'K' as i32 as u_char) == 0 {
        CStr::from_ptr(WINDOW_BUFFER_DEFAULT_KEY_FORMAT.as_ptr()).to_owned()
    } else {
        CStr::from_ptr(args_get(args, 'K' as i32 as u_char)).to_owned()
    };
    let command = if args.is_null() || args_count(args) == 0 as u_int {
        CStr::from_ptr(WINDOW_BUFFER_DEFAULT_COMMAND.as_ptr()).to_owned()
    } else {
        CStr::from_ptr(args_string(args, 0 as u_int)).to_owned()
    };
    data = Box::into_raw(Box::new(window_buffer_modedata {
        wp,
        fs: ::core::mem::zeroed(),
        data: ::core::ptr::null_mut(),
        editor: ::core::ptr::null_mut(),
        edit: ::core::ptr::null_mut(),
        command,
        format,
        key_format,
        item_list: Vec::new(),
    }));
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).wp = wp;
    cmd_find_copy_state(&raw mut (*data).fs, fs);
    (*data).data = mode_tree_start(
        wp,
        args,
        Some(
            window_buffer_build
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut sort_criteria,
                    *mut uint64_t,
                    *const ::core::ffi::c_char,
                ) -> (),
        ),
        Some(
            window_buffer_draw
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    *mut screen_write_ctx,
                    u_int,
                    u_int,
                ) -> (),
        ),
        Some(
            window_buffer_search
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    *const ::core::ffi::c_char,
                    ::core::ffi::c_int,
                ) -> ::core::ffi::c_int,
        ),
        Some(
            window_buffer_menu
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut client, key_code) -> (),
        ),
        None,
        Some(
            window_buffer_get_key
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    u_int,
                ) -> key_code,
        ),
        None,
        Some(window_buffer_sort as unsafe extern "C" fn(*mut sort_criteria) -> ()),
        Some(
            window_buffer_help
                as unsafe extern "C" fn(
                    *mut u_int,
                    *mut *const ::core::ffi::c_char,
                ) -> *mut *const ::core::ffi::c_char,
        ),
        data as *mut ::core::ffi::c_void,
        &raw const window_buffer_menu_items as *const menu_item,
        &raw mut s,
    );
    mode_tree_zoom((*data).data, args);
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    return s;
}
unsafe extern "C" fn window_buffer_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_buffer_modedata = (*wme).data as *mut window_buffer_modedata;
    if data.is_null() {
        return;
    }
    if !(*data).editor.is_null() {
        spawn_cancel_editor((*data).editor);
        window_buffer_finish_edit((*data).edit as *mut window_buffer_editdata);
    }
    mode_tree_free((*data).data);
    window_buffer_clear_items(&mut (*data).item_list);
    drop(Box::from_raw(data));
}
unsafe extern "C" fn window_buffer_resize(
    mut wme: *mut window_mode_entry,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_buffer_modedata = (*wme).data as *mut window_buffer_modedata;
    mode_tree_resize((*data).data, sx, sy);
}
unsafe extern "C" fn window_buffer_update(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_buffer_modedata = (*wme).data as *mut window_buffer_modedata;
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    window_buffer_draw_waiting(data);
    (*(*data).wp).flags |= PANE_REDRAW;
}
unsafe extern "C" fn window_buffer_do_delete(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut data: *mut window_buffer_modedata = modedata as *mut window_buffer_modedata;
    let mut item: *mut window_buffer_itemdata = itemdata as *mut window_buffer_itemdata;
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    if item == mode_tree_get_current((*data).data) as *mut window_buffer_itemdata
        && mode_tree_down((*data).data, 0 as ::core::ffi::c_int) == 0
    {
        mode_tree_up((*data).data, 0 as ::core::ffi::c_int);
    }
    pb = paste_get_name(((*item).name).as_ptr().cast_mut());
    if !pb.is_null() {
        paste_free(pb);
    }
}
unsafe extern "C" fn window_buffer_do_paste(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut data: *mut window_buffer_modedata = modedata as *mut window_buffer_modedata;
    let mut item: *mut window_buffer_itemdata = itemdata as *mut window_buffer_itemdata;
    if !paste_get_name(((*item).name).as_ptr().cast_mut()).is_null() {
        mode_tree_run_command(
            c,
            ::core::ptr::null_mut::<cmd_find_state>(),
            (*data).command.as_ptr(),
            ((*item).name).as_ptr().cast_mut(),
        );
    }
}
unsafe extern "C" fn window_buffer_finish_edit(ed: *mut window_buffer_editdata) {
    drop(Box::from_raw(ed));
}
unsafe extern "C" fn window_buffer_draw_waiting(mut data: *mut window_buffer_modedata) {
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut s: *mut screen = (*(*data).wp).screen;
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut text: [::core::ffi::c_char; 128] = [0; 128];
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut box_w: u_int = 0;
    let mut box_h: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut text_x: u_int = 0;
    let mut textlen: size_t = 0;
    let mut pid: pid_t = 0;
    if (*data).editor.is_null() {
        return;
    }
    sx = (*(*s).grid).sx;
    sy = (*(*s).grid).sy;
    if sx == 0 as u_int || sy == 0 as u_int {
        return;
    }
    pid = spawn_get_editor_pid((*data).editor);
    if pid == -(1 as ::core::ffi::c_int) {
        xsnprintf(
            &raw mut text as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"WAITING FOR EDITOR\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        xsnprintf(
            &raw mut text as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"WAITING FOR EDITOR (PID %ld)\0" as *const u8 as *const ::core::ffi::c_char,
            pid as ::core::ffi::c_long,
        );
    }
    textlen = strlen(&raw mut text as *mut ::core::ffi::c_char);
    box_w = textlen.wrapping_add(4 as size_t) as u_int;
    box_h = 3 as u_int;
    if sx < box_w || sy < box_h {
        return;
    }
    x = sx.wrapping_sub(box_w).wrapping_div(2 as u_int);
    y = sy.wrapping_sub(box_h).wrapping_div(2 as u_int);
    text_x = (x as size_t).wrapping_add(
        (box_w as size_t)
            .wrapping_sub(textlen)
            .wrapping_div(2 as size_t),
    ) as u_int;
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    screen_write_start(&raw mut ctx, s);
    screen_write_cursormove(
        &raw mut ctx,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_box(
        &raw mut ctx,
        box_w,
        box_h,
        BOX_LINES_DEFAULT,
        &raw mut gc,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
    screen_write_cursormove(
        &raw mut ctx,
        x.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        y.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_clearcharacter(&raw mut ctx, box_w.wrapping_sub(2 as u_int), gc.bg as u_int);
    screen_write_cursormove(
        &raw mut ctx,
        text_x as ::core::ffi::c_int,
        y.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_nputs(
        &raw mut ctx,
        box_w.wrapping_sub(2 as u_int) as ssize_t,
        &raw mut gc,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut text as *mut ::core::ffi::c_char,
    );
    screen_write_stop(&raw mut ctx);
}
unsafe extern "C" fn window_buffer_edit_close_cb(
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut ed: *mut window_buffer_editdata = arg as *mut window_buffer_editdata;
    let mut oldlen: size_t = 0;
    let mut oldbuf: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut data: *mut window_buffer_modedata = ::core::ptr::null_mut::<window_buffer_modedata>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    wp = window_pane_find_by_id((*ed).wp_id);
    if !wp.is_null() {
        wme = (*wp).modes.active;
        if !wme.is_null() && (*wme).mode == &raw const window_buffer_mode {
            data = (*wme).data as *mut window_buffer_modedata;
            if (*data).editor == (*ed).editor {
                (*data).editor = ::core::ptr::null_mut::<spawn_editor_state>();
                (*data).edit = ::core::ptr::null_mut::<window_buffer_editdata>();
            }
        }
    }
    if buf.is_null() || len == 0 as size_t {
        window_buffer_finish_edit(ed);
        return;
    }
    pb = paste_get_name((*ed).name);
    if pb.is_null() || pb != (*ed).pb {
        window_buffer_finish_edit(ed);
        return;
    }
    oldbuf = paste_buffer_data(pb, &raw mut oldlen);
    if oldlen != 0 as size_t
        && *oldbuf.offset(oldlen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            != '\n' as i32
        && *buf.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == '\n' as i32
    {
        len = len.wrapping_sub(1);
    }
    if len != 0 as size_t {
        paste_replace(pb, buf, len);
    }
    wp = window_pane_find_by_id((*ed).wp_id);
    if !wp.is_null() {
        wme = (*wp).modes.active;
        if !wme.is_null() && (*wme).mode == &raw const window_buffer_mode {
            data = (*wme).data as *mut window_buffer_modedata;
            mode_tree_build((*data).data);
            mode_tree_draw((*data).data);
            window_buffer_draw_waiting(data);
        }
        (*wp).flags |= PANE_REDRAW;
    }
    window_buffer_finish_edit(ed);
}
unsafe extern "C" fn window_buffer_start_edit(
    mut data: *mut window_buffer_modedata,
    mut item: *mut window_buffer_itemdata,
    mut c: *mut client,
) {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut buf: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut ed: *mut window_buffer_editdata = ::core::ptr::null_mut::<window_buffer_editdata>();
    if !(*data).editor.is_null() {
        return;
    }
    pb = paste_get_name(((*item).name).as_ptr().cast_mut());
    if pb.is_null() {
        return;
    }
    buf = paste_buffer_data(pb, &raw mut len);
    let name_owner = CStr::from_ptr(paste_buffer_name(pb)).to_owned();
    ed = Box::into_raw(Box::new(window_buffer_editdata {
        wp_id: (*(*data).wp).id,
        name: name_owner.as_ptr() as *mut ::core::ffi::c_char,
        pb,
        editor: ::core::ptr::null_mut(),
        name_owner,
    }));
    (*ed).editor = spawn_editor(
        c,
        buf,
        len,
        Some(
            window_buffer_edit_close_cb
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_char,
                    size_t,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        ed as *mut ::core::ffi::c_void,
    );
    if (*ed).editor.is_null() {
        window_buffer_finish_edit(ed);
    } else {
        (*data).editor = (*ed).editor;
        (*data).edit = ed as *mut window_buffer_editdata;
    };
}
unsafe extern "C" fn window_buffer_key(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_buffer_modedata = (*wme).data as *mut window_buffer_modedata;
    let mut mtd: *mut mode_tree_data = (*data).data;
    let mut item: *mut window_buffer_itemdata = ::core::ptr::null_mut::<window_buffer_itemdata>();
    let mut finished: ::core::ffi::c_int = 0;
    if paste_is_empty() != 0 {
        finished = 1 as ::core::ffi::c_int;
    } else if !(*data).editor.is_null() {
        if key == 'q' as i32 as key_code
            || key == '\u{1b}' as i32 as key_code
            || key == '\u{3}' as i32 as key_code
        {
            finished = 1 as ::core::ffi::c_int;
        } else {
            finished = 0 as ::core::ffi::c_int;
        }
    } else {
        finished = mode_tree_key(
            mtd,
            c,
            &raw mut key,
            m,
            ::core::ptr::null_mut::<u_int>(),
            ::core::ptr::null_mut::<u_int>(),
        );
        match key {
            101 => {
                item = mode_tree_get_current(mtd) as *mut window_buffer_itemdata;
                window_buffer_start_edit(data, item, c);
            }
            100 => {
                item = mode_tree_get_current(mtd) as *mut window_buffer_itemdata;
                window_buffer_do_delete(
                    data as *mut ::core::ffi::c_void,
                    item as *mut ::core::ffi::c_void,
                    c,
                    key,
                );
                mode_tree_build(mtd);
            }
            68 => {
                mode_tree_each_tagged(
                    mtd,
                    Some(
                        window_buffer_do_delete
                            as unsafe extern "C" fn(
                                *mut ::core::ffi::c_void,
                                *mut ::core::ffi::c_void,
                                *mut client,
                                key_code,
                            ) -> (),
                    ),
                    c,
                    key,
                    0 as ::core::ffi::c_int,
                );
                mode_tree_build(mtd);
            }
            80 => {
                mode_tree_each_tagged(
                    mtd,
                    Some(
                        window_buffer_do_paste
                            as unsafe extern "C" fn(
                                *mut ::core::ffi::c_void,
                                *mut ::core::ffi::c_void,
                                *mut client,
                                key_code,
                            ) -> (),
                    ),
                    c,
                    key,
                    0 as ::core::ffi::c_int,
                );
                finished = 1 as ::core::ffi::c_int;
            }
            112 | 13 => {
                item = mode_tree_get_current(mtd) as *mut window_buffer_itemdata;
                window_buffer_do_paste(
                    data as *mut ::core::ffi::c_void,
                    item as *mut ::core::ffi::c_void,
                    c,
                    key,
                );
                finished = 1 as ::core::ffi::c_int;
            }
            _ => {}
        }
    }
    if finished != 0 || paste_is_empty() != 0 {
        window_pane_reset_mode(wp);
    } else {
        mode_tree_draw(mtd);
        window_buffer_draw_waiting(data);
        (*wp).flags |= PANE_REDRAW;
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_items_own_names_and_keep_callback_addresses_stable() {
        let mut items = Vec::new();
        let source = CString::new(b"\xffbuffer".to_vec()).unwrap();
        let first = window_buffer_add_item(&mut items, &source);
        let first_name = unsafe { ((*first).name).as_ptr().cast_mut() };
        drop(source);

        let empty = CStr::from_bytes_with_nul(b"\0").unwrap();
        let empty_item = window_buffer_add_item(&mut items, empty);
        for _ in 0..512 {
            window_buffer_add_item(&mut items, empty);
        }

        assert_eq!(first, &mut *items[0] as *mut window_buffer_itemdata);
        assert_eq!(
            unsafe { CStr::from_ptr(first_name).to_bytes() },
            b"\xffbuffer"
        );
        assert_eq!(unsafe { ((*first).name).as_ptr().cast_mut() }, first_name);
        assert_eq!(
            unsafe { CStr::from_ptr(((*empty_item).name).as_ptr().cast_mut()).to_bytes() },
            b""
        );

        window_buffer_clear_items(&mut items);
        assert!(items.is_empty());
    }
}
