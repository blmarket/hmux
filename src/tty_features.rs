use crate::src::ffi::libc::{strcasecmp, strcmp, strlcat, strlen, strsep};
use crate::src::log::log_debug;
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
pub use crate::src::shared::client::CLIENT_UTF8;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
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
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
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
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::tty::{
    TERM_256COLOURS, TERM_DECFRA, TERM_DECSLRM, TERM_RGBCOLOURS, TERM_SIXEL,
};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::tty_term::{tty_term_apply, tty_term_has_name};
use std::ffi::CStr;

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_0;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_feature {
    pub name: *const ::core::ffi::c_char,
    pub capabilities: *const *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub name: *const ::core::ffi::c_char,
    pub version: u_int,
    pub features: *const ::core::ffi::c_char,
}
static mut tty_feature_title_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"tsl=\\E]0;\0" as *const u8 as *const ::core::ffi::c_char,
    b"fsl=\\a\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_title: tty_feature = unsafe {
    tty_feature {
        name: b"title\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_title_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_osc7_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"Swd=\\E]7;\0" as *const u8 as *const ::core::ffi::c_char,
    b"fsl=\\a\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_osc7: tty_feature = unsafe {
    tty_feature {
        name: b"osc7\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_osc7_capabilities as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_mouse_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"kmous=\\E[M\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_mouse: tty_feature = unsafe {
    tty_feature {
        name: b"mouse\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_mouse_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_clipboard_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Ms=\\E]52;%p1%s;%p2%s\\a\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_clipboard: tty_feature = unsafe {
    tty_feature {
        name: b"clipboard\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_clipboard_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_hyperlinks_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Hls=\\E]8;%?%p1%l%tid=%p1%s%;;%p2%s\\E\\\\\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_hyperlinks: tty_feature = unsafe {
    tty_feature {
        name: b"hyperlinks\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_hyperlinks_capabilities
            as *mut *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_rgb_capabilities: [*const ::core::ffi::c_char; 6] = [
    b"AX\0" as *const u8 as *const ::core::ffi::c_char,
    b"setrgbf=\\E[38;2;%p1%d;%p2%d;%p3%dm\0" as *const u8 as *const ::core::ffi::c_char,
    b"setrgbb=\\E[48;2;%p1%d;%p2%d;%p3%dm\0" as *const u8 as *const ::core::ffi::c_char,
    b"setab=\\E[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0" as *const u8
        as *const ::core::ffi::c_char,
    b"setaf=\\E[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0" as *const u8
        as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_rgb: tty_feature = unsafe {
    tty_feature {
        name: b"RGB\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_rgb_capabilities as *const *const ::core::ffi::c_char,
        flags: TERM_256COLOURS | TERM_RGBCOLOURS,
    }
};
static mut tty_feature_256_capabilities: [*const ::core::ffi::c_char; 4] = [
    b"AX\0" as *const u8 as *const ::core::ffi::c_char,
    b"setab=\\E[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0" as *const u8
        as *const ::core::ffi::c_char,
    b"setaf=\\E[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0" as *const u8
        as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_256: tty_feature = unsafe {
    tty_feature {
        name: b"256\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_256_capabilities as *const *const ::core::ffi::c_char,
        flags: TERM_256COLOURS,
    }
};
static mut tty_feature_overline_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Smol=\\E[53m\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_overline: tty_feature = unsafe {
    tty_feature {
        name: b"overline\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_overline_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_usstyle_capabilities: [*const ::core::ffi::c_char; 5] = [
    b"Smulx=\\E[4::%p1%dm\0" as *const u8 as *const ::core::ffi::c_char,
    b"Setulc=\\E[58::2::%p1%{65536}%/%d::%p1%{256}%/%{255}%&%d::%p1%{255}%&%d%;m\0" as *const u8
        as *const ::core::ffi::c_char,
    b"Setulc1=\\E[58::5::%p1%dm\0" as *const u8 as *const ::core::ffi::c_char,
    b"ol=\\E[59m\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_usstyle: tty_feature = unsafe {
    tty_feature {
        name: b"usstyle\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_usstyle_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_bpaste_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"Enbp=\\E[?2004h\0" as *const u8 as *const ::core::ffi::c_char,
    b"Dsbp=\\E[?2004l\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_bpaste: tty_feature = unsafe {
    tty_feature {
        name: b"bpaste\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_bpaste_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_focus_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"Enfcs=\\E[?1004h\0" as *const u8 as *const ::core::ffi::c_char,
    b"Dsfcs=\\E[?1004l\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_focus: tty_feature = unsafe {
    tty_feature {
        name: b"focus\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_focus_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_cstyle_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"Ss=\\E[%p1%d q\0" as *const u8 as *const ::core::ffi::c_char,
    b"Se=\\E[2 q\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_cstyle: tty_feature = unsafe {
    tty_feature {
        name: b"cstyle\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_cstyle_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_ccolour_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"Cs=\\E]12;%p1%s\\a\0" as *const u8 as *const ::core::ffi::c_char,
    b"Cr=\\E]112\\a\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_ccolour: tty_feature = unsafe {
    tty_feature {
        name: b"ccolour\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_ccolour_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_strikethrough_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"smxx=\\E[9m\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_strikethrough: tty_feature = unsafe {
    tty_feature {
        name: b"strikethrough\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_strikethrough_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_sync_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Sync=\\E[?2026%?%p1%{1}%-%tl%eh%;\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_sync: tty_feature = unsafe {
    tty_feature {
        name: b"sync\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_sync_capabilities as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_extkeys_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"Eneks=\\E[>4;2m\0" as *const u8 as *const ::core::ffi::c_char,
    b"Dseks=\\E[>4m\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_extkeys: tty_feature = unsafe {
    tty_feature {
        name: b"extkeys\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_extkeys_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_margins_capabilities: [*const ::core::ffi::c_char; 5] = [
    b"Enmg=\\E[?69h\0" as *const u8 as *const ::core::ffi::c_char,
    b"Dsmg=\\E[?69l\0" as *const u8 as *const ::core::ffi::c_char,
    b"Clmg=\\E[s\0" as *const u8 as *const ::core::ffi::c_char,
    b"Cmg=\\E[%i%p1%d;%p2%ds\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_margins: tty_feature = unsafe {
    tty_feature {
        name: b"margins\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_margins_capabilities
            as *const *const ::core::ffi::c_char,
        flags: TERM_DECSLRM,
    }
};
static mut tty_feature_rectfill_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Rect\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_rectfill: tty_feature = unsafe {
    tty_feature {
        name: b"rectfill\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_rectfill_capabilities
            as *const *const ::core::ffi::c_char,
        flags: TERM_DECFRA,
    }
};
static mut tty_feature_ignorefkeys_capabilities: [*const ::core::ffi::c_char; 65] = [
    b"kf0@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf1@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf2@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf3@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf4@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf5@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf6@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf7@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf8@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf9@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf10@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf11@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf12@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf13@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf14@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf15@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf16@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf17@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf18@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf19@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf20@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf21@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf22@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf23@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf24@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf25@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf26@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf27@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf28@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf29@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf30@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf31@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf32@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf33@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf34@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf35@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf36@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf37@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf38@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf39@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf40@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf41@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf42@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf43@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf44@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf45@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf46@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf47@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf48@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf49@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf50@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf51@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf52@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf53@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf54@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf55@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf56@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf57@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf58@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf59@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf60@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf61@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf62@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf63@\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_ignorefkeys: tty_feature = unsafe {
    tty_feature {
        name: b"ignorefkeys\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_ignorefkeys_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_sixel_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Sxl\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_sixel: tty_feature = unsafe {
    tty_feature {
        name: b"sixel\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_sixel_capabilities
            as *const *const ::core::ffi::c_char,
        flags: TERM_SIXEL,
    }
};
static mut tty_feature_progressbar_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Spb=\\E]9;4;%p1%d;%p2%d\\E\\\\\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_progressbar: tty_feature = unsafe {
    tty_feature {
        name: b"progressbar\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_progressbar_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_utf8: tty_feature = tty_feature {
    name: b"utf8\0" as *const u8 as *const ::core::ffi::c_char,
    capabilities: ::core::ptr::null::<*const ::core::ffi::c_char>(),
    flags: 0 as ::core::ffi::c_int,
};
static mut tty_features: [*const tty_feature; 22] = unsafe {
    [
        &raw const tty_feature_256,
        &raw const tty_feature_bpaste,
        &raw const tty_feature_ccolour,
        &raw const tty_feature_clipboard,
        &raw const tty_feature_hyperlinks,
        &raw const tty_feature_cstyle,
        &raw const tty_feature_extkeys,
        &raw const tty_feature_focus,
        &raw const tty_feature_ignorefkeys,
        &raw const tty_feature_margins,
        &raw const tty_feature_mouse,
        &raw const tty_feature_osc7,
        &raw const tty_feature_overline,
        &raw const tty_feature_progressbar,
        &raw const tty_feature_rectfill,
        &raw const tty_feature_rgb,
        &raw const tty_feature_sixel,
        &raw const tty_feature_strikethrough,
        &raw const tty_feature_sync,
        &raw const tty_feature_title,
        &raw const tty_feature_usstyle,
        &raw const tty_feature_utf8,
    ]
};
#[no_mangle]
pub unsafe extern "C" fn tty_parse_client_features(
    mut c: *mut client,
    mut s: *const ::core::ffi::c_char,
    mut sep: *const ::core::ffi::c_char,
) {
    tty_parse_features(
        s,
        sep,
        &raw mut (*c).term_features,
        &raw mut (*c).term_nofeatures,
    );
}
#[no_mangle]
pub unsafe extern "C" fn tty_parse_features(
    mut s: *const ::core::ffi::c_char,
    mut sep: *const ::core::ffi::c_char,
    mut enabled: *mut ::core::ffi::c_int,
    mut disabled: *mut ::core::ffi::c_int,
) {
    let mut tf: *const tty_feature = ::core::ptr::null::<tty_feature>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut loop_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut remove: ::core::ffi::c_int = 0;
    log_debug(
        b"adding terminal features %s\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    // strsep and the trailing-@ removal both write into this local copy.
    let mut copy = CStr::from_ptr(s).to_bytes_with_nul().to_vec();
    loop_0 = copy.as_mut_ptr().cast();
    loop {
        next = strsep(&raw mut loop_0, sep);
        if next.is_null() {
            break;
        }
        remove = (*next as ::core::ffi::c_int != '\0' as i32
            && *next.offset(strlen(next).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == '@' as i32) as ::core::ffi::c_int;
        if remove != 0 {
            *next.offset(strlen(next).wrapping_sub(1 as size_t) as isize) =
                '\0' as i32 as ::core::ffi::c_char;
        }
        i = 0 as u_int;
        while (i as usize)
            < (::core::mem::size_of::<[*const tty_feature; 22]>() as usize)
                .wrapping_div(::core::mem::size_of::<*const tty_feature>() as usize)
        {
            tf = tty_features[i as usize];
            if strcasecmp((*tf).name, next) == 0 as ::core::ffi::c_int {
                break;
            }
            i = i.wrapping_add(1);
        }
        if i as usize
            == (::core::mem::size_of::<[*const tty_feature; 22]>() as usize)
                .wrapping_div(::core::mem::size_of::<*const tty_feature>() as usize)
        {
            log_debug(
                b"unknown terminal feature: %s\0" as *const u8 as *const ::core::ffi::c_char,
                next,
            );
            break;
        } else if remove != 0 {
            log_debug(
                b"removing terminal feature: %s\0" as *const u8 as *const ::core::ffi::c_char,
                (*tf).name,
            );
            *enabled &= !((1 as ::core::ffi::c_int) << i);
            if !disabled.is_null() {
                *disabled |= (1 as ::core::ffi::c_int) << i;
            }
        } else {
            if !disabled.is_null() && *disabled & (1 as ::core::ffi::c_int) << i != 0 {
                continue;
            }
            if !*enabled & (1 as ::core::ffi::c_int) << i != 0 {
                log_debug(
                    b"adding terminal feature: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*tf).name,
                );
                *enabled |= (1 as ::core::ffi::c_int) << i;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn tty_get_features(
    mut feat: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut tf: *const tty_feature = ::core::ptr::null::<tty_feature>();
    static mut s: [::core::ffi::c_char; 512] = [0; 512];
    let mut i: u_int = 0;
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[*const tty_feature; 22]>() as usize)
            .wrapping_div(::core::mem::size_of::<*const tty_feature>() as usize)
    {
        if !(!feat & (1 as ::core::ffi::c_int) << i != 0) {
            tf = tty_features[i as usize];
            strlcat(
                &raw mut s as *mut ::core::ffi::c_char,
                (*tf).name,
                ::core::mem::size_of::<[::core::ffi::c_char; 512]>() as size_t,
            );
            strlcat(
                &raw mut s as *mut ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 512]>() as size_t,
            );
        }
        i = i.wrapping_add(1);
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
        s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
    }
    return &raw mut s as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn tty_feature_present(
    mut term: *mut tty_term,
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut tf: *const tty_feature = ::core::ptr::null::<tty_feature>();
    let mut capability: *const *const ::core::ffi::c_char =
        ::core::ptr::null::<*const ::core::ffi::c_char>();
    let mut i: u_int = 0;
    if strcmp(name, b"utf8\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return ((*(*(*term).tty).client).flags & CLIENT_UTF8 as uint64_t != 0 as uint64_t)
            as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[*const tty_feature; 22]>() as usize)
            .wrapping_div(::core::mem::size_of::<*const tty_feature>() as usize)
    {
        tf = tty_features[i as usize];
        if strcmp((*tf).name, name) == 0 as ::core::ffi::c_int {
            if (*term).applied_features & (1 as ::core::ffi::c_int) << i != 0 {
                return 1 as ::core::ffi::c_int;
            }
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if tf.is_null()
        || (*tf).capabilities.is_null()
        || strcmp(
            name,
            b"ignorefkeys\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if (*tf).flags != 0 as ::core::ffi::c_int && (*term).flags & (*tf).flags != (*tf).flags {
        return 0 as ::core::ffi::c_int;
    }
    capability = (*tf).capabilities;
    while !(*capability).is_null() {
        let mut copy = CStr::from_ptr(*capability).to_bytes_with_nul().to_vec();
        if let Some(equal) = copy.iter().position(|&byte| byte == b'=') {
            copy[equal] = 0;
        }
        if tty_term_has_name(term, copy.as_ptr().cast()) == 0 {
            return 0 as ::core::ffi::c_int;
        }
        capability = capability.offset(1);
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_apply_features(mut term: *mut tty_term) -> ::core::ffi::c_int {
    let mut c: *mut client = (*(*term).tty).client;
    let mut tf: *const tty_feature = ::core::ptr::null::<tty_feature>();
    let mut capability: *const *const ::core::ffi::c_char =
        ::core::ptr::null::<*const ::core::ffi::c_char>();
    let mut feat: ::core::ffi::c_int = 0;
    let mut i: u_int = 0;
    feat = (*c).term_features & !(*c).term_nofeatures;
    if feat == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"applying terminal features: %s\0" as *const u8 as *const ::core::ffi::c_char,
        tty_get_features(feat),
    );
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[*const tty_feature; 22]>() as usize)
            .wrapping_div(::core::mem::size_of::<*const tty_feature>() as usize)
    {
        if !((*term).applied_features & (1 as ::core::ffi::c_int) << i != 0
            || !feat & (1 as ::core::ffi::c_int) << i != 0)
        {
            tf = tty_features[i as usize];
            log_debug(
                b"applying terminal feature: %s\0" as *const u8 as *const ::core::ffi::c_char,
                (*tf).name,
            );
            if !(*tf).capabilities.is_null() {
                capability = (*tf).capabilities;
                while !(*capability).is_null() {
                    log_debug(
                        b"adding capability: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        *capability,
                    );
                    tty_term_apply(term, *capability, 1 as ::core::ffi::c_int);
                    capability = capability.offset(1);
                }
            }
            (*term).flags |= (*tf).flags;
            if tf == &raw const tty_feature_utf8 {
                (*c).flags |= CLIENT_UTF8 as uint64_t;
            }
        }
        i = i.wrapping_add(1);
    }
    if (*term).applied_features | feat == (*term).applied_features {
        return 0 as ::core::ffi::c_int;
    }
    (*term).applied_features |= feat;
    return 1 as ::core::ffi::c_int;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::shared::tty::{
        tty_code, TTYC_AX, TTYC_MS, TTYC_SETAB, TTYC_SETAF, TTYC_SETRGBB, TTYC_SETRGBF,
    };
    use crate::src::tty_term::{tty_term_ncodes, TTYCODE_FLAG, TTYCODE_NONE, TTYCODE_STRING};

    #[test]
    fn feature_presence_checks_capability_names_before_values() {
        unsafe {
            let mut codes = vec![std::mem::zeroed::<tty_code>(); tty_term_ncodes() as usize];
            let mut term = std::mem::zeroed::<tty_term>();
            term.codes = codes.as_mut_ptr();

            codes[TTYC_MS as usize].type_0 = TTYCODE_STRING;
            assert_eq!(tty_feature_present(&mut term, c"clipboard".as_ptr()), 1);
            codes[TTYC_MS as usize].type_0 = TTYCODE_NONE;
            assert_eq!(tty_feature_present(&mut term, c"clipboard".as_ptr()), 0);

            term.flags = TERM_256COLOURS | TERM_RGBCOLOURS;
            codes[TTYC_AX as usize].type_0 = TTYCODE_FLAG;
            for code in [TTYC_SETRGBF, TTYC_SETRGBB, TTYC_SETAB, TTYC_SETAF] {
                codes[code as usize].type_0 = TTYCODE_STRING;
            }
            assert_eq!(tty_feature_present(&mut term, c"RGB".as_ptr()), 1);
            codes[TTYC_SETAF as usize].type_0 = TTYCODE_NONE;
            assert_eq!(tty_feature_present(&mut term, c"RGB".as_ptr()), 0);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn tty_default_features(
    mut c: *mut client,
    mut name: *const ::core::ffi::c_char,
    mut version: u_int,
) {
    static mut table: [C2RustUnnamed_35; 9] = [
        C2RustUnnamed_35 {
            name: b"mintty\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,margins,overline,usstyle\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"tmux\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,focus,overline,usstyle,hyperlinks,progressbar\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"rxvt-unicode\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,bpaste,ccolour,cstyle,mouse,title,ignorefkeys\0" as *const u8
                as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"iTerm2\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,cstyle,extkeys,margins,usstyle,sync,osc7,hyperlinks,progressbar\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"foot\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,usstyle,sync,osc7,hyperlinks\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"WezTerm\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,focus,hyperlinks,usstyle\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"ghostty\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,focus,overline,hyperlinks,osc7,sync,usstyle,progressbar\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"Rio\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,focus,overline,hyperlinks,osc7,sync,usstyle,progressbar\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"XTerm\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,focus\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
    ];
    let mut i: u_int = 0;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_35; 9]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
    {
        if !(strcmp(table[i as usize].name, name) != 0 as ::core::ffi::c_int) {
            if !(version != 0 as u_int && version < table[i as usize].version) {
                tty_parse_client_features(
                    c,
                    table[i as usize].features,
                    b",\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        i = i.wrapping_add(1);
    }
}
