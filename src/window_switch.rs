use crate::src::options::options_owner_ptr;
use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::cmd::find::{cmd_find_clear_state, cmd_find_from_session, cmd_find_from_winlink};
use crate::src::cmd::parse::{cmd_parse_and_append, cmd_parse_error_uppercase_first};
use crate::src::cmd::queue::{cmdq_new_state};
use crate::src::cmd::{cmd_mouse_at, cmd_template_replace_cstring};
use crate::src::ffi::libc::__ctype_toupper_loc;
use crate::src::format::bytes::write_cstr;
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
    pub screen: screen,
    pub zoomed: ::core::ffi::c_int,
    pub format: CString,
    pub command: CString,
    pub type_0: window_switch_type,
    pub filter: CString,
    pub prompt: Option<PromptOwner>,
    pub prompt_cx: u_int,
    // Match indices are rebuilt whenever the owned row list changes.
    item_list: Vec<Box<window_switch_itemdata>>,
    matches: Vec<usize>,
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

pub const WINDOW_SWITCH_DEFAULT_COMMAND: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"switch-client -Zt '%%'\0")
};
pub const WINDOW_SWITCH_DEFAULT_FORMAT: &CStr = c"#{?window_format,#{window_name} #[dim]#{session_name}:#{window_index}#{window_flags}#[default] #[dim]#{pane_current_command}#[default] #[dim]#{?#{!=:#{pane_title},#{host_short}},#{pane_title},}#[default],#{session_name} #[dim]#{session_windows} windows#[default] #{?session_attached,attached,#[dim]detached#[default]} #[dim]#{window_name}#[default]}";
pub static window_switch_mode: window_mode = {
    window_mode {
        name: c"switch-mode",
        default_format: Some(WINDOW_SWITCH_DEFAULT_FORMAT),
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
                    &std::rc::Rc<std::cell::UnsafeCell<client>>,
                    *mut winlink,
                    key_code,
                    *mut mouse_event,
                ) -> (),
        ),
        key_table: None,
        command: None,
        formats: None,
        get_screen: None,
        display_screen: Some(window_switch_get_screen),
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
    let Some(session_owner) = (*wl).session.upgrade() else { return; };
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        session_owner.get(),
        wl,
        ::core::ptr::null_mut::<window_pane>(),
    );
    item = window_switch_add_item(data);
    (*item).type_0 = WINDOW_SWITCH_TYPE_WINDOW;
    (*item).session = (*session_owner.get()).id as ::core::ffi::c_int;
    (*item).winlink = (*wl).idx;
    let fresh4 = *order;
    *order = (*order).wrapping_add(1);
    (*item).order = fresh4;
    (*item).text = format_expand_cstring(ft, (*data).format.as_ptr());
    format_free(ft);
}
fn window_switch_matches(
    items: &mut [Box<window_switch_itemdata>],
    filter: &CStr,
    width: u_int,
) -> Vec<usize> {
    let mut matches = Vec::new();
    for (index, item) in items.iter_mut().enumerate() {
        item.score = 0;
        item.match_mask = if filter.is_empty() {
            None
        } else {
            // Both C strings and the score slot are borrowed for this call only.
            unsafe { fuzzy_match_owned(filter, &item.text, width, Some(&mut item.score)) }
        };
        if filter.is_empty() || item.match_mask.is_some() {
            matches.push(index);
        }
    }
    matches.sort_unstable_by(|&a, &b| {
        items[b]
            .score
            .cmp(&items[a].score)
            .then_with(|| items[a].order.cmp(&items[b].order))
    });
    matches
}

