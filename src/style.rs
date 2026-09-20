use crate::src::attributes::{attributes_fromstring, attributes_tostring};
use crate::src::colour::{colour_fromstring, colour_tostring};
use crate::src::compat::strtonum::strtonum;
use crate::src::ffi::libc::{
    free, memcpy, snprintf, strcasecmp, strchr, strcmp, strcspn, strlcpy, strncasecmp, strspn,
};
use crate::src::format::{format_create, format_free, format_single};
use crate::src::grid::grid_default_cell;
use crate::src::hyperlinks::{hyperlinks_get, hyperlinks_init, hyperlinks_put};
use crate::src::log::{fatalx, log_debug};
use crate::src::options::{options_get, options_get_string, options_string_to_style};
pub use crate::src::options::options_table_entry;
use crate::src::utf8::utf8_set;
use crate::src::xmalloc::xsnprintf;
pub use crate::src::shared::arguments::{args};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{options, options_entry};
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
pub use crate::src::shared::format::{FORMAT_NOJOBS};
pub use crate::src::shared::limits::{__INT_MAX__, UINT_MAX};
pub use crate::src::shared::pane::{
    PANE_SCROLLBARS_CHARACTER, PANE_SCROLLBARS_DEFAULT_PADDING, PANE_SCROLLBARS_DEFAULT_WIDTH,
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::options::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

pub const STYLE_WIDTH_DEFAULT: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const STYLE_PAD_DEFAULT: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);

