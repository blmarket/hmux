use crate::src::ffi::libc::strcmp;
pub use crate::src::shared::abi::__compar_fn_t;
pub use crate::src::shared::abi::NULL_0;
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
pub use crate::src::shared::client::CLIENT_UTF8;
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
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tty::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::tty_term::{tty_term_has, tty_term_number};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_acs_entry {
    pub key: u_char,
    pub string: &'static ::std::ffi::CStr,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_acs_reverse_entry {
    pub string: &'static ::std::ffi::CStr,
    pub key: u_char,
}

#[inline]
unsafe extern "C" fn bsearch(
    mut __key: *const ::core::ffi::c_void,
    mut __base: *const ::core::ffi::c_void,
    mut __nmemb: size_t,
    mut __size: size_t,
    mut __compar: __compar_fn_t,
) -> *mut ::core::ffi::c_void {
    let mut __p: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    let mut __comparison: ::core::ffi::c_int = 0;
    while __nmemb != 0 {
        __p = (__base as *const ::core::ffi::c_char)
            .offset((__nmemb >> 1 as ::core::ffi::c_int).wrapping_mul(__size) as isize)
            as *const ::core::ffi::c_void;
        __comparison = Some(__compar.expect("non-null function pointer"))
            .expect("non-null function pointer")(__key, __p);
        if __comparison == 0 as ::core::ffi::c_int {
            return __p as *mut ::core::ffi::c_void;
        }
        if __comparison > 0 as ::core::ffi::c_int {
            __base = (__p as *const ::core::ffi::c_char).offset(__size as isize)
                as *const ::core::ffi::c_void;
            __nmemb = __nmemb.wrapping_sub(1);
        }
        __nmemb >>= 1 as ::core::ffi::c_int;
    }
    return NULL;
}
static mut tty_acs_table: [tty_acs_entry; 36] = [
    tty_acs_entry {
        key: '+' as i32 as u_char,
        string: c"\xE2\x86\x92",
    },
    tty_acs_entry {
        key: ',' as i32 as u_char,
        string: c"\xE2\x86\x90",
    },
    tty_acs_entry {
        key: '-' as i32 as u_char,
        string: c"\xE2\x86\x91",
    },
    tty_acs_entry {
        key: '.' as i32 as u_char,
        string: c"\xE2\x86\x93",
    },
    tty_acs_entry {
        key: '0' as i32 as u_char,
        string: c"\xE2\x96\xAE",
    },
    tty_acs_entry {
        key: '`' as i32 as u_char,
        string: c"\xE2\x97\x86",
    },
    tty_acs_entry {
        key: 'a' as i32 as u_char,
        string: c"\xE2\x96\x92",
    },
    tty_acs_entry {
        key: 'b' as i32 as u_char,
        string: c"\xE2\x90\x89",
    },
    tty_acs_entry {
        key: 'c' as i32 as u_char,
        string: c"\xE2\x90\x8C",
    },
    tty_acs_entry {
        key: 'd' as i32 as u_char,
        string: c"\xE2\x90\x8D",
    },
    tty_acs_entry {
        key: 'e' as i32 as u_char,
        string: c"\xE2\x90\x8A",
    },
    tty_acs_entry {
        key: 'f' as i32 as u_char,
        string: c"\xC2\xB0",
    },
    tty_acs_entry {
        key: 'g' as i32 as u_char,
        string: c"\xC2\xB1",
    },
    tty_acs_entry {
        key: 'h' as i32 as u_char,
        string: c"\xE2\x90\xA4",
    },
    tty_acs_entry {
        key: 'i' as i32 as u_char,
        string: c"\xE2\x90\x8B",
    },
    tty_acs_entry {
        key: 'j' as i32 as u_char,
        string: c"\xE2\x94\x98",
    },
    tty_acs_entry {
        key: 'k' as i32 as u_char,
        string: c"\xE2\x94\x90",
    },
    tty_acs_entry {
        key: 'l' as i32 as u_char,
        string: c"\xE2\x94\x8C",
    },
    tty_acs_entry {
        key: 'm' as i32 as u_char,
        string: c"\xE2\x94\x94",
    },
    tty_acs_entry {
        key: 'n' as i32 as u_char,
        string: c"\xE2\x94\xBC",
    },
    tty_acs_entry {
        key: 'o' as i32 as u_char,
        string: c"\xE2\x8E\xBA",
    },
    tty_acs_entry {
        key: 'p' as i32 as u_char,
        string: c"\xE2\x8E\xBB",
    },
    tty_acs_entry {
        key: 'q' as i32 as u_char,
        string: c"\xE2\x94\x80",
    },
    tty_acs_entry {
        key: 'r' as i32 as u_char,
        string: c"\xE2\x8E\xBC",
    },
    tty_acs_entry {
        key: 's' as i32 as u_char,
        string: c"\xE2\x8E\xBD",
    },
    tty_acs_entry {
        key: 't' as i32 as u_char,
        string: c"\xE2\x94\x9C",
    },
    tty_acs_entry {
        key: 'u' as i32 as u_char,
        string: c"\xE2\x94\xA4",
    },
    tty_acs_entry {
        key: 'v' as i32 as u_char,
        string: c"\xE2\x94\xB4",
    },
    tty_acs_entry {
        key: 'w' as i32 as u_char,
        string: c"\xE2\x94\xAC",
    },
    tty_acs_entry {
        key: 'x' as i32 as u_char,
        string: c"\xE2\x94\x82",
    },
    tty_acs_entry {
        key: 'y' as i32 as u_char,
        string: c"\xE2\x89\xA4",
    },
    tty_acs_entry {
        key: 'z' as i32 as u_char,
        string: c"\xE2\x89\xA5",
    },
    tty_acs_entry {
        key: '{' as i32 as u_char,
        string: c"\xCF\x80",
    },
    tty_acs_entry {
        key: '|' as i32 as u_char,
        string: c"\xE2\x89\xA0",
    },
    tty_acs_entry {
        key: '}' as i32 as u_char,
        string: c"\xC2\xA3",
    },
    tty_acs_entry {
        key: '~' as i32 as u_char,
        string: c"\xC2\xB7",
    },
];
static mut tty_acs_reverse2: [tty_acs_reverse_entry; 1] = [tty_acs_reverse_entry {
    string: c"\xC2\xB7",
    key: '~' as i32 as u_char,
}];
static mut tty_acs_reverse3: [tty_acs_reverse_entry; 32] = [
    tty_acs_reverse_entry {
        string: c"\xE2\x94\x80",
        key: 'q' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\x81",
        key: 'q' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\x82",
        key: 'x' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\x83",
        key: 'x' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\x8C",
        key: 'l' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\x8F",
        key: 'k' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\x90",
        key: 'k' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\x93",
        key: 'l' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\x94",
        key: 'm' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\x97",
        key: 'm' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\x98",
        key: 'j' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\x9B",
        key: 'j' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\x9C",
        key: 't' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\xA3",
        key: 't' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\xA4",
        key: 'u' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\xAB",
        key: 'u' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\xB3",
        key: 'w' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\xB4",
        key: 'v' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\xBB",
        key: 'v' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x94\xBC",
        key: 'n' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x95\x8B",
        key: 'n' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x95\x90",
        key: 'q' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x95\x91",
        key: 'x' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x95\x94",
        key: 'l' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x95\x97",
        key: 'k' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x95\x9A",
        key: 'm' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x95\x9D",
        key: 'j' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x95\xA0",
        key: 't' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x95\xA3",
        key: 'u' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x95\xA6",
        key: 'w' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x95\xA9",
        key: 'v' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: c"\xE2\x95\xAC",
        key: 'n' as i32 as u_char,
    },
];
static mut tty_acs_double_borders_list: [utf8_data; 13] = unsafe {
    [
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 0 as u_char,
            width: 0 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x91\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x90\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x94\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x97\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x9A\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x9D\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xA6\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xA9\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xA3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xAC\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xC2\xB7\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 2 as u_char,
            width: 1 as u_char,
        },
    ]
};
static mut tty_acs_heavy_borders_list: [utf8_data; 13] = unsafe {
    [
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 0 as u_char,
            width: 0 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x83\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x81\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x8F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x93\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x97\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x9B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xB3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xBB\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xA3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xAB\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x8B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xC2\xB7\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 2 as u_char,
            width: 1 as u_char,
        },
    ]
};
static mut tty_acs_rounded_borders_list: [utf8_data; 13] = unsafe {
    [
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 0 as u_char,
            width: 0 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xAD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xAE\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xB0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xAF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xB3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xBB\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x9C\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xA4\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x8B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xC2\xB7\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 2 as u_char,
            width: 1 as u_char,
        },
    ]
};
#[no_mangle]
pub unsafe extern "C" fn tty_acs_double_borders(
    mut cell_type: ::core::ffi::c_int,
) -> *const utf8_data {
    return (&raw const tty_acs_double_borders_list as *const utf8_data).offset(cell_type as isize)
        as *const utf8_data;
}
#[no_mangle]
pub unsafe extern "C" fn tty_acs_heavy_borders(
    mut cell_type: ::core::ffi::c_int,
) -> *const utf8_data {
    return (&raw const tty_acs_heavy_borders_list as *const utf8_data).offset(cell_type as isize)
        as *const utf8_data;
}
#[no_mangle]
pub unsafe extern "C" fn tty_acs_rounded_borders(
    mut cell_type: ::core::ffi::c_int,
) -> *const utf8_data {
    return (&raw const tty_acs_rounded_borders_list as *const utf8_data).offset(cell_type as isize)
        as *const utf8_data;
}
unsafe extern "C" fn tty_acs_cmp(
    mut key: *const ::core::ffi::c_void,
    mut value: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut entry: *const tty_acs_entry = value as *const tty_acs_entry;
    let mut test: ::core::ffi::c_int = *(key as *mut u_char) as ::core::ffi::c_int;
    return test - (*entry).key as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_acs_reverse_cmp(
    mut key: *const ::core::ffi::c_void,
    mut value: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut entry: *const tty_acs_reverse_entry = value as *const tty_acs_reverse_entry;
    let mut test: *const ::core::ffi::c_char = key as *const ::core::ffi::c_char;
    return strcmp(test, (*entry).string.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn tty_acs_needed(mut tty: *mut tty) -> ::core::ffi::c_int {
    if tty.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if tty_term_has((*tty).term, TTYC_U8) != 0
        && tty_term_number((*tty).term, TTYC_U8) == 0 as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*(*tty).client).flags & CLIENT_UTF8 as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_acs_get(
    mut tty: *mut tty,
    mut ch: u_char,
) -> *const ::core::ffi::c_char {
    let mut entry: *const tty_acs_entry = ::core::ptr::null::<tty_acs_entry>();
    if tty_acs_needed(tty) != 0 {
        if (*(*tty).term).acs[ch as usize][0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == '\0' as i32
        {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        return (&raw mut *(&raw mut (*(*tty).term).acs as *mut [::core::ffi::c_char; 2])
            .offset(ch as isize) as *mut ::core::ffi::c_char)
            .offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
    }
    entry = bsearch(
        &raw mut ch as *const ::core::ffi::c_void,
        &raw const tty_acs_table as *const tty_acs_entry as *const ::core::ffi::c_void,
        (::core::mem::size_of::<[tty_acs_entry; 36]>() as size_t)
            .wrapping_div(::core::mem::size_of::<tty_acs_entry>() as size_t),
        ::core::mem::size_of::<tty_acs_entry>() as size_t,
        Some(
            tty_acs_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    ) as *const tty_acs_entry;
    if entry.is_null() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return (*entry).string.as_ptr();
}
#[no_mangle]
pub unsafe extern "C" fn tty_acs_reverse_get(
    mut tty: *mut tty,
    mut s: *const ::core::ffi::c_char,
    mut slen: size_t,
) -> ::core::ffi::c_int {
    let mut table: *const tty_acs_reverse_entry = ::core::ptr::null::<tty_acs_reverse_entry>();
    let mut entry: *const tty_acs_reverse_entry = ::core::ptr::null::<tty_acs_reverse_entry>();
    let mut items: u_int = 0;
    if slen == 2 as size_t {
        table = &raw const tty_acs_reverse2 as *const tty_acs_reverse_entry;
        items = (::core::mem::size_of::<[tty_acs_reverse_entry; 1]>() as usize)
            .wrapping_div(::core::mem::size_of::<tty_acs_reverse_entry>() as usize)
            as u_int;
    } else if slen == 3 as size_t {
        table = &raw const tty_acs_reverse3 as *const tty_acs_reverse_entry;
        items = (::core::mem::size_of::<[tty_acs_reverse_entry; 32]>() as usize)
            .wrapping_div(::core::mem::size_of::<tty_acs_reverse_entry>() as usize)
            as u_int;
    } else {
        return -(1 as ::core::ffi::c_int);
    }
    entry = bsearch(
        s as *const ::core::ffi::c_void,
        table as *const ::core::ffi::c_void,
        items as size_t,
        ::core::mem::size_of::<tty_acs_reverse_entry>() as size_t,
        Some(
            tty_acs_reverse_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    ) as *const tty_acs_reverse_entry;
    if entry.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    return (*entry).key as ::core::ffi::c_int;
}
