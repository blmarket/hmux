use crate::src::cmd::cmd_table;
use crate::src::cmd::find::{cmd_find_clear_state, cmd_find_valid_state};
use crate::src::ffi::libc::{memcpy, memmove, memset, strlen};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_create_defaults, format_create_from_state, format_expand_time_cstring,
    format_free,
};
use crate::src::format_draw::{format_draw, format_width};
use crate::src::grid::grid_default_cell;
use crate::src::key_string::key_string_format;
use crate::src::log::{log_cstr, log_debug};
use crate::src::options::{
    options_array_first, options_array_item_value, options_array_next, options_get_number,
    options_get_only, options_get_string,
};
use crate::src::paste::paste_get_top;
use crate::src::prompt_history::{prompt_add_history, prompt_down_history, prompt_up_history};
use crate::src::screen::screen_set_cursor_style;
use crate::src::screen_write::{
    screen_write_cell, screen_write_clearcharacter, screen_write_cursormove,
};
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::command::{cmd_entry, cmd_find_state, cmdq_item};
use crate::src::shared::format::format_tree;
use crate::src::shared::grid::*;
use crate::src::shared::key::MODEKEY_VI;
use crate::src::shared::key::*;
use crate::src::shared::options::{options, options_array_item, options_entry};
use crate::src::shared::pane::window_pane;
use crate::src::shared::prompt::prompt;
use crate::src::shared::prompt::*;
use crate::src::shared::prompt::{prompt_create_data, prompt_draw_data};
use crate::src::shared::prompt::{
    prompt_result, PROMPT_ACCEPT, PROMPT_BSPACE_EXIT, PROMPT_CLOSE, PROMPT_COMMANDMODE,
    PROMPT_CONTINUE, PROMPT_EDITARROWS, PROMPT_INCREMENTAL, PROMPT_ISMODE, PROMPT_ISPANE,
    PROMPT_KEY, PROMPT_NOFORMAT, PROMPT_NOFREEZE, PROMPT_NUMERIC, PROMPT_QUOTENEXT,
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

const PROMPT_CALLBACK_ACTIVE: ::core::ffi::c_int = 0x2000;
const PROMPT_FREE_PENDING: ::core::ffi::c_int = 0x4000;
const PROMPT_FREEING: ::core::ffi::c_int = 0x8000;

unsafe fn prompt_buffer_cells(pr: *mut prompt) -> *mut utf8_data {
    (*pr).buffer.as_mut_ptr()
}

fn prompt_trim_buffer(pr: &mut prompt) {
    let len = utf8_strlen(&pr.buffer) + 1;
    pr.buffer.truncate(len);
}

#[derive(Copy, Clone, Default)]
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

fn prompt_write_flags(
    out: &mut dyn std::io::Write,
    flags: ::core::ffi::c_int,
) -> std::io::Result<()> {
    let mut separator = "";
    for (flag, name) in [
        (PROMPT_SINGLE, "SINGLE"),
        (PROMPT_NUMERIC, "NUMERIC"),
        (PROMPT_INCREMENTAL, "INCREMENTAL"),
        (PROMPT_NOFORMAT, "NOFORMAT"),
        (PROMPT_KEY, "KEY"),
        (PROMPT_ACCEPT, "ACCEPT"),
        (PROMPT_QUOTENEXT, "QUOTENEXT"),
        (PROMPT_BSPACE_EXIT, "BSPACE_EXIT"),
        (PROMPT_NOFREEZE, "NOFREEZE"),
        (PROMPT_COMMANDMODE, "COMMANDMODE"),
        (PROMPT_ISPANE, "ISPANE"),
        (PROMPT_ISMODE, "ISMODE"),
        (PROMPT_EDITARROWS, "EDITARROWS"),
    ] {
        if flags & flag != 0 {
            out.write_all(separator.as_bytes())?;
            out.write_all(name.as_bytes())?;
            separator = ",";
        }
    }
    Ok(())
}
pub unsafe fn prompt_set_options(pd: &mut prompt_create_data<'_>, s: Option<&session>) {
    let oo = s.map_or(global_s_options, |s| s.options);
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
    style_apply(
        &raw mut pd.style,
        oo,
        b"message-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    style_apply(
        &raw mut pd.command_style,
        oo,
        b"message-command-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    pd.style_str = CStr::from_ptr(options_get_string(
        oo,
        b"message-style\0" as *const u8 as *const ::core::ffi::c_char,
    ))
    .to_owned();
    pd.command_style_str = CStr::from_ptr(options_get_string(
        oo,
        b"message-command-style\0" as *const u8 as *const ::core::ffi::c_char,
    ))
    .to_owned();
    n = options_get_number(
        oo,
        b"prompt-cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    screen_set_cursor_style(n, &mut pd.cstyle, &mut pd.cmode);
    n = options_get_number(
        oo,
        b"prompt-command-cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    screen_set_cursor_style(n, &mut pd.command_cstyle, &mut pd.command_cmode);
    style_apply(
        &raw mut gc,
        oo,
        b"prompt-cursor-colour\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    pd.ccolour = gc.fg;
    style_apply(
        &raw mut gc,
        oo,
        b"prompt-command-cursor-colour\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    pd.command_ccolour = gc.fg;
    pd.message_format = CStr::from_ptr(options_get_string(
        oo,
        b"message-format\0" as *const u8 as *const ::core::ffi::c_char,
    ))
    .to_owned();
    pd.keys = options_get_number(
        oo,
        b"status-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    pd.word_separators = CStr::from_ptr(options_get_string(
        oo,
        b"word-separators\0" as *const u8 as *const ::core::ffi::c_char,
    ))
    .to_owned();
}
pub unsafe fn prompt_create(pd: prompt_create_data<'_>) -> Box<prompt> {
    let mut pr = Box::new(prompt::default());
    let ft = if let Some(fs) = pd.fs {
        // Copy the selected target, matching cmd_find_copy_state rather than
        // inheriting the source's search flags or current-state pointer.
        pr.state = cmd_find_state {
            s: fs.s,
            wl: fs.wl,
            w: fs.w,
            wp: fs.wp,
            idx: fs.idx,
            ..Default::default()
        };
        format_create_defaults(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            fs.s,
            fs.wl,
            fs.wp,
        )
    } else {
        cmd_find_clear_state(&mut pr.state, 0);
        format_create_defaults(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    let input = pd.input.unwrap_or(c"");
    pr.string = pd.prompt.to_owned();
    let expanded = if pd.flags & PROMPT_NOFORMAT != 0 {
        None
    } else {
        Some(format_expand_time_cstring(ft, input.as_ptr()))
    };
    if pd.flags & PROMPT_INCREMENTAL != 0 {
        pr.last = Some(expanded.unwrap_or_else(|| input.to_owned()));
        pr.buffer = utf8_fromcstr_vec(c"");
    } else {
        pr.buffer = utf8_fromcstr_vec(expanded.as_deref().unwrap_or(input));
    }
    pr.index = utf8_strlen(&pr.buffer);
    pr.inputcb = pd.inputcb;
    pr.freecb = pd.freecb;
    pr.flags = pd.flags;
    pr.type_0 = pd.type_0;
    pr.style = pd.style;
    pr.command_style = pd.command_style;
    pr.style_str = pd.style_str;
    pr.command_style_str = pd.command_style_str;
    pr.cstyle = pd.cstyle;
    pr.command_cstyle = pd.command_cstyle;
    pr.ccolour = pd.ccolour;
    pr.command_ccolour = pd.command_ccolour;
    pr.cmode = pd.cmode;
    pr.command_cmode = pd.command_cmode;
    pr.message_format = pd.message_format;
    pr.keys = pd.keys;
    pr.word_separators = pd.word_separators;
    format_free(ft);
    pr
}
pub unsafe fn prompt_free(mut pr: *mut prompt) {
    if pr.is_null() {
        return;
    }
    if (*pr).flags & PROMPT_FREEING != 0 {
        return;
    }
    if (*pr).flags & PROMPT_CALLBACK_ACTIVE != 0 {
        (*pr).flags |= PROMPT_FREE_PENDING;
        return;
    }
    (*pr).flags |= PROMPT_FREEING;
    if let Some(callback) = (*pr).freecb.take() {
        callback();
    }
    prompt_clear_complete(&mut *pr);
    drop(Box::from_raw(pr));
}
fn prompt_last(pr: &prompt) -> &CStr {
    pr.last
        .as_ref()
        .expect("incremental prompt has saved input")
        .as_c_str()
}
unsafe fn prompt_fire_callback(
    mut pr: *mut prompt,
    mut s: *const ::core::ffi::c_char,
    mut type_0: prompt_key_result,
    mut redraw: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut callback = (*pr).inputcb.take().expect("non-null prompt callback");
    (*pr).flags |= PROMPT_CALLBACK_ACTIVE;
    let text = (!s.is_null()).then(|| CStr::from_ptr(s));
    let result = callback(text, type_0);
    let free_pending = (*pr).flags & PROMPT_FREE_PENDING != 0;
    (*pr).flags &= !PROMPT_CALLBACK_ACTIVE;
    if free_pending {
        prompt_free(pr);
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
pub unsafe fn prompt_incremental_start(mut pr: *mut prompt) {
    if (*pr).flags & PROMPT_INCREMENTAL != 0 {
        let input = utf8_tocstr_cstring(&(*pr).buffer);
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
pub unsafe fn prompt_update(
    mut pr: *mut prompt,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
) {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    if cmd_find_valid_state(&(*pr).state) != 0 {
        ft = format_create_from_state(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            &(*pr).state,
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
    (*pr).index = utf8_strlen(&(*pr).buffer);
    memset(
        &raw mut (*pr).hindex as *mut u_int as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[u_int; 2]>() as size_t,
    );
    (*pr).closed = 0 as ::core::ffi::c_int;
    prompt_clear_complete(&mut *pr);
    format_free(ft);
}
pub unsafe fn prompt_closed(mut pr: *mut prompt) -> ::core::ffi::c_int {
    return (*pr).closed;
}
unsafe fn prompt_redraw_character(
    ctx: &mut screen_write_ctx,
    mut offset: u_int,
    mut pwidth: u_int,
    width: &mut u_int,
    gc: &mut grid_cell,
    ud: &utf8_data,
) -> ::core::ffi::c_int {
    let mut ch: u_char = 0;
    if *width < offset {
        *width = (*width).wrapping_add(ud.width as u_int);
        return 1 as ::core::ffi::c_int;
    }
    if *width >= offset.wrapping_add(pwidth) {
        return 0 as ::core::ffi::c_int;
    }
    *width = (*width).wrapping_add(ud.width as u_int);
    if *width > offset.wrapping_add(pwidth) {
        return 0 as ::core::ffi::c_int;
    }
    ch = ud.data[0];
    if ud.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && (ch as ::core::ffi::c_int <= 0x1f as ::core::ffi::c_int
            || ch as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int)
    {
        gc.data.data[0 as ::core::ffi::c_int as usize] = '^' as i32 as u_char;
        gc.data.data[1 as ::core::ffi::c_int as usize] =
            (if ch as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int {
                '?' as i32
            } else {
                ch as ::core::ffi::c_int | 0x40 as ::core::ffi::c_int
            }) as u_char;
        gc.data.have = 2 as u_char;
        gc.data.size = gc.data.have;
        gc.data.width = 2 as u_char;
    } else {
        gc.data = utf8_copy(ud);
    }
    screen_write_cell(ctx, gc);
    return 1 as ::core::ffi::c_int;
}
unsafe fn prompt_redraw_quote(
    pr: &prompt,
    mut pcursor: u_int,
    mut input_x: u_int,
    ctx: &mut screen_write_ctx,
    mut offset: u_int,
    mut pw: u_int,
    w: &mut u_int,
    gc: &mut grid_cell,
) -> ::core::ffi::c_int {
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    if pr.flags & PROMPT_QUOTENEXT != 0
        && pcursor >= offset
        && (*ctx.s).cx == input_x.wrapping_add(pcursor).wrapping_sub(offset)
    {
        utf8_set(&mut ud, '^' as i32 as u_char);
        return prompt_redraw_character(ctx, offset, pw, w, gc, &ud);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn prompt_draw_complete(
    pr: &prompt,
    ctx: &mut screen_write_ctx,
    mut ax: u_int,
    mut aw: u_int,
    mut cx: u_int,
    mut py: u_int,
    base: &grid_cell,
) {
    let mut gc = *base;
    let mut avail: u_int = 0;
    let mut width: u_int = 0;
    let Some(display) = pr.completion.display.as_deref() else {
        return;
    };
    if pr.index != utf8_strlen(&pr.buffer) {
        return;
    }
    if cx < ax || cx.wrapping_sub(ax) >= aw {
        return;
    }
    avail = aw.wrapping_sub(cx.wrapping_sub(ax));
    gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE) as u_short;
    screen_write_cursormove(
        ctx,
        cx as ::core::ffi::c_int,
        py as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    width = 0 as u_int;
    let mut cells = utf8_fromcstr_vec(display);
    for cell in &mut cells {
        if cell.size == 0 || width.wrapping_add(cell.width as u_int) > avail {
            break;
        }
        gc.data = utf8_copy(cell);
        screen_write_cell(ctx, &gc);
        width = width.wrapping_add(cell.width as u_int);
    }
}
unsafe fn prompt_format_tree(pr: &prompt) -> *mut format_tree {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    if cmd_find_valid_state(&pr.state) != 0 {
        ft = format_create_from_state(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            &pr.state,
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
    let tmp = utf8_tocstr_cstring(&pr.buffer);
    format_add(
        ft,
        b"prompt_input\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, tmp.as_ptr()),
    );
    format_add(
        ft,
        b"prompt_flags\0" as *const u8 as *const ::core::ffi::c_char,
        |out| prompt_write_flags(out, pr.flags),
    );
    format_add(
        ft,
        b"prompt_type\0" as *const u8 as *const ::core::ffi::c_char,
        |out| out.write_all(prompt_type_string(pr.type_0).to_bytes()),
    );
    if pr.flags & PROMPT_COMMANDMODE != 0 {
        format_add(
            ft,
            b"command_prompt\0" as *const u8 as *const ::core::ffi::c_char,
            |out| out.write_all(b"1"),
        );
    } else {
        format_add(
            ft,
            b"command_prompt\0" as *const u8 as *const ::core::ffi::c_char,
            |out| out.write_all(b"0"),
        );
    }
    return ft;
}
unsafe fn prompt_expand1(pr: &prompt, mut ft: *mut format_tree) -> CString {
    let prompt = format_expand_time_cstring(ft, pr.string.as_ptr());
    format_add(
        ft,
        b"message\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, prompt.as_ptr()),
    );
    format_expand_time_cstring(ft, pr.message_format.as_ptr())
}
unsafe fn prompt_effective_style(pr: &prompt, sy: &mut style, ft: *mut format_tree) {
    let (text, cell) = if pr.flags & PROMPT_COMMANDMODE != 0 {
        (&pr.command_style_str, &pr.command_style)
    } else {
        (&pr.style_str, &pr.style)
    };
    style_set(sy, cell);
    let expanded = format_expand_time_cstring(ft, text.as_ptr());
    if style_parse(sy, &grid_default_cell, expanded.as_ptr()) != 0 {
        style_set(sy, cell);
    }
}
unsafe fn prompt_layout(
    pr: &prompt,
    mut ax: u_int,
    mut aw: u_int,
    mut sy: Option<&mut style>,
) -> (prompt_layout, CString) {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut pcursor: u_int = 0;
    let mut pwidth: u_int = 0;
    let mut end: u_int = 0;
    let mut width: u_int = 0;
    let mut offset: u_int = 0;
    let mut avail: u_int = 0;
    let mut pl = prompt_layout {
        area_x: ax,
        area_width: aw,
        ..Default::default()
    };
    ft = prompt_format_tree(pr);
    if let Some(sy) = sy.as_deref_mut() {
        prompt_effective_style(pr, sy, ft);
    }
    let expanded = prompt_expand1(pr, ft);
    format_free(ft);
    if aw == 0 as u_int {
        return (pl, expanded);
    }
    pl.label_width = format_width(expanded.as_ptr());
    if pl.label_width > aw {
        pl.label_width = aw;
    }
    pcursor = utf8_strwidth(&pr.buffer, pr.index as ssize_t);
    pwidth = utf8_strwidth(&pr.buffer, -(1 as ::core::ffi::c_int) as ssize_t);
    if pr.flags & PROMPT_QUOTENEXT != 0 {
        pwidth = pwidth.wrapping_add(1);
    }
    avail = aw.wrapping_sub(pl.label_width);
    if avail == 0 as u_int {
        pl.input_offset = 0 as u_int;
        pl.input_width = 0 as u_int;
        pl.cursor_x = pl.label_width;
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
        pl.input_offset = offset;
        pl.input_width = width;
        pl.cursor_x = pl.label_width.wrapping_add(pcursor).wrapping_sub(offset);
    }
    pl.content_width = pl.label_width.wrapping_add(pl.input_width);
    if let Some(display) = pr
        .completion
        .display
        .as_deref()
        .filter(|_| pr.index == utf8_strlen(&pr.buffer) && pl.cursor_x < aw)
    {
        avail = aw.wrapping_sub(pl.cursor_x);
        width = utf8_cstrwidth(display);
        if width > avail {
            width = avail;
        }
        end = pl.cursor_x.wrapping_add(width);
        if end > pl.content_width {
            pl.content_width = end;
        }
    }
    if pl.content_width > aw {
        pl.content_width = aw;
    }
    if let Some(sy) = sy.as_ref() {
        match sy.align as ::core::ffi::c_uint {
            2 | 4 => {
                pl.content_x =
                    ax.wrapping_add(aw.wrapping_sub(pl.content_width).wrapping_div(2 as u_int));
            }
            3 => {
                pl.content_x = ax.wrapping_add(aw).wrapping_sub(pl.content_width);
            }
            _ => {
                pl.content_x = ax;
            }
        }
    } else {
        pl.content_x = ax;
    }
    pl.input_x = pl.content_x.wrapping_add(pl.label_width);
    pl.cursor_x = pl.cursor_x.wrapping_add(pl.content_x);
    (pl, expanded)
}
unsafe fn prompt_mouse_complete(
    pr: &mut prompt,
    mut x: u_int,
    mut cx: u_int,
    mut ax: u_int,
    mut aw: u_int,
    mut redraw: Option<&mut ::core::ffi::c_int>,
) -> prompt_key_result {
    let mut avail: u_int = 0;
    let mut clicked: u_int = 0;
    let mut end: u_int = 0;
    let mut i: u_int = 0;
    let mut start: u_int = 0;
    let mut width: u_int = 0;
    let Some(display) = pr.completion.display.as_deref() else {
        return PROMPT_KEY_NOT_HANDLED;
    };
    if pr.completion.names.is_empty() {
        return PROMPT_KEY_NOT_HANDLED;
    }
    if pr.index != utf8_strlen(&pr.buffer) {
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
    while (i as usize) < pr.completion.names.len() {
        start = end.wrapping_add(1 as u_int);
        end = start.wrapping_add(utf8_cstrwidth(&pr.completion.names[i as usize]));
        if clicked < start || clicked >= end {
            i = i.wrapping_add(1);
        } else {
            let mut replacement = pr.completion.names[i as usize].as_bytes().to_vec();
            replacement.push(b' ');
            let replacement = CString::new(replacement).expect("completion name contains no NUL");
            if prompt_replace_complete(pr, Some(&replacement)) != 0 {
                prompt_clear_complete(pr);
                if let Some(redraw) = redraw {
                    *redraw = 1 as ::core::ffi::c_int;
                }
            }
            return PROMPT_KEY_HANDLED;
        }
    }
    return PROMPT_KEY_HANDLED;
}
pub unsafe fn prompt_draw(pr: &prompt, ctx: &mut screen_write_ctx, pd: prompt_draw_data) -> u_int {
    let mut s: *mut screen = ctx.s;
    let mut ax: u_int = pd.area_x;
    let mut py: u_int = pd.prompt_line;
    let mut aw: u_int = pd.area_width;
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
    let mut width: u_int = 0;
    let mut pcursor: u_int = 0;
    if pr.flags & PROMPT_COMMANDMODE != 0 {
        (*s).default_cstyle = pr.command_cstyle;
        (*s).default_mode = pr.command_cmode;
        (*s).default_ccolour = pr.command_ccolour;
    } else {
        (*s).default_cstyle = pr.cstyle;
        (*s).default_mode = pr.cmode;
        (*s).default_ccolour = pr.ccolour;
    }
    let (pl, expanded) = prompt_layout(pr, ax, aw, Some(&mut sy));
    let mut gc = sy.gc;
    screen_write_cursormove(
        ctx,
        ax as ::core::ffi::c_int,
        py as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if sy.fill != 8 as ::core::ffi::c_int {
        screen_write_clearcharacter(ctx, aw, sy.fill as u_int);
    }
    pcursor = utf8_strwidth(&pr.buffer, pr.index as ssize_t);
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
                &mut gc,
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
        for cell in pr.buffer.iter().take_while(|cell| cell.size != 0) {
            if prompt_redraw_quote(
                pr,
                pcursor,
                pl.input_x,
                ctx,
                pl.input_offset,
                pl.input_width,
                &mut width,
                &mut gc,
            ) == 0
            {
                break;
            }
            if prompt_redraw_character(
                ctx,
                pl.input_offset,
                pl.input_width,
                &mut width,
                &mut gc,
                cell,
            ) == 0
            {
                break;
            }
        }
        prompt_redraw_quote(
            pr,
            pcursor,
            pl.input_x,
            ctx,
            pl.input_offset,
            pl.input_width,
            &mut width,
            &mut gc,
        );
        prompt_draw_complete(
            pr,
            ctx,
            pl.content_x,
            pl.content_width,
            pl.cursor_x,
            py,
            &gc,
        );
    }
    pl.cursor_x
}
pub unsafe fn prompt_mouse(
    pr: &mut prompt,
    mut x: u_int,
    mut ax: u_int,
    mut aw: u_int,
    mut redraw: Option<&mut ::core::ffi::c_int>,
) -> prompt_key_result {
    let mut result: prompt_key_result = PROMPT_KEY_NOT_HANDLED;
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
    let (pl, _) = prompt_layout(pr, ax, aw, Some(&mut sy));
    if pl.input_width == 0 as u_int {
        return PROMPT_KEY_HANDLED;
    }
    pwidth = utf8_strwidth(&pr.buffer, -(1 as ::core::ffi::c_int) as ssize_t);
    if pr.flags & PROMPT_QUOTENEXT != 0 {
        pwidth = pwidth.wrapping_add(1);
    }
    result = prompt_mouse_complete(
        pr,
        x,
        pl.cursor_x,
        pl.content_x,
        pl.content_width,
        redraw.as_deref_mut(),
    );
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
    for cell in pr.buffer.iter().take_while(|cell| cell.size != 0) {
        if width >= target {
            break;
        }
        width = width.wrapping_add(cell.width as u_int);
        idx += 1;
    }
    if idx == pr.index {
        return PROMPT_KEY_HANDLED;
    }
    pr.index = idx;
    prompt_clear_complete(pr);
    if let Some(redraw) = redraw {
        *redraw = 1 as ::core::ffi::c_int;
    }
    return PROMPT_KEY_HANDLED;
}
fn prompt_in_list(separators: &CStr, cell: &utf8_data) -> ::core::ffi::c_int {
    // strchr also matches the terminator when the cell contains a literal NUL.
    (cell.size == 1 && cell.width == 1 && separators.to_bytes_with_nul().contains(&cell.data[0]))
        as ::core::ffi::c_int
}
fn prompt_space(cell: &utf8_data) -> ::core::ffi::c_int {
    (cell.size == 1 && cell.width == 1 && cell.data[0] == b' ') as ::core::ffi::c_int
}
fn prompt_keypad_key(key: key_code) -> key_code {
    if key & KEYC_MASK_MODIFIERS != 0 {
        return key;
    }
    match key {
        KEYC_KP_SLASH => b'/' as key_code,
        KEYC_KP_STAR => b'*' as key_code,
        KEYC_KP_MINUS => b'-' as key_code,
        KEYC_KP_SEVEN => b'7' as key_code,
        KEYC_KP_EIGHT => b'8' as key_code,
        KEYC_KP_NINE => b'9' as key_code,
        KEYC_KP_PLUS => b'+' as key_code,
        KEYC_KP_FOUR => b'4' as key_code,
        KEYC_KP_FIVE => b'5' as key_code,
        KEYC_KP_SIX => b'6' as key_code,
        KEYC_KP_ONE => b'1' as key_code,
        KEYC_KP_TWO => b'2' as key_code,
        KEYC_KP_THREE => b'3' as key_code,
        KEYC_KP_ENTER => b'\r' as key_code,
        KEYC_KP_ZERO => b'0' as key_code,
        KEYC_KP_PERIOD => b'.' as key_code,
        _ => key,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PromptTranslatedKey {
    Handled,
    Process(key_code),
    Append(key_code),
}

fn prompt_translate_key(
    pr: &mut prompt,
    key: key_code,
    redraw: &mut ::core::ffi::c_int,
) -> PromptTranslatedKey {
    use PromptTranslatedKey::{Append, Handled, Process};

    if pr.flags & PROMPT_COMMANDMODE == 0 {
        if key == 27 || key == KEYC_CTRL | b'[' as key_code {
            pr.flags |= PROMPT_COMMANDMODE;
            pr.index = pr.index.saturating_sub(1);
            *redraw = 1;
            return Handled;
        }
        if matches!(
            key,
            9 | 10
                | 13
                | KEYC_BSPACE
                | KEYC_DC
                | KEYC_DOWN
                | KEYC_END
                | KEYC_HOME
                | KEYC_LEFT
                | KEYC_RIGHT
                | KEYC_UP
        ) || key == KEYC_LEFT | KEYC_CTRL
            || key == KEYC_RIGHT | KEYC_CTRL
            || b"aceghknptuvwy"
                .iter()
                .any(|&byte| key == KEYC_CTRL | byte as key_code)
        {
            return Process(key);
        }
        return Append(key);
    }

    match key {
        KEYC_BSPACE => return Process(KEYC_LEFT),
        KEYC_DC | KEYC_DOWN | KEYC_LEFT | KEYC_RIGHT | KEYC_UP | 10 | 13 => {
            return Process(key);
        }
        _ => {}
    }
    if key == KEYC_CTRL | b'h' as key_code || key == KEYC_CTRL | b'c' as key_code {
        return Process(key);
    }
    let Ok(byte) = u8::try_from(key) else {
        return Handled;
    };
    match byte {
        b'A' | b'I' | b'C' | b's' | b'a' | b'S' | b'i' => {
            pr.flags &= !PROMPT_COMMANDMODE;
            *redraw = 1;
        }
        _ => {}
    }
    match byte {
        b'A' | b'$' => Process(KEYC_END),
        b'I' | b'0' | b'^' => Process(KEYC_HOME),
        b'C' | b'D' => Process(KEYC_CTRL | b'k' as key_code),
        b'X' => Process(KEYC_BSPACE),
        b'b' => Process(KEYC_META | b'b' as key_code),
        b'B' => Process(KEYC_VI | b'B' as key_code),
        b'd' | b'S' => Process(KEYC_CTRL | b'u' as key_code),
        b'e' | b'E' | b'w' | b'W' => Process(KEYC_VI | key),
        b'p' => Process(KEYC_CTRL | b'y' as key_code),
        b'q' => Process(KEYC_CTRL | b'c' as key_code),
        b's' | b'x' => Process(KEYC_DC),
        b'j' => Process(KEYC_DOWN),
        b'h' => Process(KEYC_LEFT),
        b'a' | b'l' => Process(KEYC_RIGHT),
        b'k' => Process(KEYC_UP),
        _ => Handled,
    }
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

unsafe fn prompt_decode_paste(data: &[u8]) -> Vec<utf8_data> {
    let mut cells = vec![utf8_data::default(); data.len().wrapping_add(1)];
    let mut count = 0;
    let mut i: u_int = 0;
    while i as usize != data.len() {
        let cell = &mut cells[count];
        let mut more = utf8_open(cell, data[i as usize]);
        if more == UTF8_MORE {
            loop {
                i = i.wrapping_add(1);
                if i as usize == data.len() || more != UTF8_MORE {
                    break;
                }
                more = utf8_append(cell, data[i as usize]);
            }
            if more == UTF8_DONE {
                count += 1;
                continue;
            }
            i = i.wrapping_sub(cell.have as u_int);
        }
        if !(b' '..=b'~').contains(&data[i as usize]) {
            break;
        }
        utf8_set(cell, data[i as usize]);
        count += 1;
        i = i.wrapping_add(1);
    }
    cells[count].size = 0;
    cells.truncate(count + 1);
    cells
}

unsafe fn prompt_paste(pr: &mut prompt) -> ::core::ffi::c_int {
    let decoded;
    let pasted = if let Some(copied) = pr.copied.as_deref() {
        &copied[..utf8_strlen(copied)]
    } else {
        let Some(buffer) = paste_get_top(None).as_ref() else {
            return 0;
        };
        let bytes = &buffer.data.as_deref().unwrap_or_default()[..buffer.size];
        decoded = prompt_decode_paste(bytes);
        &decoded[..decoded.len() - 1]
    };
    if !pasted.is_empty() {
        pr.buffer.splice(pr.index..pr.index, pasted.iter().copied());
        pr.index = pr.index.wrapping_add(pasted.len());
    }
    1
}
unsafe fn prompt_replace_complete(
    pr: &mut prompt,
    replacement: Option<&CStr>,
) -> ::core::ffi::c_int {
    let mut word = [0_u8; 64];
    let size = utf8_strlen(&pr.buffer);
    let index = pr.index.saturating_sub(1);
    let mut first = index;
    while first > 0 && prompt_space(&pr.buffer[first]) == 0 {
        first -= 1;
    }
    while pr.buffer[first].size != 0 && prompt_space(&pr.buffer[first]) != 0 {
        first += 1;
    }
    let mut last = index;
    while pr.buffer[last].size != 0 && prompt_space(&pr.buffer[last]) == 0 {
        last += 1;
    }
    while last > 0 && prompt_space(&pr.buffer[last]) != 0 {
        last -= 1;
    }
    if pr.buffer[last].size != 0 {
        last += 1;
    }
    if last < first {
        return 0;
    }
    let completed = if replacement.is_none() {
        let mut used = 0;
        for cell in &pr.buffer[first..last] {
            let bytes = &cell.data[..cell.size as usize];
            if used + bytes.len() >= word.len() {
                return 0;
            }
            word[used..used + bytes.len()].copy_from_slice(bytes);
            used += bytes.len();
        }
        // Quoted control cells may contain NUL, which ends the C completion word.
        let word =
            CStr::from_bytes_until_nul(&word[..used + 1]).expect("terminated completion word");
        prompt_complete(pr, word, first as u_int)
    } else {
        None
    };
    let Some(replacement) = replacement.or(completed.as_deref()) else {
        return 0;
    };
    let replacement_bytes = replacement.to_bytes();
    let mut cells = Vec::with_capacity(first + replacement_bytes.len() + size + 1 - last);
    cells.extend_from_slice(&pr.buffer[..first]);
    for &byte in replacement_bytes {
        let mut cell = utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        };
        utf8_set(&mut cell, byte);
        cells.push(cell);
    }
    cells.extend_from_slice(&pr.buffer[last..=size]);
    pr.buffer = cells;
    pr.index = first + replacement_bytes.len();
    1
}
fn prompt_forward_word(pr: &prompt, size: usize, vi: bool, separators: &CStr) -> usize {
    let buffer = &pr.buffer;
    let mut index = pr.index;
    if !vi {
        while index != size && prompt_space(&buffer[index]) != 0 {
            index += 1;
        }
    }
    if index == size {
        return index;
    }
    let word_is_separators =
        prompt_in_list(separators, &buffer[index]) != 0 && prompt_space(&buffer[index]) == 0;
    loop {
        index += 1;
        if prompt_space(&buffer[index]) != 0 {
            if vi {
                while index != size && prompt_space(&buffer[index]) != 0 {
                    index += 1;
                }
            }
            break;
        }
        if index == size || word_is_separators != (prompt_in_list(separators, &buffer[index]) != 0)
        {
            break;
        }
    }
    index
}
fn prompt_end_word(pr: &prompt, size: usize, separators: &CStr) -> usize {
    let buffer = &pr.buffer;
    let mut index = pr.index;
    if index == size {
        return index;
    }
    loop {
        index += 1;
        if index == size {
            return index;
        }
        if prompt_space(&buffer[index]) == 0 {
            break;
        }
    }
    let word_is_separators = prompt_in_list(separators, &buffer[index]);
    loop {
        index += 1;
        if index == size
            || prompt_space(&buffer[index]) != 0
            || word_is_separators != prompt_in_list(separators, &buffer[index])
        {
            break;
        }
    }
    index - 1
}
fn prompt_backward_word(pr: &prompt, separators: &CStr) -> usize {
    let buffer = &pr.buffer;
    let mut index = pr.index;
    while index != 0 {
        index -= 1;
        if prompt_space(&buffer[index]) == 0 {
            break;
        }
    }
    let word_is_separators = prompt_in_list(separators, &buffer[index]);
    while index != 0 {
        index -= 1;
        if prompt_space(&buffer[index]) != 0
            || word_is_separators != prompt_in_list(separators, &buffer[index])
        {
            index += 1;
            break;
        }
    }
    index
}
unsafe fn prompt_done(
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
    let s = utf8_tocstr_cstring(&(*pr).buffer);
    if !s.as_bytes().is_empty() {
        prompt_add_history(s.as_ptr(), (*pr).type_0 as u_int);
    }
    prompt_done(pr, s.as_ptr(), redraw)
}
unsafe fn prompt_check_move(mut pr: *mut prompt, mut key: key_code) -> prompt_key_result {
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
    let s = utf8_tocstr_cstring(&(*pr).buffer);
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
pub unsafe fn prompt_key(
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
    prompt_clear_complete(&mut *pr);
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
    size = utf8_strlen(&(*pr).buffer);
    key &= !KEYC_MASK_FLAGS;
    key = prompt_keypad_key(key);
    if (*pr).flags & PROMPT_NUMERIC != 0 {
        if key >= '0' as i32 as key_code && key <= '9' as i32 as key_code {
            current_block = 1115217863795707468;
        } else {
            let input = utf8_tocstr_cstring(&(*pr).buffer);
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
            match prompt_translate_key(&mut *pr, key, &mut *redraw) {
                PromptTranslatedKey::Process(translated) => {
                    key = translated;
                    current_block = 11090587058695514569;
                }
                PromptTranslatedKey::Append(translated) => {
                    key = translated;
                    current_block = 1115217863795707468;
                }
                PromptTranslatedKey::Handled => return PROMPT_KEY_HANDLED,
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                                    let cells = &mut (*pr).buffer;
                                    tmp = utf8_copy(&cells[idx - 2]);
                                    cells[idx - 2] = utf8_copy(&cells[idx - 1]);
                                    cells[idx - 1] = utf8_copy(&tmp);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                    (*pr).index = utf8_strlen(&(*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(&(&(*pr).buffer)[idx]) == 0 {
                                        break;
                                    }
                                }
                                word_is_separators =
                                    prompt_in_list(&(*pr).word_separators, &(&(*pr).buffer)[idx]);
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(&(&(*pr).buffer)[idx]) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                &(*pr).word_separators,
                                                &(&(*pr).buffer)[idx],
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
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
                                        (*pr).index = utf8_strlen(&(*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(&mut *pr, None) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, false, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                (*pr).index = prompt_end_word(&*pr, size, c"");
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                (*pr).index = prompt_end_word(&*pr, size, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                (*pr).index = prompt_forward_word(&*pr, size, true, c"");
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                (*pr).index =
                                    prompt_forward_word(&*pr, size, true, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                (*pr).index = prompt_backward_word(&*pr, c"");
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                (*pr).index = prompt_backward_word(&*pr, &(*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(&mut *pr) != 0 {
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
                utf8_set(&mut tmp, key as u_char);
                if key <= 0x1f as key_code || key == 0x7f as key_code {
                    tmp.width = 2 as u_char;
                }
            } else if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                == (KEYC_TYPE_UNICODE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    > 0x7f as ::core::ffi::c_ulonglong
            {
                utf8_to_data(key as utf8_char, &mut tmp);
                if tmp.size as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    return PROMPT_KEY_HANDLED;
                }
            } else {
                return PROMPT_KEY_HANDLED;
            }
            (*pr).buffer.insert((*pr).index, tmp);
            (*pr).index = (*pr).index.wrapping_add(1);
            if (*pr).flags & PROMPT_SINGLE != 0 {
                if utf8_strlen(&(*pr).buffer) != 1 as size_t {
                    (*pr).closed = 1 as ::core::ffi::c_int;
                    result = PROMPT_KEY_CLOSE;
                } else {
                    let input = utf8_tocstr_cstring(&(*pr).buffer);
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
        let input = utf8_tocstr_cstring(&(*pr).buffer);
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
        let name = (**cmdent).name;
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
fn prompt_clear_complete(pr: &mut prompt) {
    pr.completion.names = Vec::new();
    pr.completion.display = None;
}
fn prompt_store_complete(pr: &mut prompt, list: Vec<CString>) {
    prompt_clear_complete(pr);
    pr.completion.names = list;
    let mut display = Vec::new();
    for name in &pr.completion.names {
        display.push(b' ');
        display.extend_from_slice(name.as_bytes());
    }
    pr.completion.display = Some(CString::new(display).expect("names contain no NUL"));
}
unsafe fn prompt_complete(pr: &mut prompt, word: &CStr, mut offset: u_int) -> Option<CString> {
    let mut list: Vec<CString>;
    let mut i: u_int = 0;
    if pr.type_0 as ::core::ffi::c_uint
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
        log_debug(format_args!(
            "complete {}: {}",
            (i) as u32,
            log_cstr((list[i as usize].as_ptr()) as *const _)
        ));
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
pub fn prompt_type(name: &CStr) -> prompt_type {
    match name.to_bytes() {
        b"command" => PROMPT_TYPE_COMMAND,
        b"search" => PROMPT_TYPE_SEARCH,
        _ => PROMPT_TYPE_INVALID,
    }
}
pub fn prompt_type_string(kind: prompt_type) -> &'static CStr {
    match kind {
        PROMPT_TYPE_COMMAND => c"command",
        PROMPT_TYPE_SEARCH => c"search",
        PROMPT_TYPE_INVALID => c"invalid",
        _ => c"unknown",
    }
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
        utf8_set(&mut result, byte);
        result
    }

    #[test]
    fn word_motion_preserves_tmux_separator_classes_and_end_positions() {
        // Each row is (Emacs forward, vi forward, end of word, backward),
        // for every cursor position including the sentinel. Golden indices
        // come from the pinned tmux e880cf63e0a9 helpers.
        type Motion = (usize, usize, usize, usize);
        let cases: &[(&CStr, &CStr, &[Motion])] = &[
            (c"", c",/", &[(0, 0, 0, 0)]),
            (c"   ", c",/", &[(3, 3, 3, 0); 4]),
            (
                c"ab,cd",
                c",/",
                &[
                    (2, 2, 1, 0),
                    (2, 2, 2, 0),
                    (3, 3, 4, 0),
                    (5, 5, 4, 2),
                    (5, 5, 5, 3),
                    (5, 5, 5, 3),
                ],
            ),
            (
                c"ab,cd",
                c"",
                &[
                    (5, 5, 4, 0),
                    (5, 5, 4, 0),
                    (5, 5, 4, 0),
                    (5, 5, 4, 0),
                    (5, 5, 5, 0),
                    (5, 5, 5, 0),
                ],
            ),
            (
                c"  ab,,é漢 /z  ",
                c",/",
                &[
                    (4, 2, 3, 0),
                    (4, 4, 3, 0),
                    (4, 4, 3, 0),
                    (4, 4, 5, 2),
                    (6, 6, 5, 2),
                    (6, 6, 7, 4),
                    (8, 9, 7, 4),
                    (8, 9, 9, 6),
                    (10, 9, 9, 6),
                    (10, 10, 10, 6),
                    (11, 13, 13, 9),
                    (13, 13, 13, 10),
                    (13, 13, 13, 10),
                    (13, 13, 13, 10),
                ],
            ),
            (
                c"  ab,,é漢 /z  ",
                c"",
                &[
                    (8, 2, 7, 0),
                    (8, 9, 7, 0),
                    (8, 9, 7, 0),
                    (8, 9, 7, 2),
                    (8, 9, 7, 2),
                    (8, 9, 7, 2),
                    (8, 9, 7, 2),
                    (8, 9, 10, 2),
                    (11, 13, 10, 2),
                    (11, 13, 10, 2),
                    (11, 13, 13, 9),
                    (13, 13, 13, 9),
                    (13, 13, 13, 9),
                    (13, 13, 13, 9),
                ],
            ),
            (
                c"a\tb c",
                c",/",
                &[
                    (3, 4, 2, 0),
                    (3, 4, 2, 0),
                    (3, 4, 4, 0),
                    (5, 5, 4, 0),
                    (5, 5, 5, 0),
                    (5, 5, 5, 4),
                ],
            ),
        ];
        for &(text, separators, expected) in cases {
            let mut pr = make_prompt(text, 0, None);
            let size = utf8_strlen(&pr.buffer);
            assert_eq!(expected.len(), size + 1);
            for (index, &positions) in expected.iter().enumerate() {
                pr.index = index;
                assert_eq!(
                    (
                        prompt_forward_word(&pr, size, false, separators),
                        prompt_forward_word(&pr, size, true, separators),
                        prompt_end_word(&pr, size, separators),
                        prompt_backward_word(&pr, separators)
                    ),
                    positions,
                    "{text:?}, separators {separators:?}, cursor {index}",
                );
            }
        }
        assert_eq!(prompt_in_list(c",/", &cell(0)), 1);
        let mut wide_space = cell(b' ');
        wide_space.width = 2;
        assert_eq!(prompt_space(&wide_space), 0);
        assert_eq!(prompt_in_list(c" ", &wide_space), 0);
    }

    #[test]
    fn completion_keeps_prefix_suffix_bytes_and_only_the_logical_sentinel() {
        let input = CString::new("cmd old tail").unwrap();
        let replacement = CString::new(vec![0xff]).unwrap();
        let mut pr = make_prompt(&input, 7, None);
        pr.buffer.push(cell(b'!'));

        unsafe {
            assert_eq!(prompt_replace_complete(&mut *pr, Some(&replacement)), 1);
            assert_eq!(utf8_tocstr_cstring(&pr.buffer).as_bytes(), b"cmd \xff tail");
            assert_eq!(pr.index, 5);
            assert_eq!(pr.buffer.len(), utf8_strlen(&pr.buffer) + 1);

            for (input, index, replaced, expected, next) in [
                (c"", 0, 1, c"new ", 4),
                (c"cmd old tail", 0, 1, c"new  old tail", 4),
                (c"cmd old tail", 4, 0, c"cmd old tail", 4),
                (c"cmd old tail", 5, 1, c"cmd new  tail", 8),
                (c"cmd old tail", 12, 1, c"cmd old new ", 12),
                (c"   ", 0, 0, c"   ", 0),
                (c"   ", 3, 0, c"   ", 3),
                (c"é漢 old", 1, 1, c"new  old", 4),
            ] {
                let mut pr = make_prompt(input, index, None);
                assert_eq!(prompt_replace_complete(&mut pr, Some(c"new ")), replaced);
                assert_eq!(utf8_tocstr_cstring(&pr.buffer).as_c_str(), expected);
                assert_eq!(pr.index, next);
            }
            // Explicit replacement (mouse completion) does not use the
            // 64-byte scratch word needed by command-name lookup.
            let long = CString::new(vec![b'x'; 64]).unwrap();
            let mut pr = make_prompt(&long, 64, None);
            assert_eq!(prompt_replace_complete(&mut pr, Some(c"new ")), 1);
            assert_eq!(utf8_tocstr_cstring(&pr.buffer).as_c_str(), c"new ");
        }
    }

    #[test]
    fn vi_translation_preserves_mode_switches_and_multibyte_cursor_edits() {
        let mut pr = make_prompt(c"é漢Z", 3, None);
        pr.keys = MODEKEY_VI;
        // Expected text and cursor positions follow the pinned tmux vi
        // insert/command transitions, including backspace as a movement.
        for (key, expected, index, command) in [
            (27, c"é漢Z", 2, true),
            (KEYC_BSPACE, c"é漢Z", 1, true),
            (b'a' as key_code, c"é漢Z", 2, false),
            (KEYC_KP_PLUS, c"é漢+Z", 3, false),
            (27, c"é漢+Z", 2, true),
            (b's' as key_code, c"é漢Z", 2, false),
            (b'!' as key_code, c"é漢!Z", 3, false),
            (27, c"é漢!Z", 2, true),
            (b'S' as key_code, c"", 0, false),
        ] {
            let mut redraw = 0;
            unsafe {
                assert_eq!(prompt_key(&mut *pr, key, &mut redraw), PROMPT_KEY_HANDLED);
                assert_eq!(utf8_tocstr_cstring(&pr.buffer).as_c_str(), expected);
            }
            assert_eq!(redraw, 1);
            assert_eq!(pr.index, index);
            assert_eq!(pr.flags & PROMPT_COMMANDMODE != 0, command);
            assert_eq!(pr.buffer.len(), utf8_strlen(&pr.buffer) + 1);
        }

        // A translated key must preserve an already pending redraw. Only
        // changing vi modes requests a new one, including at an empty input.
        let mut redraw = 7;
        let modified_keypad = KEYC_KP_PLUS | KEYC_CTRL;
        assert_eq!(prompt_keypad_key(modified_keypad), modified_keypad);
        assert_eq!(
            prompt_translate_key(&mut pr, modified_keypad, &mut redraw),
            PromptTranslatedKey::Append(modified_keypad)
        );
        assert_eq!(redraw, 7);
        assert_eq!(
            prompt_translate_key(&mut pr, 27, &mut redraw),
            PromptTranslatedKey::Handled
        );
        assert_eq!((pr.index, redraw), (0, 1));
        redraw = 7;
        assert_eq!(
            prompt_translate_key(&mut pr, 27, &mut redraw),
            PromptTranslatedKey::Handled
        );
        assert_eq!((pr.index, redraw), (0, 7));
    }

    #[test]
    fn transpose_keeps_multibyte_cells_and_the_logical_sentinel() {
        for (index, expected, next) in [(0, c"é漢Z", 0), (1, c"漢éZ", 2), (3, c"éZ漢", 3)] {
            let mut pr = make_prompt(c"é漢Z", index, None);
            let mut redraw = 0;
            unsafe {
                assert_eq!(
                    prompt_key(&mut *pr, KEYC_CTRL | b't' as key_code, &mut redraw),
                    PROMPT_KEY_HANDLED
                );
                assert_eq!(utf8_tocstr_cstring(&pr.buffer).as_c_str(), expected);
            }
            assert_eq!(pr.index, next);
            assert_eq!(pr.buffer.len(), 4);
            assert_eq!(pr.buffer[3].size, 0);
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
                utf8_tocstr_cstring(&pr.buffer).as_bytes(),
                b"a\xce\xbb\xc3\xa9Z"
            );
            assert_eq!(pr.index, 2);
            assert_eq!(pr.buffer.len(), utf8_strlen(&pr.buffer) + 1);

            // Even an empty copied word takes precedence over the clipboard.
            pr.copied = Some(vec![utf8_data::default()].into_boxed_slice());
            assert_eq!(prompt_paste(&mut pr), 1);
            assert_eq!(pr.index, 2);
            assert_eq!(
                utf8_tocstr_cstring(&pr.buffer).as_bytes(),
                b"a\xce\xbb\xc3\xa9Z"
            );
        }
    }

    #[test]
    fn paste_decoding_preserves_utf8_and_stops_at_control_or_invalid_bytes() {
        unsafe {
            assert!(!libc::setlocale(libc::LC_CTYPE, c"C.UTF-8".as_ptr()).is_null());
            for (input, expected, count) in [
                (&b""[..], &b""[..], 0),
                (&b"ab"[..], &b"ab"[..], 2),
                ("Aé漢Z".as_bytes(), "Aé漢Z".as_bytes(), 4),
                ("éZ".as_bytes(), "éZ".as_bytes(), 3),
                (&b"abc\nignored"[..], &b"abc"[..], 3),
                (&b"abc\tignored"[..], &b"abc"[..], 3),
                (&b"ab\0ignored"[..], &b"ab"[..], 2),
                (&b"x\x7fz"[..], &b"x"[..], 1),
                (&b"x\x1bz"[..], &b"x"[..], 1),
                (&b"x\xffz"[..], &b"x"[..], 1),
                (&b"x\xc0\xafz"[..], &b"x"[..], 1),
                (&b"x\xe2(\xa1Z"[..], &b"x"[..], 1),
                (&b"x\xe2\x82"[..], &b"x"[..], 1),
                (&b"x\xf0\x9f"[..], &b"x"[..], 1),
                (&b"x\xed\xa0\x80Z"[..], &b"x"[..], 1),
                (&b"x\xf4\x90\x80\x80z"[..], &b"x"[..], 1),
            ] {
                let cells = prompt_decode_paste(input);
                assert_eq!(
                    utf8_tocstr_cstring(&cells).as_bytes(),
                    expected,
                    "{input:?}"
                );
                assert_eq!(utf8_strlen(&cells), count, "{input:?}");
                assert_eq!(cells.len(), count + 1);
                assert_eq!(cells.last().unwrap().size, 0);
            }
        }
    }
}
