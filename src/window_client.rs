use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::ffi::libc::memcpy;
use crate::src::format::{
    format_add, format_create, format_create_defaults, format_defaults, format_expand_cstring,
    format_free, format_single_cstring, format_true,
};
use crate::src::format_draw::format_draw;
use crate::src::grid::grid_default_cell;
use crate::src::key_string::key_string_parse_cstr;
use crate::src::mode_tree::{
    mode_tree_add, mode_tree_build, mode_tree_down, mode_tree_draw, mode_tree_each_tagged,
    mode_tree_free, mode_tree_get_current, mode_tree_key, mode_tree_resize, mode_tree_run_command,
    mode_tree_start, mode_tree_view_name, mode_tree_zoom,
};
use crate::src::screen_write::{
    screen_write_cursormove, screen_write_fast_copy, screen_write_hline, screen_write_preview,
    screen_write_vline,
};
use crate::src::server_client::{
    server_client_detach, server_client_how_many, server_client_suspend, server_client_unref,
};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_UNATTACHEDFLAGS;
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::layout::*;
use crate::src::shared::menu::menu_item;
use crate::src::shared::message::*;
use crate::src::shared::mode_tree::{mode_tree_data, mode_tree_help_info, mode_tree_item};
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_REDRAW;
use crate::src::shared::screen::screen;
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::style::*;
use crate::src::shared::tty::TERM_INVALIDMS;
use crate::src::shared::window::{window, window_mode, window_mode_entry, winlink};
use crate::src::sort::sort_get_clients;
use crate::src::status::{status_at_line, status_line_size};
use crate::src::style::style_apply;
use crate::src::window::{window_pane_reset_mode, window_pane_stack_first};
use std::ffi::{CStr, CString};

#[repr(C)]
pub struct window_client_modedata {
    pub wp: *mut window_pane,
    pub data: *mut mode_tree_data,
    pub format: CString,
    pub key_format: CString,
    pub command: CString,
    pub hide_preview_this_pane: ::core::ffi::c_int,
    pub preview_is_info: ::core::ffi::c_int,
    // Boxes keep the mode-tree itemdata pointers stable when the list grows.
    pub items: Vec<Box<window_client_itemdata>>,
}
pub struct window_client_itemdata {
    pub c: *mut client,
    pub ttyname: CString,
}

impl window_client_itemdata {
    fn new(c: *mut client, ttyname: &CStr) -> Self {
        Self {
            c,
            ttyname: ttyname.to_owned(),
        }
    }
}

impl Drop for window_client_itemdata {
    fn drop(&mut self) {
        // The item retains its client until its borrowed mode-tree row is gone.
        if !self.c.is_null() {
            unsafe { server_client_unref(self.c) };
        }
        // CString is released after unref, matching the former free path.
    }
}

