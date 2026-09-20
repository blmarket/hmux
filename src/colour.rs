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
pub use crate::src::shared::options::{
    options, options_array, options_array_item, options_entry, options_value,
};
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
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tty::{TERM_256COLOURS, TERM_RGBCOLOURS, TTY_OPENED};
pub use crate::src::shared::colour::{
    COLOUR_FLAG_256, COLOUR_FLAG_RGB, COLOUR_FLAG_THEME, COLOUR_THEME_COUNT,
};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
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
extern "C" {

    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn round(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    fn sscanf(
        __s: *const ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrndup(_: *const ::core::ffi::c_char, _: size_t) -> *mut ::core::ffi::c_char;
    fn xsnprintf(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn options_get(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_array_getv(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        ...
    ) -> *mut options_value;
    fn options_array_first(_: *mut options_entry) -> *mut options_array_item;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn free(__ptr: *mut ::core::ffi::c_void);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_13 {
    pub offset: u_int,
    pub data: C2RustUnnamed_14,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_14 {
    pub attr: u_char,
    pub fg: u_char,
    pub bg: u_char,
    pub data: u_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub name: *const ::core::ffi::c_char,
    pub dark_option: *const ::core::ffi::c_char,
    pub light_option: *const ::core::ffi::c_char,
    pub terminal_colour: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub name: *const ::core::ffi::c_char,
    pub c: ::core::ffi::c_int,
}
static mut colour_theme_table: [C2RustUnnamed_36; 10] = [
    C2RustUnnamed_36 {
        name: b"themeblack\0" as *const u8 as *const ::core::ffi::c_char,
        dark_option: b"dark-theme-black\0" as *const u8 as *const ::core::ffi::c_char,
        light_option: b"light-theme-black\0" as *const u8 as *const ::core::ffi::c_char,
        terminal_colour: 0 as ::core::ffi::c_int,
    },
    C2RustUnnamed_36 {
        name: b"themewhite\0" as *const u8 as *const ::core::ffi::c_char,
        dark_option: b"dark-theme-white\0" as *const u8 as *const ::core::ffi::c_char,
        light_option: b"light-theme-white\0" as *const u8 as *const ::core::ffi::c_char,
        terminal_colour: 7 as ::core::ffi::c_int,
    },
    C2RustUnnamed_36 {
        name: b"themelightgrey\0" as *const u8 as *const ::core::ffi::c_char,
        dark_option: b"dark-theme-light-grey\0" as *const u8 as *const ::core::ffi::c_char,
        light_option: b"light-theme-light-grey\0" as *const u8 as *const ::core::ffi::c_char,
        terminal_colour: 7 as ::core::ffi::c_int,
    },
    C2RustUnnamed_36 {
        name: b"themedarkgrey\0" as *const u8 as *const ::core::ffi::c_char,
        dark_option: b"dark-theme-dark-grey\0" as *const u8 as *const ::core::ffi::c_char,
        light_option: b"light-theme-dark-grey\0" as *const u8 as *const ::core::ffi::c_char,
        terminal_colour: 0 as ::core::ffi::c_int,
    },
    C2RustUnnamed_36 {
        name: b"themegreen\0" as *const u8 as *const ::core::ffi::c_char,
        dark_option: b"dark-theme-green\0" as *const u8 as *const ::core::ffi::c_char,
        light_option: b"light-theme-green\0" as *const u8 as *const ::core::ffi::c_char,
        terminal_colour: 2 as ::core::ffi::c_int,
    },
    C2RustUnnamed_36 {
        name: b"themeyellow\0" as *const u8 as *const ::core::ffi::c_char,
        dark_option: b"dark-theme-yellow\0" as *const u8 as *const ::core::ffi::c_char,
        light_option: b"light-theme-yellow\0" as *const u8 as *const ::core::ffi::c_char,
        terminal_colour: 3 as ::core::ffi::c_int,
    },
    C2RustUnnamed_36 {
        name: b"themered\0" as *const u8 as *const ::core::ffi::c_char,
        dark_option: b"dark-theme-red\0" as *const u8 as *const ::core::ffi::c_char,
        light_option: b"light-theme-red\0" as *const u8 as *const ::core::ffi::c_char,
        terminal_colour: 1 as ::core::ffi::c_int,
    },
    C2RustUnnamed_36 {
        name: b"themeblue\0" as *const u8 as *const ::core::ffi::c_char,
        dark_option: b"dark-theme-blue\0" as *const u8 as *const ::core::ffi::c_char,
        light_option: b"light-theme-blue\0" as *const u8 as *const ::core::ffi::c_char,
        terminal_colour: 4 as ::core::ffi::c_int,
    },
    C2RustUnnamed_36 {
        name: b"themecyan\0" as *const u8 as *const ::core::ffi::c_char,
        dark_option: b"dark-theme-cyan\0" as *const u8 as *const ::core::ffi::c_char,
        light_option: b"light-theme-cyan\0" as *const u8 as *const ::core::ffi::c_char,
        terminal_colour: 6 as ::core::ffi::c_int,
    },
    C2RustUnnamed_36 {
        name: b"thememagenta\0" as *const u8 as *const ::core::ffi::c_char,
        dark_option: b"dark-theme-magenta\0" as *const u8 as *const ::core::ffi::c_char,
        light_option: b"light-theme-magenta\0" as *const u8 as *const ::core::ffi::c_char,
        terminal_colour: 5 as ::core::ffi::c_int,
    },
];
#[no_mangle]
pub unsafe extern "C" fn colour_theme_option(
    mut n: u_int,
    mut theme: client_theme,
) -> *const ::core::ffi::c_char {
    if n as usize
        >= (::core::mem::size_of::<[C2RustUnnamed_36; 10]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_36>() as usize)
    {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if theme as ::core::ffi::c_uint == THEME_LIGHT as ::core::ffi::c_int as ::core::ffi::c_uint {
        return colour_theme_table[n as usize].light_option;
    }
    return colour_theme_table[n as usize].dark_option;
}
#[no_mangle]
pub unsafe extern "C" fn colour_theme_terminal_colour(mut n: u_int) -> ::core::ffi::c_int {
    if n as usize
        >= (::core::mem::size_of::<[C2RustUnnamed_36; 10]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_36>() as usize)
    {
        return 8 as ::core::ffi::c_int;
    }
    return colour_theme_table[n as usize].terminal_colour;
}
unsafe extern "C" fn colour_dist_sq(
    mut R: ::core::ffi::c_int,
    mut G: ::core::ffi::c_int,
    mut B: ::core::ffi::c_int,
    mut r: ::core::ffi::c_int,
    mut g: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return (R - r) * (R - r) + (G - g) * (G - g) + (B - b) * (B - b);
}
unsafe extern "C" fn colour_to_6cube(mut v: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if v < 48 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if v < 114 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    return (v - 35 as ::core::ffi::c_int) / 40 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn colour_find_rgb(
    mut r: u_char,
    mut g: u_char,
    mut b: u_char,
) -> ::core::ffi::c_int {
    static mut q2c: [::core::ffi::c_int; 6] = [
        0 as ::core::ffi::c_int,
        0x5f as ::core::ffi::c_int,
        0x87 as ::core::ffi::c_int,
        0xaf as ::core::ffi::c_int,
        0xd7 as ::core::ffi::c_int,
        0xff as ::core::ffi::c_int,
    ];
    let mut qr: ::core::ffi::c_int = 0;
    let mut qg: ::core::ffi::c_int = 0;
    let mut qb: ::core::ffi::c_int = 0;
    let mut cr: ::core::ffi::c_int = 0;
    let mut cg: ::core::ffi::c_int = 0;
    let mut cb: ::core::ffi::c_int = 0;
    let mut d: ::core::ffi::c_int = 0;
    let mut idx: ::core::ffi::c_int = 0;
    let mut grey_avg: ::core::ffi::c_int = 0;
    let mut grey_idx: ::core::ffi::c_int = 0;
    let mut grey: ::core::ffi::c_int = 0;
    qr = colour_to_6cube(r as ::core::ffi::c_int);
    cr = q2c[qr as usize];
    qg = colour_to_6cube(g as ::core::ffi::c_int);
    cg = q2c[qg as usize];
    qb = colour_to_6cube(b as ::core::ffi::c_int);
    cb = q2c[qb as usize];
    if cr == r as ::core::ffi::c_int
        && cg == g as ::core::ffi::c_int
        && cb == b as ::core::ffi::c_int
    {
        return 16 as ::core::ffi::c_int
            + 36 as ::core::ffi::c_int * qr
            + 6 as ::core::ffi::c_int * qg
            + qb
            | COLOUR_FLAG_256;
    }
    grey_avg = (r as ::core::ffi::c_int + g as ::core::ffi::c_int + b as ::core::ffi::c_int)
        / 3 as ::core::ffi::c_int;
    if grey_avg > 238 as ::core::ffi::c_int {
        grey_idx = 23 as ::core::ffi::c_int;
    } else {
        grey_idx = (grey_avg - 3 as ::core::ffi::c_int) / 10 as ::core::ffi::c_int;
    }
    grey = 8 as ::core::ffi::c_int + 10 as ::core::ffi::c_int * grey_idx;
    d = colour_dist_sq(
        cr,
        cg,
        cb,
        r as ::core::ffi::c_int,
        g as ::core::ffi::c_int,
        b as ::core::ffi::c_int,
    );
    if colour_dist_sq(
        grey,
        grey,
        grey,
        r as ::core::ffi::c_int,
        g as ::core::ffi::c_int,
        b as ::core::ffi::c_int,
    ) < d
    {
        idx = 232 as ::core::ffi::c_int + grey_idx;
    } else {
        idx = 16 as ::core::ffi::c_int
            + 36 as ::core::ffi::c_int * qr
            + 6 as ::core::ffi::c_int * qg
            + qb;
    }
    return idx | COLOUR_FLAG_256;
}
#[no_mangle]
pub unsafe extern "C" fn colour_join_rgb(
    mut r: u_char,
    mut g: u_char,
    mut b: u_char,
) -> ::core::ffi::c_int {
    return (r as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
        | (g as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
        | b as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
        | COLOUR_FLAG_RGB;
}
#[no_mangle]
pub unsafe extern "C" fn colour_split_rgb(
    mut c: ::core::ffi::c_int,
    mut r: *mut u_char,
    mut g: *mut u_char,
    mut b: *mut u_char,
) {
    *r = (c >> 16 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as u_char;
    *g = (c >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as u_char;
    *b = (c & 0xff as ::core::ffi::c_int) as u_char;
}
#[no_mangle]
pub unsafe extern "C" fn colour_force_rgb(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c & COLOUR_FLAG_RGB != 0 {
        return c;
    }
    if c & COLOUR_FLAG_256 != 0 {
        return colour_256toRGB(c);
    }
    if c >= 0 as ::core::ffi::c_int && c <= 7 as ::core::ffi::c_int {
        return colour_256toRGB(c);
    }
    if c >= 90 as ::core::ffi::c_int && c <= 97 as ::core::ffi::c_int {
        return colour_256toRGB(8 as ::core::ffi::c_int + c - 90 as ::core::ffi::c_int);
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn colour_dim(
    mut c: ::core::ffi::c_int,
    mut dim: u_int,
) -> ::core::ffi::c_int {
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    if dim == 0 as u_int
        || (c == 8 as ::core::ffi::c_int || c == 9 as ::core::ffi::c_int)
        || c & COLOUR_FLAG_THEME != 0
    {
        return c;
    }
    if dim >= 100 as u_int {
        return colour_join_rgb(0 as u_char, 0 as u_char, 0 as u_char);
    }
    c = colour_force_rgb(c);
    if c == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    colour_split_rgb(c, &raw mut r, &raw mut g, &raw mut b);
    r = (r as u_int)
        .wrapping_mul((100 as u_int).wrapping_sub(dim))
        .wrapping_div(100 as u_int) as u_char;
    g = (g as u_int)
        .wrapping_mul((100 as u_int).wrapping_sub(dim))
        .wrapping_div(100 as u_int) as u_char;
    b = (b as u_int)
        .wrapping_mul((100 as u_int).wrapping_sub(dim))
        .wrapping_div(100 as u_int) as u_char;
    return colour_join_rgb(r, g, b);
}
#[no_mangle]
pub unsafe extern "C" fn colour_tostring(mut c: ::core::ffi::c_int) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 32] = [0; 32];
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    if c == -(1 as ::core::ffi::c_int) {
        return b"none\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if c & COLOUR_FLAG_THEME != 0 {
        c &= 0xff as ::core::ffi::c_int;
        if c >= 0 as ::core::ffi::c_int
            && (c as u_int as usize)
                < (::core::mem::size_of::<[C2RustUnnamed_36; 10]>() as usize)
                    .wrapping_div(::core::mem::size_of::<C2RustUnnamed_36>() as usize)
        {
            return colour_theme_table[c as usize].name;
        }
        return b"invalid\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if c & COLOUR_FLAG_RGB != 0 {
        colour_split_rgb(c, &raw mut r, &raw mut g, &raw mut b);
        xsnprintf(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"#%02x%02x%02x\0" as *const u8 as *const ::core::ffi::c_char,
            r as ::core::ffi::c_int,
            g as ::core::ffi::c_int,
            b as ::core::ffi::c_int,
        );
        return &raw mut s as *mut ::core::ffi::c_char;
    }
    if c & COLOUR_FLAG_256 != 0 {
        xsnprintf(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"colour%u\0" as *const u8 as *const ::core::ffi::c_char,
            c & 0xff as ::core::ffi::c_int,
        );
        return &raw mut s as *mut ::core::ffi::c_char;
    }
    match c {
        0 => return b"black\0" as *const u8 as *const ::core::ffi::c_char,
        1 => return b"red\0" as *const u8 as *const ::core::ffi::c_char,
        2 => return b"green\0" as *const u8 as *const ::core::ffi::c_char,
        3 => return b"yellow\0" as *const u8 as *const ::core::ffi::c_char,
        4 => return b"blue\0" as *const u8 as *const ::core::ffi::c_char,
        5 => return b"magenta\0" as *const u8 as *const ::core::ffi::c_char,
        6 => return b"cyan\0" as *const u8 as *const ::core::ffi::c_char,
        7 => return b"white\0" as *const u8 as *const ::core::ffi::c_char,
        8 => return b"default\0" as *const u8 as *const ::core::ffi::c_char,
        9 => return b"terminal\0" as *const u8 as *const ::core::ffi::c_char,
        90 => return b"brightblack\0" as *const u8 as *const ::core::ffi::c_char,
        91 => return b"brightred\0" as *const u8 as *const ::core::ffi::c_char,
        92 => return b"brightgreen\0" as *const u8 as *const ::core::ffi::c_char,
        93 => return b"brightyellow\0" as *const u8 as *const ::core::ffi::c_char,
        94 => return b"brightblue\0" as *const u8 as *const ::core::ffi::c_char,
        95 => return b"brightmagenta\0" as *const u8 as *const ::core::ffi::c_char,
        96 => return b"brightcyan\0" as *const u8 as *const ::core::ffi::c_char,
        97 => return b"brightwhite\0" as *const u8 as *const ::core::ffi::c_char,
        _ => {}
    }
    return b"invalid\0" as *const u8 as *const ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn colour_toescape(
    mut c: *mut client,
    mut colour: ::core::ffi::c_int,
    mut bg: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 32] = [0; 32];
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = TERM_256COLOURS | TERM_RGBCOLOURS;
    let mut o: u_int = (if bg != 0 {
        40 as ::core::ffi::c_int
    } else {
        30 as ::core::ffi::c_int
    }) as u_int;
    if !c.is_null() && (*c).tty.flags & TTY_OPENED != 0 && !(*c).tty.term.is_null() {
        flags = (*(*c).tty.term).flags;
    }
    if colour & COLOUR_FLAG_THEME != 0 {
        n = colour & 0xff as ::core::ffi::c_int;
        if !c.is_null() && (n as u_int) < COLOUR_THEME_COUNT as u_int {
            colour = (*c).theme_colours[n as usize];
        } else {
            colour = colour_theme_terminal_colour(n as u_int);
        }
    }
    if colour == 8 as ::core::ffi::c_int || colour == 9 as ::core::ffi::c_int {
        xsnprintf(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"\x1B[%dm\0" as *const u8 as *const ::core::ffi::c_char,
            o.wrapping_add(9 as u_int),
        );
        return &raw mut s as *mut ::core::ffi::c_char;
    }
    if !flags & TERM_RGBCOLOURS & (colour & COLOUR_FLAG_RGB) != 0 {
        colour_split_rgb(colour, &raw mut r, &raw mut g, &raw mut b);
        colour = colour_find_rgb(r, g, b);
    }
    if !flags & TERM_256COLOURS & (colour & COLOUR_FLAG_256) != 0 {
        colour = colour_256to16(colour);
    }
    if colour & COLOUR_FLAG_RGB != 0 {
        colour_split_rgb(colour, &raw mut r, &raw mut g, &raw mut b);
        xsnprintf(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"\x1B[%d;2;%u;%u;%um\0" as *const u8 as *const ::core::ffi::c_char,
            o.wrapping_add(8 as u_int),
            r as ::core::ffi::c_int,
            g as ::core::ffi::c_int,
            b as ::core::ffi::c_int,
        );
        return &raw mut s as *mut ::core::ffi::c_char;
    }
    if colour & COLOUR_FLAG_256 != 0 {
        xsnprintf(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"\x1B[%d;5;%um\0" as *const u8 as *const ::core::ffi::c_char,
            o.wrapping_add(8 as u_int),
            colour & 0xff as ::core::ffi::c_int,
        );
        return &raw mut s as *mut ::core::ffi::c_char;
    }
    if colour >= 0 as ::core::ffi::c_int && colour <= 7 as ::core::ffi::c_int {
        xsnprintf(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"\x1B[%dm\0" as *const u8 as *const ::core::ffi::c_char,
            (colour as u_int).wrapping_add(o),
        );
        return &raw mut s as *mut ::core::ffi::c_char;
    }
    if colour >= 90 as ::core::ffi::c_int && colour <= 97 as ::core::ffi::c_int {
        xsnprintf(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"\x1B[%dm\0" as *const u8 as *const ::core::ffi::c_char,
            (colour as u_int).wrapping_add(o).wrapping_sub(30 as u_int),
        );
        return &raw mut s as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn colour_totheme(mut c: ::core::ffi::c_int) -> client_theme {
    let mut r: ::core::ffi::c_int = 0;
    let mut g: ::core::ffi::c_int = 0;
    let mut b: ::core::ffi::c_int = 0;
    let mut brightness: ::core::ffi::c_int = 0;
    if c == -(1 as ::core::ffi::c_int) {
        return THEME_UNKNOWN;
    }
    if c & COLOUR_FLAG_RGB != 0 {
        r = c >> 16 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int;
        g = c >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int;
        b = c >> 0 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int;
        brightness = r + g + b;
        if brightness > 382 as ::core::ffi::c_int {
            return THEME_LIGHT;
        }
        return THEME_DARK;
    }
    if c & COLOUR_FLAG_256 != 0 {
        return colour_totheme(colour_256toRGB(c));
    }
    match c {
        0 | 90 => return THEME_DARK,
        7 | 97 => return THEME_LIGHT,
        _ => {
            if c >= 0 as ::core::ffi::c_int && c <= 7 as ::core::ffi::c_int {
                return colour_totheme(colour_256toRGB(c));
            }
            if c >= 90 as ::core::ffi::c_int && c <= 97 as ::core::ffi::c_int {
                return colour_totheme(colour_256toRGB(
                    8 as ::core::ffi::c_int + c - 90 as ::core::ffi::c_int,
                ));
            }
        }
    }
    return THEME_UNKNOWN;
}
#[no_mangle]
pub unsafe extern "C" fn colour_fromstring(
    mut s: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: ::core::ffi::c_int = 0;
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut i: u_int = 0;
    if *s as ::core::ffi::c_int == '#' as i32 && strlen(s) == 7 as size_t {
        cp = s.offset(1 as ::core::ffi::c_int as isize);
        while *(*__ctype_b_loc()).offset(*cp as u_char as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        {
            cp = cp.offset(1);
        }
        if *cp as ::core::ffi::c_int != '\0' as i32 {
            return -(1 as ::core::ffi::c_int);
        }
        n = sscanf(
            s.offset(1 as ::core::ffi::c_int as isize),
            b"%2hhx%2hhx%2hhx\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut r,
            &raw mut g,
            &raw mut b,
        );
        if n != 3 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        return colour_join_rgb(r, g, b);
    }
    if strncasecmp(
        s,
        b"colour\0" as *const u8 as *const ::core::ffi::c_char,
        (::core::mem::size_of::<[::core::ffi::c_char; 7]>() as size_t).wrapping_sub(1 as size_t),
    ) == 0 as ::core::ffi::c_int
    {
        n = strtonum(
            s.offset(::core::mem::size_of::<[::core::ffi::c_char; 7]>() as usize as isize)
                .offset(-(1 as ::core::ffi::c_int as isize)),
            0 as ::core::ffi::c_longlong,
            255 as ::core::ffi::c_longlong,
            &raw mut errstr,
        ) as ::core::ffi::c_int;
        if !errstr.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return n | COLOUR_FLAG_256;
    }
    if strncasecmp(
        s,
        b"color\0" as *const u8 as *const ::core::ffi::c_char,
        (::core::mem::size_of::<[::core::ffi::c_char; 6]>() as size_t).wrapping_sub(1 as size_t),
    ) == 0 as ::core::ffi::c_int
    {
        n = strtonum(
            s.offset(::core::mem::size_of::<[::core::ffi::c_char; 6]>() as usize as isize)
                .offset(-(1 as ::core::ffi::c_int as isize)),
            0 as ::core::ffi::c_longlong,
            255 as ::core::ffi::c_longlong,
            &raw mut errstr,
        ) as ::core::ffi::c_int;
        if !errstr.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return n | COLOUR_FLAG_256;
    }
    if strcasecmp(s, b"default\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        return 8 as ::core::ffi::c_int;
    }
    if strcasecmp(s, b"terminal\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        return 9 as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_36; 10]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_36>() as usize)
    {
        if strcasecmp(s, colour_theme_table[i as usize].name) == 0 as ::core::ffi::c_int {
            return (i | COLOUR_FLAG_THEME as u_int) as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    if strcasecmp(s, b"black\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(s, b"0\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if strcasecmp(s, b"red\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        || strcmp(s, b"1\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    if strcasecmp(s, b"green\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(s, b"2\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 2 as ::core::ffi::c_int;
    }
    if strcasecmp(s, b"yellow\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(s, b"3\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 3 as ::core::ffi::c_int;
    }
    if strcasecmp(s, b"blue\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(s, b"4\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 4 as ::core::ffi::c_int;
    }
    if strcasecmp(s, b"magenta\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(s, b"5\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 5 as ::core::ffi::c_int;
    }
    if strcasecmp(s, b"cyan\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(s, b"6\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 6 as ::core::ffi::c_int;
    }
    if strcasecmp(s, b"white\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(s, b"7\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 7 as ::core::ffi::c_int;
    }
    if strcasecmp(
        s,
        b"brightblack\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(s, b"90\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 90 as ::core::ffi::c_int;
    }
    if strcasecmp(s, b"brightred\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(s, b"91\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 91 as ::core::ffi::c_int;
    }
    if strcasecmp(
        s,
        b"brightgreen\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(s, b"92\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 92 as ::core::ffi::c_int;
    }
    if strcasecmp(
        s,
        b"brightyellow\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(s, b"93\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 93 as ::core::ffi::c_int;
    }
    if strcasecmp(
        s,
        b"brightblue\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(s, b"94\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 94 as ::core::ffi::c_int;
    }
    if strcasecmp(
        s,
        b"brightmagenta\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(s, b"95\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 95 as ::core::ffi::c_int;
    }
    if strcasecmp(
        s,
        b"brightcyan\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(s, b"96\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 96 as ::core::ffi::c_int;
    }
    if strcasecmp(
        s,
        b"brightwhite\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(s, b"97\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return 97 as ::core::ffi::c_int;
    }
    return colour_byname(s);
}
#[no_mangle]
pub unsafe extern "C" fn colour_256toRGB(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    static mut table: [::core::ffi::c_int; 256] = [
        0 as ::core::ffi::c_int,
        0x800000 as ::core::ffi::c_int,
        0x8000 as ::core::ffi::c_int,
        0x808000 as ::core::ffi::c_int,
        0x80 as ::core::ffi::c_int,
        0x800080 as ::core::ffi::c_int,
        0x8080 as ::core::ffi::c_int,
        0xc0c0c0 as ::core::ffi::c_int,
        0x808080 as ::core::ffi::c_int,
        0xff0000 as ::core::ffi::c_int,
        0xff00 as ::core::ffi::c_int,
        0xffff00 as ::core::ffi::c_int,
        0xff as ::core::ffi::c_int,
        0xff00ff as ::core::ffi::c_int,
        0xffff as ::core::ffi::c_int,
        0xffffff as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0x5f as ::core::ffi::c_int,
        0x87 as ::core::ffi::c_int,
        0xaf as ::core::ffi::c_int,
        0xd7 as ::core::ffi::c_int,
        0xff as ::core::ffi::c_int,
        0x5f00 as ::core::ffi::c_int,
        0x5f5f as ::core::ffi::c_int,
        0x5f87 as ::core::ffi::c_int,
        0x5faf as ::core::ffi::c_int,
        0x5fd7 as ::core::ffi::c_int,
        0x5fff as ::core::ffi::c_int,
        0x8700 as ::core::ffi::c_int,
        0x875f as ::core::ffi::c_int,
        0x8787 as ::core::ffi::c_int,
        0x87af as ::core::ffi::c_int,
        0x87d7 as ::core::ffi::c_int,
        0x87ff as ::core::ffi::c_int,
        0xaf00 as ::core::ffi::c_int,
        0xaf5f as ::core::ffi::c_int,
        0xaf87 as ::core::ffi::c_int,
        0xafaf as ::core::ffi::c_int,
        0xafd7 as ::core::ffi::c_int,
        0xafff as ::core::ffi::c_int,
        0xd700 as ::core::ffi::c_int,
        0xd75f as ::core::ffi::c_int,
        0xd787 as ::core::ffi::c_int,
        0xd7af as ::core::ffi::c_int,
        0xd7d7 as ::core::ffi::c_int,
        0xd7ff as ::core::ffi::c_int,
        0xff00 as ::core::ffi::c_int,
        0xff5f as ::core::ffi::c_int,
        0xff87 as ::core::ffi::c_int,
        0xffaf as ::core::ffi::c_int,
        0xffd7 as ::core::ffi::c_int,
        0xffff as ::core::ffi::c_int,
        0x5f0000 as ::core::ffi::c_int,
        0x5f005f as ::core::ffi::c_int,
        0x5f0087 as ::core::ffi::c_int,
        0x5f00af as ::core::ffi::c_int,
        0x5f00d7 as ::core::ffi::c_int,
        0x5f00ff as ::core::ffi::c_int,
        0x5f5f00 as ::core::ffi::c_int,
        0x5f5f5f as ::core::ffi::c_int,
        0x5f5f87 as ::core::ffi::c_int,
        0x5f5faf as ::core::ffi::c_int,
        0x5f5fd7 as ::core::ffi::c_int,
        0x5f5fff as ::core::ffi::c_int,
        0x5f8700 as ::core::ffi::c_int,
        0x5f875f as ::core::ffi::c_int,
        0x5f8787 as ::core::ffi::c_int,
        0x5f87af as ::core::ffi::c_int,
        0x5f87d7 as ::core::ffi::c_int,
        0x5f87ff as ::core::ffi::c_int,
        0x5faf00 as ::core::ffi::c_int,
        0x5faf5f as ::core::ffi::c_int,
        0x5faf87 as ::core::ffi::c_int,
        0x5fafaf as ::core::ffi::c_int,
        0x5fafd7 as ::core::ffi::c_int,
        0x5fafff as ::core::ffi::c_int,
        0x5fd700 as ::core::ffi::c_int,
        0x5fd75f as ::core::ffi::c_int,
        0x5fd787 as ::core::ffi::c_int,
        0x5fd7af as ::core::ffi::c_int,
        0x5fd7d7 as ::core::ffi::c_int,
        0x5fd7ff as ::core::ffi::c_int,
        0x5fff00 as ::core::ffi::c_int,
        0x5fff5f as ::core::ffi::c_int,
        0x5fff87 as ::core::ffi::c_int,
        0x5fffaf as ::core::ffi::c_int,
        0x5fffd7 as ::core::ffi::c_int,
        0x5fffff as ::core::ffi::c_int,
        0x870000 as ::core::ffi::c_int,
        0x87005f as ::core::ffi::c_int,
        0x870087 as ::core::ffi::c_int,
        0x8700af as ::core::ffi::c_int,
        0x8700d7 as ::core::ffi::c_int,
        0x8700ff as ::core::ffi::c_int,
        0x875f00 as ::core::ffi::c_int,
        0x875f5f as ::core::ffi::c_int,
        0x875f87 as ::core::ffi::c_int,
        0x875faf as ::core::ffi::c_int,
        0x875fd7 as ::core::ffi::c_int,
        0x875fff as ::core::ffi::c_int,
        0x878700 as ::core::ffi::c_int,
        0x87875f as ::core::ffi::c_int,
        0x878787 as ::core::ffi::c_int,
        0x8787af as ::core::ffi::c_int,
        0x8787d7 as ::core::ffi::c_int,
        0x8787ff as ::core::ffi::c_int,
        0x87af00 as ::core::ffi::c_int,
        0x87af5f as ::core::ffi::c_int,
        0x87af87 as ::core::ffi::c_int,
        0x87afaf as ::core::ffi::c_int,
        0x87afd7 as ::core::ffi::c_int,
        0x87afff as ::core::ffi::c_int,
        0x87d700 as ::core::ffi::c_int,
        0x87d75f as ::core::ffi::c_int,
        0x87d787 as ::core::ffi::c_int,
        0x87d7af as ::core::ffi::c_int,
        0x87d7d7 as ::core::ffi::c_int,
        0x87d7ff as ::core::ffi::c_int,
        0x87ff00 as ::core::ffi::c_int,
        0x87ff5f as ::core::ffi::c_int,
        0x87ff87 as ::core::ffi::c_int,
        0x87ffaf as ::core::ffi::c_int,
        0x87ffd7 as ::core::ffi::c_int,
        0x87ffff as ::core::ffi::c_int,
        0xaf0000 as ::core::ffi::c_int,
        0xaf005f as ::core::ffi::c_int,
        0xaf0087 as ::core::ffi::c_int,
        0xaf00af as ::core::ffi::c_int,
        0xaf00d7 as ::core::ffi::c_int,
        0xaf00ff as ::core::ffi::c_int,
        0xaf5f00 as ::core::ffi::c_int,
        0xaf5f5f as ::core::ffi::c_int,
        0xaf5f87 as ::core::ffi::c_int,
        0xaf5faf as ::core::ffi::c_int,
        0xaf5fd7 as ::core::ffi::c_int,
        0xaf5fff as ::core::ffi::c_int,
        0xaf8700 as ::core::ffi::c_int,
        0xaf875f as ::core::ffi::c_int,
        0xaf8787 as ::core::ffi::c_int,
        0xaf87af as ::core::ffi::c_int,
        0xaf87d7 as ::core::ffi::c_int,
        0xaf87ff as ::core::ffi::c_int,
        0xafaf00 as ::core::ffi::c_int,
        0xafaf5f as ::core::ffi::c_int,
        0xafaf87 as ::core::ffi::c_int,
        0xafafaf as ::core::ffi::c_int,
        0xafafd7 as ::core::ffi::c_int,
        0xafafff as ::core::ffi::c_int,
        0xafd700 as ::core::ffi::c_int,
        0xafd75f as ::core::ffi::c_int,
        0xafd787 as ::core::ffi::c_int,
        0xafd7af as ::core::ffi::c_int,
        0xafd7d7 as ::core::ffi::c_int,
        0xafd7ff as ::core::ffi::c_int,
        0xafff00 as ::core::ffi::c_int,
        0xafff5f as ::core::ffi::c_int,
        0xafff87 as ::core::ffi::c_int,
        0xafffaf as ::core::ffi::c_int,
        0xafffd7 as ::core::ffi::c_int,
        0xafffff as ::core::ffi::c_int,
        0xd70000 as ::core::ffi::c_int,
        0xd7005f as ::core::ffi::c_int,
        0xd70087 as ::core::ffi::c_int,
        0xd700af as ::core::ffi::c_int,
        0xd700d7 as ::core::ffi::c_int,
        0xd700ff as ::core::ffi::c_int,
        0xd75f00 as ::core::ffi::c_int,
        0xd75f5f as ::core::ffi::c_int,
        0xd75f87 as ::core::ffi::c_int,
        0xd75faf as ::core::ffi::c_int,
        0xd75fd7 as ::core::ffi::c_int,
        0xd75fff as ::core::ffi::c_int,
        0xd78700 as ::core::ffi::c_int,
        0xd7875f as ::core::ffi::c_int,
        0xd78787 as ::core::ffi::c_int,
        0xd787af as ::core::ffi::c_int,
        0xd787d7 as ::core::ffi::c_int,
        0xd787ff as ::core::ffi::c_int,
        0xd7af00 as ::core::ffi::c_int,
        0xd7af5f as ::core::ffi::c_int,
        0xd7af87 as ::core::ffi::c_int,
        0xd7afaf as ::core::ffi::c_int,
        0xd7afd7 as ::core::ffi::c_int,
        0xd7afff as ::core::ffi::c_int,
        0xd7d700 as ::core::ffi::c_int,
        0xd7d75f as ::core::ffi::c_int,
        0xd7d787 as ::core::ffi::c_int,
        0xd7d7af as ::core::ffi::c_int,
        0xd7d7d7 as ::core::ffi::c_int,
        0xd7d7ff as ::core::ffi::c_int,
        0xd7ff00 as ::core::ffi::c_int,
        0xd7ff5f as ::core::ffi::c_int,
        0xd7ff87 as ::core::ffi::c_int,
        0xd7ffaf as ::core::ffi::c_int,
        0xd7ffd7 as ::core::ffi::c_int,
        0xd7ffff as ::core::ffi::c_int,
        0xff0000 as ::core::ffi::c_int,
        0xff005f as ::core::ffi::c_int,
        0xff0087 as ::core::ffi::c_int,
        0xff00af as ::core::ffi::c_int,
        0xff00d7 as ::core::ffi::c_int,
        0xff00ff as ::core::ffi::c_int,
        0xff5f00 as ::core::ffi::c_int,
        0xff5f5f as ::core::ffi::c_int,
        0xff5f87 as ::core::ffi::c_int,
        0xff5faf as ::core::ffi::c_int,
        0xff5fd7 as ::core::ffi::c_int,
        0xff5fff as ::core::ffi::c_int,
        0xff8700 as ::core::ffi::c_int,
        0xff875f as ::core::ffi::c_int,
        0xff8787 as ::core::ffi::c_int,
        0xff87af as ::core::ffi::c_int,
        0xff87d7 as ::core::ffi::c_int,
        0xff87ff as ::core::ffi::c_int,
        0xffaf00 as ::core::ffi::c_int,
        0xffaf5f as ::core::ffi::c_int,
        0xffaf87 as ::core::ffi::c_int,
        0xffafaf as ::core::ffi::c_int,
        0xffafd7 as ::core::ffi::c_int,
        0xffafff as ::core::ffi::c_int,
        0xffd700 as ::core::ffi::c_int,
        0xffd75f as ::core::ffi::c_int,
        0xffd787 as ::core::ffi::c_int,
        0xffd7af as ::core::ffi::c_int,
        0xffd7d7 as ::core::ffi::c_int,
        0xffd7ff as ::core::ffi::c_int,
        0xffff00 as ::core::ffi::c_int,
        0xffff5f as ::core::ffi::c_int,
        0xffff87 as ::core::ffi::c_int,
        0xffffaf as ::core::ffi::c_int,
        0xffffd7 as ::core::ffi::c_int,
        0xffffff as ::core::ffi::c_int,
        0x80808 as ::core::ffi::c_int,
        0x121212 as ::core::ffi::c_int,
        0x1c1c1c as ::core::ffi::c_int,
        0x262626 as ::core::ffi::c_int,
        0x303030 as ::core::ffi::c_int,
        0x3a3a3a as ::core::ffi::c_int,
        0x444444 as ::core::ffi::c_int,
        0x4e4e4e as ::core::ffi::c_int,
        0x585858 as ::core::ffi::c_int,
        0x626262 as ::core::ffi::c_int,
        0x6c6c6c as ::core::ffi::c_int,
        0x767676 as ::core::ffi::c_int,
        0x808080 as ::core::ffi::c_int,
        0x8a8a8a as ::core::ffi::c_int,
        0x949494 as ::core::ffi::c_int,
        0x9e9e9e as ::core::ffi::c_int,
        0xa8a8a8 as ::core::ffi::c_int,
        0xb2b2b2 as ::core::ffi::c_int,
        0xbcbcbc as ::core::ffi::c_int,
        0xc6c6c6 as ::core::ffi::c_int,
        0xd0d0d0 as ::core::ffi::c_int,
        0xdadada as ::core::ffi::c_int,
        0xe4e4e4 as ::core::ffi::c_int,
        0xeeeeee as ::core::ffi::c_int,
    ];
    return table[(c & 0xff as ::core::ffi::c_int) as usize] | COLOUR_FLAG_RGB;
}
#[no_mangle]
pub unsafe extern "C" fn colour_256to16(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    static mut table: [::core::ffi::c_char; 256] = [
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        3 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        5 as ::core::ffi::c_int as ::core::ffi::c_char,
        6 as ::core::ffi::c_int as ::core::ffi::c_char,
        7 as ::core::ffi::c_int as ::core::ffi::c_char,
        8 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        11 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        13 as ::core::ffi::c_int as ::core::ffi::c_char,
        14 as ::core::ffi::c_int as ::core::ffi::c_char,
        15 as ::core::ffi::c_int as ::core::ffi::c_char,
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        6 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        6 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        6 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        14 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        14 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        5 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        3 as ::core::ffi::c_int as ::core::ffi::c_char,
        8 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        6 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        6 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        14 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        14 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        5 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        5 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        3 as ::core::ffi::c_int as ::core::ffi::c_char,
        3 as ::core::ffi::c_int as ::core::ffi::c_char,
        8 as ::core::ffi::c_int as ::core::ffi::c_char,
        4 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        2 as ::core::ffi::c_int as ::core::ffi::c_char,
        6 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        14 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        14 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        5 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        5 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        1 as ::core::ffi::c_int as ::core::ffi::c_char,
        5 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        3 as ::core::ffi::c_int as ::core::ffi::c_char,
        3 as ::core::ffi::c_int as ::core::ffi::c_char,
        3 as ::core::ffi::c_int as ::core::ffi::c_char,
        7 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        14 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        14 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        13 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        13 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        13 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        13 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        11 as ::core::ffi::c_int as ::core::ffi::c_char,
        11 as ::core::ffi::c_int as ::core::ffi::c_char,
        11 as ::core::ffi::c_int as ::core::ffi::c_char,
        11 as ::core::ffi::c_int as ::core::ffi::c_char,
        7 as ::core::ffi::c_int as ::core::ffi::c_char,
        12 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        10 as ::core::ffi::c_int as ::core::ffi::c_char,
        14 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        13 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        13 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        13 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        13 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        9 as ::core::ffi::c_int as ::core::ffi::c_char,
        13 as ::core::ffi::c_int as ::core::ffi::c_char,
        11 as ::core::ffi::c_int as ::core::ffi::c_char,
        11 as ::core::ffi::c_int as ::core::ffi::c_char,
        11 as ::core::ffi::c_int as ::core::ffi::c_char,
        11 as ::core::ffi::c_int as ::core::ffi::c_char,
        11 as ::core::ffi::c_int as ::core::ffi::c_char,
        15 as ::core::ffi::c_int as ::core::ffi::c_char,
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        8 as ::core::ffi::c_int as ::core::ffi::c_char,
        8 as ::core::ffi::c_int as ::core::ffi::c_char,
        8 as ::core::ffi::c_int as ::core::ffi::c_char,
        8 as ::core::ffi::c_int as ::core::ffi::c_char,
        8 as ::core::ffi::c_int as ::core::ffi::c_char,
        8 as ::core::ffi::c_int as ::core::ffi::c_char,
        7 as ::core::ffi::c_int as ::core::ffi::c_char,
        7 as ::core::ffi::c_int as ::core::ffi::c_char,
        7 as ::core::ffi::c_int as ::core::ffi::c_char,
        7 as ::core::ffi::c_int as ::core::ffi::c_char,
        7 as ::core::ffi::c_int as ::core::ffi::c_char,
        7 as ::core::ffi::c_int as ::core::ffi::c_char,
        15 as ::core::ffi::c_int as ::core::ffi::c_char,
        15 as ::core::ffi::c_int as ::core::ffi::c_char,
        15 as ::core::ffi::c_int as ::core::ffi::c_char,
        15 as ::core::ffi::c_int as ::core::ffi::c_char,
        15 as ::core::ffi::c_int as ::core::ffi::c_char,
        15 as ::core::ffi::c_int as ::core::ffi::c_char,
    ];
    return table[(c & 0xff as ::core::ffi::c_int) as usize] as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn colour_byname(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    static mut colours: [C2RustUnnamed_37; 578] = [
        C2RustUnnamed_37 {
            name: b"AliceBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf0f8ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"AntiqueWhite\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfaebd7 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"AntiqueWhite1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffefdb as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"AntiqueWhite2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeedfcc as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"AntiqueWhite3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdc0b0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"AntiqueWhite4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b8378 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"BlanchedAlmond\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffebcd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"BlueViolet\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8a2be2 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"CadetBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x5f9ea0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"CadetBlue1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x98f5ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"CadetBlue2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8ee5ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"CadetBlue3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7ac5cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"CadetBlue4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x53868b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"CornflowerBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x6495ed as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkCyan\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkGoldenrod\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb8860b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkGoldenrod1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffb90f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkGoldenrod2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeead0e as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkGoldenrod3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd950c as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkGoldenrod4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b6508 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkGray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xa9a9a9 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x6400 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkGrey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xa9a9a9 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkKhaki\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbdb76b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkMagenta\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b008b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOliveGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x556b2f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOliveGreen1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcaff70 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOliveGreen2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbcee68 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOliveGreen3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xa2cd5a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOliveGreen4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x6e8b3d as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOrange\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff8c00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOrange1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff7f00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOrange2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee7600 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOrange3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd6600 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOrange4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b4500 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOrchid\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9932cc as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOrchid1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbf3eff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOrchid2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb23aee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOrchid3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9a32cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkOrchid4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x68228b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkRed\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b0000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkSalmon\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xe9967a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkSeaGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8fbc8f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkSeaGreen1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xc1ffc1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkSeaGreen2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb4eeb4 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkSeaGreen3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9bcd9b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkSeaGreen4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x698b69 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkSlateBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x483d8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkSlateGray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x2f4f4f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkSlateGray1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x97ffff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkSlateGray2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8deeee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkSlateGray3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x79cdcd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkSlateGray4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x528b8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkSlateGrey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x2f4f4f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkTurquoise\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xced1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DarkViolet\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9400d3 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DeepPink\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff1493 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DeepPink1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff1493 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DeepPink2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee1289 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DeepPink3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd1076 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DeepPink4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b0a50 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DeepSkyBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbfff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DeepSkyBlue1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbfff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DeepSkyBlue2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb2ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DeepSkyBlue3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9acd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DeepSkyBlue4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x688b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DimGray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x696969 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DimGrey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x696969 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DodgerBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x1e90ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DodgerBlue1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x1e90ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DodgerBlue2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x1c86ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DodgerBlue3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x1874cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"DodgerBlue4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x104e8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"FloralWhite\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfffaf0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"ForestGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x228b22 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"GhostWhite\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf8f8ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"GreenYellow\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xadff2f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"HotPink\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff69b4 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"HotPink1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff6eb4 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"HotPink2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee6aa7 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"HotPink3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd6090 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"HotPink4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b3a62 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"IndianRed\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd5c5c as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"IndianRed1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff6a6a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"IndianRed2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee6363 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"IndianRed3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd5555 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"IndianRed4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b3a3a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LavenderBlush\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfff0f5 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LavenderBlush1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfff0f5 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LavenderBlush2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeee0e5 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LavenderBlush3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdc1c5 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LavenderBlush4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b8386 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LawnGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7cfc00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LemonChiffon\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfffacd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LemonChiffon1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfffacd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LemonChiffon2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeee9bf as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LemonChiffon3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdc9a5 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LemonChiffon4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b8970 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xadd8e6 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightBlue1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbfefff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightBlue2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb2dfee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightBlue3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9ac0cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightBlue4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x68838b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightCoral\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf08080 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightCyan\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xe0ffff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightCyan1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xe0ffff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightCyan2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xd1eeee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightCyan3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb4cdcd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightCyan4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7a8b8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightGoldenrod\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeedd82 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightGoldenrod1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffec8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightGoldenrod2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeedc82 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightGoldenrod3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdbe70 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightGoldenrod4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b814c as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightGoldenrodYellow\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfafad2 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightGray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xd3d3d3 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x90ee90 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightGrey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xd3d3d3 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightPink\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffb6c1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightPink1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffaeb9 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightPink2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeea2ad as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightPink3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd8c95 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightPink4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b5f65 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSalmon\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffa07a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSalmon1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffa07a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSalmon2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee9572 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSalmon3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd8162 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSalmon4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b5742 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSeaGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x20b2aa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSkyBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x87cefa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSkyBlue1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb0e2ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSkyBlue2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xa4d3ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSkyBlue3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8db6cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSkyBlue4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x607b8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSlateBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8470ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSlateGray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x778899 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSlateGrey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x778899 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSteelBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb0c4de as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSteelBlue1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcae1ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSteelBlue2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbcd2ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSteelBlue3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xa2b5cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightSteelBlue4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x6e7b8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightYellow\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffffe0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightYellow1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffffe0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightYellow2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeeeed1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightYellow3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdcdb4 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LightYellow4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b8b7a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"LimeGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x32cd32 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumAquamarine\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x66cdaa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumOrchid\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xba55d3 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumOrchid1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xe066ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumOrchid2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xd15fee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumOrchid3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb452cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumOrchid4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7a378b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumPurple\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9370db as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumPurple1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xab82ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumPurple2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9f79ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumPurple3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8968cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumPurple4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x5d478b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumSeaGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x3cb371 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumSlateBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7b68ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumSpringGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfa9a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumTurquoise\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x48d1cc as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MediumVioletRed\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xc71585 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MidnightBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x191970 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MintCream\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf5fffa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MistyRose\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffe4e1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MistyRose1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffe4e1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MistyRose2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeed5d2 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MistyRose3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdb7b5 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"MistyRose4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b7d7b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"NavajoWhite\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffdead as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"NavajoWhite1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffdead as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"NavajoWhite2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeecfa1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"NavajoWhite3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdb38b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"NavajoWhite4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b795e as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"NavyBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x80 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"OldLace\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfdf5e6 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"OliveDrab\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x6b8e23 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"OliveDrab1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xc0ff3e as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"OliveDrab2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb3ee3a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"OliveDrab3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9acd32 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"OliveDrab4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x698b22 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"OrangeRed\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff4500 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"OrangeRed1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff4500 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"OrangeRed2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee4000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"OrangeRed3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd3700 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"OrangeRed4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b2500 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleGoldenrod\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeee8aa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x98fb98 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleGreen1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9aff9a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleGreen2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x90ee90 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleGreen3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7ccd7c as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleGreen4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x548b54 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleTurquoise\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xafeeee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleTurquoise1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbbffff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleTurquoise2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xaeeeee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleTurquoise3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x96cdcd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleTurquoise4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x668b8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleVioletRed\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xdb7093 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleVioletRed1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff82ab as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleVioletRed2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee799f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleVioletRed3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd6889 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PaleVioletRed4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b475d as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PapayaWhip\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffefd5 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PeachPuff\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffdab9 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PeachPuff1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffdab9 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PeachPuff2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeecbad as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PeachPuff3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdaf95 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PeachPuff4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b7765 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"PowderBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb0e0e6 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"RebeccaPurple\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x663399 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"RosyBrown\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbc8f8f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"RosyBrown1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffc1c1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"RosyBrown2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeeb4b4 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"RosyBrown3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd9b9b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"RosyBrown4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b6969 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"RoyalBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x4169e1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"RoyalBlue1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x4876ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"RoyalBlue2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x436eee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"RoyalBlue3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x3a5fcd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"RoyalBlue4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x27408b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SaddleBrown\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b4513 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SandyBrown\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf4a460 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SeaGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x2e8b57 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SeaGreen1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x54ff9f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SeaGreen2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x4eee94 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SeaGreen3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x43cd80 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SeaGreen4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x2e8b57 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SkyBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x87ceeb as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SkyBlue1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x87ceff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SkyBlue2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7ec0ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SkyBlue3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x6ca6cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SkyBlue4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x4a708b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SlateBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x6a5acd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SlateBlue1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x836fff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SlateBlue2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7a67ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SlateBlue3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x6959cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SlateBlue4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x473c8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SlateGray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x708090 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SlateGray1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xc6e2ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SlateGray2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb9d3ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SlateGray3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9fb6cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SlateGray4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x6c7b8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SlateGrey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x708090 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SpringGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff7f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SpringGreen1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff7f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SpringGreen2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee76 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SpringGreen3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd66 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SpringGreen4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b45 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SteelBlue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x4682b4 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SteelBlue1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x63b8ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SteelBlue2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x5cacee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SteelBlue3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x4f94cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"SteelBlue4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x36648b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"VioletRed\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xd02090 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"VioletRed1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff3e96 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"VioletRed2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee3a8c as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"VioletRed3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd3278 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"VioletRed4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b2252 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"WebGray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x808080 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"WebGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"WebGrey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x808080 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"WebMaroon\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x800000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"WebPurple\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x800080 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"WhiteSmoke\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf5f5f5 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"X11Gray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbebebe as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"X11Green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"X11Grey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbebebe as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"X11Maroon\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb03060 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"X11Purple\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xa020f0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"YellowGreen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9acd32 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"alice blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf0f8ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"antique white\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfaebd7 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"aqua\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"aquamarine\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7fffd4 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"aquamarine1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7fffd4 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"aquamarine2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x76eec6 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"aquamarine3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x66cdaa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"aquamarine4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x458b74 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"azure\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf0ffff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"azure1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf0ffff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"azure2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xe0eeee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"azure3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xc1cdcd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"azure4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x838b8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"beige\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf5f5dc as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"bisque\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffe4c4 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"bisque1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffe4c4 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"bisque2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeed5b7 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"bisque3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdb79e as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"bisque4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b7d6b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"black\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"blanched almond\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffebcd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"blue violet\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8a2be2 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"blue1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"blue2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"blue3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"blue4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"brown\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xa52a2a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"brown1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff4040 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"brown2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee3b3b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"brown3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd3333 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"brown4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b2323 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"burlywood\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xdeb887 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"burlywood1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffd39b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"burlywood2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeec591 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"burlywood3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdaa7d as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"burlywood4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b7355 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"cadet blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x5f9ea0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"chartreuse\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7fff00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"chartreuse1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7fff00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"chartreuse2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x76ee00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"chartreuse3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x66cd00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"chartreuse4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x458b00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"chocolate\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xd2691e as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"chocolate1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff7f24 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"chocolate2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee7621 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"chocolate3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd661d as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"chocolate4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b4513 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"coral\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff7f50 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"coral1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff7256 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"coral2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee6a50 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"coral3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd5b45 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"coral4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b3e2f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"cornflower blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x6495ed as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"cornsilk\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfff8dc as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"cornsilk1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfff8dc as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"cornsilk2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeee8cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"cornsilk3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdc8b1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"cornsilk4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b8878 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"crimson\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xdc143c as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"cyan\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"cyan1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"cyan2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeeee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"cyan3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdcd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"cyan4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark cyan\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark goldenrod\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb8860b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark gray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xa9a9a9 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x6400 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark grey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xa9a9a9 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark khaki\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbdb76b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark magenta\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b008b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark olive green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x556b2f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark orange\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff8c00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark orchid\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9932cc as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark red\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b0000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark salmon\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xe9967a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark sea green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8fbc8f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark slate blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x483d8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark slate gray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x2f4f4f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark slate grey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x2f4f4f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark turquoise\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xced1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dark violet\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9400d3 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"deep pink\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff1493 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"deep sky blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbfff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dim gray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x696969 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dim grey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x696969 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"dodger blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x1e90ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"firebrick\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb22222 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"firebrick1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff3030 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"firebrick2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee2c2c as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"firebrick3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd2626 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"firebrick4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b1a1a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"floral white\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfffaf0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"forest green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x228b22 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"fuchsia\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff00ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"gainsboro\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xdcdcdc as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"ghost white\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf8f8ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"gold\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffd700 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"gold1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffd700 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"gold2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeec900 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"gold3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdad00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"gold4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b7500 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"goldenrod\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xdaa520 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"goldenrod1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffc125 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"goldenrod2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeeb422 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"goldenrod3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd9b1d as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"goldenrod4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b6914 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"green yellow\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xadff2f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"green1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"green2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"green3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"green4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"honeydew\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf0fff0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"honeydew1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf0fff0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"honeydew2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xe0eee0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"honeydew3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xc1cdc1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"honeydew4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x838b83 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"hot pink\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff69b4 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"indian red\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd5c5c as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"indigo\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x4b0082 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"ivory\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfffff0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"ivory1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfffff0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"ivory2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeeeee0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"ivory3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdcdc1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"ivory4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b8b83 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"khaki\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf0e68c as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"khaki1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfff68f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"khaki2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeee685 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"khaki3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdc673 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"khaki4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b864e as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"lavender blush\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfff0f5 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"lavender\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xe6e6fa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"lawn green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7cfc00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"lemon chiffon\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfffacd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xadd8e6 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light coral\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf08080 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light cyan\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xe0ffff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light goldenrod yellow\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfafad2 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light goldenrod\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeedd82 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light gray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xd3d3d3 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x90ee90 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light grey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xd3d3d3 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light pink\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffb6c1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light salmon\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffa07a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light sea green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x20b2aa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light sky blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x87cefa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light slate blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8470ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light slate gray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x778899 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light slate grey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x778899 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light steel blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb0c4de as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"light yellow\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffffe0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"lime green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x32cd32 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"lime\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"linen\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfaf0e6 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"magenta\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff00ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"magenta1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff00ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"magenta2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee00ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"magenta3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd00cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"magenta4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b008b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"maroon\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb03060 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"maroon1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff34b3 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"maroon2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee30a7 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"maroon3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd2990 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"maroon4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b1c62 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"medium aquamarine\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x66cdaa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"medium blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"medium orchid\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xba55d3 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"medium purple\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9370db as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"medium sea green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x3cb371 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"medium slate blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7b68ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"medium spring green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfa9a as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"medium turquoise\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x48d1cc as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"medium violet red\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xc71585 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"midnight blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x191970 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"mint cream\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf5fffa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"misty rose\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffe4e1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"moccasin\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffe4b5 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"navajo white\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffdead as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"navy blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x80 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"navy\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x80 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"old lace\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfdf5e6 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"olive drab\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x6b8e23 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"olive\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x808000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"orange red\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff4500 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"orange\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffa500 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"orange1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffa500 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"orange2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee9a00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"orange3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd8500 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"orange4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b5a00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"orchid\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xda70d6 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"orchid1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff83fa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"orchid2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee7ae9 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"orchid3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd69c9 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"orchid4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b4789 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"pale goldenrod\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeee8aa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"pale green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x98fb98 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"pale turquoise\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xafeeee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"pale violet red\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xdb7093 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"papaya whip\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffefd5 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"peach puff\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffdab9 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"peru\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd853f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"pink\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffc0cb as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"pink1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffb5c5 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"pink2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeea9b8 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"pink3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd919e as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"pink4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b636c as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"plum\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xdda0dd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"plum1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffbbff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"plum2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeeaeee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"plum3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd96cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"plum4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b668b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"powder blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb0e0e6 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"purple\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xa020f0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"purple1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9b30ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"purple2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x912cee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"purple3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x7d26cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"purple4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x551a8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"rebecca purple\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x663399 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"red\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff0000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"red1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff0000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"red2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee0000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"red3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd0000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"red4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b0000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"rosy brown\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbc8f8f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"royal blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x4169e1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"saddle brown\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b4513 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"salmon\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfa8072 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"salmon1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff8c69 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"salmon2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee8262 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"salmon3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd7054 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"salmon4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b4c39 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"sandy brown\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf4a460 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"sea green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x2e8b57 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"seashell\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfff5ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"seashell1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfff5ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"seashell2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeee5de as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"seashell3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdc5bf as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"seashell4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b8682 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"sienna\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xa0522d as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"sienna1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff8247 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"sienna2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee7942 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"sienna3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd6839 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"sienna4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b4726 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"silver\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xc0c0c0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"sky blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x87ceeb as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"slate blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x6a5acd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"slate gray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x708090 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"slate grey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x708090 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"snow\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfffafa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"snow1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xfffafa as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"snow2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeee9e9 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"snow3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdc9c9 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"snow4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b8989 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"spring green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff7f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"steel blue\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x4682b4 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"tan\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xd2b48c as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"tan1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffa54f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"tan2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee9a49 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"tan3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd853f as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"tan4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b5a2b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"teal\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8080 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"thistle\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xd8bfd8 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"thistle1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffe1ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"thistle2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeed2ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"thistle3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdb5cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"thistle4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b7b8b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"tomato\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff6347 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"tomato1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff6347 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"tomato2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee5c42 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"tomato3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcd4f39 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"tomato4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b3626 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"turquoise\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x40e0d0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"turquoise1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf5ff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"turquoise2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xe5ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"turquoise3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xc5cd as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"turquoise4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x868b as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"violet red\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xd02090 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"violet\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xee82ee as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"web gray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x808080 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"web green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"web grey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x808080 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"web maroon\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x800000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"web purple\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x800080 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"wheat\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf5deb3 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"wheat1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffe7ba as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"wheat2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeed8ae as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"wheat3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdba96 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"wheat4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b7e66 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"white smoke\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xf5f5f5 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"white\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffffff as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"x11 gray\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbebebe as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"x11 green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xff00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"x11 grey\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xbebebe as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"x11 maroon\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xb03060 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"x11 purple\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xa020f0 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"yellow green\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x9acd32 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"yellow\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffff00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"yellow1\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xffff00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"yellow2\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xeeee00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"yellow3\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0xcdcd00 as ::core::ffi::c_int,
        },
        C2RustUnnamed_37 {
            name: b"yellow4\0" as *const u8 as *const ::core::ffi::c_char,
            c: 0x8b8b00 as ::core::ffi::c_int,
        },
    ];
    let mut i: u_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if strncasecmp(
        name,
        b"grey\0" as *const u8 as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncasecmp(
            name,
            b"gray\0" as *const u8 as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        if *name.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32 {
            return 0xbebebe as ::core::ffi::c_int | COLOUR_FLAG_RGB;
        }
        c = strtonum(
            name.offset(4 as ::core::ffi::c_int as isize),
            0 as ::core::ffi::c_longlong,
            100 as ::core::ffi::c_longlong,
            &raw mut errstr,
        ) as ::core::ffi::c_int;
        if !errstr.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        c = round(2.55f64 * c as ::core::ffi::c_double) as ::core::ffi::c_int;
        if c < 0 as ::core::ffi::c_int || c > 255 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        return colour_join_rgb(c as u_char, c as u_char, c as u_char);
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_37; 578]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_37>() as usize)
    {
        if strcasecmp(colours[i as usize].name, name) == 0 as ::core::ffi::c_int {
            return colours[i as usize].c | COLOUR_FLAG_RGB;
        }
        i = i.wrapping_add(1);
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn colour_parseX11(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_double = 0.;
    let mut m: ::core::ffi::c_double = 0.;
    let mut y: ::core::ffi::c_double = 0.;
    let mut k: ::core::ffi::c_double = 0 as ::core::ffi::c_int as ::core::ffi::c_double;
    let mut r: u_int = 0;
    let mut g: u_int = 0;
    let mut b: u_int = 0;
    let mut len: size_t = strlen(p);
    let mut colour: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if len == 12 as size_t
        && sscanf(
            p,
            b"rgb:%02x/%02x/%02x\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut r,
            &raw mut g,
            &raw mut b,
        ) == 3 as ::core::ffi::c_int
        || len == 7 as size_t
            && sscanf(
                p,
                b"#%02x%02x%02x\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut r,
                &raw mut g,
                &raw mut b,
            ) == 3 as ::core::ffi::c_int
        || sscanf(
            p,
            b"%d,%d,%d\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut r,
            &raw mut g,
            &raw mut b,
        ) == 3 as ::core::ffi::c_int
    {
        colour = colour_join_rgb(r as u_char, g as u_char, b as u_char);
    } else if len == 18 as size_t
        && sscanf(
            p,
            b"rgb:%04x/%04x/%04x\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut r,
            &raw mut g,
            &raw mut b,
        ) == 3 as ::core::ffi::c_int
        || len == 13 as size_t
            && sscanf(
                p,
                b"#%04x%04x%04x\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut r,
                &raw mut g,
                &raw mut b,
            ) == 3 as ::core::ffi::c_int
    {
        colour = colour_join_rgb(
            (r >> 8 as ::core::ffi::c_int) as u_char,
            (g >> 8 as ::core::ffi::c_int) as u_char,
            (b >> 8 as ::core::ffi::c_int) as u_char,
        );
    } else if (sscanf(
        p,
        b"cmyk:%lf/%lf/%lf/%lf\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut c,
        &raw mut m,
        &raw mut y,
        &raw mut k,
    ) == 4 as ::core::ffi::c_int
        || sscanf(
            p,
            b"cmy:%lf/%lf/%lf\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut c,
            &raw mut m,
            &raw mut y,
        ) == 3 as ::core::ffi::c_int)
        && c >= 0 as ::core::ffi::c_int as ::core::ffi::c_double
        && c <= 1 as ::core::ffi::c_int as ::core::ffi::c_double
        && m >= 0 as ::core::ffi::c_int as ::core::ffi::c_double
        && m <= 1 as ::core::ffi::c_int as ::core::ffi::c_double
        && y >= 0 as ::core::ffi::c_int as ::core::ffi::c_double
        && y <= 1 as ::core::ffi::c_int as ::core::ffi::c_double
        && k >= 0 as ::core::ffi::c_int as ::core::ffi::c_double
        && k <= 1 as ::core::ffi::c_int as ::core::ffi::c_double
    {
        colour = colour_join_rgb(
            ((1 as ::core::ffi::c_int as ::core::ffi::c_double - c)
                * (1 as ::core::ffi::c_int as ::core::ffi::c_double - k)
                * 255 as ::core::ffi::c_int as ::core::ffi::c_double) as u_char,
            ((1 as ::core::ffi::c_int as ::core::ffi::c_double - m)
                * (1 as ::core::ffi::c_int as ::core::ffi::c_double - k)
                * 255 as ::core::ffi::c_int as ::core::ffi::c_double) as u_char,
            ((1 as ::core::ffi::c_int as ::core::ffi::c_double - y)
                * (1 as ::core::ffi::c_int as ::core::ffi::c_double - k)
                * 255 as ::core::ffi::c_int as ::core::ffi::c_double) as u_char,
        );
    } else {
        while len != 0 as size_t && *p as ::core::ffi::c_int == ' ' as i32 {
            p = p.offset(1);
            len = len.wrapping_sub(1);
        }
        while len != 0 as size_t
            && *p.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == ' ' as i32
        {
            len = len.wrapping_sub(1);
        }
        copy = xstrndup(p, len);
        colour = colour_byname(copy);
        free(copy as *mut ::core::ffi::c_void);
    }
    log_debug(
        b"%s: %s = %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"colour_parseX11\0" as *const u8 as *const ::core::ffi::c_char,
        p,
        colour_tostring(colour),
    );
    return colour;
}
#[no_mangle]
pub unsafe extern "C" fn colour_palette_init(mut p: *mut colour_palette) {
    (*p).fg = 8 as ::core::ffi::c_int;
    (*p).bg = 8 as ::core::ffi::c_int;
    (*p).palette = ::core::ptr::null_mut::<::core::ffi::c_int>();
    (*p).default_palette = ::core::ptr::null_mut::<::core::ffi::c_int>();
}
#[no_mangle]
pub unsafe extern "C" fn colour_palette_clear(mut p: *mut colour_palette) {
    if !p.is_null() {
        (*p).fg = 8 as ::core::ffi::c_int;
        (*p).bg = 8 as ::core::ffi::c_int;
        free((*p).palette as *mut ::core::ffi::c_void);
        (*p).palette = ::core::ptr::null_mut::<::core::ffi::c_int>();
    }
}
#[no_mangle]
pub unsafe extern "C" fn colour_palette_free(mut p: *mut colour_palette) {
    if !p.is_null() {
        free((*p).palette as *mut ::core::ffi::c_void);
        (*p).palette = ::core::ptr::null_mut::<::core::ffi::c_int>();
        free((*p).default_palette as *mut ::core::ffi::c_void);
        (*p).default_palette = ::core::ptr::null_mut::<::core::ffi::c_int>();
    }
}
#[no_mangle]
pub unsafe extern "C" fn colour_palette_get(
    mut p: *mut colour_palette,
    mut n: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if p.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if n >= 90 as ::core::ffi::c_int && n <= 97 as ::core::ffi::c_int {
        n = 8 as ::core::ffi::c_int + n - 90 as ::core::ffi::c_int;
    } else if n & COLOUR_FLAG_256 != 0 {
        n &= !COLOUR_FLAG_256;
    } else if n >= 8 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    if !(*p).palette.is_null() && *(*p).palette.offset(n as isize) != -(1 as ::core::ffi::c_int) {
        return *(*p).palette.offset(n as isize);
    }
    if !(*p).default_palette.is_null()
        && *(*p).default_palette.offset(n as isize) != -(1 as ::core::ffi::c_int)
    {
        return *(*p).default_palette.offset(n as isize);
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn colour_palette_set(
    mut p: *mut colour_palette,
    mut n: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    if p.is_null() || n < 0 as ::core::ffi::c_int || n > 255 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if c == -(1 as ::core::ffi::c_int) && (*p).palette.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*p).palette.is_null() {
        (*p).palette = xcalloc(
            256 as size_t,
            ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
        ) as *mut ::core::ffi::c_int;
        i = 0 as u_int;
        while i < 256 as u_int {
            *(*p).palette.offset(i as isize) = -(1 as ::core::ffi::c_int);
            i = i.wrapping_add(1);
        }
    }
    *(*p).palette.offset(n as isize) = c;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn colour_palette_from_option(
    mut p: *mut colour_palette,
    mut oo: *mut options,
) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut i: u_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    if p.is_null() {
        return;
    }
    o = options_get(
        oo,
        b"pane-colours\0" as *const u8 as *const ::core::ffi::c_char,
    );
    a = options_array_first(o);
    if a.is_null() {
        if !(*p).default_palette.is_null() {
            free((*p).default_palette as *mut ::core::ffi::c_void);
            (*p).default_palette = ::core::ptr::null_mut::<::core::ffi::c_int>();
        }
        return;
    }
    if (*p).default_palette.is_null() {
        (*p).default_palette = xcalloc(
            256 as size_t,
            ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
        ) as *mut ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while i < 256 as u_int {
        *(*p).default_palette.offset(i as isize) = -(1 as ::core::ffi::c_int);
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while i < 256 as u_int {
        ov = options_array_getv(o, b"%u\0" as *const u8 as *const ::core::ffi::c_char, i);
        if !ov.is_null() {
            c = (*ov).number as ::core::ffi::c_int;
            *(*p).default_palette.offset(i as isize) = c;
        }
        i = i.wrapping_add(1);
    }
}
