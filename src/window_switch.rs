use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::cmd::find::{cmd_find_clear_state, cmd_find_from_session, cmd_find_from_winlink};
use crate::src::cmd::parse::{cmd_parse_and_append, cmd_parse_error_uppercase_first};
use crate::src::cmd::queue::{cmdq_free_state, cmdq_new_state};
use crate::src::cmd::{cmd_mouse_at, cmd_template_replace_cstring};
use crate::src::ffi::libc::__ctype_toupper_loc;
use crate::src::format::{format_create, format_defaults, format_expand_cstring, format_free};
use crate::src::format_draw::format_draw;
use crate::src::fuzzy::fuzzy_match_owned;
use crate::src::grid::{grid_default_cell, grid_get_cell};
use crate::src::prompt::{
    prompt_create, prompt_draw, prompt_free, prompt_incremental_start, prompt_key, prompt_mouse,
    prompt_set_options, prompt_update,
};
use crate::src::screen::{screen_free, screen_init, screen_resize};
use crate::src::screen_write::{
    screen_write_cell, screen_write_clearendofline, screen_write_clearscreen,
    screen_write_cursormove, screen_write_start, screen_write_stop,
};
use crate::src::server_fn::{server_redraw_window, server_unzoom_window};
use crate::src::session::session_find_by_id;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::client;
use crate::src::shared::command::cmd_parse_input;
use crate::src::shared::command::{cmd_find_state, cmdq_item, cmdq_state};
use crate::src::shared::display::*;
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::grid::*;
use crate::src::shared::key::key_event;
use crate::src::shared::key::*;
use crate::src::shared::mouse::{mouse_event, MOUSE_BUTTON_1, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG};
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_REDRAW;
use crate::src::shared::prompt::prompt;
use crate::src::shared::prompt::*;
use crate::src::shared::prompt::{prompt_create_data, prompt_draw_data};
use crate::src::shared::prompt::{
    prompt_result, PROMPT_CONTINUE, PROMPT_EDITARROWS, PROMPT_INCREMENTAL, PROMPT_ISMODE,
    PROMPT_NOFORMAT,
};
use crate::src::shared::screen::{screen, MODE_CURSOR};
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::style::*;
use crate::src::shared::window::WINDOW_ZOOMED;
use crate::src::shared::window::{window, window_mode, window_mode_entry, winlink};
use crate::src::sort::{sort_get_sessions, sort_get_winlinks};
use crate::src::status::status_message_set;
use crate::src::style::style_apply;
use crate::src::window::{window_pane_reset_mode, window_zoom, winlink_find_by_index};
use std::ffi::{CStr, CString};

#[repr(C)]
pub struct window_switch_modedata {
    pub wp: *mut window_pane,
    pub screen: screen,
    pub zoomed: ::core::ffi::c_int,
    pub format: CString,
    pub command: CString,
    pub type_0: window_switch_type,
    pub filter: CString,
    pub prompt: *mut prompt,
    pub prompt_cx: u_int,
    // Matches only borrow rows; boxes keep their addresses stable as the list grows.
    item_list: Vec<Box<window_switch_itemdata>>,
    matches: Vec<*mut window_switch_itemdata>,
    pub current: u_int,
    pub offset: u_int,
}
#[repr(C)]
pub struct window_switch_itemdata {
    pub type_0: window_switch_type,
    pub session: ::core::ffi::c_int,
    pub winlink: ::core::ffi::c_int,
    pub text: CString,
    match_mask: Option<Vec<bitstr_t>>,
    pub score: u_int,
    pub order: u_int,
}

pub type window_switch_type = ::core::ffi::c_uint;
pub const WINDOW_SWITCH_TYPE_WINDOW: window_switch_type = 1;
pub const WINDOW_SWITCH_TYPE_SESSION: window_switch_type = 0;
#[inline]
unsafe fn toupper(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_toupper_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}