static mut style_default: style = unsafe {
    style {
        gc: grid_cell {
            data: utf8_data {
                data: [
                    ' ' as i32 as u_char,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                have: 0 as u_char,
                size: 1 as u_char,
                width: 1 as u_char,
            },
            attr: 0 as u_short,
            flags: 0 as u_char,
            fg: 8 as ::core::ffi::c_int,
            bg: 8 as ::core::ffi::c_int,
            us: 0 as ::core::ffi::c_int,
            link: 0 as u_int,
        },
        ignore: 0 as ::core::ffi::c_int,
        dim: 0 as ::core::ffi::c_int,
        fill: 8 as ::core::ffi::c_int,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0 as u_int,
        range_string: ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(
            *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
        width: STYLE_WIDTH_DEFAULT,
        width_percentage: 0 as ::core::ffi::c_int,
        pad: STYLE_PAD_DEFAULT,
        default_type: STYLE_DEFAULT_BASE,
        link: 0 as u_int,
    }
};
static mut style_hyperlinks: *mut hyperlinks = ::core::ptr::null::<hyperlinks>() as *mut hyperlinks;
unsafe extern "C" fn style_set_range_string(mut sy: *mut style, mut s: *const ::core::ffi::c_char) {
    strlcpy(
        &raw mut (*sy).range_string as *mut ::core::ffi::c_char,
        s,
        ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn style_parse(
    mut sy: *mut style,
    mut base: *const grid_cell,
    mut in_0: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut saved: style = style {
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
    let delimiters: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b" ,\n\0");
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tmp: [::core::ffi::c_char; 256] = [0; 256];
    let mut found: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: ::core::ffi::c_int = 0;
    let mut end: size_t = 0;
    let mut n: u_int = 0;
    if *in_0 as ::core::ffi::c_int == '\0' as i32 {
        return 0 as ::core::ffi::c_int;
    }
    style_copy(&raw mut saved, sy);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"style_parse\0" as *const u8 as *const ::core::ffi::c_char,
        in_0,
    );
    loop {
        while *in_0 as ::core::ffi::c_int != '\0' as i32
            && !strchr(
                &raw const delimiters as *const ::core::ffi::c_char,
                *in_0 as ::core::ffi::c_int,
            )
            .is_null()
        {
            in_0 = in_0.offset(1);
        }
        if *in_0 as ::core::ffi::c_int == '\0' as i32 {
            current_block = 5832582820025303349;
            break;
        }
        end = strcspn(in_0, &raw const delimiters as *const ::core::ffi::c_char) as size_t;
        if end
            > (::core::mem::size_of::<[::core::ffi::c_char; 256]>() as usize)
                .wrapping_sub(1 as usize)
        {
            current_block = 6605876559004397942;
            break;
        }
        memcpy(
            &raw mut tmp as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            in_0 as *const ::core::ffi::c_void,
            end,
        );
        tmp[end as usize] = '\0' as i32 as ::core::ffi::c_char;
        log_debug(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"style_parse\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut tmp as *mut ::core::ffi::c_char,
        );
        if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"default\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).gc.fg = (*base).fg;
            (*sy).gc.bg = (*base).bg;
            (*sy).gc.us = (*base).us;
            (*sy).gc.attr = (*base).attr;
            (*sy).gc.flags = (*base).flags;
            (*sy).link = 0 as u_int;
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"ignore\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).ignore = 1 as ::core::ffi::c_int;
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"noignore\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).ignore = 0 as ::core::ffi::c_int;
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"push-default\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).default_type = STYLE_DEFAULT_PUSH;
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"pop-default\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).default_type = STYLE_DEFAULT_POP;
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"set-default\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).default_type = STYLE_DEFAULT_SET;
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"nolist\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).list = STYLE_LIST_OFF;
        } else if strncasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"list=\0" as *const u8 as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(5 as ::core::ffi::c_int as isize),
                b"on\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).list = STYLE_LIST_ON;
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(5 as ::core::ffi::c_int as isize),
                b"focus\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).list = STYLE_LIST_FOCUS;
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(5 as ::core::ffi::c_int as isize),
                b"left-marker\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).list = STYLE_LIST_LEFT_MARKER;
            } else {
                if !(strcasecmp(
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(5 as ::core::ffi::c_int as isize),
                    b"right-marker\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int)
                {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).list = STYLE_LIST_RIGHT_MARKER;
            }
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"norange\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).range_type = style_default.range_type;
            (*sy).range_argument = style_default.range_type as u_int;
            strlcpy(
                &raw mut (*sy).range_string as *mut ::core::ffi::c_char,
                &raw mut style_default.range_string as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
            );
        } else if end > 6 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"range=\0" as *const u8 as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            found = strchr(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                '|' as i32,
            );
            if !found.is_null() {
                let fresh0 = found;
                found = found.offset(1);
                *fresh0 = '\0' as i32 as ::core::ffi::c_char;
                if *found as ::core::ffi::c_int == '\0' as i32 {
                    current_block = 6605876559004397942;
                    break;
                }
            }
            if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"left\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if !found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_LEFT;
                (*sy).range_argument = 0 as u_int;
                style_set_range_string(sy, b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"right\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if !found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_RIGHT;
                (*sy).range_argument = 0 as u_int;
                style_set_range_string(sy, b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"control\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                n = strtonum(
                    found,
                    0 as ::core::ffi::c_longlong,
                    9 as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_CONTROL;
                (*sy).range_argument = n;
                style_set_range_string(sy, b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                if *found as ::core::ffi::c_int != '%' as i32
                    || *found.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\0' as i32
                {
                    current_block = 6605876559004397942;
                    break;
                }
                n = strtonum(
                    found.offset(1 as ::core::ffi::c_int as isize),
                    0 as ::core::ffi::c_longlong,
                    UINT_MAX as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_PANE;
                (*sy).range_argument = n;
                style_set_range_string(sy, b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"window\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                n = strtonum(
                    found,
                    0 as ::core::ffi::c_longlong,
                    UINT_MAX as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_WINDOW;
                (*sy).range_argument = n;
                style_set_range_string(sy, b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"session\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                if *found as ::core::ffi::c_int != '$' as i32
                    || *found.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\0' as i32
                {
                    current_block = 6605876559004397942;
                    break;
                }
                n = strtonum(
                    found.offset(1 as ::core::ffi::c_int as isize),
                    0 as ::core::ffi::c_longlong,
                    UINT_MAX as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_SESSION;
                (*sy).range_argument = n;
                style_set_range_string(sy, b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"user\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_USER;
                (*sy).range_argument = 0 as u_int;
                style_set_range_string(sy, found);
            }
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"noalign\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).align = style_default.align;
        } else if end > 6 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"align=\0" as *const u8 as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"left\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).align = STYLE_ALIGN_LEFT;
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"centre\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).align = STYLE_ALIGN_CENTRE;
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"right\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).align = STYLE_ALIGN_RIGHT;
            } else {
                if !(strcasecmp(
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(6 as ::core::ffi::c_int as isize),
                    b"absolute-centre\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int)
                {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).align = STYLE_ALIGN_ABSOLUTE_CENTRE;
            }
        } else if end > 5 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"fill=\0" as *const u8 as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            value = colour_fromstring(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(5 as ::core::ffi::c_int as isize),
            );
            if value == -(1 as ::core::ffi::c_int) {
                current_block = 6605876559004397942;
                break;
            }
            (*sy).fill = value;
        } else if end > 4 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"dim=\0" as *const u8 as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if tmp[end.wrapping_sub(1 as size_t) as usize] as ::core::ffi::c_int == '%' as i32 {
                tmp[end.wrapping_sub(1 as size_t) as usize] = '\0' as i32 as ::core::ffi::c_char;
            }
            n = strtonum(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(4 as ::core::ffi::c_int as isize),
                0 as ::core::ffi::c_longlong,
                100 as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as u_int;
            if !errstr.is_null() {
                current_block = 6605876559004397942;
                break;
            }
            (*sy).dim = n as ::core::ffi::c_int;
        } else if end > 3 as size_t
            && strncasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize),
                b"g=\0" as *const u8 as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            value = colour_fromstring(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize),
            );
            if value == -(1 as ::core::ffi::c_int) {
                current_block = 6605876559004397942;
                break;
            }
            if *in_0 as ::core::ffi::c_int == 'f' as i32
                || *in_0 as ::core::ffi::c_int == 'F' as i32
            {
                if value != 8 as ::core::ffi::c_int {
                    (*sy).gc.fg = value;
                } else {
                    (*sy).gc.fg = (*base).fg;
                }
            } else {
                if !(*in_0 as ::core::ffi::c_int == 'b' as i32
                    || *in_0 as ::core::ffi::c_int == 'B' as i32)
                {
                    current_block = 6605876559004397942;
                    break;
                }
                if value != 8 as ::core::ffi::c_int {
                    (*sy).gc.bg = value;
                } else {
                    (*sy).gc.bg = (*base).bg;
                }
            }
        } else if end > 3 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"us=\0" as *const u8 as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            value = colour_fromstring(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize),
            );
            if value == -(1 as ::core::ffi::c_int) {
                current_block = 6605876559004397942;
                break;
            }
            if value != 8 as ::core::ffi::c_int {
                (*sy).gc.us = value;
            } else {
                (*sy).gc.us = (*base).us;
            }
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"none\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).gc.attr = 0 as u_short;
        } else if end > 2 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"no\0" as *const u8 as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if strcmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(2 as ::core::ffi::c_int as isize),
                b"link\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).link = 0 as u_int;
            } else if strcmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(2 as ::core::ffi::c_int as isize),
                b"attr\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).gc.attr = ((*sy).gc.attr as ::core::ffi::c_int | GRID_ATTR_NOATTR) as u_short;
            } else {
                value = attributes_fromstring(
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(2 as ::core::ffi::c_int as isize),
                );
                if value == -(1 as ::core::ffi::c_int) {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).gc.attr = ((*sy).gc.attr as ::core::ffi::c_int & !value) as u_short;
            }
        } else if end > 6 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"width=\0" as *const u8 as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if end > 7 as size_t
                && tmp[end.wrapping_sub(1 as size_t) as usize] as ::core::ffi::c_int == '%' as i32
            {
                tmp[end.wrapping_sub(1 as size_t) as usize] = '\0' as i32 as ::core::ffi::c_char;
                n = strtonum(
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(6 as ::core::ffi::c_int as isize),
                    0 as ::core::ffi::c_longlong,
                    100 as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).width = n as ::core::ffi::c_int;
                (*sy).width_percentage = 1 as ::core::ffi::c_int;
            } else {
                n = strtonum(
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(6 as ::core::ffi::c_int as isize),
                    0 as ::core::ffi::c_longlong,
                    UINT_MAX as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).width = n as ::core::ffi::c_int;
                (*sy).width_percentage = 0 as ::core::ffi::c_int;
            }
        } else if end > 4 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"pad=\0" as *const u8 as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            n = strtonum(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(4 as ::core::ffi::c_int as isize),
                0 as ::core::ffi::c_longlong,
                UINT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as u_int;
            if !errstr.is_null() {
                current_block = 6605876559004397942;
                break;
            }
            (*sy).pad = n as ::core::ffi::c_int;
        } else if strncasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"link=\0" as *const u8 as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            if tmp[5 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == '\0' as i32 {
                (*sy).link = 0 as u_int;
            } else {
                if style_hyperlinks.is_null() {
                    style_hyperlinks = hyperlinks_init();
                }
                (*sy).link = hyperlinks_put(
                    style_hyperlinks,
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(5 as ::core::ffi::c_int as isize),
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(5 as ::core::ffi::c_int as isize),
                );
            }
        } else {
            value = attributes_fromstring(&raw mut tmp as *mut ::core::ffi::c_char);
            if value == -(1 as ::core::ffi::c_int) {
                current_block = 6605876559004397942;
                break;
            }
            (*sy).gc.attr = ((*sy).gc.attr as ::core::ffi::c_int | value) as u_short;
        }
        in_0 = in_0.offset(end.wrapping_add(strspn(
            in_0.offset(end as isize),
            &raw const delimiters as *const ::core::ffi::c_char,
        ) as size_t) as isize);
        if !(*in_0 as ::core::ffi::c_int != '\0' as i32) {
            current_block = 5832582820025303349;
            break;
        }
    }
    match current_block {
        5832582820025303349 => return 0 as ::core::ffi::c_int,
        _ => {
            style_copy(sy, &raw mut saved);
            return -(1 as ::core::ffi::c_int);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn style_tostring(mut sy: *mut style) -> *const ::core::ffi::c_char {
    let mut gc: *mut grid_cell = &raw mut (*sy).gc;
    let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut comma: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut tmp: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    static mut s: [::core::ffi::c_char; 2048] = [0; 2048];
    let mut b: [::core::ffi::c_char; 21] = [0; 21];
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if (*sy).list as ::core::ffi::c_uint
        != STYLE_LIST_OFF as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*sy).list as ::core::ffi::c_uint
            == STYLE_LIST_ON as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"on\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).list as ::core::ffi::c_uint
            == STYLE_LIST_FOCUS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"focus\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).list as ::core::ffi::c_uint
            == STYLE_LIST_LEFT_MARKER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"left-marker\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).list as ::core::ffi::c_uint
            == STYLE_LIST_RIGHT_MARKER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"right-marker\0" as *const u8 as *const ::core::ffi::c_char;
        }
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%slist=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            tmp,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).range_type as ::core::ffi::c_uint
        != STYLE_RANGE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*sy).range_type as ::core::ffi::c_uint
            == STYLE_RANGE_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"left\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).range_type as ::core::ffi::c_uint
            == STYLE_RANGE_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"right\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).range_type as ::core::ffi::c_uint
            == STYLE_RANGE_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            snprintf(
                &raw mut b as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 21]>() as size_t,
                b"pane|%%%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*sy).range_argument,
            );
            tmp = &raw mut b as *mut ::core::ffi::c_char;
        } else if (*sy).range_type as ::core::ffi::c_uint
            == STYLE_RANGE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            snprintf(
                &raw mut b as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 21]>() as size_t,
                b"window|%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*sy).range_argument,
            );
            tmp = &raw mut b as *mut ::core::ffi::c_char;
        } else if (*sy).range_type as ::core::ffi::c_uint
            == STYLE_RANGE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            snprintf(
                &raw mut b as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 21]>() as size_t,
                b"session|$%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*sy).range_argument,
            );
            tmp = &raw mut b as *mut ::core::ffi::c_char;
        } else if (*sy).range_type as ::core::ffi::c_uint
            == STYLE_RANGE_USER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            snprintf(
                &raw mut b as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 21]>() as size_t,
                b"user|%s\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut (*sy).range_string as *mut ::core::ffi::c_char,
            );
            tmp = &raw mut b as *mut ::core::ffi::c_char;
        }
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%srange=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            tmp,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).align as ::core::ffi::c_uint
        != STYLE_ALIGN_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*sy).align as ::core::ffi::c_uint
            == STYLE_ALIGN_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"left\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).align as ::core::ffi::c_uint
            == STYLE_ALIGN_CENTRE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"centre\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).align as ::core::ffi::c_uint
            == STYLE_ALIGN_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"right\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).align as ::core::ffi::c_uint
            == STYLE_ALIGN_ABSOLUTE_CENTRE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"absolute-centre\0" as *const u8 as *const ::core::ffi::c_char;
        }
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%salign=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            tmp,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).default_type as ::core::ffi::c_uint
        != STYLE_DEFAULT_BASE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*sy).default_type as ::core::ffi::c_uint
            == STYLE_DEFAULT_PUSH as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"push-default\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).default_type as ::core::ffi::c_uint
            == STYLE_DEFAULT_POP as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"pop-default\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).default_type as ::core::ffi::c_uint
            == STYLE_DEFAULT_SET as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"set-default\0" as *const u8 as *const ::core::ffi::c_char;
        }
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            tmp,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).fill != 8 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%sfill=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            colour_tostring((*sy).fill),
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).dim != 0 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%sdim=%d%%\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            (*sy).dim,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*gc).fg != 8 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%sfg=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            colour_tostring((*gc).fg),
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*gc).bg != 8 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%sbg=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            colour_tostring((*gc).bg),
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*gc).us != 8 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%sus=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            colour_tostring((*gc).us),
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*gc).attr as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            attributes_tostring((*gc).attr as ::core::ffi::c_int),
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).width >= 0 as ::core::ffi::c_int {
        if (*sy).width_percentage != 0 {
            off += xsnprintf(
                (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
                (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                    .wrapping_sub(off as size_t),
                b"%swidth=%u%%\0" as *const u8 as *const ::core::ffi::c_char,
                comma,
                (*sy).width,
            );
        } else {
            off += xsnprintf(
                (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
                (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                    .wrapping_sub(off as size_t),
                b"%swidth=%u\0" as *const u8 as *const ::core::ffi::c_char,
                comma,
                (*sy).width,
            );
        }
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).pad >= 0 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%spad=%u\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            (*sy).pad,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    uri = style_link(sy);
    if !uri.is_null() {
        xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%slink=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            uri,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int == '\0' as i32 {
        return b"default\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return &raw mut s as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn style_link(mut sy: *mut style) -> *const ::core::ffi::c_char {
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*sy).link == 0 as u_int || style_hyperlinks.is_null() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if hyperlinks_get(
        style_hyperlinks,
        (*sy).link,
        &raw mut uri,
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
    ) == 0
    {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return uri;
}
#[no_mangle]
pub unsafe extern "C" fn style_add(
    mut gc: *mut grid_cell,
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut ft: *mut format_tree,
) -> *mut style {
    let mut sy: *mut style = ::core::ptr::null_mut::<style>();
    let mut ft0: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    if ft.is_null() {
        ft0 = format_create(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<cmdq_item>(),
            0 as ::core::ffi::c_int,
            FORMAT_NOJOBS,
        );
        ft = ft0;
    }
    sy = options_string_to_style(oo, name, ft);
    if sy.is_null() {
        sy = &raw mut style_default;
    }
    if (*sy).gc.fg != 8 as ::core::ffi::c_int {
        (*gc).fg = (*sy).gc.fg;
    }
    if (*sy).gc.bg != 8 as ::core::ffi::c_int {
        (*gc).bg = (*sy).gc.bg;
    }
    if (*sy).gc.us != 8 as ::core::ffi::c_int {
        (*gc).us = (*sy).gc.us;
    }
    (*gc).attr =
        ((*gc).attr as ::core::ffi::c_int | (*sy).gc.attr as ::core::ffi::c_int) as u_short;
    if !ft0.is_null() {
        format_free(ft0);
    }
    return sy;
}
#[no_mangle]
pub unsafe extern "C" fn style_apply(
    mut gc: *mut grid_cell,
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut ft: *mut format_tree,
) {
    memcpy(
        gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_add(gc, oo, name, ft);
}
#[no_mangle]
pub unsafe extern "C" fn style_parse_colour(
    mut sy: *mut style,
    mut base: *const grid_cell,
    mut s: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    style_set(sy, base);
    if *s as ::core::ffi::c_int == '\0' as i32 {
        (*sy).gc.fg = -(1 as ::core::ffi::c_int);
        return 0 as ::core::ffi::c_int;
    }
    c = colour_fromstring(s);
    if c == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    if c == 8 as ::core::ffi::c_int {
        (*sy).gc.fg = (*base).fg;
    } else {
        (*sy).gc.fg = c;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn style_set(mut sy: *mut style, mut gc: *const grid_cell) {
    memcpy(
        sy as *mut ::core::ffi::c_void,
        &raw mut style_default as *const ::core::ffi::c_void,
        ::core::mem::size_of::<style>() as size_t,
    );
    memcpy(
        &raw mut (*sy).gc as *mut ::core::ffi::c_void,
        gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn style_copy(mut dst: *mut style, mut src: *mut style) {
    memcpy(
        dst as *mut ::core::ffi::c_void,
        src as *const ::core::ffi::c_void,
        ::core::mem::size_of::<style>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn style_set_scrollbar_style_from_option(
    mut sb_style: *mut style,
    mut oo: *mut options,
) {
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut style: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    style_set(sb_style, &raw const grid_default_cell);
    o = options_get(
        oo,
        b"pane-scrollbars-style\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if o.is_null() {
        fatalx(b"missing pane-scrollbars-style\0" as *const u8 as *const ::core::ffi::c_char);
    }
    oe = options_table_entry(o);
    style = format_single(
        ::core::ptr::null_mut::<cmdq_item>(),
        (*oe).default_str,
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    if style_parse(sb_style, &raw const grid_default_cell, style) != 0 as ::core::ffi::c_int {
        fatalx(b"bad pane-scrollbars-style default\0" as *const u8 as *const ::core::ffi::c_char);
    }
    s = options_get_string(
        oo,
        b"pane-scrollbars-style\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !s.is_null() {
        expanded = format_single(
            ::core::ptr::null_mut::<cmdq_item>(),
            s,
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
        if style_parse(sb_style, &raw const grid_default_cell, expanded) != 0 as ::core::ffi::c_int
        {
            style_parse(sb_style, &raw const grid_default_cell, style);
        }
        free(expanded as *mut ::core::ffi::c_void);
    }
    free(style as *mut ::core::ffi::c_void);
    if (*sb_style).width < 1 as ::core::ffi::c_int {
        (*sb_style).width = PANE_SCROLLBARS_DEFAULT_WIDTH;
    }
    if (*sb_style).pad < 0 as ::core::ffi::c_int {
        (*sb_style).pad = PANE_SCROLLBARS_DEFAULT_PADDING;
    }
    utf8_set(
        &raw mut (*sb_style).gc.data,
        PANE_SCROLLBARS_CHARACTER as u_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn style_ranges_init(mut srs: *mut style_ranges) {
    (*srs).tqh_first = ::core::ptr::null_mut::<style_range>();
    (*srs).tqh_last = &raw mut (*srs).tqh_first;
}
#[no_mangle]
pub unsafe extern "C" fn style_ranges_free(mut srs: *mut style_ranges) {
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    let mut sr1: *mut style_range = ::core::ptr::null_mut::<style_range>();
    sr = (*srs).tqh_first;
    while !sr.is_null() && {
        sr1 = (*sr).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*sr).entry.tqe_next.is_null() {
            (*(*sr).entry.tqe_next).entry.tqe_prev = (*sr).entry.tqe_prev;
        } else {
            (*srs).tqh_last = (*sr).entry.tqe_prev;
        }
        *(*sr).entry.tqe_prev = (*sr).entry.tqe_next;
        free(sr as *mut ::core::ffi::c_void);
        sr = sr1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn style_ranges_get_range(
    mut srs: *mut style_ranges,
    mut x: u_int,
) -> *mut style_range {
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    if srs.is_null() {
        return ::core::ptr::null_mut::<style_range>();
    }
    sr = (*srs).tqh_first;
    while !sr.is_null() {
        if x >= (*sr).start && x < (*sr).end {
            return sr;
        }
        sr = (*sr).entry.tqe_next;
    }
    return ::core::ptr::null_mut::<style_range>();
}
