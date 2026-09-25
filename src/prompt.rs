use crate::src::cmd::cmd_table;
use crate::src::cmd::find::{cmd_find_clear_state, cmd_find_copy_state, cmd_find_valid_state};
use crate::src::ffi::libc::{memcpy, memmove, memset, strchr, strcmp, strlcat, strlen};
use crate::src::format::{
    format_add, format_create_defaults, format_create_from_state, format_expand_time_cstring,
    format_free,
};
use crate::src::format_draw::{format_draw, format_width};
use crate::src::grid::grid_default_cell;
use crate::src::key_string::key_string_format;
use crate::src::log::log_debug;
use crate::src::options::{
    options_array_first, options_array_item_value, options_array_next, options_get_number,
    options_get_only, options_get_string,
};
use crate::src::paste::{paste_buffer_data, paste_get_top};
use crate::src::prompt_history::{prompt_add_history, prompt_down_history, prompt_up_history};
use crate::src::screen::screen_set_cursor_style;
use crate::src::screen_write::{
    screen_write_cell, screen_write_clearcharacter, screen_write_cursormove,
};
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::command::{cmd_entry, cmdq_item};
use crate::src::shared::format::format_tree;
use crate::src::shared::grid::*;
use crate::src::shared::key::MODEKEY_VI;
use crate::src::shared::key::*;
use crate::src::shared::options::{options, options_array_item, options_entry};
use crate::src::shared::pane::window_pane;
use crate::src::shared::paste::paste_buffer;
use crate::src::shared::prompt::prompt;
use crate::src::shared::prompt::*;
use crate::src::shared::prompt::{prompt_create_data, prompt_draw_data};
use crate::src::shared::prompt::{
    prompt_result, PROMPT_ACCEPT, PROMPT_BSPACE_EXIT, PROMPT_CLOSE, PROMPT_COMMANDMODE,
    PROMPT_CONTINUE, PROMPT_EDITARROWS, PROMPT_INCREMENTAL, PROMPT_ISMODE, PROMPT_ISPANE,
    PROMPT_KEY, PROMPT_NOFORMAT, PROMPT_NOFREEZE, PROMPT_NTYPES, PROMPT_NUMERIC, PROMPT_QUOTENEXT,
    PROMPT_SINGLE,
};
use crate::src::shared::screen::screen;
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::style::*;
use crate::src::shared::utf8::*;
use crate::src::shared::window::winlink;
use crate::src::style::{style_apply, style_parse, style_set};
use crate::src::text::utf8::{
    utf8_append, utf8_copy, utf8_cstrwidth, utf8_fromcstr_vec, utf8_open, utf8_set, utf8_strlen,
    utf8_strwidth, utf8_to_data, utf8_tocstr_cstring,
};
use crate::src::tmux::{global_options, global_s_options};
use std::ffi::{CStr, CString};

unsafe fn prompt_buffer_cells(pr: *mut prompt) -> *mut utf8_data {
    (*pr).buffer.as_mut_ptr()
}

