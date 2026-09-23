use crate::src::ffi::libc::{
    clock_gettime, gmtime_r, localtime, memcpy, strftime, strlcat, strlen, time,
};
use crate::src::format::{format_create_defaults, format_free};
use crate::src::grid::grid_default_cell;
use crate::src::options::options_get_number;
use crate::src::reactor::{event_add, event_del, event_set};
use crate::src::screen::{screen_free, screen_init, screen_resize};
use crate::src::screen_write::{
    screen_write_clearscreen, screen_write_cursormove, screen_write_putc, screen_write_puts,
    screen_write_start, screen_write_stop,
};
use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{__clockid_t, __syscall_slong_t, clockid_t};
pub use crate::src::shared::arguments::args;
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
    PANE_REDRAW,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles, MODE_CURSOR};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::screen_write::{screen_write_ctx, screen_write_init_ctx_cb};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::time::{timespec, tm, CLOCK_REALTIME};
pub use crate::src::shared::tty::{
    tty, tty_code, tty_ctx, tty_ctx_c2rust_unnamed, tty_ctx_c2rust_unnamed_data,
    tty_ctx_c2rust_unnamed_sel, tty_ctx_redraw_cb, tty_ctx_set_client_cb, tty_key, tty_style_ctx,
    tty_term, tty_term_entry,
};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::style::style_apply;
use crate::src::window::window_pane_reset_mode;

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[repr(C)]
pub struct window_clock_mode_data {
    pub screen: screen,
    pub tim: time_t,
    pub timer: event,
}