unsafe fn window_switch_build(mut data: *mut window_switch_modedata) {
    let mut i: u_int = 0;
    let mut order: u_int = 0 as u_int;
    let mut sx: u_int = (*data).screen.grid().sx;
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_ACTIVITY,
        reversed: 0,
        order_seq: &[],
    };
    sort_crit.order = SORT_NAME;
    sort_crit.reversed = 0 as ::core::ffi::c_int;
    (*data).matches.clear();
    (*data).item_list.clear();
    match (*data).type_0 as ::core::ffi::c_uint {
        0 => {
            let sl = sort_get_sessions(&sort_crit);
            let ns = u_int::try_from(sl.len()).expect("too many sessions for window switch");
            i = 0 as u_int;
            while i < ns {
                window_switch_add_session(data, sl[i as usize].get(), &raw mut order);
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
    (*data).matches = window_switch_matches(&mut (*data).item_list, &(*data).filter, sx);
}
unsafe fn window_switch_visible(mut data: *mut window_switch_modedata) -> u_int {
    let mut sy: u_int = (*data).screen.grid().sy;
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
unsafe fn window_switch_data(wme: *mut window_mode_entry) -> *mut window_switch_modedata {
    (*wme)
        .boxed_data_ptr::<window_switch_modedata>()
        .expect("switch mode payload")
}

unsafe fn window_switch_draw_screen(mut wme: *mut window_mode_entry) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_switch_modedata = window_switch_data(wme);
    let mut oo: *mut options = options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options);
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut sx: u_int = (*s).grid().sx;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut sy: u_int = (*s).grid().sy;
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
    screen_write_start(&mut ctx, s);
    screen_write_clearscreen(&mut ctx, 8 as u_int);
    if sy <= 1 as u_int {
        screen_write_stop(&mut ctx);
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
        let row = (&(*data).matches)[idx as usize];
        item = &raw mut *(&mut (*data).item_list)[row];
        screen_write_cursormove(
            &mut ctx,
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
            screen_write_clearendofline(&mut ctx, sgc.bg as u_int);
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
                    grid_get_cell((*s).grid(), j, i, &mut gc);
                    gc.attr = mgc.attr;
                    gc.fg = mgc.fg;
                    gc.bg = mgc.bg;
                    screen_write_cursormove(
                        &mut ctx,
                        j as ::core::ffi::c_int,
                        i as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    screen_write_cell(&mut ctx, &gc);
                }
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    if (*data).prompt.is_some() {
        let pdd = prompt_draw_data {
            area_x: 0 as u_int,
            area_width: sx,
            prompt_line: sy.wrapping_sub(1 as u_int),
        };
        (*s).mode |= MODE_CURSOR;
        (*data).prompt_cx = prompt_draw(
            &(*data).prompt.as_ref().expect("active prompt").try_borrow_mut().expect("unborrowed prompt"),
            &mut ctx,
            pdd,
        );
        screen_write_cursormove(
            &mut ctx,
            (*data).prompt_cx as ::core::ffi::c_int,
            sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    screen_write_stop(&mut ctx);
}
unsafe fn window_switch_init(
    mut wme: *mut window_mode_entry,
    _item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_switch_modedata = ::core::ptr::null_mut::<window_switch_modedata>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let mut pd = prompt_create_data::default();
    let format = if args.is_null() || args_has(args, 'F' as i32 as u_char) == 0 {
        WINDOW_SWITCH_DEFAULT_FORMAT.as_ptr()
    } else {
        args_get(&*(args), 'F' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr())
    };
    let command = if args.is_null() || args_count(args) == 0 as u_int {
        WINDOW_SWITCH_DEFAULT_COMMAND.as_ptr()
    } else {
        args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr())
    };
    let owner = Box::new(std::cell::UnsafeCell::new(window_switch_modedata {
        screen: screen::empty(),
        zoomed: 0,
        format: CStr::from_ptr(format).to_owned(),
        command: CStr::from_ptr(command).to_owned(),
        type_0: WINDOW_SWITCH_TYPE_SESSION,
        filter: CString::default(),
        prompt: None,
        prompt_cx: 0,
        item_list: Vec::new(),
        matches: Vec::new(),
        current: 0,
        offset: 0,
    }));
    data = owner.get();
    (*wme).boxed_data = Some(owner);
    if args_has(args, 'w' as i32 as u_char) != 0 {
        (*data).type_0 = WINDOW_SWITCH_TYPE_WINDOW;
    } else {
        (*data).type_0 = WINDOW_SWITCH_TYPE_SESSION;
    }
    prompt_set_options(&mut pd, ((*fs).s_ptr()).as_mut());
    pd.fs = fs.as_ref();
    pd.prompt = c"(search) ";
    pd.input = Some(c"");
    pd.type_0 = PROMPT_TYPE_SEARCH;
    pd.flags = PROMPT_INCREMENTAL | PROMPT_NOFORMAT | PROMPT_ISMODE | PROMPT_EDITARROWS;
    pd.inputcb = Some(Box::new(move |s, key| unsafe {
        window_switch_prompt_callback(data, s, key)
    }));
    let prompt = prompt_create(pd);
    let prompt_observer = prompt.downgrade();
    (*data).prompt = Some(prompt);
    let prompt = prompt_observer;
    prompt_update(
        &mut prompt.try_borrow_mut().expect("live unborrowed prompt"),
        c"(search) ",
        Some(&(*data).filter),
    );
    s = &raw mut (*data).screen;
    screen_init(
        &mut *s,
        (*wp).base.grid().sx,
        (*wp).base.grid().sy,
        0 as u_int,
    );
    if args_has(args, 'Z' as i32 as u_char) == 0 {
        (*data).zoomed = -(1 as ::core::ffi::c_int);
    } else {
        (*data).zoomed = (*(*wp).window).flags & WINDOW_ZOOMED;
        if (*data).zoomed == 0 && window_zoom(&mode_pane_owner) == 0 as ::core::ffi::c_int {
            server_redraw_window(&*((*wp).window));
        }
    }
    window_switch_build(data);
    prompt_incremental_start(&prompt);
    window_switch_draw_screen(wme);
    return s;
}
unsafe fn window_switch_get_screen(wme: *mut window_mode_entry) -> *mut screen {
    let data = (*wme).boxed_data_ptr::<window_switch_modedata>();
    data.map_or(std::ptr::null_mut(), |data| &raw mut (*data).screen)
}

unsafe fn window_switch_free(mut wme: *mut window_mode_entry) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_switch_modedata = window_switch_data(wme);
    if (*data).zoomed == 0 as ::core::ffi::c_int {
        server_unzoom_window((*mode_pane).window as *mut window);
    }
    (*data).matches.clear();
    (*data).item_list.clear();
    if let Some(prompt) = (*data).prompt.take() {
        prompt_free(&prompt.downgrade());
    }
    screen_free(&mut (*data).screen);
    drop((*wme).boxed_data.take());
}
unsafe fn window_switch_resize(mut wme: *mut window_mode_entry, mut sx: u_int, mut sy: u_int) {
    let mut data: *mut window_switch_modedata = window_switch_data(wme);
    let mut s: *mut screen = &raw mut (*data).screen;
    screen_resize(&mut *s, sx, sy, 0 as ::core::ffi::c_int);
    window_switch_build(data);
    window_switch_set_current(data, (*data).current);
    window_switch_draw_screen(wme);
}
unsafe fn window_switch_run_command(
    mut data: *mut window_switch_modedata,
    client_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
) -> ::core::ffi::c_int {
    let mut item: *mut window_switch_itemdata = ::core::ptr::null_mut::<window_switch_itemdata>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut target: Option<CString> = None;
    let state;
    if (*data).matches.is_empty() {
        return 0 as ::core::ffi::c_int;
    }
    let row = (&(*data).matches)[(*data).current as usize];
    item = &raw mut *(&mut (*data).item_list)[row];
    cmd_find_clear_state(&raw mut fs, 0 as ::core::ffi::c_int);
    match (*item).type_0 as ::core::ffi::c_uint {
        0 => {
            s = session_find_by_id((*item).session as u_int).as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
            if !s.is_null() {
                let mut bytes = Vec::from(b"=".as_slice());
                bytes.extend_from_slice((*s).name.as_bytes());
                bytes.push(b':');
                target = Some(CString::new(bytes).expect("session target contains no NUL"));
                cmd_find_from_session(&raw mut fs, s, 0 as ::core::ffi::c_int);
            }
        }
        1 => {
            s = session_find_by_id((*item).session as u_int).as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
            if !s.is_null() {
                wl = winlink_find_by_index(&raw mut (*s).windows, (*item).winlink);
                if !s.is_null() && !wl.is_null() {
                    let mut bytes = Vec::from(b"=".as_slice());
                    bytes.extend_from_slice((*s).name.as_bytes());
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
        if let Err(mut error) = cmd_parse_and_append(command.as_c_str(), client_owner, Some(&state)) {
            if let Some(owner) = client_owner {
                cmd_parse_error_uppercase_first(&mut error);
                status_message_set(
                    owner.get(),
                    -(1 as ::core::ffi::c_int),
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    |out| {
                        write_cstr(
                            out,
                            error
                                .as_ref()
                                .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
                        )
                    },
                );
            }
        }

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
    client_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>,
    _wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let c = client_owner.get();
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut current_block: u64;
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_switch_modedata = window_switch_data(wme);
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
        if (*data).prompt.is_some()
            && (*data).screen.grid().sy != 0 as u_int
            && y == (*data).screen.grid().sy.wrapping_sub(1 as u_int)
            && (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_1 as u_int
            && (*m).b & MOUSE_MASK_DRAG as u_int == 0
            && !((*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int)
        {
            result = prompt_mouse(
                &mut (*data).prompt.as_ref().expect("active prompt").try_borrow_mut().expect("unborrowed prompt"),
                x,
                0 as u_int,
                (*data).screen.grid().sx,
                Some(&mut redraw),
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
                    if window_switch_run_command(data, Some(client_owner)) != 0 {
                        window_pane_reset_mode(&mode_pane_owner);
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
                if window_switch_run_command(data, Some(client_owner)) != 0 {
                    window_pane_reset_mode(&mode_pane_owner);
                }
                return;
            }
            27 | 35184372088923 | 35184372088931 | 35184372088935 => {
                window_pane_reset_mode(&mode_pane_owner);
                return;
            }
            _ => {}
        }
        if let Some(prompt) = (*data).prompt.as_ref().map(|prompt| prompt.downgrade()) {
            result = prompt_key(&prompt, key, &mut redraw);
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

#[cfg(test)]
mod match_tests {
    use super::*;

    fn row(text: &CStr, order: u_int) -> Box<window_switch_itemdata> {
        Box::new(window_switch_itemdata {
            type_0: WINDOW_SWITCH_TYPE_SESSION,
            session: 0,
            winlink: 0,
            text: text.to_owned(),
            match_mask: None,
            score: 0,
            order,
        })
    }

    #[test]
    fn matches_rank_rows_and_refresh_after_list_changes() {
        let mut rows = vec![row(c"same", 9), row(c"other", 1), row(c"same", 2)];
        assert_eq!(window_switch_matches(&mut rows, c"same", 80), [2, 0]);
        assert!(rows[0].match_mask.is_some());
        assert!(rows[1].match_mask.is_none());
        rows.push(row(c"same", 0));
        assert_eq!(window_switch_matches(&mut rows, c"same", 80), [3, 2, 0]);
        assert_eq!(window_switch_matches(&mut rows, c"", 80), [3, 1, 2, 0]);
        assert!(rows.iter().all(|row| row.match_mask.is_none() && row.score == 0));
        rows = vec![row(c"new", 0)];
        assert!(window_switch_matches(&mut rows, c"same", 80).is_empty());
        assert_eq!(window_switch_matches(&mut rows, c"new", 80), [0]);
    }
}