pub const WINDOW_SWITCH_DEFAULT_COMMAND: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"switch-client -Zt '%%'\0")
};
pub const WINDOW_SWITCH_DEFAULT_FORMAT: [::core::ffi::c_char; 350] = unsafe {
    ::core::mem::transmute::<
        [u8; 350],
        [::core::ffi::c_char; 350],
    >(
        *b"#{?window_format,#{window_name} #[dim]#{session_name}:#{window_index}#{window_flags}#[default] #[dim]#{pane_current_command}#[default] #[dim]#{?#{!=:#{pane_title},#{host_short}},#{pane_title},}#[default],#{session_name} #[dim]#{session_windows} windows#[default] #{?session_attached,attached,#[dim]detached#[default]} #[dim]#{window_name}#[default]}\0",
    )
};
pub static mut window_switch_mode: window_mode = {
    window_mode {
        name: c"switch-mode",
        default_format: WINDOW_SWITCH_DEFAULT_FORMAT.as_ptr(),
        flags: 0,
        init: Some(
            window_switch_init
                as unsafe fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_switch_free as unsafe fn(*mut window_mode_entry) -> ()),
        resize: Some(window_switch_resize as unsafe fn(*mut window_mode_entry, u_int, u_int) -> ()),
        update: None,
        style_changed: None,
        key: Some(
            window_switch_key
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
unsafe fn window_switch_add_item(
    mut data: *mut window_switch_modedata,
) -> *mut window_switch_itemdata {
    let mut item = Box::new(window_switch_itemdata {
        type_0: WINDOW_SWITCH_TYPE_SESSION,
        session: 0,
        winlink: 0,
        text: CString::default(),
        match_mask: None,
        score: 0,
        order: 0,
    });
    let ptr = &raw mut *item;
    (*data).item_list.push(item);
    ptr
}
unsafe fn window_switch_add_session(
    mut data: *mut window_switch_modedata,
    mut s: *mut session,
    mut order: *mut u_int,
) {
    let mut item: *mut window_switch_itemdata = ::core::ptr::null_mut::<window_switch_itemdata>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        s,
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    item = window_switch_add_item(data);
    (*item).type_0 = WINDOW_SWITCH_TYPE_SESSION;
    (*item).session = (*s).id as ::core::ffi::c_int;
    (*item).winlink = -(1 as ::core::ffi::c_int);
    let fresh7 = *order;
    *order = (*order).wrapping_add(1);
    (*item).order = fresh7;
    (*item).text = format_expand_cstring(ft, (*data).format.as_ptr());
    format_free(ft);
}
unsafe fn window_switch_add_window(
    mut data: *mut window_switch_modedata,
    mut wl: *mut winlink,
    mut order: *mut u_int,
) {
    let mut item: *mut window_switch_itemdata = ::core::ptr::null_mut::<window_switch_itemdata>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        (*wl).session,
        wl,
        ::core::ptr::null_mut::<window_pane>(),
    );
    item = window_switch_add_item(data);
    (*item).type_0 = WINDOW_SWITCH_TYPE_WINDOW;
    (*item).session = (*(*wl).session).id as ::core::ffi::c_int;
    (*item).winlink = (*wl).idx;
    let fresh4 = *order;
    *order = (*order).wrapping_add(1);
    (*item).order = fresh4;
    (*item).text = format_expand_cstring(ft, (*data).format.as_ptr());
    format_free(ft);
}
unsafe fn window_switch_build(mut data: *mut window_switch_modedata) {
    let mut item: *mut window_switch_itemdata = ::core::ptr::null_mut::<window_switch_itemdata>();
    let mut m: Vec<*mut window_switch_itemdata> = Vec::new();
    let mut f: *const ::core::ffi::c_char = (*data).filter.as_ptr();
    let mut i: u_int = 0;
    let mut order: u_int = 0 as u_int;
    let mut sx: u_int = (*(*data).screen.grid).sx;
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_ACTIVITY,
        reversed: 0,
        order_seq: &[],
    };
    sort_crit.order = SORT_NAME;
    sort_crit.reversed = 0 as ::core::ffi::c_int;
    (*data).item_list.clear();
    match (*data).type_0 as ::core::ffi::c_uint {
        0 => {
            let sl = sort_get_sessions(&raw mut sort_crit);
            let ns = u_int::try_from(sl.len()).expect("too many sessions for window switch");
            i = 0 as u_int;
            while i < ns {
                window_switch_add_session(data, sl[i as usize], &raw mut order);
                i = i.wrapping_add(1);
            }
        }
        1 => {
            let links = sort_get_winlinks(&raw mut sort_crit);
            for wl in links {
                window_switch_add_window(data, wl, &raw mut order);
            }
        }
        _ => {}
    }
    i = 0 as u_int;
    while (i as usize) < (*data).item_list.len() {
        item = &raw mut *(&mut (*data).item_list)[i as usize];
        if *f as ::core::ffi::c_int == '\0' as i32 {
            m.push(item);
        } else {
            (*item).match_mask = fuzzy_match_owned(
                std::ffi::CStr::from_ptr(f),
                (*item).text.as_c_str(),
                sx,
                Some(&mut (*item).score),
            );
            if (*item).match_mask.is_some() {
                m.push(item);
            }
        }
        i = i.wrapping_add(1);
    }
    m.sort_unstable_by(|a, b| unsafe {
        (**b)
            .score
            .cmp(&(**a).score)
            .then_with(|| (**a).order.cmp(&(**b).order))
    });
    (*data).matches = m;
}
unsafe fn window_switch_visible(mut data: *mut window_switch_modedata) -> u_int {
    let mut sy: u_int = (*(*data).screen.grid).sy;
    if sy <= 1 as u_int {
        return 0 as u_int;
    }
    return sy.wrapping_sub(1 as u_int);
}
unsafe fn window_switch_set_current(mut data: *mut window_switch_modedata, mut current: u_int) {
    let mut visible: u_int = window_switch_visible(data);
    if (*data).matches.is_empty() {
        (*data).current = 0 as u_int;
        (*data).offset = 0 as u_int;
        return;
    }
    if (current as usize) >= (*data).matches.len() {
        current = ((*data).matches.len() - 1) as u_int;
    }
    (*data).current = current;
    if (*data).current < (*data).offset {
        (*data).offset = (*data).current;
    } else if visible != 0 as u_int && (*data).current >= (*data).offset.wrapping_add(visible) {
        (*data).offset = (*data)
            .current
            .wrapping_sub(visible)
            .wrapping_add(1 as u_int);
    }
}
unsafe fn window_switch_draw_screen(mut wme: *mut window_mode_entry) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_switch_modedata = (*wme).data as *mut window_switch_modedata;
    let mut oo: *mut options = (*wp).options;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut sx: u_int = (*(*s).grid).sx;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut sy: u_int = (*(*s).grid).sy;
    let mut visible: u_int = 0;
    let mut idx: u_int = 0;
    let mut item: *mut window_switch_itemdata = ::core::ptr::null_mut::<window_switch_itemdata>();
    let mut mgc: grid_cell = grid_cell {
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
    let mut sgc: grid_cell = grid_cell {
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
    let mut dgc: *const grid_cell = &raw const grid_default_cell;
    let mut pdd: prompt_draw_data = prompt_draw_data {
        ctx: ::core::ptr::null_mut::<screen_write_ctx>(),
        cursor_x: ::core::ptr::null_mut::<u_int>(),
        area_x: 0,
        area_width: 0,
        prompt_line: 0,
    };
    screen_write_start(&raw mut ctx, s);
    screen_write_clearscreen(&raw mut ctx, 8 as u_int);
    if sy <= 1 as u_int {
        screen_write_stop(&raw mut ctx);
        return;
    }
    style_apply(
        &raw mut mgc,
        oo,
        b"switch-mode-match-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    style_apply(
        &raw mut sgc,
        oo,
        b"mode-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    visible = window_switch_visible(data);
    i = 0 as u_int;
    while i < visible {
        idx = (*data).offset.wrapping_add(i);
        if (idx as usize) >= (*data).matches.len() {
            break;
        }
        item = (&(*data).matches)[idx as usize];
        screen_write_cursormove(
            &raw mut ctx,
            0 as ::core::ffi::c_int,
            i as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if idx != (*data).current {
            format_draw(
                &raw mut ctx,
                dgc,
                sx,
                (*item).text.as_ptr(),
                ::core::ptr::null_mut::<style_ranges>(),
                0 as ::core::ffi::c_int,
            );
        } else {
            screen_write_clearendofline(&raw mut ctx, sgc.bg as u_int);
            format_draw(
                &raw mut ctx,
                &raw mut sgc,
                sx,
                (*item).text.as_ptr(),
                ::core::ptr::null_mut::<style_ranges>(),
                0 as ::core::ffi::c_int,
            );
        }
        let match_mask = (*item).match_mask.as_ref().map(|mask| mask.as_ptr());
        if let Some(match_mask) = match_mask {
            j = 0 as u_int;
            while j < sx {
                if !(*match_mask.offset((j >> 3 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << (j & 0x7 as u_int)
                    == 0)
                {
                    grid_get_cell((*s).grid, j, i, &raw mut gc);
                    gc.attr = mgc.attr;
                    gc.fg = mgc.fg;
                    gc.bg = mgc.bg;
                    screen_write_cursormove(
                        &raw mut ctx,
                        j as ::core::ffi::c_int,
                        i as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    screen_write_cell(&raw mut ctx, &raw mut gc);
                }
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    if !(*data).prompt.is_null() {
        pdd.ctx = &raw mut ctx;
        pdd.cursor_x = &raw mut (*data).prompt_cx;
        pdd.area_x = 0 as u_int;
        pdd.area_width = sx;
        pdd.prompt_line = sy.wrapping_sub(1 as u_int);
        (*s).mode |= MODE_CURSOR;
        prompt_draw((*data).prompt, &raw mut pdd);
        screen_write_cursormove(
            &raw mut ctx,
            (*data).prompt_cx as ::core::ffi::c_int,
            sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    screen_write_stop(&raw mut ctx);
}
unsafe fn window_switch_init(
    mut wme: *mut window_mode_entry,
    _item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_switch_modedata = ::core::ptr::null_mut::<window_switch_modedata>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let mut pd = prompt_create_data::default();
    let format = if args.is_null() || args_has(args, 'F' as i32 as u_char) == 0 {
        WINDOW_SWITCH_DEFAULT_FORMAT.as_ptr()
    } else {
        args_get(args, 'F' as i32 as u_char)
    };
    let command = if args.is_null() || args_count(args) == 0 as u_int {
        WINDOW_SWITCH_DEFAULT_COMMAND.as_ptr()
    } else {
        args_string(args, 0 as u_int)
    };
    data = Box::into_raw(Box::new(window_switch_modedata {
        wp: ::core::ptr::null_mut(),
        screen: screen::empty(),
        zoomed: 0,
        format: CStr::from_ptr(format).to_owned(),
        command: CStr::from_ptr(command).to_owned(),
        type_0: WINDOW_SWITCH_TYPE_SESSION,
        filter: CString::default(),
        prompt: ::core::ptr::null_mut(),
        prompt_cx: 0,
        item_list: Vec::new(),
        matches: Vec::new(),
        current: 0,
        offset: 0,
    }));
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).wp = wp;
    if args_has(args, 'w' as i32 as u_char) != 0 {
        (*data).type_0 = WINDOW_SWITCH_TYPE_WINDOW;
    } else {
        (*data).type_0 = WINDOW_SWITCH_TYPE_SESSION;
    }
    prompt_set_options(&raw mut pd, (*fs).s);
    pd.fs = fs;
    pd.prompt = b"(search) \0" as *const u8 as *const ::core::ffi::c_char;
    pd.input = b"\0" as *const u8 as *const ::core::ffi::c_char;
    pd.type_0 = PROMPT_TYPE_SEARCH;
    pd.flags = PROMPT_INCREMENTAL | PROMPT_NOFORMAT | PROMPT_ISMODE | PROMPT_EDITARROWS;
    pd.inputcb = Some(Box::new(move |s, key| unsafe {
        window_switch_prompt_callback(data, s, key)
    }));
    (*data).prompt = prompt_create(&raw mut pd);
    prompt_update(
        (*data).prompt,
        b"(search) \0" as *const u8 as *const ::core::ffi::c_char,
        (*data).filter.as_ptr(),
    );
    s = &raw mut (*data).screen;
    screen_init(s, (*(*wp).base.grid).sx, (*(*wp).base.grid).sy, 0 as u_int);
    if args_has(args, 'Z' as i32 as u_char) == 0 {
        (*data).zoomed = -(1 as ::core::ffi::c_int);
    } else {
        (*data).zoomed = (*(*wp).window).flags & WINDOW_ZOOMED;
        if (*data).zoomed == 0 && window_zoom(wp) == 0 as ::core::ffi::c_int {
            server_redraw_window((*wp).window as *mut window);
        }
    }
    window_switch_build(data);
    prompt_incremental_start((*data).prompt);
    window_switch_draw_screen(wme);
    return s;
}
unsafe fn window_switch_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_switch_modedata = (*wme).data as *mut window_switch_modedata;
    if (*data).zoomed == 0 as ::core::ffi::c_int {
        server_unzoom_window((*(*wme).wp).window as *mut window);
    }
    (*data).item_list.clear();
    (*data).matches.clear();
    prompt_free((*data).prompt);
    screen_free(&raw mut (*data).screen);
    drop(Box::from_raw(data));
}
unsafe fn window_switch_resize(mut wme: *mut window_mode_entry, mut sx: u_int, mut sy: u_int) {
    let mut data: *mut window_switch_modedata = (*wme).data as *mut window_switch_modedata;
    let mut s: *mut screen = &raw mut (*data).screen;
    screen_resize(s, sx, sy, 0 as ::core::ffi::c_int);
    window_switch_build(data);
    window_switch_set_current(data, (*data).current);
    window_switch_draw_screen(wme);
}
unsafe fn window_switch_run_command(
    mut data: *mut window_switch_modedata,
    mut c: *mut client,
) -> ::core::ffi::c_int {
    let mut item: *mut window_switch_itemdata = ::core::ptr::null_mut::<window_switch_itemdata>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut target: Option<CString> = None;
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    if (*data).matches.is_empty() {
        return 0 as ::core::ffi::c_int;
    }
    item = (&(*data).matches)[(*data).current as usize];
    cmd_find_clear_state(&raw mut fs, 0 as ::core::ffi::c_int);
    match (*item).type_0 as ::core::ffi::c_uint {
        0 => {
            s = session_find_by_id((*item).session as u_int);
            if !s.is_null() {
                let mut bytes = Vec::from(b"=".as_slice());
                bytes.extend_from_slice(CStr::from_ptr(((*s).name).as_ptr().cast_mut()).to_bytes());
                bytes.push(b':');
                target = Some(CString::new(bytes).expect("session target contains no NUL"));
                cmd_find_from_session(&raw mut fs, s, 0 as ::core::ffi::c_int);
            }
        }
        1 => {
            s = session_find_by_id((*item).session as u_int);
            if !s.is_null() {
                wl = winlink_find_by_index(&raw mut (*s).windows, (*item).winlink);
                if !s.is_null() && !wl.is_null() {
                    let mut bytes = Vec::from(b"=".as_slice());
                    bytes.extend_from_slice(
                        CStr::from_ptr(((*s).name).as_ptr().cast_mut()).to_bytes(),
                    );
                    bytes.push(b':');
                    bytes.extend_from_slice(((*wl).idx as u32).to_string().as_bytes());
                    bytes.push(b'.');
                    target = Some(CString::new(bytes).expect("window target contains no NUL"));
                    cmd_find_from_winlink(&raw mut fs, wl, 0 as ::core::ffi::c_int);
                }
            }
        }
        _ => {}
    }
    let Some(target) = target else {
        return 0 as ::core::ffi::c_int;
    };
    let command = cmd_template_replace_cstring(
        (*data).command.as_c_str(),
        target.as_c_str(),
        1 as ::core::ffi::c_int,
    );
    if !command.as_bytes().is_empty() {
        state = cmdq_new_state(
            &raw mut fs,
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
        if let Err(mut error) = cmd_parse_and_append(
            command.as_c_str(),
            ::core::ptr::null_mut::<cmd_parse_input>(),
            c,
            state,
        ) {
            if !c.is_null() {
                cmd_parse_error_uppercase_first(&mut error);
                status_message_set(
                    c,
                    -(1 as ::core::ffi::c_int),
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    error
                        .as_ref()
                        .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
                );
            }
        }
        cmdq_free_state(state);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_switch_prompt_callback(
    mut data: *mut window_switch_modedata,
    s: Option<&CStr>,
    mut key: prompt_key_result,
) -> prompt_result {
    if key as ::core::ffi::c_uint != PROMPT_KEY_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return PROMPT_CONTINUE;
    }
    let value = s.map_or(&[][..], CStr::to_bytes);
    let value = if s.is_some() && !value.is_empty() {
        &value[1..]
    } else {
        value
    };
    (*data).filter = CString::new(value).expect("prompt input contains no NUL");
    window_switch_build(data);
    (*data).current = 0 as u_int;
    (*data).offset = 0 as u_int;
    return PROMPT_CONTINUE;
}
unsafe fn window_switch_key(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    _s: *mut session,
    _wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let mut current_block: u64;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_switch_modedata = (*wme).data as *mut window_switch_modedata;
    let mut visible: u_int = 0;
    let mut current: u_int = (*data).current;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut size: u_int = (*data).matches.len() as u_int;
    let mut result: prompt_key_result = PROMPT_KEY_NOT_HANDLED;
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        if m.is_null()
            || cmd_mouse_at(wp, m, &raw mut x, &raw mut y, 0 as ::core::ffi::c_int)
                != 0 as ::core::ffi::c_int
        {
            return;
        }
        if !(*data).prompt.is_null()
            && (*(*data).screen.grid).sy != 0 as u_int
            && y == (*(*data).screen.grid).sy.wrapping_sub(1 as u_int)
            && (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_1 as u_int
            && (*m).b & MOUSE_MASK_DRAG as u_int == 0
            && !((*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int)
        {
            result = prompt_mouse(
                (*data).prompt,
                x,
                0 as u_int,
                (*(*data).screen.grid).sx,
                &raw mut redraw,
            );
            if redraw != 0
                || result as ::core::ffi::c_uint
                    == PROMPT_KEY_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_switch_draw_screen(wme);
                (*wp).flags |= PANE_REDRAW;
            }
            return;
        }
        match key {
            38654705664 => {
                if size != 0 as u_int && current != 0 as u_int {
                    window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                }
            }
            34359738368 => {
                if size != 0 as u_int && current != size.wrapping_sub(1 as u_int) {
                    window_switch_set_current(data, current.wrapping_add(1 as u_int));
                }
            }
            17179869440 | 47244640512 => {
                if y >= window_switch_visible(data) || (*data).offset.wrapping_add(y) >= size {
                    return;
                }
                window_switch_set_current(data, (*data).offset.wrapping_add(y));
                if key == KEYC_DOUBLECLICK1_PANE as ::core::ffi::c_ulong as key_code {
                    if window_switch_run_command(data, c) != 0 {
                        window_pane_reset_mode(wp);
                    }
                    return;
                }
            }
            _ => return,
        }
    } else {
        match key {
            35184372088944 | 35184372088939 => {
                key = KEYC_UP as ::core::ffi::c_ulong as key_code;
            }
            35184372088942 | 35184372088938 => {
                key = KEYC_DOWN as ::core::ffi::c_ulong as key_code;
            }
            _ => {}
        }
        match key {
            13 => {
                if window_switch_run_command(data, c) != 0 {
                    window_pane_reset_mode(wp);
                }
                return;
            }
            27 | 35184372088923 | 35184372088931 | 35184372088935 => {
                window_pane_reset_mode(wp);
                return;
            }
            _ => {}
        }
        if !(*data).prompt.is_null() {
            result = prompt_key((*data).prompt, key, &raw mut redraw);
            if redraw != 0 {
                window_switch_draw_screen(wme);
                (*wp).flags |= PANE_REDRAW;
            }
            if result as ::core::ffi::c_uint
                == PROMPT_KEY_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
                || result as ::core::ffi::c_uint
                    == PROMPT_KEY_NOT_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return;
            }
            current = (*data).current;
            size = (*data).matches.len() as u_int;
        }
        match key {
            8589934619 => {
                current_block = 2855313901578758422;
                match current_block {
                    10411635476396469942 => {
                        if size > 0 as u_int {
                            window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                        }
                    }
                    6377616723418564240 => {
                        if !(size == 0 as u_int) {
                            if current == size.wrapping_sub(1 as u_int) {
                                window_switch_set_current(data, 0 as u_int);
                            } else {
                                window_switch_set_current(data, current.wrapping_add(1 as u_int));
                            }
                        }
                    }
                    10953541916736708013 => {
                        visible = window_switch_visible(data);
                        if current >= visible {
                            window_switch_set_current(data, current.wrapping_sub(visible));
                        } else {
                            window_switch_set_current(data, 0 as u_int);
                        }
                    }
                    14247072322348479886 => {
                        visible = window_switch_visible(data);
                        window_switch_set_current(data, current.wrapping_add(visible));
                    }
                    12328944762703010638 => {
                        window_switch_set_current(data, 0 as u_int);
                    }
                    _ => {
                        if !(size == 0 as u_int) {
                            if current == 0 as u_int {
                                window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                            } else {
                                window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                            }
                        }
                    }
                }
            }
            8589934620 => {
                current_block = 6377616723418564240;
                match current_block {
                    10411635476396469942 => {
                        if size > 0 as u_int {
                            window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                        }
                    }
                    6377616723418564240 => {
                        if !(size == 0 as u_int) {
                            if current == size.wrapping_sub(1 as u_int) {
                                window_switch_set_current(data, 0 as u_int);
                            } else {
                                window_switch_set_current(data, current.wrapping_add(1 as u_int));
                            }
                        }
                    }
                    10953541916736708013 => {
                        visible = window_switch_visible(data);
                        if current >= visible {
                            window_switch_set_current(data, current.wrapping_sub(visible));
                        } else {
                            window_switch_set_current(data, 0 as u_int);
                        }
                    }
                    14247072322348479886 => {
                        visible = window_switch_visible(data);
                        window_switch_set_current(data, current.wrapping_add(visible));
                    }
                    12328944762703010638 => {
                        window_switch_set_current(data, 0 as u_int);
                    }
                    _ => {
                        if !(size == 0 as u_int) {
                            if current == 0 as u_int {
                                window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                            } else {
                                window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                            }
                        }
                    }
                }
            }
            8589934617 => {
                current_block = 10953541916736708013;
                match current_block {
                    10411635476396469942 => {
                        if size > 0 as u_int {
                            window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                        }
                    }
                    6377616723418564240 => {
                        if !(size == 0 as u_int) {
                            if current == size.wrapping_sub(1 as u_int) {
                                window_switch_set_current(data, 0 as u_int);
                            } else {
                                window_switch_set_current(data, current.wrapping_add(1 as u_int));
                            }
                        }
                    }
                    10953541916736708013 => {
                        visible = window_switch_visible(data);
                        if current >= visible {
                            window_switch_set_current(data, current.wrapping_sub(visible));
                        } else {
                            window_switch_set_current(data, 0 as u_int);
                        }
                    }
                    14247072322348479886 => {
                        visible = window_switch_visible(data);
                        window_switch_set_current(data, current.wrapping_add(visible));
                    }
                    12328944762703010638 => {
                        window_switch_set_current(data, 0 as u_int);
                    }
                    _ => {
                        if !(size == 0 as u_int) {
                            if current == 0 as u_int {
                                window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                            } else {
                                window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                            }
                        }
                    }
                }
            }
            8589934616 => {
                current_block = 14247072322348479886;
                match current_block {
                    10411635476396469942 => {
                        if size > 0 as u_int {
                            window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                        }
                    }
                    6377616723418564240 => {
                        if !(size == 0 as u_int) {
                            if current == size.wrapping_sub(1 as u_int) {
                                window_switch_set_current(data, 0 as u_int);
                            } else {
                                window_switch_set_current(data, current.wrapping_add(1 as u_int));
                            }
                        }
                    }
                    10953541916736708013 => {
                        visible = window_switch_visible(data);
                        if current >= visible {
                            window_switch_set_current(data, current.wrapping_sub(visible));
                        } else {
                            window_switch_set_current(data, 0 as u_int);
                        }
                    }
                    14247072322348479886 => {
                        visible = window_switch_visible(data);
                        window_switch_set_current(data, current.wrapping_add(visible));
                    }
                    12328944762703010638 => {
                        window_switch_set_current(data, 0 as u_int);
                    }
                    _ => {
                        if !(size == 0 as u_int) {
                            if current == 0 as u_int {
                                window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                            } else {
                                window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                            }
                        }
                    }
                }
            }
            8589934614 => {
                current_block = 12328944762703010638;
                match current_block {
                    10411635476396469942 => {
                        if size > 0 as u_int {
                            window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                        }
                    }
                    6377616723418564240 => {
                        if !(size == 0 as u_int) {
                            if current == size.wrapping_sub(1 as u_int) {
                                window_switch_set_current(data, 0 as u_int);
                            } else {
                                window_switch_set_current(data, current.wrapping_add(1 as u_int));
                            }
                        }
                    }
                    10953541916736708013 => {
                        visible = window_switch_visible(data);
                        if current >= visible {
                            window_switch_set_current(data, current.wrapping_sub(visible));
                        } else {
                            window_switch_set_current(data, 0 as u_int);
                        }
                    }
                    14247072322348479886 => {
                        visible = window_switch_visible(data);
                        window_switch_set_current(data, current.wrapping_add(visible));
                    }
                    12328944762703010638 => {
                        window_switch_set_current(data, 0 as u_int);
                    }
                    _ => {
                        if !(size == 0 as u_int) {
                            if current == 0 as u_int {
                                window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                            } else {
                                window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                            }
                        }
                    }
                }
            }
            8589934615 => {
                current_block = 10411635476396469942;
                match current_block {
                    10411635476396469942 => {
                        if size > 0 as u_int {
                            window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                        }
                    }
                    6377616723418564240 => {
                        if !(size == 0 as u_int) {
                            if current == size.wrapping_sub(1 as u_int) {
                                window_switch_set_current(data, 0 as u_int);
                            } else {
                                window_switch_set_current(data, current.wrapping_add(1 as u_int));
                            }
                        }
                    }
                    10953541916736708013 => {
                        visible = window_switch_visible(data);
                        if current >= visible {
                            window_switch_set_current(data, current.wrapping_sub(visible));
                        } else {
                            window_switch_set_current(data, 0 as u_int);
                        }
                    }
                    14247072322348479886 => {
                        visible = window_switch_visible(data);
                        window_switch_set_current(data, current.wrapping_add(visible));
                    }
                    12328944762703010638 => {
                        window_switch_set_current(data, 0 as u_int);
                    }
                    _ => {
                        if !(size == 0 as u_int) {
                            if current == 0 as u_int {
                                window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                            } else {
                                window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    window_switch_draw_screen(wme);
    (*wp).flags |= PANE_REDRAW;
}