unsafe fn prompt_trim_buffer(pr: &mut prompt) {
    let len = utf8_strlen(pr.buffer.as_mut_ptr()) + 1;
    pr.buffer.truncate(len);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct prompt_layout {
    pub area_x: u_int,
    pub area_width: u_int,
    pub content_x: u_int,
    pub content_width: u_int,
    pub label_width: u_int,
    pub input_x: u_int,
    pub cursor_x: u_int,
    pub input_offset: u_int,
    pub input_width: u_int,
}

unsafe extern "C" fn prompt_flags_to_string(
    mut flags: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static mut tmp: [::core::ffi::c_char; 256] = [0; 256];
    *(&raw mut tmp as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if flags & PROMPT_SINGLE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"SINGLE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_NUMERIC != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"NUMERIC,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_INCREMENTAL != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"INCREMENTAL,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_NOFORMAT != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"NOFORMAT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_KEY != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"KEY,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_ACCEPT != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"ACCEPT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_QUOTENEXT != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"QUOTENEXT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_BSPACE_EXIT != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"BSPACE_EXIT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_NOFREEZE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"NOFREEZE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_COMMANDMODE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"COMMANDMODE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_ISPANE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"ISPANE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_ISMODE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"ISMODE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_EDITARROWS != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"EDITARROWS,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if *(&raw mut tmp as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
        tmp[strlen(&raw mut tmp as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
    }
    return &raw mut tmp as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_set_options(mut pd: *mut prompt_create_data, mut s: *mut session) {
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
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
    let mut n: u_int = 0;
    if !s.is_null() {
        oo = (*s).options;
    } else {
        oo = global_s_options;
    }
    style_apply(
        &raw mut (*pd).style,
        oo,
        b"message-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    style_apply(
        &raw mut (*pd).command_style,
        oo,
        b"message-command-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    (*pd).style_str = options_get_string(
        oo,
        b"message-style\0" as *const u8 as *const ::core::ffi::c_char,
    );
    (*pd).command_style_str = options_get_string(
        oo,
        b"message-command-style\0" as *const u8 as *const ::core::ffi::c_char,
    );
    n = options_get_number(
        oo,
        b"prompt-cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    screen_set_cursor_style(n, &raw mut (*pd).cstyle, &raw mut (*pd).cmode);
    n = options_get_number(
        oo,
        b"prompt-command-cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    screen_set_cursor_style(
        n,
        &raw mut (*pd).command_cstyle,
        &raw mut (*pd).command_cmode,
    );
    style_apply(
        &raw mut gc,
        oo,
        b"prompt-cursor-colour\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    (*pd).ccolour = gc.fg;
    style_apply(
        &raw mut gc,
        oo,
        b"prompt-command-cursor-colour\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    (*pd).command_ccolour = gc.fg;
    (*pd).message_format = options_get_string(
        oo,
        b"message-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    (*pd).keys = options_get_number(
        oo,
        b"status-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*pd).word_separators = options_get_string(
        oo,
        b"word-separators\0" as *const u8 as *const ::core::ffi::c_char,
    );
}

#[no_mangle]
pub unsafe extern "C" fn prompt_create(mut pd: *mut prompt_create_data) -> *mut prompt {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut input: *const ::core::ffi::c_char = (*pd).input;
    let mut allocation = Box::new(prompt::default());
    let pr = &mut *allocation as *mut prompt;
    if !(*pd).fs.is_null() {
        ft = format_create_from_state(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            (*pd).fs,
        );
        cmd_find_copy_state(&raw mut (*pr).state, (*pd).fs);
    } else {
        ft = format_create_defaults(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
        cmd_find_clear_state(&raw mut (*pr).state, 0 as ::core::ffi::c_int);
    }
    if input.is_null() {
        input = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    (*pr).string = CStr::from_ptr((*pd).prompt).to_owned();
    let expanded = if (*pd).flags & PROMPT_NOFORMAT != 0 {
        None
    } else {
        Some(format_expand_time_cstring(ft, input))
    };
    if (*pd).flags & PROMPT_INCREMENTAL != 0 {
        let last = expanded.unwrap_or_else(|| CStr::from_ptr(input).to_owned());
        (*pr).last = Some(last);
        (*pr).buffer = utf8_fromcstr_vec(CStr::from_bytes_with_nul_unchecked(b"\0"));
    } else {
        (*pr).last = None;
        let tmp = expanded.as_ref().map_or(input, |value| value.as_ptr());
        (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(tmp));
    }
    (*pr).index = utf8_strlen((*pr).buffer.as_ptr());
    (*pr).inputcb = ::core::mem::take(&mut (*pd).inputcb);
    (*pr).freecb = ::core::mem::take(&mut (*pd).freecb);
    (*pr).flags = (*pd).flags;
    (*pr).type_0 = (*pd).type_0;
    memcpy(
        &raw mut (*pr).style as *mut ::core::ffi::c_void,
        &raw const (*pd).style as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    memcpy(
        &raw mut (*pr).command_style as *mut ::core::ffi::c_void,
        &raw const (*pd).command_style as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    (*pr).style_str = CStr::from_ptr((*pd).style_str).to_owned();
    (*pr).command_style_str = CStr::from_ptr((*pd).command_style_str).to_owned();
    (*pr).cstyle = (*pd).cstyle;
    (*pr).command_cstyle = (*pd).command_cstyle;
    (*pr).ccolour = (*pd).ccolour;
    (*pr).command_ccolour = (*pd).command_ccolour;
    (*pr).cmode = (*pd).cmode;
    (*pr).command_cmode = (*pd).command_cmode;
    (*pr).message_format = CStr::from_ptr((*pd).message_format).to_owned();
    (*pr).keys = (*pd).keys;
    (*pr).word_separators = CStr::from_ptr((*pd).word_separators).to_owned();
    format_free(ft);
    return Box::into_raw(allocation);
}
#[no_mangle]
pub unsafe extern "C" fn prompt_free(mut pr: *mut prompt) {
    if !pr.is_null() {
        if let Some(callback) = (*pr).freecb.take() {
            callback();
        }
        prompt_clear_complete(pr);
        drop(Box::from_raw(pr));
    }
}
fn prompt_last(pr: &prompt) -> &CStr {
    pr.last
        .as_ref()
        .expect("incremental prompt has saved input")
        .as_c_str()
}
unsafe extern "C" fn prompt_fire_callback(
    mut pr: *mut prompt,
    mut s: *const ::core::ffi::c_char,
    mut type_0: prompt_key_result,
    mut redraw: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let alive = (*pr).alive.clone();
    let mut callback = (*pr).inputcb.take().expect("non-null prompt callback");
    let text = (!s.is_null()).then(|| CStr::from_ptr(s));
    let result = callback(text, type_0);
    if ::std::rc::Rc::strong_count(&alive) == 1 {
        return 1 as ::core::ffi::c_int;
    }
    if (*pr).inputcb.is_none() {
        (*pr).inputcb = Some(callback);
    }
    if result as ::core::ffi::c_uint == PROMPT_CLOSE as ::core::ffi::c_int as ::core::ffi::c_uint {
        (*pr).closed = 1 as ::core::ffi::c_int;
        return 1 as ::core::ffi::c_int;
    }
    if !redraw.is_null() {
        *redraw = 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_incremental_start(mut pr: *mut prompt) {
    if (*pr).flags & PROMPT_INCREMENTAL != 0 {
        let input = utf8_tocstr_cstring(prompt_buffer_cells(pr));
        let mut bytes = Vec::with_capacity(input.as_bytes().len() + 1);
        bytes.push(b'=');
        bytes.extend_from_slice(input.as_bytes());
        let callback_input = CString::new(bytes).expect("the first NUL ends the prompt input");
        prompt_fire_callback(
            pr,
            callback_input.as_ptr(),
            PROMPT_KEY_HANDLED,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn prompt_update(
    mut pr: *mut prompt,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
) {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    if cmd_find_valid_state(&raw mut (*pr).state) != 0 {
        ft = format_create_from_state(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            &raw mut (*pr).state,
        );
    } else {
        ft = format_create_defaults(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
    }
    (*pr).string = CStr::from_ptr(msg).to_owned();
    if input.is_null() {
        input = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let expanded = if (*pr).flags & PROMPT_NOFORMAT != 0 {
        None
    } else {
        Some(format_expand_time_cstring(ft, input))
    };
    let tmp = expanded.as_ref().map_or(input, |value| value.as_ptr());
    // Decode first because input may point into the current buffer.
    let replacement = utf8_fromcstr_vec(CStr::from_ptr(tmp));
    (*pr).buffer = replacement;
    (*pr).index = utf8_strlen((*pr).buffer.as_ptr());
    memset(
        &raw mut (*pr).hindex as *mut u_int as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[u_int; 2]>() as size_t,
    );
    (*pr).closed = 0 as ::core::ffi::c_int;
    prompt_clear_complete(pr);
    format_free(ft);
}
#[no_mangle]
pub unsafe extern "C" fn prompt_closed(mut pr: *mut prompt) -> ::core::ffi::c_int {
    return (*pr).closed;
}
unsafe extern "C" fn prompt_redraw_character(
    mut ctx: *mut screen_write_ctx,
    mut offset: u_int,
    mut pwidth: u_int,
    mut width: *mut u_int,
    mut gc: *mut grid_cell,
    mut ud: *const utf8_data,
) -> ::core::ffi::c_int {
    let mut ch: u_char = 0;
    if *width < offset {
        *width = (*width).wrapping_add((*ud).width as u_int);
        return 1 as ::core::ffi::c_int;
    }
    if *width >= offset.wrapping_add(pwidth) {
        return 0 as ::core::ffi::c_int;
    }
    *width = (*width).wrapping_add((*ud).width as u_int);
    if *width > offset.wrapping_add(pwidth) {
        return 0 as ::core::ffi::c_int;
    }
    ch = *(&raw const (*ud).data as *const u_char);
    if (*ud).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && (ch as ::core::ffi::c_int <= 0x1f as ::core::ffi::c_int
            || ch as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int)
    {
        (*gc).data.data[0 as ::core::ffi::c_int as usize] = '^' as i32 as u_char;
        (*gc).data.data[1 as ::core::ffi::c_int as usize] =
            (if ch as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int {
                '?' as i32
            } else {
                ch as ::core::ffi::c_int | 0x40 as ::core::ffi::c_int
            }) as u_char;
        (*gc).data.have = 2 as u_char;
        (*gc).data.size = (*gc).data.have;
        (*gc).data.width = 2 as u_char;
    } else {
        utf8_copy(&raw mut (*gc).data, ud);
    }
    screen_write_cell(ctx, gc);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn prompt_redraw_quote(
    mut pr: *const prompt,
    mut pcursor: u_int,
    mut input_x: u_int,
    mut ctx: *mut screen_write_ctx,
    mut offset: u_int,
    mut pw: u_int,
    mut w: *mut u_int,
    mut gc: *mut grid_cell,
) -> ::core::ffi::c_int {
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    if (*pr).flags & PROMPT_QUOTENEXT != 0
        && pcursor >= offset
        && (*(*ctx).s).cx == input_x.wrapping_add(pcursor).wrapping_sub(offset)
    {
        utf8_set(&raw mut ud, '^' as i32 as u_char);
        return prompt_redraw_character(ctx, offset, pw, w, gc, &raw mut ud);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn prompt_draw_complete(
    mut pr: *mut prompt,
    mut ctx: *mut screen_write_ctx,
    mut ax: u_int,
    mut aw: u_int,
    mut cx: u_int,
    mut py: u_int,
    mut base: *const grid_cell,
) {
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
    let mut avail: u_int = 0;
    let mut width: u_int = 0;
    let display = (*pr)
        .completion
        .display
        .as_ref()
        .map_or(::core::ptr::null(), |s| s.as_ptr());
    if display.is_null() {
        return;
    }
    if (*pr).index != utf8_strlen(prompt_buffer_cells(pr)) {
        return;
    }
    if cx < ax || cx.wrapping_sub(ax) >= aw {
        return;
    }
    avail = aw.wrapping_sub(cx.wrapping_sub(ax));
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        base as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE) as u_short;
    screen_write_cursormove(
        ctx,
        cx as ::core::ffi::c_int,
        py as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    width = 0 as u_int;
    let mut cells = utf8_fromcstr_vec(CStr::from_ptr(display));
    for cell in &mut cells {
        if cell.size == 0 || width.wrapping_add(cell.width as u_int) > avail {
            break;
        }
        utf8_copy(&raw mut gc.data, cell);
        screen_write_cell(ctx, &raw mut gc);
        width = width.wrapping_add(cell.width as u_int);
    }
}
unsafe extern "C" fn prompt_format_tree(mut pr: *mut prompt) -> *mut format_tree {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    if cmd_find_valid_state(&raw mut (*pr).state) != 0 {
        ft = format_create_from_state(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            &raw mut (*pr).state,
        );
    } else {
        ft = format_create_defaults(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
    }
    let tmp = utf8_tocstr_cstring(prompt_buffer_cells(pr));
    format_add(
        ft,
        b"prompt_input\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        tmp.as_ptr(),
    );
    format_add(
        ft,
        b"prompt_flags\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        prompt_flags_to_string((*pr).flags),
    );
    format_add(
        ft,
        b"prompt_type\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        prompt_type_string((*pr).type_0),
    );
    if (*pr).flags & PROMPT_COMMANDMODE != 0 {
        format_add(
            ft,
            b"command_prompt\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            ft,
            b"command_prompt\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return ft;
}
unsafe fn prompt_expand1(mut pr: *mut prompt, mut ft: *mut format_tree) -> CString {
    let prompt = format_expand_time_cstring(ft, (*pr).string.as_ptr());
    format_add(
        ft,
        b"message\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        prompt.as_ptr(),
    );
    format_expand_time_cstring(ft, (*pr).message_format.as_ptr())
}
unsafe extern "C" fn prompt_effective_style(
    mut pr: *mut prompt,
    mut sy: *mut style,
    mut ft: *mut format_tree,
) {
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut gc: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    if (*pr).flags & PROMPT_COMMANDMODE != 0 {
        s = (*pr).command_style_str.as_ptr();
        gc = &raw mut (*pr).command_style;
    } else {
        s = (*pr).style_str.as_ptr();
        gc = &raw mut (*pr).style;
    }
    style_set(sy, gc);
    if !s.is_null() {
        let expanded = format_expand_time_cstring(ft, s);
        if style_parse(sy, &raw const grid_default_cell, expanded.as_ptr())
            != 0 as ::core::ffi::c_int
        {
            style_set(sy, gc);
        }
    }
}
unsafe fn prompt_layout(
    mut pr: *mut prompt,
    mut ax: u_int,
    mut aw: u_int,
    mut pl: *mut prompt_layout,
    mut sy: *mut style,
) -> CString {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut pcursor: u_int = 0;
    let mut pwidth: u_int = 0;
    let mut end: u_int = 0;
    let mut width: u_int = 0;
    let mut offset: u_int = 0;
    let mut avail: u_int = 0;
    memset(
        pl as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<prompt_layout>() as size_t,
    );
    (*pl).area_x = ax;
    (*pl).area_width = aw;
    ft = prompt_format_tree(pr);
    if !sy.is_null() {
        prompt_effective_style(pr, sy, ft);
    }
    let expanded = prompt_expand1(pr, ft);
    format_free(ft);
    if aw == 0 as u_int {
        return expanded;
    }
    (*pl).label_width = format_width(expanded.as_ptr());
    if (*pl).label_width > aw {
        (*pl).label_width = aw;
    }
    pcursor = utf8_strwidth(prompt_buffer_cells(pr), (*pr).index as ssize_t);
    pwidth = utf8_strwidth(
        prompt_buffer_cells(pr),
        -(1 as ::core::ffi::c_int) as ssize_t,
    );
    if (*pr).flags & PROMPT_QUOTENEXT != 0 {
        pwidth = pwidth.wrapping_add(1);
    }
    avail = aw.wrapping_sub((*pl).label_width);
    if avail == 0 as u_int {
        (*pl).input_offset = 0 as u_int;
        (*pl).input_width = 0 as u_int;
        (*pl).cursor_x = (*pl).label_width;
    } else {
        if pcursor >= avail {
            offset = pcursor.wrapping_sub(avail).wrapping_add(1 as u_int);
            width = avail;
        } else {
            offset = 0 as u_int;
            width = pwidth;
        }
        if width > avail {
            width = avail;
        }
        (*pl).input_offset = offset;
        (*pl).input_width = width;
        (*pl).cursor_x = (*pl).label_width.wrapping_add(pcursor).wrapping_sub(offset);
    }
    (*pl).content_width = (*pl).label_width.wrapping_add((*pl).input_width);
    let display = (*pr)
        .completion
        .display
        .as_ref()
        .map_or(::core::ptr::null(), |s| s.as_ptr());
    if !display.is_null()
        && (*pr).index == utf8_strlen(prompt_buffer_cells(pr))
        && (*pl).cursor_x < aw
    {
        avail = aw.wrapping_sub((*pl).cursor_x);
        width = utf8_cstrwidth(display);
        if width > avail {
            width = avail;
        }
        end = (*pl).cursor_x.wrapping_add(width);
        if end > (*pl).content_width {
            (*pl).content_width = end;
        }
    }
    if (*pl).content_width > aw {
        (*pl).content_width = aw;
    }
    if !sy.is_null() {
        match (*sy).align as ::core::ffi::c_uint {
            2 | 4 => {
                (*pl).content_x = ax.wrapping_add(
                    aw.wrapping_sub((*pl).content_width)
                        .wrapping_div(2 as u_int),
                );
            }
            3 => {
                (*pl).content_x = ax.wrapping_add(aw).wrapping_sub((*pl).content_width);
            }
            _ => {
                (*pl).content_x = ax;
            }
        }
    } else {
        (*pl).content_x = ax;
    }
    (*pl).input_x = (*pl).content_x.wrapping_add((*pl).label_width);
    (*pl).cursor_x = (*pl).cursor_x.wrapping_add((*pl).content_x);
    expanded
}
unsafe extern "C" fn prompt_mouse_complete(
    mut pr: *mut prompt,
    mut x: u_int,
    mut cx: u_int,
    mut ax: u_int,
    mut aw: u_int,
    mut redraw: *mut ::core::ffi::c_int,
) -> prompt_key_result {
    let mut avail: u_int = 0;
    let mut clicked: u_int = 0;
    let mut end: u_int = 0;
    let mut i: u_int = 0;
    let mut start: u_int = 0;
    let mut width: u_int = 0;
    let display = (*pr)
        .completion
        .display
        .as_ref()
        .map_or(::core::ptr::null(), |s| s.as_ptr());
    if display.is_null() || (*pr).completion.names.is_empty() {
        return PROMPT_KEY_NOT_HANDLED;
    }
    if (*pr).index != utf8_strlen(prompt_buffer_cells(pr)) {
        return PROMPT_KEY_NOT_HANDLED;
    }
    if cx < ax || cx.wrapping_sub(ax) >= aw || x < cx {
        return PROMPT_KEY_NOT_HANDLED;
    }
    avail = aw.wrapping_sub(cx.wrapping_sub(ax));
    clicked = x.wrapping_sub(cx);
    width = utf8_cstrwidth(display);
    if width > avail {
        width = avail;
    }
    if clicked >= width {
        return PROMPT_KEY_NOT_HANDLED;
    }
    end = 0 as u_int;
    i = 0 as u_int;
    while (i as usize) < (*pr).completion.names.len() {
        start = end.wrapping_add(1 as u_int);
        end = start.wrapping_add(utf8_cstrwidth(
            (&(*pr).completion.names)[i as usize].as_ptr(),
        ));
        if clicked < start || clicked >= end {
            i = i.wrapping_add(1);
        } else {
            let mut replacement = (&(*pr).completion.names)[i as usize].as_bytes().to_vec();
            replacement.push(b' ');
            let replacement = CString::new(replacement).expect("completion name contains no NUL");
            if prompt_replace_complete(pr, replacement.as_ptr()) != 0 {
                prompt_clear_complete(pr);
                if !redraw.is_null() {
                    *redraw = 1 as ::core::ffi::c_int;
                }
            }
            return PROMPT_KEY_HANDLED;
        }
    }
    return PROMPT_KEY_HANDLED;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_draw(mut pr: *mut prompt, mut pd: *mut prompt_draw_data) {
    let mut ctx: *mut screen_write_ctx = (*pd).ctx;
    let mut s: *mut screen = (*ctx).s;
    let mut ax: u_int = (*pd).area_x;
    let mut py: u_int = (*pd).prompt_line;
    let mut cx: *mut u_int = ::core::ptr::null_mut::<u_int>();
    let mut aw: u_int = (*pd).area_width;
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
    let mut pl: prompt_layout = prompt_layout {
        area_x: 0,
        area_width: 0,
        content_x: 0,
        content_width: 0,
        label_width: 0,
        input_x: 0,
        cursor_x: 0,
        input_offset: 0,
        input_width: 0,
    };
    let mut sy: style = style {
        gc: grid_cell {
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
        },
        ignore: 0,
        dim: 0,
        fill: 0,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0,
        range_string: [0; 16],
        width: 0,
        width_percentage: 0,
        pad: 0,
        default_type: STYLE_DEFAULT_BASE,
        link: 0,
    };
    let mut i: u_int = 0;
    let mut width: u_int = 0;
    let mut pcursor: u_int = 0;
    if (*pr).flags & PROMPT_COMMANDMODE != 0 {
        (*s).default_cstyle = (*pr).command_cstyle;
        (*s).default_mode = (*pr).command_cmode;
        (*s).default_ccolour = (*pr).command_ccolour;
    } else {
        (*s).default_cstyle = (*pr).cstyle;
        (*s).default_mode = (*pr).cmode;
        (*s).default_ccolour = (*pr).ccolour;
    }
    let expanded = prompt_layout(pr, ax, aw, &raw mut pl, &raw mut sy);
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw mut sy.gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    cx = (*pd).cursor_x;
    *cx = pl.cursor_x;
    screen_write_cursormove(
        ctx,
        ax as ::core::ffi::c_int,
        py as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if sy.fill != 8 as ::core::ffi::c_int {
        screen_write_clearcharacter(ctx, aw, sy.fill as u_int);
    }
    pcursor = utf8_strwidth(prompt_buffer_cells(pr), (*pr).index as ssize_t);
    if pl.content_width != 0 as u_int {
        screen_write_cursormove(
            ctx,
            pl.content_x as ::core::ffi::c_int,
            py as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if pl.label_width != 0 as u_int {
            format_draw(
                ctx,
                &raw mut gc,
                pl.label_width,
                expanded.as_ptr(),
                ::core::ptr::null_mut::<style_ranges>(),
                0 as ::core::ffi::c_int,
            );
        }
        screen_write_cursormove(
            ctx,
            pl.input_x as ::core::ffi::c_int,
            py as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        width = 0 as u_int;
        i = 0 as u_int;
        while (*prompt_buffer_cells(pr).offset(i as isize)).size as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if prompt_redraw_quote(
                pr,
                pcursor,
                pl.input_x,
                ctx,
                pl.input_offset,
                pl.input_width,
                &raw mut width,
                &raw mut gc,
            ) == 0
            {
                break;
            }
            if prompt_redraw_character(
                ctx,
                pl.input_offset,
                pl.input_width,
                &raw mut width,
                &raw mut gc,
                prompt_buffer_cells(pr).offset(i as isize) as *mut utf8_data,
            ) == 0
            {
                break;
            }
            i = i.wrapping_add(1);
        }
        prompt_redraw_quote(
            pr,
            pcursor,
            pl.input_x,
            ctx,
            pl.input_offset,
            pl.input_width,
            &raw mut width,
            &raw mut gc,
        );
        prompt_draw_complete(
            pr,
            ctx,
            pl.content_x,
            pl.content_width,
            pl.cursor_x,
            py,
            &raw mut gc,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn prompt_mouse(
    mut pr: *mut prompt,
    mut x: u_int,
    mut ax: u_int,
    mut aw: u_int,
    mut redraw: *mut ::core::ffi::c_int,
) -> prompt_key_result {
    let mut ud: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut result: prompt_key_result = PROMPT_KEY_NOT_HANDLED;
    let mut pl: prompt_layout = prompt_layout {
        area_x: 0,
        area_width: 0,
        content_x: 0,
        content_width: 0,
        label_width: 0,
        input_x: 0,
        cursor_x: 0,
        input_offset: 0,
        input_width: 0,
    };
    let mut sy: style = style {
        gc: grid_cell {
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
        },
        ignore: 0,
        dim: 0,
        fill: 0,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0,
        range_string: [0; 16],
        width: 0,
        width_percentage: 0,
        pad: 0,
        default_type: STYLE_DEFAULT_BASE,
        link: 0,
    };
    let mut pwidth: u_int = 0;
    let mut width: u_int = 0;
    let mut target: u_int = 0;
    let mut idx: size_t = 0;
    if x < ax || x >= ax.wrapping_add(aw) {
        return PROMPT_KEY_NOT_HANDLED;
    }
    drop(prompt_layout(pr, ax, aw, &raw mut pl, &raw mut sy));
    if pl.input_width == 0 as u_int {
        return PROMPT_KEY_HANDLED;
    }
    pwidth = utf8_strwidth(
        prompt_buffer_cells(pr),
        -(1 as ::core::ffi::c_int) as ssize_t,
    );
    if (*pr).flags & PROMPT_QUOTENEXT != 0 {
        pwidth = pwidth.wrapping_add(1);
    }
    result = prompt_mouse_complete(pr, x, pl.cursor_x, pl.content_x, pl.content_width, redraw);
    if result as ::core::ffi::c_uint
        != PROMPT_KEY_NOT_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return result;
    }
    if x <= pl.input_x {
        target = pl.input_offset;
    } else {
        target = pl.input_offset.wrapping_add(x).wrapping_sub(pl.input_x);
    }
    if target > pwidth {
        target = pwidth;
    }
    width = 0 as u_int;
    idx = 0 as size_t;
    while (*prompt_buffer_cells(pr).offset(idx as isize)).size as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        ud = prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data;
        if width >= target {
            break;
        }
        width = width.wrapping_add((*ud).width as u_int);
        idx = idx.wrapping_add(1);
    }
    if idx == (*pr).index {
        return PROMPT_KEY_HANDLED;
    }
    (*pr).index = idx;
    prompt_clear_complete(pr);
    if !redraw.is_null() {
        *redraw = 1 as ::core::ffi::c_int;
    }
    return PROMPT_KEY_HANDLED;
}
unsafe extern "C" fn prompt_in_list(
    mut ws: *const ::core::ffi::c_char,
    mut ud: *const utf8_data,
) -> ::core::ffi::c_int {
    if (*ud).size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        || (*ud).width as ::core::ffi::c_int != 1 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    return (strchr(
        ws,
        *(&raw const (*ud).data as *const u_char) as ::core::ffi::c_int,
    ) != NULL as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
}
unsafe extern "C" fn prompt_space(mut ud: *const utf8_data) -> ::core::ffi::c_int {
    if (*ud).size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        || (*ud).width as ::core::ffi::c_int != 1 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    return (*(&raw const (*ud).data as *const u_char) as ::core::ffi::c_int == ' ' as i32)
        as ::core::ffi::c_int;
}
unsafe extern "C" fn prompt_keypad_key(mut key: key_code) -> key_code {
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS != 0 {
        return key;
    }
    match key {
        8589934623 => return '/' as i32 as key_code,
        8589934624 => return '*' as i32 as key_code,
        8589934625 => return '-' as i32 as key_code,
        8589934626 => return '7' as i32 as key_code,
        8589934627 => return '8' as i32 as key_code,
        8589934628 => return '9' as i32 as key_code,
        8589934629 => return '+' as i32 as key_code,
        8589934630 => return '4' as i32 as key_code,
        8589934631 => return '5' as i32 as key_code,
        8589934632 => return '6' as i32 as key_code,
        8589934633 => return '1' as i32 as key_code,
        8589934634 => return '2' as i32 as key_code,
        8589934635 => return '3' as i32 as key_code,
        8589934636 => return '\r' as i32 as key_code,
        8589934637 => return '0' as i32 as key_code,
        8589934638 => return '.' as i32 as key_code,
        _ => {}
    }
    return key;
}
unsafe extern "C" fn prompt_translate_key(
    mut pr: *mut prompt,
    mut key: key_code,
    mut new_key: *mut key_code,
    mut redraw: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if !(*pr).flags & PROMPT_COMMANDMODE != 0 {
        match key {
            35184372088929 | 35184372088931 | 35184372088933 | 35184372088935 | 35184372088936
            | 9 | 35184372088939 | 35184372088942 | 35184372088944 | 35184372088948
            | 35184372088949 | 35184372088950 | 35184372088951 | 35184372088953 | 10 | 13
            | 35192962023453 | 35192962023454 | 8589934599 | 8589934613 | 8589934620
            | 8589934615 | 8589934614 | 8589934621 | 8589934622 | 8589934619 => {
                *new_key = key;
                return 1 as ::core::ffi::c_int;
            }
            27 | 35184372088923 => {
                (*pr).flags |= PROMPT_COMMANDMODE;
                if (*pr).index != 0 as size_t {
                    (*pr).index = (*pr).index.wrapping_sub(1);
                }
                *redraw = 1 as ::core::ffi::c_int;
                return 0 as ::core::ffi::c_int;
            }
            _ => {}
        }
        *new_key = key;
        return 2 as ::core::ffi::c_int;
    }
    match key {
        8589934599 => {
            *new_key = KEYC_LEFT as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        65 | 73 | 67 | 115 | 97 => {
            (*pr).flags &= !PROMPT_COMMANDMODE;
            *redraw = 1 as ::core::ffi::c_int;
        }
        83 => {
            (*pr).flags &= !PROMPT_COMMANDMODE;
            *redraw = 1 as ::core::ffi::c_int;
            *new_key = ('u' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        105 => {
            (*pr).flags &= !PROMPT_COMMANDMODE;
            *redraw = 1 as ::core::ffi::c_int;
            return 0 as ::core::ffi::c_int;
        }
        27 | 35184372088923 => return 0 as ::core::ffi::c_int,
        _ => {}
    }
    match key {
        65 | 36 => {
            *new_key = KEYC_END as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        73 | 48 | 94 => {
            *new_key = KEYC_HOME as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        67 | 68 => {
            *new_key = ('k' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        8589934599 | 88 => {
            *new_key = KEYC_BSPACE as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        98 => {
            *new_key = ('b' as i32 as ::core::ffi::c_ulonglong | KEYC_META) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        66 => {
            *new_key = ('B' as i32 as ::core::ffi::c_ulonglong | KEYC_VI) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        100 => {
            *new_key = ('u' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        101 => {
            *new_key = ('e' as i32 as ::core::ffi::c_ulonglong | KEYC_VI) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        69 => {
            *new_key = ('E' as i32 as ::core::ffi::c_ulonglong | KEYC_VI) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        119 => {
            *new_key = ('w' as i32 as ::core::ffi::c_ulonglong | KEYC_VI) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        87 => {
            *new_key = ('W' as i32 as ::core::ffi::c_ulonglong | KEYC_VI) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        112 => {
            *new_key = ('y' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        113 => {
            *new_key = ('c' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        115 | 8589934613 | 120 => {
            *new_key = KEYC_DC as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        8589934620 | 106 => {
            *new_key = KEYC_DOWN as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        8589934621 | 104 => {
            *new_key = KEYC_LEFT as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        97 | 8589934622 | 108 => {
            *new_key = KEYC_RIGHT as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        8589934619 | 107 => {
            *new_key = KEYC_UP as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        35184372088936 | 35184372088931 | 10 | 13 => return 1 as ::core::ffi::c_int,
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
fn prompt_save_copied(pr: &mut prompt, idx: size_t) {
    let count = pr.index.wrapping_sub(idx);
    let empty = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut copied = vec![empty; count.wrapping_add(1)];
    let buffer = &pr.buffer;
    copied[..count].copy_from_slice(&buffer[idx..idx + count]);
    pr.copied = Some(copied.into_boxed_slice());
}

unsafe extern "C" fn prompt_paste(mut pr: *mut prompt) -> ::core::ffi::c_int {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut bufdata: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: size_t = 0;
    let mut bufsize: size_t = 0;
    let mut i: u_int = 0;
    let mut ud: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut udp: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut scratch: Vec<utf8_data> = Vec::new();
    let mut more: utf8_state = UTF8_MORE;
    if let Some(copied) = (*pr).copied.as_ref() {
        ud = copied.as_ptr() as *mut utf8_data;
        n = utf8_strlen(copied.as_ptr());
    } else {
        pb = paste_get_top(None);
        if pb.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        bufdata = paste_buffer_data(pb, &raw mut bufsize);
        scratch.resize(
            bufsize.wrapping_add(1 as size_t),
            utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            },
        );
        udp = scratch.as_mut_ptr();
        ud = udp;
        i = 0 as u_int;
        while i as size_t != bufsize {
            more = utf8_open(udp, *bufdata.offset(i as isize) as u_char);
            if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                loop {
                    i = i.wrapping_add(1);
                    if !(i as size_t != bufsize
                        && more as ::core::ffi::c_uint
                            == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        break;
                    }
                    more = utf8_append(udp, *bufdata.offset(i as isize) as u_char);
                }
                if more as ::core::ffi::c_uint
                    == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    udp = udp.offset(1);
                    continue;
                } else {
                    i = i.wrapping_sub((*udp).have as u_int);
                }
            }
            if *bufdata.offset(i as isize) as ::core::ffi::c_int <= 31 as ::core::ffi::c_int
                || *bufdata.offset(i as isize) as ::core::ffi::c_int >= 127 as ::core::ffi::c_int
            {
                break;
            }
            utf8_set(udp, *bufdata.offset(i as isize) as u_char);
            udp = udp.offset(1);
            i = i.wrapping_add(1);
        }
        (*udp).size = 0 as u_char;
        n = udp.offset_from(ud) as ::core::ffi::c_long as size_t;
    }
    if n != 0 as size_t {
        let pasted = std::slice::from_raw_parts(ud, n);
        (*pr)
            .buffer
            .splice((*pr).index..(*pr).index, pasted.iter().copied());
        (*pr).index = (*pr).index.wrapping_add(n);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn prompt_replace_complete(
    mut pr: *mut prompt,
    mut s: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut word: [::core::ffi::c_char; 64] = [0; 64];
    let mut allocated: Option<CString> = None;
    let mut size: size_t = 0;
    let mut idx: size_t = 0;
    let mut used: size_t = 0;
    let mut first: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut last: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut ud: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    idx = (*pr).index;
    if idx != 0 as size_t {
        idx = idx.wrapping_sub(1);
    }
    size = utf8_strlen(prompt_buffer_cells(pr));
    first = prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data;
    while first > prompt_buffer_cells(pr) && prompt_space(first) == 0 {
        first = first.offset(-1);
    }
    while (*first).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int && prompt_space(first) != 0
    {
        first = first.offset(1);
    }
    last = prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data;
    while (*last).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int && prompt_space(last) == 0 {
        last = last.offset(1);
    }
    while last > prompt_buffer_cells(pr) && prompt_space(last) != 0 {
        last = last.offset(-1);
    }
    if (*last).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        last = last.offset(1);
    }
    if last < first {
        return 0 as ::core::ffi::c_int;
    }
    let first_index = first.offset_from(prompt_buffer_cells(pr)) as usize;
    let last_index = last.offset_from(prompt_buffer_cells(pr)) as usize;
    if s.is_null() {
        used = 0 as size_t;
        ud = first;
        while ud < last {
            if used.wrapping_add((*ud).size as size_t)
                >= ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as usize
            {
                break;
            }
            memcpy(
                (&raw mut word as *mut ::core::ffi::c_char).offset(used as isize)
                    as *mut ::core::ffi::c_void,
                &raw mut (*ud).data as *mut u_char as *const ::core::ffi::c_void,
                (*ud).size as size_t,
            );
            used = used.wrapping_add((*ud).size as size_t);
            ud = ud.offset(1);
        }
        if ud != last {
            return 0 as ::core::ffi::c_int;
        }
        word[used as usize] = '\0' as i32 as ::core::ffi::c_char;
    }
    if s.is_null() {
        allocated = prompt_complete(
            pr,
            CStr::from_ptr(word.as_ptr()),
            first.offset_from(prompt_buffer_cells(pr)) as ::core::ffi::c_long as u_int,
        );
        let Some(completion) = allocated.as_ref() else {
            return 0 as ::core::ffi::c_int;
        };
        s = completion.as_ptr();
    }
    let replacement_bytes = CStr::from_ptr(s).to_bytes();
    let mut replacement =
        Vec::with_capacity(first_index + replacement_bytes.len() + size + 1 - last_index);
    let buffer = &(*pr).buffer;
    replacement.extend_from_slice(&buffer[..first_index]);
    for &byte in replacement_bytes {
        let mut cell = utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        };
        utf8_set(&raw mut cell, byte);
        replacement.push(cell);
    }
    replacement.extend_from_slice(&buffer[last_index..=size]);
    (*pr).buffer = replacement;
    (*pr).index = first_index + replacement_bytes.len();
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn prompt_forward_word(
    mut pr: *mut prompt,
    mut size: size_t,
    mut vi: ::core::ffi::c_int,
    mut separators: *const ::core::ffi::c_char,
) {
    let mut idx: size_t = (*pr).index;
    let mut word_is_separators: ::core::ffi::c_int = 0;
    if vi == 0 {
        while idx != size
            && prompt_space(prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data) != 0
        {
            idx = idx.wrapping_add(1);
        }
    }
    if idx == size {
        (*pr).index = idx;
        return;
    }
    word_is_separators = (prompt_in_list(
        separators,
        prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
    ) != 0
        && prompt_space(prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data) == 0)
        as ::core::ffi::c_int;
    loop {
        idx = idx.wrapping_add(1);
        if prompt_space(prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data) != 0 {
            if vi != 0 {
                while idx != size
                    && prompt_space(prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data)
                        != 0
                {
                    idx = idx.wrapping_add(1);
                }
            }
            break;
        } else if !(idx != size
            && word_is_separators
                == prompt_in_list(
                    separators,
                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                ))
        {
            break;
        }
    }
    (*pr).index = idx;
}
unsafe extern "C" fn prompt_end_word(
    mut pr: *mut prompt,
    mut size: size_t,
    mut separators: *const ::core::ffi::c_char,
) {
    let mut idx: size_t = (*pr).index;
    let mut word_is_separators: ::core::ffi::c_int = 0;
    if idx == size {
        return;
    }
    loop {
        idx = idx.wrapping_add(1);
        if idx == size {
            (*pr).index = idx;
            return;
        }
        if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data) != 0) {
            break;
        }
    }
    word_is_separators = prompt_in_list(
        separators,
        prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
    );
    loop {
        idx = idx.wrapping_add(1);
        if idx == size {
            break;
        }
        if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data) == 0
            && word_is_separators
                == prompt_in_list(
                    separators,
                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                ))
        {
            break;
        }
    }
    (*pr).index = idx.wrapping_sub(1 as size_t);
}
unsafe extern "C" fn prompt_backward_word(
    mut pr: *mut prompt,
    mut separators: *const ::core::ffi::c_char,
) {
    let mut idx: size_t = (*pr).index;
    let mut word_is_separators: ::core::ffi::c_int = 0;
    while idx != 0 as size_t {
        idx = idx.wrapping_sub(1);
        if prompt_space(prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data) == 0 {
            break;
        }
    }
    word_is_separators = prompt_in_list(
        separators,
        prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
    );
    while idx != 0 as size_t {
        idx = idx.wrapping_sub(1);
        if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data) != 0
            || word_is_separators
                != prompt_in_list(
                    separators,
                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                ))
        {
            continue;
        }
        idx = idx.wrapping_add(1);
        break;
    }
    (*pr).index = idx;
}
unsafe extern "C" fn prompt_done(
    mut pr: *mut prompt,
    mut s: *const ::core::ffi::c_char,
    mut redraw: *mut ::core::ffi::c_int,
) -> prompt_key_result {
    if prompt_fire_callback(pr, s, PROMPT_KEY_CLOSE, redraw) != 0 {
        return PROMPT_KEY_CLOSE;
    }
    return PROMPT_KEY_HANDLED;
}
unsafe fn prompt_done_with_history(
    pr: *mut prompt,
    redraw: *mut ::core::ffi::c_int,
) -> prompt_key_result {
    let s = utf8_tocstr_cstring(prompt_buffer_cells(pr));
    if !s.as_bytes().is_empty() {
        prompt_add_history(s.as_ptr(), (*pr).type_0 as u_int);
    }
    prompt_done(pr, s.as_ptr(), redraw)
}
unsafe extern "C" fn prompt_check_move(
    mut pr: *mut prompt,
    mut key: key_code,
) -> prompt_key_result {
    if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
        return PROMPT_KEY_NOT_HANDLED;
    }
    match key {
        8589934619 | 8589934620 | 8589934617 | 8589934616 => {}
        8589934621 | 8589934622 => {
            if (*pr).flags & PROMPT_EDITARROWS != 0 {
                return PROMPT_KEY_NOT_HANDLED;
            }
        }
        _ => return PROMPT_KEY_NOT_HANDLED,
    }
    let s = utf8_tocstr_cstring(prompt_buffer_cells(pr));
    if prompt_fire_callback(
        pr,
        s.as_ptr(),
        PROMPT_KEY_MOVE,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    ) != 0
    {
        return PROMPT_KEY_CLOSE;
    }
    return PROMPT_KEY_MOVE;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_key(
    mut pr: *mut prompt,
    mut key: key_code,
    mut redraw: *mut ::core::ffi::c_int,
) -> prompt_key_result {
    let mut current_block: u64;
    let mut prefix: ::core::ffi::c_char = '=' as i32 as ::core::ffi::c_char;
    let mut histstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    let mut idx: size_t = 0;
    let mut tmp: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut result: prompt_key_result = PROMPT_KEY_HANDLED;
    let mut word_is_separators: ::core::ffi::c_int = 0;
    (*pr).closed = 0 as ::core::ffi::c_int;
    prompt_clear_complete(pr);
    if (*pr).flags & PROMPT_KEY != 0 {
        let key_string = key_string_format(key, false);
        if prompt_fire_callback(
            pr,
            key_string.as_ptr(),
            PROMPT_KEY_CLOSE,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ) == 0
        {
            (*pr).closed = 1 as ::core::ffi::c_int;
        }
        return PROMPT_KEY_CLOSE;
    }
    size = utf8_strlen(prompt_buffer_cells(pr));
    key &= !KEYC_MASK_FLAGS;
    key = prompt_keypad_key(key);
    if (*pr).flags & PROMPT_NUMERIC != 0 {
        if key >= '0' as i32 as key_code && key <= '9' as i32 as key_code {
            current_block = 1115217863795707468;
        } else {
            let input = utf8_tocstr_cstring(prompt_buffer_cells(pr));
            if prompt_fire_callback(
                pr,
                input.as_ptr(),
                PROMPT_KEY_CLOSE,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ) == 0
            {
                (*pr).closed = 1 as ::core::ffi::c_int;
            }
            return PROMPT_KEY_NOT_HANDLED;
        }
    } else if (*pr).flags & (PROMPT_SINGLE | PROMPT_QUOTENEXT) != 0 {
        if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_BSPACE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        {
            key = 0x7f as key_code;
        } else if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY > 0x7f as ::core::ffi::c_ulonglong
        {
            if !(key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                == (KEYC_TYPE_UNICODE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    > 0x7f as ::core::ffi::c_ulonglong)
            {
                return PROMPT_KEY_HANDLED;
            }
            key &= KEYC_MASK_KEY;
        } else {
            key &= if key as ::core::ffi::c_ulonglong & KEYC_CTRL != 0 {
                0x1f as ::core::ffi::c_ulonglong
            } else {
                KEYC_MASK_KEY
            };
        }
        (*pr).flags &= !PROMPT_QUOTENEXT;
        current_block = 1115217863795707468;
    } else {
        if (*pr).keys == MODEKEY_VI {
            match prompt_translate_key(pr, key, &raw mut key, redraw) {
                1 => {
                    current_block = 11090587058695514569;
                }
                2 => {
                    current_block = 1115217863795707468;
                }
                _ => return PROMPT_KEY_HANDLED,
            }
        } else {
            current_block = 11090587058695514569;
        }
        match current_block {
            1115217863795707468 => {}
            _ => {
                result = prompt_check_move(pr, key);
                if result as ::core::ffi::c_uint
                    != PROMPT_KEY_NOT_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    return result;
                }
                result = PROMPT_KEY_HANDLED;
                match key {
                    8589934621 | 35184372088930 => {
                        current_block = 14309557416411021540;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934622 | 35184372088934 => {
                        current_block = 9249683890344250718;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934614 | 35184372088929 => {
                        current_block = 12295586438617123170;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934615 | 35184372088933 => {
                        current_block = 14414701084776968413;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    9 => {
                        current_block = 460814018713664829;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934599 | 35184372088936 => {
                        current_block = 2263409105760785053;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934613 | 35184372088932 => {
                        current_block = 5070726005990265983;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088949 => {
                        current_block = 1491684083417518595;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088939 => {
                        current_block = 8994603623389184299;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088951 => {
                        current_block = 12879184554692362543;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35192962023454 | 17592186044518 => {
                        current_block = 15759124641699640388;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    9007199254741061 => {
                        current_block = 15097225335540574411;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    9007199254741093 => {
                        current_block = 4818991882628172305;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    9007199254741079 => {
                        current_block = 5183579720934817709;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    9007199254741111 => {
                        current_block = 17166280686405466987;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    9007199254741058 => {
                        current_block = 227872999036956190;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35192962023453 | 17592186044514 => {
                        current_block = 8252555365493261901;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934619 | 35184372088944 => {
                        current_block = 410082779658746517;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934620 | 35184372088942 => {
                        current_block = 10877725664641927171;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088953 => {
                        current_block = 12682153168616704965;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088948 => {
                        current_block = 10468295484844996031;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    13 | 10 => {
                        current_block = 1087518874050103606;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    27 | 35184372088923 | 35184372088931 | 35184372088935 => {
                        current_block = 2284014684288272695;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088946 => {
                        current_block = 11643096306346113746;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088947 => {
                        current_block = 4238185747604537484;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088950 => {
                        current_block = 13376399739742518040;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        prompt_buffer_cells(pr)
                                            .offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    (*pr).buffer = utf8_fromcstr_vec(CStr::from_ptr(histstr));
                                    (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators.as_ptr(),
                                    prompt_buffer_cells(pr).offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut utf8_data)
                                        != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators.as_ptr(),
                                                prompt_buffer_cells(pr).offset(idx as isize)
                                                    as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                prompt_save_copied(&mut *pr, idx);
                                memmove(
                                    prompt_buffer_cells(pr).offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    prompt_buffer_cells(pr).offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    prompt_buffer_cells(pr)
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*prompt_buffer_cells(pr).offset((*pr).index as isize)).size =
                                        0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*prompt_buffer_cells(pr)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .size = 0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        prompt_buffer_cells(pr).offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        prompt_buffer_cells(pr)
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*prompt_buffer_cells(pr).offset((*pr).index as isize))
                                            .size = 0 as u_char;
                                    } else {
                                        memmove(
                                            prompt_buffer_cells(pr)
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            prompt_buffer_cells(pr).offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*prompt_buffer_cells(pr)
                                        .offset(0 as ::core::ffi::c_int as isize))
                                    .size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        (*pr).buffer = utf8_fromcstr_vec(prompt_last(&*pr));
                                        (*pr).index = utf8_strlen(prompt_buffer_cells(pr));
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators.as_ptr(),
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators.as_ptr());
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                return prompt_done_with_history(pr, redraw);
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    _ => {
                        current_block = 1115217863795707468;
                    }
                }
            }
        }
    }
    match current_block {
        1115217863795707468 => {
            if key <= 0x7f as key_code {
                utf8_set(&raw mut tmp, key as u_char);
                if key <= 0x1f as key_code || key == 0x7f as key_code {
                    tmp.width = 2 as u_char;
                }
            } else if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                == (KEYC_TYPE_UNICODE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    > 0x7f as ::core::ffi::c_ulonglong
            {
                utf8_to_data(key as utf8_char, &raw mut tmp);
                if tmp.size as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    return PROMPT_KEY_HANDLED;
                }
            } else {
                return PROMPT_KEY_HANDLED;
            }
            (*pr).buffer.insert((*pr).index, tmp);
            (*pr).index = (*pr).index.wrapping_add(1);
            if (*pr).flags & PROMPT_SINGLE != 0 {
                if utf8_strlen(prompt_buffer_cells(pr)) != 1 as size_t {
                    (*pr).closed = 1 as ::core::ffi::c_int;
                    result = PROMPT_KEY_CLOSE;
                } else {
                    let input = utf8_tocstr_cstring(prompt_buffer_cells(pr));
                    result = prompt_done(pr, input.as_ptr(), redraw);
                }
            }
        }
        _ => {}
    }
    // Key editing changes the C-visible end marker in place. Keep the Vec
    // length at that marker so later inserts and replacements never retain
    // cells that the prompt no longer exposes.
    prompt_trim_buffer(&mut *pr);
    *redraw = 1 as ::core::ffi::c_int;
    if (*pr).flags & PROMPT_INCREMENTAL != 0 {
        let input = utf8_tocstr_cstring(prompt_buffer_cells(pr));
        let mut bytes = Vec::with_capacity(input.as_bytes().len() + 1);
        bytes.push(prefix as u8);
        bytes.extend_from_slice(input.as_bytes());
        let callback_input = CString::new(bytes).expect("the first NUL ends the prompt input");
        prompt_fire_callback(
            pr,
            callback_input.as_ptr(),
            PROMPT_KEY_HANDLED,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
    }
    return result;
}
fn prompt_complete_add(list: &mut Vec<CString>, s: &CStr) {
    if !list.iter().any(|name| name.as_bytes() == s.to_bytes()) {
        list.push(s.to_owned());
    }
}
unsafe fn prompt_complete_commands(s: &CStr) -> Vec<CString> {
    let mut list = Vec::new();
    let mut cmdent: *mut *const cmd_entry = ::core::ptr::null_mut::<*const cmd_entry>();
    let prefix = s.to_bytes();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    cmdent = &raw mut cmd_table as *mut *const cmd_entry;
    while !(*cmdent).is_null() {
        let name = CStr::from_ptr((**cmdent).name);
        if name.to_bytes().starts_with(prefix) {
            prompt_complete_add(&mut list, name);
        }
        cmdent = cmdent.offset(1);
    }
    o = options_get_only(
        global_options,
        b"command-alias\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !o.is_null() {
        a = options_array_first(o);
        while !a.is_null() {
            let value = CStr::from_ptr((*options_array_item_value(a)).string_ptr());
            if let Some(separator) = value.to_bytes().iter().position(|&byte| byte == b'=') {
                if prefix.len() <= separator && &value.to_bytes()[..prefix.len()] == prefix {
                    let alias = CString::new(&value.to_bytes()[..separator])
                        .expect("alias prefix contains no NUL");
                    prompt_complete_add(&mut list, alias.as_c_str());
                }
            }
            a = options_array_next(a);
        }
    }
    return list;
}
fn prompt_complete_prefix(list: &[CString]) -> CString {
    let first = list
        .first()
        .expect("completion list is nonempty")
        .as_bytes();
    let mut prefix_len = first.len();
    for name in &list[1..] {
        prefix_len = first[..prefix_len]
            .iter()
            .zip(name.as_bytes())
            .take_while(|(a, b)| a == b)
            .count();
    }
    CString::new(&first[..prefix_len]).expect("completion names contain no NUL")
}
unsafe extern "C" fn prompt_clear_complete(mut pr: *mut prompt) {
    (*pr).completion.names = Vec::new();
    (*pr).completion.display = None;
}
unsafe fn prompt_store_complete(mut pr: *mut prompt, list: Vec<CString>) {
    prompt_clear_complete(pr);
    (*pr).completion.names = list;
    let mut display = Vec::new();
    for name in &(*pr).completion.names {
        display.push(b' ');
        display.extend_from_slice(name.as_bytes());
    }
    (*pr).completion.display = Some(CString::new(display).expect("names contain no NUL"));
}
unsafe fn prompt_complete(mut pr: *mut prompt, word: &CStr, mut offset: u_int) -> Option<CString> {
    let mut list: Vec<CString>;
    let mut i: u_int = 0;
    if (*pr).type_0 as ::core::ffi::c_uint
        != PROMPT_TYPE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
        || offset != 0 as u_int
        || word.to_bytes().is_empty()
    {
        return None;
    }
    list = prompt_complete_commands(word);
    if list.is_empty() {
        return None;
    }
    list.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    i = 0 as u_int;
    while (i as usize) < list.len() {
        log_debug(
            b"complete %u: %s\0" as *const u8 as *const ::core::ffi::c_char,
            i,
            list[i as usize].as_ptr(),
        );
        i = i.wrapping_add(1);
    }
    let out = if list.len() == 1 {
        let mut bytes = list[0].as_bytes().to_vec();
        bytes.push(b' ');
        CString::new(bytes).expect("completion name contains no NUL")
    } else {
        prompt_complete_prefix(&list)
    };
    if word != out.as_c_str() {
        return Some(out);
    }
    if list.len() <= 1 {
        return None;
    }
    prompt_store_complete(pr, list);
    None
}
#[no_mangle]
pub unsafe extern "C" fn prompt_type(mut type_0: *const ::core::ffi::c_char) -> prompt_type {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < PROMPT_NTYPES as u_int {
        if strcmp(type_0, prompt_type_string(i as prompt_type)) == 0 as ::core::ffi::c_int {
            return i as prompt_type;
        }
        i = i.wrapping_add(1);
    }
    return PROMPT_TYPE_INVALID;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_type_string(mut type_0: prompt_type) -> *const ::core::ffi::c_char {
    match type_0 as ::core::ffi::c_uint {
        0 => return b"command\0" as *const u8 as *const ::core::ffi::c_char,
        1 => return b"search\0" as *const u8 as *const ::core::ffi::c_char,
        255 => return b"invalid\0" as *const u8 as *const ::core::ffi::c_char,
        _ => {}
    }
    return b"unknown\0" as *const u8 as *const ::core::ffi::c_char;
}

#[cfg(test)]
mod prompt_buffer_tests {
    use super::*;

    fn make_prompt(input: &CStr, index: usize, copied: Option<Box<[utf8_data]>>) -> Box<prompt> {
        let mut allocation = Box::new(prompt::default());
        let raw = &mut *allocation as *mut prompt;
        unsafe {
            (*raw).string = CString::new(Vec::new()).unwrap();
            (*raw).buffer = utf8_fromcstr_vec(input);
            (*raw).last = None;
            (*raw).message_format = CString::new(Vec::new()).unwrap();
            (*raw).word_separators = CString::new(Vec::new()).unwrap();
            (*raw).style_str = CString::new(Vec::new()).unwrap();
            (*raw).command_style_str = CString::new(Vec::new()).unwrap();
            (*raw).copied = copied;
            (*raw).index = index;
        }
        allocation
    }

    fn cell(byte: u8) -> utf8_data {
        let mut result = utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        };
        unsafe { utf8_set(&raw mut result, byte) };
        result
    }

    #[test]
    fn completion_keeps_prefix_suffix_bytes_and_only_the_logical_sentinel() {
        let input = CString::new("cmd old tail").unwrap();
        let replacement = CString::new(vec![0xff]).unwrap();
        let mut pr = make_prompt(&input, 7, None);
        pr.buffer.push(cell(b'!'));

        unsafe {
            assert_eq!(prompt_replace_complete(&mut *pr, replacement.as_ptr()), 1);
            assert_eq!(
                utf8_tocstr_cstring(pr.buffer.as_ptr()).as_bytes(),
                b"cmd \xff tail"
            );
            assert_eq!(pr.index, 5);
            assert_eq!(pr.buffer.len(), utf8_strlen(pr.buffer.as_ptr()) + 1);
        }
    }

    #[test]
    fn copied_paste_inserts_cells_before_the_sentinel() {
        let input = CString::new(&b"a\xc3\xa9Z"[..]).unwrap();
        let copied = CString::new(&b"\xce\xbb"[..]).unwrap();
        let copied_cells = utf8_fromcstr_vec(copied.as_c_str()).into_boxed_slice();
        let mut pr = make_prompt(&input, 1, Some(copied_cells));
        pr.buffer.push(cell(b'!'));

        unsafe {
            assert_eq!(prompt_paste(&mut *pr), 1);
            prompt_trim_buffer(&mut pr);
            assert_eq!(
                utf8_tocstr_cstring(pr.buffer.as_ptr()).as_bytes(),
                b"a\xce\xbb\xc3\xa9Z"
            );
            assert_eq!(pr.index, 2);
            assert_eq!(pr.buffer.len(), utf8_strlen(pr.buffer.as_ptr()) + 1);
        }
    }
}