#[no_mangle]
pub static mut window_clock_mode: window_mode = unsafe {
    window_mode {
        name: b"clock-mode\0" as *const u8 as *const ::core::ffi::c_char,
        default_format: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
        init: Some(
            window_clock_init
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_clock_free as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        resize: Some(
            window_clock_resize as unsafe extern "C" fn(*mut window_mode_entry, u_int, u_int) -> (),
        ),
        update: None,
        style_changed: None,
        key: Some(
            window_clock_key
                as unsafe extern "C" fn(
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
#[no_mangle]
pub static mut window_clock_table: [[[::core::ffi::c_char; 5]; 5]; 14] = [
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
];
unsafe extern "C" fn window_clock_start_timer(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_clock_mode_data = (*wme).data as *mut window_clock_mode_data;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut ts: timespec = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let mut delay: ::core::ffi::c_long = 0;
    clock_gettime(CLOCK_REALTIME, &raw mut ts);
    delay = (1000000 as __syscall_slong_t - ts.tv_nsec / 1000 as __syscall_slong_t)
        as ::core::ffi::c_long;
    tv.tv_sec = (delay / 1000000 as ::core::ffi::c_long) as __time_t;
    tv.tv_usec = (delay % 1000000 as ::core::ffi::c_long) as __suseconds_t;
    if tv.tv_sec < 0 as __time_t || tv.tv_sec == 0 as __time_t && tv.tv_usec <= 0 as __suseconds_t {
        tv.tv_sec = 1 as __time_t;
        tv.tv_usec = 0 as __suseconds_t;
    }
    event_add(&raw mut (*data).timer, &raw mut tv);
}
unsafe extern "C" fn window_clock_timer_callback(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut wme: *mut window_mode_entry = arg as *mut window_mode_entry;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_clock_mode_data = (*wme).data as *mut window_clock_mode_data;
    let mut now: tm = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut then: tm = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut t: time_t = 0;
    event_del(&raw mut (*data).timer);
    t = time(::core::ptr::null_mut::<time_t>());
    gmtime_r(&raw mut t, &raw mut now);
    gmtime_r(&raw mut (*data).tim, &raw mut then);
    if now.tm_sec != then.tm_sec {
        (*data).tim = t;
        window_clock_draw_screen(wme);
        (*wp).flags |= PANE_REDRAW;
    }
    window_clock_start_timer(wme);
}
unsafe extern "C" fn window_clock_init(
    mut wme: *mut window_mode_entry,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_clock_mode_data = ::core::ptr::null_mut::<window_clock_mode_data>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    data = Box::into_raw(Box::new(window_clock_mode_data {
        screen: ::core::mem::zeroed(),
        tim: 0,
        timer: ::core::mem::zeroed(),
    }));
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).tim = time(::core::ptr::null_mut::<time_t>());
    event_set(
        &raw mut (*data).timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            window_clock_timer_callback
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        wme as *mut ::core::ffi::c_void,
    );
    window_clock_start_timer(wme);
    s = &raw mut (*data).screen;
    screen_init(s, (*(*wp).base.grid).sx, (*(*wp).base.grid).sy, 0 as u_int);
    (*s).mode &= !MODE_CURSOR;
    window_clock_draw_screen(wme);
    return s;
}
unsafe extern "C" fn window_clock_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_clock_mode_data = (*wme).data as *mut window_clock_mode_data;
    event_del(&raw mut (*data).timer);
    screen_free(&raw mut (*data).screen);
    drop(Box::from_raw(data));
}
unsafe extern "C" fn window_clock_resize(
    mut wme: *mut window_mode_entry,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_clock_mode_data = (*wme).data as *mut window_clock_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    screen_resize(s, sx, sy, 0 as ::core::ffi::c_int);
    window_clock_draw_screen(wme);
}
unsafe extern "C" fn window_clock_key(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    window_pane_reset_mode((*wme).wp);
}
unsafe extern "C" fn window_clock_draw_screen(mut wme: *mut window_mode_entry) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut w: *mut window = (*wp).window as *mut window;
    let mut data: *mut window_clock_mode_data = (*wme).data as *mut window_clock_mode_data;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    };
    let mut colour: ::core::ffi::c_int = 0;
    let mut style: ::core::ffi::c_int = 0;
    let mut s: *mut screen = &raw mut (*data).screen;
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
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut tim: [::core::ffi::c_char; 64] = [0; 64];
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut t: time_t = 0;
    let mut tm: *mut tm = ::core::ptr::null_mut::<tm>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut idx: u_int = 0;
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        wp,
    );
    style_apply(
        &raw mut gc,
        (*w).options,
        b"clock-mode-colour\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    format_free(ft);
    colour = gc.fg;
    style = options_get_number(
        (*w).options,
        b"clock-mode-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    screen_write_start(&raw mut ctx, s);
    t = time(::core::ptr::null_mut::<time_t>());
    tm = localtime(&raw mut t);
    if style == 0 as ::core::ffi::c_int || style == 2 as ::core::ffi::c_int {
        if style == 2 as ::core::ffi::c_int {
            strftime(
                &raw mut tim as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                b"%l:%M:%S \0" as *const u8 as *const ::core::ffi::c_char,
                localtime(&raw mut t),
            );
        } else {
            strftime(
                &raw mut tim as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                b"%l:%M \0" as *const u8 as *const ::core::ffi::c_char,
                localtime(&raw mut t),
            );
        }
        if (*tm).tm_hour >= 12 as ::core::ffi::c_int {
            strlcat(
                &raw mut tim as *mut ::core::ffi::c_char,
                b"PM\0" as *const u8 as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
            );
        } else {
            strlcat(
                &raw mut tim as *mut ::core::ffi::c_char,
                b"AM\0" as *const u8 as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
            );
        }
    } else if style == 3 as ::core::ffi::c_int {
        strftime(
            &raw mut tim as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
            b"%H:%M:%S\0" as *const u8 as *const ::core::ffi::c_char,
            tm,
        );
    } else {
        strftime(
            &raw mut tim as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
            b"%H:%M\0" as *const u8 as *const ::core::ffi::c_char,
            tm,
        );
    }
    screen_write_clearscreen(&raw mut ctx, 8 as u_int);
    if ((*(*s).grid).sx as size_t)
        < (6 as size_t).wrapping_mul(strlen(&raw mut tim as *mut ::core::ffi::c_char))
        || (*(*s).grid).sy < 6 as u_int
    {
        if (*(*s).grid).sx as size_t >= strlen(&raw mut tim as *mut ::core::ffi::c_char)
            && (*(*s).grid).sy != 0 as u_int
        {
            x = ((*(*s).grid).sx.wrapping_div(2 as u_int) as size_t).wrapping_sub(
                strlen(&raw mut tim as *mut ::core::ffi::c_char).wrapping_div(2 as size_t),
            ) as u_int;
            y = (*(*s).grid).sy.wrapping_div(2 as u_int);
            screen_write_cursormove(
                &raw mut ctx,
                x as ::core::ffi::c_int,
                y as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            memcpy(
                &raw mut gc as *mut ::core::ffi::c_void,
                &raw const grid_default_cell as *const ::core::ffi::c_void,
                ::core::mem::size_of::<grid_cell>() as size_t,
            );
            gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
            gc.fg = colour;
            screen_write_puts(
                &raw mut ctx,
                &raw mut gc,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut tim as *mut ::core::ffi::c_char,
            );
        }
        screen_write_stop(&raw mut ctx);
        return;
    }
    x = ((*(*s).grid).sx.wrapping_div(2 as u_int) as size_t)
        .wrapping_sub((3 as size_t).wrapping_mul(strlen(&raw mut tim as *mut ::core::ffi::c_char)))
        as u_int;
    y = (*(*s).grid)
        .sy
        .wrapping_div(2 as u_int)
        .wrapping_sub(3 as u_int);
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    gc.bg = colour;
    gc.fg = colour;
    let mut current_block_56: u64;
    ptr = &raw mut tim as *mut ::core::ffi::c_char;
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        if *ptr as ::core::ffi::c_int >= '0' as i32 && *ptr as ::core::ffi::c_int <= '9' as i32 {
            idx = (*ptr as ::core::ffi::c_int - '0' as i32) as u_int;
            current_block_56 = 2543120759711851213;
        } else if *ptr as ::core::ffi::c_int == ':' as i32 {
            idx = 10 as u_int;
            current_block_56 = 2543120759711851213;
        } else if *ptr as ::core::ffi::c_int == 'A' as i32 {
            idx = 11 as u_int;
            current_block_56 = 2543120759711851213;
        } else if *ptr as ::core::ffi::c_int == 'P' as i32 {
            idx = 12 as u_int;
            current_block_56 = 2543120759711851213;
        } else if *ptr as ::core::ffi::c_int == 'M' as i32 {
            idx = 13 as u_int;
            current_block_56 = 2543120759711851213;
        } else {
            x = x.wrapping_add(6 as u_int);
            current_block_56 = 11913429853522160501;
        }
        match current_block_56 {
            2543120759711851213 => {
                j = 0 as u_int;
                while j < 5 as u_int {
                    i = 0 as u_int;
                    while i < 5 as u_int {
                        screen_write_cursormove(
                            &raw mut ctx,
                            x.wrapping_add(i) as ::core::ffi::c_int,
                            y.wrapping_add(j) as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        if window_clock_table[idx as usize][j as usize][i as usize] != 0 {
                            screen_write_putc(&raw mut ctx, &raw mut gc, '#' as i32 as u_char);
                        }
                        i = i.wrapping_add(1);
                    }
                    j = j.wrapping_add(1);
                }
                x = x.wrapping_add(6 as u_int);
            }
            _ => {}
        }
        ptr = ptr.offset(1);
    }
    screen_write_stop(&raw mut ctx);
}
