use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::cmd::find::{cmd_find_copy_state, cmd_find_valid_state};
use crate::src::ffi::libc::{memcpy, strcasestr, strlen, strstr};
use crate::src::format::bytes::write_cstr;
use crate::src::format::bytes::xformat;
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
    paste_is_empty, paste_replace_owned,
};
use crate::src::screen_write::{
    screen_write_box, screen_write_clearcharacter, screen_write_cursormove, screen_write_nputs,
    screen_write_start, screen_write_stop,
};
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::client;
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::layout::*;
use crate::src::shared::menu::menu_item;
use crate::src::shared::mode_tree::ModeTreeItemSnapshot;
use crate::src::shared::mode_tree::{
    mode_tree_data, mode_tree_help_info, mode_tree_item, ModeTreeItemData,
};
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_REDRAW;
use crate::src::shared::paste::PasteBufferWeak;
use crate::src::shared::screen::screen;
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::vis::{VIS_CSTYLE, VIS_OCTAL, VIS_TAB};
use crate::src::shared::window::{window_mode, window_mode_entry, winlink};
use crate::src::sort::sort_get_buffers;
use crate::src::spawn::{
    spawn_cancel_editor, spawn_editor, spawn_editor_write, spawn_get_editor_pid,
};
use crate::src::text::utf8::utf8_strvis;
use crate::src::window::{window_pane_find_by_id, window_pane_reset_mode};
use std::ffi::{CStr, CString};
use std::ptr::NonNull;
use std::rc::Rc;

#[repr(C)]
pub struct window_buffer_modedata {
    pub wp: *mut window_pane,
    pub fs: cmd_find_state,
    pub data: *mut mode_tree_data,
    pub editor: *mut spawn_editor_state,
    pub command: CString,
    pub format: CString,
    pub key_format: CString,
    item_list: Vec<refbox::RefBox<window_buffer_itemdata>>,
}
#[repr(C)]
#[derive(Clone)]
pub struct window_buffer_itemdata {
    pub name: std::ffi::CString,
    pub order: u_int,
    pub size: size_t,
}

