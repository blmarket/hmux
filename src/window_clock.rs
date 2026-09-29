use crate::src::ffi::libc::{
    clock_gettime, gmtime_r, localtime, memcpy, strftime, strlcat, strlen, time,
};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{format_create_defaults, format_free};
use crate::src::grid::grid_default_cell;
use crate::src::options::options_get_number;
use crate::src::options::options_owner_ptr;
use crate::src::reactor::{event_add, event_del, event_set};
use crate::src::screen::{screen_free, screen_init, screen_resize};
use crate::src::screen_write::{
    screen_write_clearscreen, screen_write_cursormove, screen_write_putc, screen_write_puts,
    screen_write_start, screen_write_stop,
};
use crate::src::shared::abi::__syscall_slong_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::client;
use crate::src::shared::client::ClientRef;
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::event::*;
use crate::src::shared::format::format_tree;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_REDRAW;
use crate::src::shared::screen::{screen, MODE_CURSOR};
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::time::{timespec, tm, CLOCK_REALTIME};
use crate::src::shared::window::{window, window_mode, window_mode_entry, winlink};
use crate::src::style::style_apply;
use crate::src::window::{window_pane_mode_weak, window_pane_reset_mode};

#[repr(C)]
pub struct window_clock_mode_data {
    pub screen: screen,
    pub tim: time_t,
    pub timer: event,
}
pub static window_clock_mode: window_mode = {
    window_mode {
        name: c"clock-mode",
        default_format: None,
        flags: 0,
        init: Some(
            window_clock_init
                as unsafe fn(
                    refbox::Weak<window_mode_entry>,
                    Option<&std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>>,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_clock_free as unsafe fn(refbox::Weak<window_mode_entry>) -> ()),
        resize: Some(
            window_clock_resize as unsafe fn(refbox::Weak<window_mode_entry>, u_int, u_int) -> (),
        ),
        update: None,
        style_changed: None,
        key: Some(
            window_clock_key
                as unsafe fn(
                    refbox::Weak<window_mode_entry>,
                    &ClientRef,
                    refbox::Weak<winlink>,
                    key_code,
                    *mut mouse_event,
                ) -> (),
        ),
        key_table: None,
        command: None,
        formats: None,
        get_screen: None,
        display_screen: Some(window_clock_get_screen),
    }
};
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
unsafe fn window_clock_data(wme: refbox::Weak<window_mode_entry>) -> *mut window_clock_mode_data {
    wme.get_unchecked()
        .boxed_data_ptr::<window_clock_mode_data>()
        .expect("clock mode payload")
}

unsafe fn window_clock_start_timer(mut wme: refbox::Weak<window_mode_entry>) {
    let mut data: *mut window_clock_mode_data = window_clock_data(wme.clone());
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
unsafe fn window_clock_timer_callback(wme: refbox::Weak<window_mode_entry>) {
    let mode_pane_owner = wme
        .get_unchecked()
        .wp
        .upgrade()
        .expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_clock_mode_data = window_clock_data(wme.clone());
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
        window_clock_draw_screen(wme.clone());
        (*wp).flags |= PANE_REDRAW;
    }
    window_clock_start_timer(wme.clone());
}
unsafe fn window_clock_init(
    mut wme: refbox::Weak<window_mode_entry>,
    _item_handle: Option<&std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>>,
    _fs: *mut cmd_find_state,
    _args: *mut args,
) -> *mut screen {
    let mode_pane_owner = wme
        .get_unchecked()
        .wp
        .upgrade()
        .expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_clock_mode_data = ::core::ptr::null_mut::<window_clock_mode_data>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let owner = Box::new(std::cell::UnsafeCell::new(window_clock_mode_data {
        screen: screen::empty(),
        tim: 0,
        timer: Default::default(),
    }));
    data = owner.get();
    wme.get_mut_unchecked().boxed_data = Some(owner);
    (*data).tim = time(::core::ptr::null_mut::<time_t>());
    let mode_observer = window_pane_mode_weak(wme.clone());
    event_set(
        &raw mut (*data).timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        move |_, _| unsafe {
            match mode_observer.try_borrow_mut() {
                Ok(_mode) => window_clock_timer_callback(mode_observer.clone()),
                Err(refbox::BorrowError::Dropped) => {}
                Err(refbox::BorrowError::Borrowed) => panic!("clock mode already borrowed"),
            }
        },
    );
    window_clock_start_timer(wme.clone());
    s = &raw mut (*data).screen;
    screen_init(
        &mut *s,
        (*wp).base.grid().sx,
        (*wp).base.grid().sy,
        0 as u_int,
    );
    (*s).mode &= !MODE_CURSOR;
    window_clock_draw_screen(wme.clone());
    return s;
}
unsafe fn window_clock_get_screen(wme: refbox::Weak<window_mode_entry>) -> *mut screen {
    let data = wme
        .get_unchecked()
        .boxed_data_ptr::<window_clock_mode_data>();
    data.map_or(std::ptr::null_mut(), |data| &raw mut (*data).screen)
}

unsafe fn window_clock_free(mut wme: refbox::Weak<window_mode_entry>) {
    let mut data: *mut window_clock_mode_data = window_clock_data(wme.clone());
    event_del(&raw mut (*data).timer);
    screen_free(&mut (*data).screen);
    drop(wme.get_mut_unchecked().boxed_data.take());
}
unsafe fn window_clock_resize(
    mut wme: refbox::Weak<window_mode_entry>,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_clock_mode_data = window_clock_data(wme.clone());
    let mut s: *mut screen = &raw mut (*data).screen;
    screen_resize(&mut *s, sx, sy, 0 as ::core::ffi::c_int);
    window_clock_draw_screen(wme.clone());
}
unsafe fn window_clock_key(
    mut wme: refbox::Weak<window_mode_entry>,
    _client_owner: &ClientRef,
    _wl: refbox::Weak<winlink>,
    _key: key_code,
    _m: *mut mouse_event,
) {
    let mode_pane_owner = wme
        .get_unchecked()
        .wp
        .upgrade()
        .expect("mode belongs to a live pane");
    let _mode_pane = mode_pane_owner.get();
    window_pane_reset_mode(&mode_pane_owner);
}
unsafe fn window_clock_draw_screen(mut wme: refbox::Weak<window_mode_entry>) {
    let mode_pane_owner = wme
        .get_unchecked()
        .wp
        .upgrade()
        .expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut w: *mut window = (*wp)
        .window_handle()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut data: *mut window_clock_mode_data = window_clock_data(wme.clone());
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        target: Default::default(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
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
    let mut ft_owner = format_create_defaults(
        None,
        None,
        None,
        (refbox::Weak::new()).clone(),
        (wp).as_ref()
            .and_then(|model| model.observer.upgrade())
            .as_ref(),
    );
    ft = &raw mut *ft_owner;
    style_apply(
        &raw mut gc,
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"clock-mode-colour\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    format_free(ft_owner);
    colour = gc.fg;
    style = options_get_number(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"clock-mode-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    screen_write_start(&mut ctx, s);
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
    screen_write_clearscreen(&mut ctx, 8 as u_int);
    if ((*s).grid().sx as size_t)
        < (6 as size_t).wrapping_mul(strlen(&raw mut tim as *mut ::core::ffi::c_char))
        || (*s).grid().sy < 6 as u_int
    {
        if (*s).grid().sx as size_t >= strlen(&raw mut tim as *mut ::core::ffi::c_char)
            && (*s).grid().sy != 0 as u_int
        {
            x = ((*s).grid().sx.wrapping_div(2 as u_int) as size_t).wrapping_sub(
                strlen(&raw mut tim as *mut ::core::ffi::c_char).wrapping_div(2 as size_t),
            ) as u_int;
            y = (*s).grid().sy.wrapping_div(2 as u_int);
            screen_write_cursormove(
                &mut ctx,
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
            screen_write_puts(&mut ctx, &gc, |out| {
                write_cstr(out, &raw mut tim as *mut ::core::ffi::c_char)
            });
        }
        screen_write_stop(&mut ctx);
        return;
    }
    x = ((*s).grid().sx.wrapping_div(2 as u_int) as size_t)
        .wrapping_sub((3 as size_t).wrapping_mul(strlen(&raw mut tim as *mut ::core::ffi::c_char)))
        as u_int;
    y = (*s)
        .grid()
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
                            &mut ctx,
                            x.wrapping_add(i) as ::core::ffi::c_int,
                            y.wrapping_add(j) as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        if window_clock_table[idx as usize][j as usize][i as usize] != 0 {
                            screen_write_putc(&mut ctx, &gc, '#' as i32 as u_char);
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
    screen_write_stop(&mut ctx);
}