pub const WINDOW_CLIENT_DEFAULT_COMMAND: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"detach-client -t '%%'\0")
};
pub const WINDOW_CLIENT_DEFAULT_FORMAT: [::core::ffi::c_char; 78] = unsafe {
    ::core::mem::transmute::<[u8; 78], [::core::ffi::c_char; 78]>(
        *b"#[fg=themelightgrey]#{t/p:client_activity}: session #[default]#{session_name}\0",
    )
};
pub const WINDOW_CLIENT_DEFAULT_KEY_FORMAT: [::core::ffi::c_char; 83] = unsafe {
    ::core::mem::transmute::<[u8; 83], [::core::ffi::c_char; 83]>(
        *b"#{?#{e|<:#{line},10},#{line},#{e|<:#{line},36},M-#{a:#{e|+:97,#{e|-:#{line},10}}}}\0",
    )
};
static mut window_client_info_lines: [*const ::core::ffi::c_char; 23] = [
    b"#[fg=themelightgrey]Client Name   #[#{E:tree-mode-border-style},acs]x#[default] #{client_name} #[fg=themelightgrey]#[fg=themelightgrey](PID #{client_pid})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Session       #[#{E:tree-mode-border-style},acs]x#[default] #{session_name}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Attach Time   #[#{E:tree-mode-border-style},acs]x#[default] #{t:client_created} #[fg=themelightgrey](#{t/r:client_created})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Activity Time #[#{E:tree-mode-border-style},acs]x#[default] #{t:client_activity} #[fg=themelightgrey](#{t/r:client_activity})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Terminal Type #[#{E:tree-mode-border-style},acs]x#[default] #{?client_termtype,#{client_termtype},Unknown}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]TERM          #[#{E:tree-mode-border-style},acs]x#[default] #{client_termname}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Size          #[#{E:tree-mode-border-style},acs]x#[default] #{client_width}x#{client_height} #[fg=themelightgrey](cell #{client_cell_width}x#{client_cell_height})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Bytes Written #[#{E:tree-mode-border-style},acs]x#[default] #{client_written} #[fg=themelightgrey](#{client_discarded} discarded)#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Features      #[#{E:tree-mode-border-style},acs]x#[default] #{?#{I/f:256},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:256}}#[default] #{?#{I/f:RGB},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:RGB}}#[default] #{?#{I/f:bpaste},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:bpaste}}#[default] #{?#{I/f:ccolour},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:ccolour}}#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"              #[#{E:tree-mode-border-style},acs]x#[default] #{?#{I/f:clipboard},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:clipboard}}#[default] #{?#{I/f:cstyle},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:cstyle}}#[default] #{?#{I/f:extkeys},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:extkeys}}#[default] #{?#{I/f:focus},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:focus}}#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"              #[#{E:tree-mode-border-style},acs]x#[default] #{?#{I/f:hyperlinks},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:hyperlinks}}#[default] #{?#{I/f:ignorefkeys},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:ignorefkeys}}#[default] #{?#{I/f:margins},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:margins}}#[default] #{?#{I/f:mouse},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:mouse}}#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"              #[#{E:tree-mode-border-style},acs]x#[default] #{?#{I/f:osc7},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:osc7}}#[default] #{?#{I/f:overline},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:overline}}#[default] #{?#{I/f:progressbar},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:progressbar}}#[default] #{?#{I/f:rectfill},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:rectfill}}#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"              #[#{E:tree-mode-border-style},acs]x#[default] #{?#{I/f:sixel},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:sixel}}#[default] #{?#{I/f:strikethrough},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:strikethrough}}#[default] #{?#{I/f:sync},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:sync}}#[default] #{?#{I/f:title},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:title}}#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"              #[#{E:tree-mode-border-style},acs]x#[default] #{?#{I/f:usstyle},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:usstyle}}#[default] #{?#{I/f:utf8},#[fg=themegreen],#[fg=themelightgrey]}#{p/15:#{l:utf8}}#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[#{E:tree-mode-border-style},acs]qqqqqqqqqqqqqqn#{R:q,#{window_width}}#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]prefix        #[#{E:tree-mode-border-style},acs]x#[default] #{prefix}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]mouse         #[#{E:tree-mode-border-style},acs]x#[default] #{?mouse,#{?#{I/c:kmous},,#[fg=themered]}on,#[fg=themelightgrey]off} #{?#{I/c:kmous},,#[align=right]unavailable: [kmous] missing}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]set-clipboard #[#{E:tree-mode-border-style},acs]x#[default] #{?#{!=:#{set-clipboard},off},#{?#{I/c:Ms},,#[fg=themered]}#{set-clipboard},#[fg=themelightgrey]off} #{?#{I/c:Ms},,#[align=right]unavailable: [Ms] #{?clipboard_invalid,invalid,missing}}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]get-clipboard #[#{E:tree-mode-border-style},acs]x#[default] #{?#{!=:#{get-clipboard},off},#{?#{I/c:Ms},,#[fg=themered]}#{get-clipboard},#[fg=themelightgrey]off} #{?#{I/c:Ms},,#[align=right]unavailable: [Ms] #{?clipboard_invalid,invalid,missing}}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]focus-events  #[#{E:tree-mode-border-style},acs]x#[default] #{?focus-events,#{?#{I/f:focus},,#[fg=themered]}on,#[fg=themelightgrey]off} #{?#{I/f:focus},,#[align=right]unavailable: [Enfcs] or [Dcfcs] missing}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]extended-keys #[#{E:tree-mode-border-style},acs]x#[default] #{?#{!=:#{extended-keys},off},#{?#{I/f:extkeys},,#[fg=themered]}#{extended-keys},#[fg=themelightgrey]off} #{?#{I/f:extkeys},,#[align=right]unavailable: [Eneks] or [Dseks] missing}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]set-titles    #[#{E:tree-mode-border-style},acs]x#[default] #{?set-titles,on,#[fg=themelightgrey]off}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]escape-time   #[#{E:tree-mode-border-style},acs]x#[default] #{escape-time} ms\0"
        as *const u8 as *const ::core::ffi::c_char,
];
static mut window_client_menu_items: [menu_item; 9] = [
    menu_item {
        name: b"Detach\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'd' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Detach Tagged\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'D' as i32 as key_code,
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
pub static mut window_client_mode: window_mode = unsafe {
    window_mode {
        name: c"client-mode",
        default_format: WINDOW_CLIENT_DEFAULT_FORMAT.as_ptr(),
        flags: 0,
        init: Some(
            window_client_init
                as unsafe fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_client_free as unsafe fn(*mut window_mode_entry) -> ()),
        resize: Some(
            window_client_resize
                as unsafe fn(*mut window_mode_entry, u_int, u_int) -> (),
        ),
        update: Some(window_client_update as unsafe fn(*mut window_mode_entry) -> ()),
        style_changed: None,
        key: Some(
            window_client_key
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
static mut window_client_order_seq: [sort_order; 5] =
    [SORT_NAME, SORT_SIZE, SORT_CREATION, SORT_ACTIVITY, SORT_END];
unsafe fn window_client_add_item(data: *mut window_client_modedata, c: *mut client) {
    let item = Box::new(window_client_itemdata::new(
        c,
        CStr::from_ptr(
            ((*c).ttyname)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ),
    ));
    (*c).references += 1;
    (*data).items.push(item);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_items_keep_stable_mode_tree_pointers_and_snapshot_ttynames() {
        let ttyname = CString::new(b"/dev/pts/7".as_slice()).unwrap();
        let mut items = Vec::<Box<window_client_itemdata>>::new();
        items.push(Box::new(window_client_itemdata::new(
            ::core::ptr::null_mut(),
            ttyname.as_c_str(),
        )));
        let first = &*items[0] as *const window_client_itemdata;
        drop(ttyname);

        for _ in 0..256 {
            items.push(Box::new(window_client_itemdata::new(
                ::core::ptr::null_mut(),
                c"another terminal",
            )));
        }
        assert_eq!(&*items[0] as *const _, first);
        assert_eq!(unsafe { (*first).ttyname.as_bytes() }, b"/dev/pts/7");

        items.clear();
        assert!(items.is_empty());
        items.push(Box::new(window_client_itemdata::new(
            ::core::ptr::null_mut(),
            c"replacement",
        )));
        assert_eq!(items[0].ttyname.as_bytes(), b"replacement");
    }
}
unsafe fn window_client_build(
    mut modedata: *mut ::core::ffi::c_void,
    mut sort_crit: *mut sort_criteria,
    _tag: *mut uint64_t,
    mut filter: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_client_modedata = modedata as *mut window_client_modedata;
    let mut item: *mut window_client_itemdata = ::core::ptr::null_mut::<window_client_itemdata>();
    let mut i: u_int = 0;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    (*data).items.clear();
    let clients_sorted = sort_get_clients(sort_crit);
    i = 0 as u_int;
    while (i as usize) < clients_sorted.len() {
        let c = clients_sorted[i as usize];
        if !((*c).session.is_null() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            window_client_add_item(data, c);
        }
        i = i.wrapping_add(1);
    }
    let mut current_block_21: u64;
    i = 0 as u_int;
    while (i as usize) < (*data).items.len() {
        item = {
            let slot = (*data).items.as_ptr().add(i as usize);
            Box::as_ref(&*slot) as *const window_client_itemdata as *mut window_client_itemdata
        };
        c = (*item).c;
        if !filter.is_null() {
            let cp = format_single_cstring(
                ::core::ptr::null_mut::<cmdq_item>(),
                filter,
                c,
                ::core::ptr::null_mut::<session>(),
                ::core::ptr::null_mut::<winlink>(),
                ::core::ptr::null_mut::<window_pane>(),
            );
            if format_true(cp.as_ptr()) == 0 {
                current_block_21 = 3512920355445576850;
            } else {
                current_block_21 = 12147880666119273379;
            }
        } else {
            current_block_21 = 12147880666119273379;
        }
        match current_block_21 {
            12147880666119273379 => {
                let text = format_single_cstring(
                    ::core::ptr::null_mut::<cmdq_item>(),
                    (*data).format.as_ptr(),
                    c,
                    ::core::ptr::null_mut::<session>(),
                    ::core::ptr::null_mut::<winlink>(),
                    ::core::ptr::null_mut::<window_pane>(),
                );
                mode_tree_add(
                    (*data).data,
                    ::core::ptr::null_mut::<mode_tree_item>(),
                    item as *mut ::core::ffi::c_void,
                    c as uint64_t,
                    ((*c).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    text.as_ptr(),
                    -(1 as ::core::ffi::c_int),
                );
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn window_client_draw_info(
    _modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut item: *mut window_client_itemdata = itemdata as *mut window_client_itemdata;
    let mut c: *mut client = (*item).c;
    let mut s: *mut screen = (*ctx).s;
    let mut w: *mut window = (*(*(*c).session).curw).window;
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
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut i: u_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    if (*(*c).tty.term).flags & TERM_INVALIDMS != 0 {
        format_add(
            ft,
            b"clipboard_invalid\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            ft,
            b"clipboard_invalid\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    screen_write_cursormove(
        ctx,
        cx as ::core::ffi::c_int,
        cy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[*const ::core::ffi::c_char; 23]>() as usize)
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>() as usize)
    {
        if i == sy {
            break;
        }
        let expanded = format_expand_cstring(ft, window_client_info_lines[i as usize]);
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            cy.wrapping_add(i) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        format_draw(
            ctx,
            &raw const grid_default_cell,
            sx,
            expanded.as_ptr(),
            ::core::ptr::null_mut::<style_ranges>(),
            0 as ::core::ffi::c_int,
        );
        i = i.wrapping_add(1);
    }
    if sx > 14 as u_int && i < sy {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
        style_apply(
            &raw mut gc,
            (*w).options,
            b"tree-mode-border-style\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::ptr::null_mut::<format_tree>(),
        );
        screen_write_cursormove(
            ctx,
            cx.wrapping_add(14 as u_int) as ::core::ffi::c_int,
            cy.wrapping_add(i) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_vline(
            ctx,
            sy.wrapping_sub(i),
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            &raw mut gc,
        );
    }
    format_free(ft);
}
unsafe fn window_client_draw(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_client_modedata = modedata as *mut window_client_modedata;
    let mut item: *mut window_client_itemdata = itemdata as *mut window_client_itemdata;
    let mut c: *mut client = (*item).c;
    let mut session: *mut session = (*c).session;
    let mut s: *mut screen = (*ctx).s;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
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
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut lines: u_int = 0;
    let mut at: u_int = 0;
    if session.is_null() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
        return;
    }
    if (*data).preview_is_info != 0 {
        window_client_draw_info(modedata, itemdata, ctx, sx, sy);
        return;
    }
    w = (*(*session).curw).window;
    wp = (*w).active;
    if (*data).hide_preview_this_pane != 0 && wp == (*data).wp {
        if !window_pane_stack_first(w).is_null() {
            wp = window_pane_stack_first(w);
        } else {
            wp = ::core::ptr::null_mut::<window_pane>();
        }
    }
    lines = status_line_size(c);
    if lines >= sy {
        lines = 0 as u_int;
    }
    if status_at_line(c) == 0 as ::core::ffi::c_int {
        at = lines;
    } else {
        at = 0 as u_int;
    }
    screen_write_cursormove(
        ctx,
        cx as ::core::ffi::c_int,
        cy.wrapping_add(at) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if !wp.is_null() {
        screen_write_preview(
            ctx,
            &raw mut (*wp).base,
            sx,
            sy.wrapping_sub(2 as u_int).wrapping_sub(lines),
        );
    }
    if at != 0 as u_int {
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            cy.wrapping_add(2 as u_int) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    } else {
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            cy.wrapping_add(sy)
                .wrapping_sub(1 as u_int)
                .wrapping_sub(lines) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut gc,
        (*w).options,
        b"tree-mode-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    screen_write_hline(
        ctx,
        sx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        BOX_LINES_DEFAULT,
        &raw mut gc,
    );
    if at != 0 as u_int {
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    } else {
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            cy.wrapping_add(sy).wrapping_sub(lines) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    screen_write_fast_copy(
        ctx,
        &raw mut (*c).status.screen,
        0 as u_int,
        0 as u_int,
        sx,
        lines,
    );
}
unsafe fn window_client_menu(
    mut modedata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut data: *mut window_client_modedata = modedata as *mut window_client_modedata;
    let mut wp: *mut window_pane = (*data).wp;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    wme = (*wp).modes.active;
    if wme.is_null() || (*wme).data != modedata {
        return;
    }
    window_client_key(
        wme,
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        key,
        ::core::ptr::null_mut::<mouse_event>(),
    );
}
unsafe fn window_client_get_key(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut line: u_int,
) -> key_code {
    let mut data: *mut window_client_modedata = modedata as *mut window_client_modedata;
    let mut item: *mut window_client_itemdata = itemdata as *mut window_client_itemdata;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut key: key_code = 0;
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        (*item).c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
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
fn window_client_sort(sort_crit: &mut sort_criteria) {
    unsafe {
        sort_crit.order_seq = &raw mut window_client_order_seq as *mut sort_order;
        if sort_crit.order as ::core::ffi::c_uint
        == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            sort_crit.order = *sort_crit.order_seq;
        }
    }
}
static window_client_help_lines: &[&'static CStr] = &[
    c"#[fg=themelightgrey]          i #[#{E:tree-mode-border-style},acs]x#[default] Toggle info view",
    c"#[fg=themelightgrey]      Enter #[#{E:tree-mode-border-style},acs]x#[default] Choose selected %1",
    c"#[fg=themelightgrey]          d #[#{E:tree-mode-border-style},acs]x#[default] Detach selected %1",
    c"#[fg=themelightgrey]          D #[#{E:tree-mode-border-style},acs]x#[default] Detach tagged %1s",
    c"#[fg=themelightgrey]          x #[#{E:tree-mode-border-style},acs]x#[default] Detach selected %1",
    c"#[fg=themelightgrey]          X #[#{E:tree-mode-border-style},acs]x#[default] Detach tagged %1s",
    c"#[fg=themelightgrey]          z #[#{E:tree-mode-border-style},acs]x#[default] Suspend selected %1",
    c"#[fg=themelightgrey]          Z #[#{E:tree-mode-border-style},acs]x#[default] Suspend tagged %1s",
    c"#[fg=themelightgrey]          f #[#{E:tree-mode-border-style},acs]x#[default] Enter a filter",
];
fn window_client_help() -> mode_tree_help_info {
    mode_tree_help_info {
        width: 0 as u_int,
        item: c"client",
        lines: window_client_help_lines,
    }
}
unsafe fn window_client_init(
    mut wme: *mut window_mode_entry,
    _item: *mut cmdq_item,
    _fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_client_modedata = ::core::ptr::null_mut::<window_client_modedata>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let format = if args.is_null() || args_has(args, 'F' as i32 as u_char) == 0 {
        WINDOW_CLIENT_DEFAULT_FORMAT.as_ptr()
    } else {
        args_get(args, 'F' as i32 as u_char)
    };
    let key_format = if args.is_null() || args_has(args, 'K' as i32 as u_char) == 0 {
        WINDOW_CLIENT_DEFAULT_KEY_FORMAT.as_ptr()
    } else {
        args_get(args, 'K' as i32 as u_char)
    };
    let command = if args.is_null() || args_count(args) == 0 as u_int {
        WINDOW_CLIENT_DEFAULT_COMMAND.as_ptr()
    } else {
        args_string(args, 0 as u_int)
    };
    data = Box::into_raw(Box::new(window_client_modedata {
        wp: ::core::ptr::null_mut(),
        data: ::core::ptr::null_mut(),
        format: CStr::from_ptr(format).to_owned(),
        key_format: CStr::from_ptr(key_format).to_owned(),
        command: CStr::from_ptr(command).to_owned(),
        hide_preview_this_pane: 0,
        preview_is_info: 0,
        items: Vec::new(),
    }));
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).wp = wp;
    (*data).hide_preview_this_pane =
        (!args.is_null() && args_has(args, 'h' as i32 as u_char) != 0) as ::core::ffi::c_int;
    (*data).preview_is_info =
        (!args.is_null() && args_has(args, 'i' as i32 as u_char) != 0) as ::core::ffi::c_int;
    (*data).data = mode_tree_start(
        wp,
        args,
        Some(
            window_client_build
                as unsafe fn(
                    *mut ::core::ffi::c_void,
                    *mut sort_criteria,
                    *mut uint64_t,
                    *const ::core::ffi::c_char,
                ) -> (),
        ),
        Some(
            window_client_draw
                as unsafe fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    *mut screen_write_ctx,
                    u_int,
                    u_int,
                ) -> (),
        ),
        None,
        Some(
            window_client_menu
                as unsafe fn(*mut ::core::ffi::c_void, *mut client, key_code) -> (),
        ),
        None,
        Some(
            window_client_get_key
                as unsafe fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    u_int,
                ) -> key_code,
        ),
        None,
        Some(window_client_sort),
        Some(window_client_help),
        data as *mut ::core::ffi::c_void,
        &raw const window_client_menu_items as *const menu_item,
        &raw mut s,
    );
    mode_tree_zoom((*data).data, args);
    if (*data).preview_is_info != 0 {
        mode_tree_view_name(
            (*data).data,
            b"info\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        mode_tree_view_name(
            (*data).data,
            b"preview\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    return s;
}
unsafe fn window_client_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_client_modedata = (*wme).data as *mut window_client_modedata;
    if data.is_null() {
        return;
    }
    mode_tree_free((*data).data);
    (*data).items.clear();
    drop(Box::from_raw(data));
}
unsafe fn window_client_resize(
    mut wme: *mut window_mode_entry,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_client_modedata = (*wme).data as *mut window_client_modedata;
    mode_tree_resize((*data).data, sx, sy);
}
unsafe fn window_client_update(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_client_modedata = (*wme).data as *mut window_client_modedata;
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
}
unsafe fn window_client_do_detach(
    mut data: *mut window_client_modedata,
    mut item: *mut window_client_itemdata,
    _c: *mut client,
    mut key: key_code,
) {
    if item == mode_tree_get_current((*data).data) as *mut window_client_itemdata {
        mode_tree_down((*data).data, 0 as ::core::ffi::c_int);
    }
    if key == 'd' as i32 as key_code || key == 'D' as i32 as key_code {
        server_client_detach((*item).c, MSG_DETACH);
    } else if key == 'x' as i32 as key_code || key == 'X' as i32 as key_code {
        server_client_detach((*item).c, MSG_DETACHKILL);
    } else if key == 'z' as i32 as key_code || key == 'Z' as i32 as key_code {
        server_client_suspend((*item).c);
    }
}
unsafe fn window_client_key(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    _s: *mut session,
    _wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_client_modedata = (*wme).data as *mut window_client_modedata;
    let mut mtd: *mut mode_tree_data = (*data).data;
    let mut item: *mut window_client_itemdata = ::core::ptr::null_mut::<window_client_itemdata>();
    let mut finished: ::core::ffi::c_int = 0;
    finished = mode_tree_key(
        mtd,
        c,
        &raw mut key,
        m,
        ::core::ptr::null_mut::<u_int>(),
        ::core::ptr::null_mut::<u_int>(),
    );
    match key {
        100 | 120 | 122 => {
            item = mode_tree_get_current(mtd) as *mut window_client_itemdata;
            window_client_do_detach(
                data,
                item,
                c,
                key,
            );
            mode_tree_build(mtd);
        }
        68 | 88 | 90 => {
            mode_tree_each_tagged(
                mtd,
                |row, c, key| unsafe {
                    window_client_do_detach(data, (*row).itemdata.cast(), c, key)
                },
                c,
                key,
                0 as ::core::ffi::c_int,
            );
            mode_tree_build(mtd);
        }
        105 => {
            (*data).preview_is_info = ((*data).preview_is_info == 0) as ::core::ffi::c_int;
            if (*data).preview_is_info != 0 {
                mode_tree_view_name(mtd, b"info\0" as *const u8 as *const ::core::ffi::c_char);
            } else {
                mode_tree_view_name(mtd, b"preview\0" as *const u8 as *const ::core::ffi::c_char);
            }
            mode_tree_build(mtd);
        }
        13 => {
            item = mode_tree_get_current(mtd) as *mut window_client_itemdata;
            mode_tree_run_command(
                c,
                ::core::ptr::null_mut::<cmd_find_state>(),
                (*data).command.as_ptr(),
                (*item).ttyname.as_ptr(),
            );
            finished = 1 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if finished != 0 || server_client_how_many() == 0 as u_int {
        window_pane_reset_mode(wp);
    } else {
        mode_tree_draw(mtd);
        (*wp).flags |= PANE_REDRAW;
    };
}