pub struct window_buffer_editdata {
    pub wp_id: u_int,
    pub name: Option<::std::ffi::CString>,
    pub pb: PasteBufferWeak,
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
static window_buffer_menu_items: [menu_item<'static>; 11] = [
    menu_item {
        name: c"Paste",
        key: 'p' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"Paste Tagged",
        key: 'P' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"",
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: c"Tag",
        key: 't' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"Tag All",
        key: '\u{14}' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"Tag None",
        key: 'T' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"",
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: c"Delete",
        key: 'd' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"Delete Tagged",
        key: 'D' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"",
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: c"Cancel",
        key: 'q' as i32 as key_code,
        command: None,
    },
];
pub static mut window_buffer_mode: window_mode = {
    window_mode {
        name: c"buffer-mode",
        default_format: WINDOW_BUFFER_DEFAULT_FORMAT.as_ptr(),
        flags: 0,
        init: Some(
            window_buffer_init
                as unsafe fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_buffer_free as unsafe fn(*mut window_mode_entry) -> ()),
        resize: Some(window_buffer_resize as unsafe fn(*mut window_mode_entry, u_int, u_int) -> ()),
        update: Some(window_buffer_update as unsafe fn(*mut window_mode_entry) -> ()),
        style_changed: None,
        key: Some(
            window_buffer_key
                as unsafe fn(
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
static window_buffer_order_seq: [sort_order; 3] = [SORT_CREATION, SORT_NAME, SORT_SIZE];
fn window_buffer_add_item(
    items: &mut Vec<refbox::RefBox<window_buffer_itemdata>>,
    name: &CStr,
    order: u_int,
    size: size_t,
) -> refbox::Weak<window_buffer_itemdata> {
    let item = refbox::RefBox::new(window_buffer_itemdata {
        name: name.to_owned(),
        order,
        size,
    });
    let handle = item.downgrade();
    items.push(item);
    handle
}

fn window_buffer_clear_items(items: &mut Vec<refbox::RefBox<window_buffer_itemdata>>) {
    items.clear();
}
unsafe fn window_buffer_build(
    mut modedata: *mut ::core::ffi::c_void,
    mut sort_crit: *mut sort_criteria,
    mut filter: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_buffer_modedata = modedata as *mut window_buffer_modedata;
    let mut i: u_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    window_buffer_clear_items(&mut (*data).item_list);
    let buffers = sort_get_buffers(&*sort_crit);
    for pb in buffers {
        let buffer = pb.borrow();
        let name = paste_buffer_name(&buffer);
        window_buffer_add_item(
            &mut (*data).item_list,
            name,
            paste_buffer_order(&buffer),
            buffer.size,
        );
    }
    if cmd_find_valid_state(&(*data).fs) != 0 {
        s = (*data).fs.s;
        wl = (*data).fs.wl;
        wp = (*data).fs.wp;
    }
    let mut current_block_32: u64;
    i = 0 as u_int;
    while (i as usize) < (*data).item_list.len() {
        let item_handle = (&(*data).item_list)[i as usize].downgrade();
        let item = ModeTreeItemData::Buffer(item_handle.clone()).as_buffer().unwrap();
        if let Some(pb) = paste_get_name(&item.name) {
            ft = format_create(
                ::core::ptr::null_mut::<client>(),
                ::core::ptr::null_mut::<cmdq_item>(),
                FORMAT_NONE,
                0 as ::core::ffi::c_int,
            );
            format_defaults(ft, ::core::ptr::null_mut::<client>(), s, wl, wp);
            format_defaults_paste_buffer(&mut *ft, &pb);
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
                        None,
                        ModeTreeItemData::Buffer(item_handle.clone()),
                        item.order as uint64_t,
                        &item.name,
                        Some(&text),
                        -(1 as ::core::ffi::c_int),
                    );
                    format_free(ft);
                }
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe fn window_buffer_draw(
    item: &window_buffer_itemdata,
    ctx: &mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let cx = (*(*ctx).s).cx;
    let cy = (*(*ctx).s).cy;
    let Some(pb) = paste_get_name(&item.name) else {
        return;
    };
    let buffer = pb.borrow();
    let data = paste_buffer_data(&buffer).unwrap_or_default();
    let mut buf = Vec::<u8>::new();
    for (row, line) in data
        .split(|&byte| byte == b'\n')
        .take(sy as usize)
        .enumerate()
    {
        buf.resize(4 * (line.len() + 1), 0);
        utf8_strvis(&mut buf, line, VIS_OCTAL | VIS_CSTYLE | VIS_TAB);
        let escaped = CStr::from_bytes_until_nul(&buf).expect("escaped line is terminated");
        if !escaped.is_empty() {
            screen_write_cursormove(
                &mut *ctx,
                cx as i32,
                cy.wrapping_add(row as u_int) as i32,
                0,
            );
            screen_write_nputs(&mut *ctx, sx as ssize_t, &grid_default_cell, |out| {
                out.write_all(escaped.to_bytes())
            });
        }
    }
}
fn window_buffer_find(data: &[u8], find: &[u8], icase: bool) -> bool {
    if find.is_empty() {
        return false;
    }
    data.windows(find.len()).any(|candidate| {
        if icase {
            candidate.iter().zip(find).all(|(&a, &b)| unsafe {
                // C tolower follows the active locale; both inputs are
                // unsigned bytes, including when the buffer is binary.
                libc::tolower(i32::from(a)) == libc::tolower(i32::from(b))
            })
        } else {
            candidate == find
        }
    })
}
unsafe fn window_buffer_search(item: &window_buffer_itemdata, search: &CStr, icase: bool) -> bool {
    let Some(buffer) = paste_get_name(&item.name) else {
        return false;
    };
    let name_match = if icase {
        strcasestr(item.name.as_ptr(), search.as_ptr())
    } else {
        strstr(item.name.as_ptr(), search.as_ptr())
    };
    if !name_match.is_null() {
        return true;
    }
    let buffer = buffer.borrow();
    window_buffer_find(
        paste_buffer_data(&buffer).unwrap_or_default(),
        search.to_bytes(),
        icase,
    )
}
unsafe fn window_buffer_menu(
    mut modedata: *mut ::core::ffi::c_void,
    c: &std::rc::Rc<std::cell::UnsafeCell<client>>,
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
        crate::src::shared::rc::as_ptr(c),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        key,
        ::core::ptr::null_mut::<mouse_event>(),
    );
}
unsafe fn window_buffer_get_key(
    mut modedata: *mut ::core::ffi::c_void,
    item: &window_buffer_itemdata,
    mut line: u_int,
) -> key_code {
    let mut data: *mut window_buffer_modedata = modedata as *mut window_buffer_modedata;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut key: key_code = 0;
    if cmd_find_valid_state(&(*data).fs) != 0 {
        s = (*data).fs.s;
        wl = (*data).fs.wl;
        wp = (*data).fs.wp;
    }
    let Some(pb) = paste_get_name(&item.name) else {
        return KEYC_NONE;
    };
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
    format_defaults_paste_buffer(&mut *ft, &pb);
    format_add(
        ft,
        b"line\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (line) as u32),
    );
    let expanded = format_expand_cstring(ft, (*data).key_format.as_ptr());
    key = key_string_parse_cstr(expanded.as_c_str()).unwrap_or(KEYC_UNKNOWN);
    format_free(ft);
    return key;
}
fn window_buffer_sort(sort_crit: &mut sort_criteria) {
    unsafe {
        sort_crit.order_seq = &window_buffer_order_seq;
        if sort_crit.order as ::core::ffi::c_uint
            == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            sort_crit.order = sort_crit.order_seq[0];
        }
    }
}
static window_buffer_help_lines: &[&'static CStr] = &[
    c"#[fg=themelightgrey]      Enter #[#{E:tree-mode-border-style},acs]x#[default] Paste selected %1",
    c"#[fg=themelightgrey]          p #[#{E:tree-mode-border-style},acs]x#[default] Paste selected %1",
    c"#[fg=themelightgrey]          P #[#{E:tree-mode-border-style},acs]x#[default] Paste tagged %1s",
    c"#[fg=themelightgrey]          d #[#{E:tree-mode-border-style},acs]x#[default] Delete selected %1",
    c"#[fg=themelightgrey]          D #[#{E:tree-mode-border-style},acs]x#[default] Delete tagged %1s",
    c"#[fg=themelightgrey]          e #[#{E:tree-mode-border-style},acs]x#[default] Open %1 in editor",
    c"#[fg=themelightgrey]          f #[#{E:tree-mode-border-style},acs]x#[default] Enter a filter",
];
fn window_buffer_help() -> mode_tree_help_info {
    mode_tree_help_info {
        width: 0 as u_int,
        item: c"buffer",
        lines: window_buffer_help_lines,
    }
}
unsafe fn window_buffer_init(
    mut wme: *mut window_mode_entry,
    _item: *mut cmdq_item,
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
        fs: Default::default(),
        data: ::core::ptr::null_mut(),
        editor: ::core::ptr::null_mut(),
        command,
        format,
        key_format,
        item_list: Vec::new(),
    }));
    (*wme).data = data as *mut ::core::ffi::c_void;
    let data_handle = std::ptr::NonNull::new(data).expect("live buffer mode data");
    (*data).wp = wp;
    cmd_find_copy_state(&raw mut (*data).fs, fs);
    (*data).data = mode_tree_start(
        wp,
        args,
        Some(Box::new(move |sort, tag, filter| {
            let mut selected = tag.unwrap_or(::core::primitive::u64::MAX as uint64_t);
            window_buffer_build(
                data_handle.as_ptr().cast(),
                sort as *mut sort_criteria,
                filter.map_or(::core::ptr::null(), |value| value.as_ptr()),
            );
            (selected != ::core::primitive::u64::MAX as uint64_t).then_some(selected)
        })),
        Some(Box::new(move |itemdata, ctx, sx, sy| {
            let item = itemdata.as_buffer().expect("buffer row payload");
            window_buffer_draw(&item, ctx, sx, sy)
        })),
        Some(Box::new(move |itemdata, search, icase| {
            let item = itemdata.as_buffer().expect("buffer row payload");
            window_buffer_search(&item, search, icase)
        })),
        Some(Box::new(move |client, key| {
            window_buffer_menu(data_handle.as_ptr().cast(), client, key)
        })),
        None,
        Some(Box::new(move |itemdata, line| {
            let item = itemdata.as_buffer().expect("buffer row payload");
            window_buffer_get_key(data_handle.as_ptr().cast(), &item, line)
        })),
        None,
        Some(window_buffer_sort),
        Some(window_buffer_help),
        &window_buffer_menu_items,
        &raw mut s,
    );
    mode_tree_zoom((*data).data, args);
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    return s;
}
unsafe fn window_buffer_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_buffer_modedata = (*wme).data as *mut window_buffer_modedata;
    if data.is_null() {
        return;
    }
    if !(*data).editor.is_null() {
        spawn_cancel_editor((*data).editor);
    }
    mode_tree_free((*data).data);
    window_buffer_clear_items(&mut (*data).item_list);
    drop(Box::from_raw(data));
}
unsafe fn window_buffer_resize(mut wme: *mut window_mode_entry, mut sx: u_int, mut sy: u_int) {
    let mut data: *mut window_buffer_modedata = (*wme).data as *mut window_buffer_modedata;
    mode_tree_resize((*data).data, sx, sy);
}
unsafe fn window_buffer_update(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_buffer_modedata = (*wme).data as *mut window_buffer_modedata;
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    window_buffer_draw_waiting(data);
    (*(*data).wp).flags |= PANE_REDRAW;
}
unsafe fn window_buffer_do_delete(
    mut data: *mut window_buffer_modedata,
    item: &ModeTreeItemSnapshot<window_buffer_itemdata>,
) {
    if mode_tree_get_current(&*(*data).data)
        .is_buffer(item)
        && mode_tree_down((*data).data, 0 as ::core::ffi::c_int) == 0
    {
        mode_tree_up((*data).data, 0 as ::core::ffi::c_int);
    }
    if let Some(pb) = paste_get_name(&item.name) {
        paste_free(&pb);
    }
}
unsafe fn window_buffer_do_paste(
    mut data: *mut window_buffer_modedata,
    item: &window_buffer_itemdata,
    mut c: *mut client,
) {
    if paste_get_name(&item.name).is_some() {
        mode_tree_run_command(c, None, &(*data).command, &item.name);
    }
}
unsafe fn window_buffer_draw_waiting(mut data: *mut window_buffer_modedata) {
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
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
    sx = (*s).grid().sx;
    sy = (*s).grid().sy;
    if sx == 0 as u_int || sy == 0 as u_int {
        return;
    }
    pid = spawn_get_editor_pid((*data).editor);
    if pid == -(1 as ::core::ffi::c_int) {
        xformat(&mut text, format_args!("WAITING FOR EDITOR"));
    } else {
        xformat(
            &mut text,
            format_args!("WAITING FOR EDITOR (PID {})", pid as ::core::ffi::c_long),
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
    screen_write_start(&mut ctx, s);
    screen_write_cursormove(
        &mut ctx,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_box(
        &mut ctx,
        box_w,
        box_h,
        BOX_LINES_DEFAULT,
        Some(&gc),
        None,
    );
    screen_write_cursormove(
        &mut ctx,
        x.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        y.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_clearcharacter(&mut ctx, box_w.wrapping_sub(2 as u_int), gc.bg as u_int);
    screen_write_cursormove(
        &mut ctx,
        text_x as ::core::ffi::c_int,
        y.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_nputs(
        &mut ctx,
        box_w.wrapping_sub(2 as u_int) as ssize_t,
        &gc,
        |out| write_cstr(out, &raw mut text as *mut ::core::ffi::c_char),
    );
    screen_write_stop(&mut ctx);
}
unsafe fn window_buffer_edit_close_cb(
    editor: NonNull<spawn_editor_state>,
    buf: Option<Vec<u8>>,
    ed: Box<window_buffer_editdata>,
) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut data: *mut window_buffer_modedata = ::core::ptr::null_mut::<window_buffer_modedata>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    wp = window_pane_find_by_id(ed.wp_id);
    if !wp.is_null() {
        wme = (*wp).modes.active;
        if !wme.is_null() && (*wme).mode == &raw const window_buffer_mode {
            data = (*wme).data as *mut window_buffer_modedata;
            if NonNull::new((*data).editor) == Some(editor) {
                (*data).editor = ::core::ptr::null_mut::<spawn_editor_state>();
            }
        }
    }
    let Some(mut buf) = buf else {
        return;
    };
    let mut len = buf.len();
    if len == 0 {
        return;
    }
    let Some(pb) = ed.name.as_deref().and_then(paste_get_name) else {
        return;
    };
    if ed.pb != pb {
        return;
    }
    let strip_newline = {
        let buffer = pb.borrow();
        paste_buffer_data(&buffer)
            .unwrap_or_default()
            .last()
            .is_some_and(|&byte| byte != b'\n')
    };
    if strip_newline && buf[len - 1] == b'\n' {
        len = len.wrapping_sub(1);
    }
    if len != 0 as size_t {
        buf.truncate(len);
        paste_replace_owned(&pb, buf.into_boxed_slice());
    }
    wp = window_pane_find_by_id(ed.wp_id);
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
}
unsafe fn window_buffer_start_edit(
    mut data: *mut window_buffer_modedata,
    item: &window_buffer_itemdata,
    mut c: *mut client,
) {
    if !(*data).editor.is_null() {
        return;
    }
    let Some(pb) = paste_get_name(&item.name) else {
        return;
    };
    let name = paste_buffer_name(&pb.borrow()).to_owned();
    // The completion callback owns the record. Cancellation drops its capture;
    // dispatch supplies editor identity without sharing the startup record.
    let ed = Box::new(window_buffer_editdata {
        wp_id: (*(*data).wp).id,
        name: Some(name),
        pb: pb.clone(),
    });
    let editor = spawn_editor(
        c,
        |stream| spawn_editor_write(stream, paste_buffer_data(&pb.borrow()).unwrap_or_default()),
        Some(Box::new(move |editor, buf| unsafe {
            window_buffer_edit_close_cb(editor, buf, ed)
        })),
    );
    if let Some(editor) = NonNull::new(editor) {
        (*data).editor = editor.as_ptr();
    }
}
unsafe fn window_buffer_key(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    _s: *mut session,
    _wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_buffer_modedata = (*wme).data as *mut window_buffer_modedata;
    let mut mtd: *mut mode_tree_data = (*data).data;
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
                let item_owner = mode_tree_get_current(&*mtd);
                if let Some(item) = item_owner.as_buffer() {
                    window_buffer_start_edit(data, &item, c);
                }
            }
            100 => {
                let item_owner = mode_tree_get_current(&*mtd);
                if let Some(item) = item_owner.as_buffer() {
                    window_buffer_do_delete(data, &item);
                }
                mode_tree_build(mtd);
            }
            68 => {
                mode_tree_each_tagged(
                    mtd,
                    |row, _, _| unsafe {
                        let itemdata = row.borrow().itemdata.clone();
                        window_buffer_do_delete(
                            data,
                            &itemdata.as_buffer().expect("buffer row payload"),
                        )
                    },
                    c,
                    key,
                    0 as ::core::ffi::c_int,
                );
                mode_tree_build(mtd);
            }
            80 => {
                mode_tree_each_tagged(
                    mtd,
                    |row, c, _| unsafe {
                        let itemdata = row.borrow().itemdata.clone();
                        window_buffer_do_paste(
                            data,
                            &itemdata.as_buffer().expect("buffer row payload"),
                            c,
                        )
                    },
                    c,
                    key,
                    0 as ::core::ffi::c_int,
                );
                finished = 1 as ::core::ffi::c_int;
            }
            112 | 13 => {
                let item_owner = mode_tree_get_current(&*mtd);
                if let Some(item) = item_owner.as_buffer() {
                    window_buffer_do_paste(data, &item, c);
                }
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
    fn editor_record_releases_on_completion_cancellation_and_replaced_buffer() {
        unsafe {
            for finish in ["complete", "cancel", "replace"] {
                let name = c"editor-callback-owner";
                assert_eq!(
                    crate::src::paste::paste_set_owned(
                        b"before".to_vec().into_boxed_slice(),
                        Some(name),
                        None,
                    ),
                    0,
                );
                let original = paste_get_name(name).unwrap();
                let edit = Box::new(window_buffer_editdata {
                    wp_id: u_int::MAX,
                    name: Some(name.to_owned()),
                    pb: original.clone(),
                            });
                let buffer_observer = edit.pb.clone();
                drop(original);
                let capture = (edit, Rc::new(()));
                let observer = Rc::downgrade(&capture.1);
                let mut editor = spawn_editor_state {
                    path: c"unused".to_owned(),
                    pid: 0,
                    cb: Some(Box::new(move |editor, bytes| {
                        let (edit, _lifetime) = capture;
                        window_buffer_edit_close_cb(editor, bytes, edit)
                    })),
                };
                if finish == "replace" {
                    assert_eq!(
                        crate::src::paste::paste_set_owned(
                            b"replacement".to_vec().into_boxed_slice(),
                            Some(name),
                            None,
                        ),
                        0,
                    );
                    assert!(!buffer_observer.is_alive());
                }
                assert!(observer.upgrade().is_some());
                if finish == "cancel" {
                    spawn_cancel_editor(&mut editor);
                    spawn_cancel_editor(&mut editor);
                } else {
                    editor.cb.take().unwrap()(NonNull::from(&mut editor), Some(b"after\0\xff\n".to_vec()));
                }
                assert!(observer.upgrade().is_none());
                assert!(editor.cb.is_none());
                let buffer = paste_get_name(name).unwrap();
                let expected = match finish {
                    "complete" => b"after\0\xff".as_slice(),
                    "cancel" => b"before",
                    _ => b"replacement",
                };
                assert_eq!(paste_buffer_data(&buffer.borrow()).unwrap(), expected);
                paste_free(&buffer);
            }
        }
    }

    #[test]
    fn buffer_search_matches_binary_substrings_without_unicode_case_folding() {
        unsafe {
            assert!(!libc::setlocale(libc::LC_CTYPE, c"C.UTF-8".as_ptr()).is_null());
        }
        let bytes = b"\xffA\0ZbA\n";
        for (find, sensitive, insensitive) in [
            (&b""[..], false, false),
            (&b"A\0Z"[..], true, true),
            (&b"a\0z"[..], false, true),
            (&b"\xffa"[..], false, true),
            (&b"A\n"[..], true, true),
            (&b"\n"[..], true, true),
            (&b"\nZ"[..], false, false),
            (&b"\xffA\0ZbA\nextra"[..], false, false),
        ] {
            assert_eq!(
                window_buffer_find(bytes, find, false),
                sensitive,
                "{find:?}"
            );
            assert_eq!(
                window_buffer_find(bytes, find, true),
                insensitive,
                "{find:?}"
            );
        }
        assert!(!window_buffer_find(b"", b"x", true));
        assert!(!window_buffer_find("É".as_bytes(), "é".as_bytes(), true));
    }

    #[test]
    fn buffer_items_own_names_and_keep_callback_addresses_stable() {
        let mut items = Vec::new();
        let source = CString::new(b"\xffbuffer".to_vec()).unwrap();
        let first = window_buffer_add_item(&mut items, &source, 7, 42);
        let first_name = first.try_borrow_mut().unwrap().name.as_ptr();
        drop(source);

        let empty = CStr::from_bytes_with_nul(b"\0").unwrap();
        let empty_item = window_buffer_add_item(&mut items, empty, 0, 0);
        for _ in 0..512 {
            window_buffer_add_item(&mut items, empty, 0, 0);
        }

        assert!(first.is(&items[0]));
        assert_eq!(
            unsafe { CStr::from_ptr(first_name).to_bytes() },
            b"\xffbuffer"
        );
        assert_eq!(first.try_borrow_mut().unwrap().name.as_ptr(), first_name);
        assert_eq!(empty_item.try_borrow_mut().unwrap().name.as_bytes(), b"");

        let selected = ModeTreeItemData::Buffer(first.clone());
        let snapshot = selected.as_buffer().unwrap();
        window_buffer_clear_items(&mut items);
        assert!(items.is_empty());
        assert!(!first.is_alive());
        assert!(selected.as_buffer().is_none());
        assert_eq!(snapshot.name.as_bytes(), b"\xffbuffer");
        assert_eq!((snapshot.order, snapshot.size), (7, 42));
    }
}
