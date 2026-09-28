use crate::src::options::options_owner_ptr;
use crate::src::arguments::args_parse;
use crate::src::arguments::{args_count, args_has, args_string};
use crate::src::cmd::{cmd_mouse_at, cmd_mouse_pane};
use crate::src::compat::strtonum::strtonum;
use crate::src::events::events_fire_pane;
use crate::src::ffi::libc::{
    __ctype_tolower_loc, abs, llabs, memcmp, memcpy, strcasecmp, strchr, strcmp, strcspn, strlen,
    strncmp,
};
use crate::src::ffi::regex::{CompiledRegex, RegexMatch, RegexStorage};
use crate::src::format::bytes::format_message_with;
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_add_owned_cb, format_create_defaults, format_expand_cstring, format_free,
    format_get_pane, format_grid_hyperlink_cstring, format_single_cstring,
};
use crate::src::format_draw::format_draw;
use crate::src::grid::reader::{
    grid_reader_cursor_back_to_indentation, grid_reader_cursor_end_of_line,
    grid_reader_cursor_jump, grid_reader_cursor_jump_back, grid_reader_cursor_left,
    grid_reader_cursor_next_word, grid_reader_cursor_next_word_end,
    grid_reader_cursor_previous_word, grid_reader_cursor_right, grid_reader_cursor_start_of_line,
    grid_reader_get_cursor, grid_reader_in_set, grid_reader_start,
};
use crate::src::grid::{
    grid_default_cell, grid_duplicate_lines, grid_free_lines, grid_get_cell, grid_get_line,
    grid_in_set, grid_line_length, grid_line_limit, grid_line_time, grid_peek_line,
    grid_unwrap_position, grid_wrap_position,
};
use crate::src::input::{input_free, input_init, input_parse_screen};
use crate::src::job::{job_get_event, job_run};
use crate::src::log::{fatal, fatalx, log_cstr, log_debug};
use crate::src::options::{options_get_number, options_get_string};
use crate::src::paste::{paste_add_owned, paste_buffer_data, paste_get_top, paste_set_owned};
use crate::src::reactor::{bufferevent_write, event_add, event_del, event_set};
use crate::src::screen::screen_share_hyperlinks;
use crate::src::screen::{
    screen_check_selection, screen_clear_selection, screen_free, screen_hide_selection,
    screen_init, screen_resize, screen_resize_cursor, screen_set_default_cursor,
    screen_set_selection,
};
use crate::src::screen_write::{
    screen_write_carriagereturn, screen_write_cell, screen_write_cursormove,
    screen_write_deleteline, screen_write_insertline, screen_write_linefeed, screen_write_nputs,
    screen_write_putc, screen_write_setselection, screen_write_start, screen_write_start_pane,
    screen_write_stop, screen_write_strlen,
};
use crate::src::shared::abi::*;
use crate::src::shared::abi::{__int32_t, ssize_t};
use crate::src::shared::arguments::args;
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_READONLY;
use crate::src::shared::colour::*;
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
use crate::src::shared::format::format_tree;
use crate::src::shared::grid::WHITESPACE;
use crate::src::shared::grid::*;
use crate::src::shared::input::input_ctx;
use crate::src::shared::job::job;
use crate::src::shared::job::JOB_NOWAIT;
use crate::src::shared::key::{MODEKEY_EMACS, MODEKEY_VI};
use crate::src::shared::limits::{INT_MAX, UCHAR_MAX, UINT_MAX};
use crate::src::shared::mouse::{
    mouse_event, MOUSE_MASK_BUTTONS, MOUSE_WHEEL_DOWN, MOUSE_WHEEL_UP,
};
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{PANE_REDRAW, PANE_REDRAWSCROLLBAR, PANE_UNSEENCHANGES};
use crate::src::shared::screen::screen;
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::style::*;
use crate::src::shared::tty::{tty, tty_ctx};
pub use crate::src::shared::window::{window, window_mode, window_mode_entry, winlink};
use crate::src::status::status_message_set;
use crate::src::style::style_apply;
use crate::src::text::utf8::{utf8_copy, utf8_fromcstr_vec, utf8_set, utf8_to_data};
use crate::src::tmux::{get_timer, global_options, global_w_options};
use crate::src::tty::tty_window_offset;
use crate::src::tty_acs::tty_acs_get;
use crate::src::window::{
    window_pane_reset_mode, window_pane_scrollbar_overlay_visible, window_pane_scrollbar_redraw,
    window_pane_scrollbar_show, window_pane_set_searchstr, window_set_active_pane,
};
use libc::{REG_EXTENDED, REG_ICASE};
use std::borrow::Cow;
use std::ffi::{CStr, CString};

#[repr(C)]
pub struct window_copy_mode_data {
    pub screen: screen,
    /// Owned snapshot for copy mode, or command output for view mode.
    backing: Option<Box<screen>>,
    pub backing_written: ::core::ffi::c_int,
    pub ictx: Option<Box<input_ctx>>,
    pub sync_added: u_int,
    pub sync_collected: u_int,
    pub sync_generation: u_int,
    pub viewmode: ::core::ffi::c_int,
    pub oy: u_int,
    pub selx: u_int,
    pub sely: u_int,
    pub endselx: u_int,
    pub endsely: u_int,
    pub cursordrag: C2RustUnnamed_45,
    pub modekeys: ::core::ffi::c_int,
    pub lineflag: C2RustUnnamed_44,
    pub rectflag: ::core::ffi::c_int,
    pub scroll_exit: ::core::ffi::c_int,
    pub hide_position: ::core::ffi::c_int,
    pub line_numbers: ::core::ffi::c_int,
    pub selflag: C2RustUnnamed_43,
    pub recentre_state: C2RustUnnamed_42,
    pub recentre_line: u_int,
    pub separators: Option<CString>,
    pub dx: u_int,
    pub dy: u_int,
    pub selrx: u_int,
    pub selry: u_int,
    pub endselrx: u_int,
    pub endselry: u_int,
    pub cx: u_int,
    pub cy: u_int,
    pub lastcx: u_int,
    pub lastsx: u_int,
    pub mx: u_int,
    pub my: u_int,
    pub showmark: ::core::ffi::c_int,
    pub searchtype: ::core::ffi::c_int,
    pub searchdirection: ::core::ffi::c_int,
    pub searchregex: ::core::ffi::c_int,
    searchstr: Option<CString>,
    /// Empty when no search marks are active; retains capacity between searches.
    pub searchmark: Vec<u8>,
    pub searchcount: ::core::ffi::c_int,
    pub searchmore: ::core::ffi::c_int,
    pub searchall: ::core::ffi::c_int,
    pub searchx: ::core::ffi::c_int,
    pub searchy: ::core::ffi::c_int,
    pub searcho: ::core::ffi::c_int,
    pub searchgen: u_char,
    pub timeout: ::core::ffi::c_int,
    pub jumptype: ::core::ffi::c_int,
    pub jumpchar: Vec<utf8_data>,
    pub dragtimer: event,
    pub refresh_timer: event,
    pub refresh_active: ::core::ffi::c_int,
}

impl window_copy_mode_data {
    fn backing(&self) -> &screen {
        self.backing
            .as_deref()
            .expect("copy backing is initialized")
    }

    fn backing_mut(&mut self) -> &mut screen {
        self.backing
            .as_deref_mut()
            .expect("copy backing is initialized")
    }

    fn clear_backing(&mut self) {
        if let Some(mut backing) = self.backing.take() {
            unsafe { screen_free(&mut backing) };
        }
    }
}

impl Drop for window_copy_mode_data {
    fn drop(&mut self) {
        drop(self.ictx.take());
        self.clear_backing();
    }
}

impl Default for window_copy_mode_data {
    fn default() -> Self {
        Self {
            screen: screen::empty(),
            backing: None,
            backing_written: 0,
            ictx: None,
            sync_added: 0,
            sync_collected: 0,
            sync_generation: 0,
            viewmode: 0,
            oy: 0,
            selx: 0,
            sely: 0,
            endselx: 0,
            endsely: 0,
            cursordrag: CURSORDRAG_NONE,
            modekeys: 0,
            lineflag: LINE_SEL_NONE,
            rectflag: 0,
            scroll_exit: 0,
            hide_position: 0,
            line_numbers: 0,
            selflag: SEL_CHAR,
            recentre_state: RECENTRE_TOP,
            recentre_line: 0,
            separators: None,
            dx: 0,
            dy: 0,
            selrx: 0,
            selry: 0,
            endselrx: 0,
            endselry: 0,
            cx: 0,
            cy: 0,
            lastcx: 0,
            lastsx: 0,
            mx: 0,
            my: 0,
            showmark: 0,
            searchtype: 0,
            searchdirection: 0,
            searchregex: 0,
            searchstr: None,
            searchmark: Vec::new(),
            searchcount: 0,
            searchmore: 0,
            searchall: 0,
            searchx: 0,
            searchy: 0,
            searcho: 0,
            searchgen: 0,
            timeout: 0,
            jumptype: 0,
            jumpchar: Vec::new(),
            dragtimer: event::new(),
            refresh_timer: event::new(),
            refresh_active: 0,
        }
    }
}

fn window_copy_searchstr(data: &window_copy_mode_data) -> *const ::core::ffi::c_char {
    data.searchstr
        .as_ref()
        .map_or(::core::ptr::null(), |value| value.as_ptr())
}

fn window_copy_separators(data: &window_copy_mode_data) -> *const ::core::ffi::c_char {
    data.separators
        .as_ref()
        .map_or(::core::ptr::null(), |value| value.as_ptr())
}

unsafe fn window_copy_snapshot_separators(so: *mut options) -> Option<CString> {
    let separators = options_get_string(so, c"word-separators".as_ptr());
    (!separators.is_null()).then(|| CStr::from_ptr(separators).to_owned())
}

#[cfg(test)]
mod separator_snapshot_tests {
    use super::*;
    use crate::src::options::{options_create_owned, options_default, options_set_string};
    use crate::src::options_table::options_table;

    #[test]
    fn selected_word_separators_survive_option_replacement() {
        unsafe {
            let mut options = options_create_owned(std::ptr::null_mut());
            let so = &mut *options as *mut options;
            let definition = options_table
                .iter()
                .find(|entry| entry.name == Some(c"word-separators"))
                .unwrap();
            options_default(so, definition);
            options_set_string(so, c"word-separators".as_ptr(), 0, |out| out.write_all(b"original"));
            let mut data = window_copy_mode_data::default();
            data.separators = window_copy_snapshot_separators(so);
            options_set_string(so, c"word-separators".as_ptr(), 0, |out| out.write_all(b"replacement"));
            assert_eq!(CStr::from_ptr(window_copy_separators(&data)), c"original");
        }
    }
}
pub type C2RustUnnamed_42 = ::core::ffi::c_uint;
pub const RECENTRE_BOTTOM: C2RustUnnamed_42 = 2;
pub const RECENTRE_MIDDLE: C2RustUnnamed_42 = 1;
pub const RECENTRE_TOP: C2RustUnnamed_42 = 0;
pub type C2RustUnnamed_43 = ::core::ffi::c_uint;
pub const SEL_LINE: C2RustUnnamed_43 = 2;
pub const SEL_WORD: C2RustUnnamed_43 = 1;
pub const SEL_CHAR: C2RustUnnamed_43 = 0;
pub type C2RustUnnamed_44 = ::core::ffi::c_uint;
pub const LINE_SEL_RIGHT_LEFT: C2RustUnnamed_44 = 2;
pub const LINE_SEL_LEFT_RIGHT: C2RustUnnamed_44 = 1;
pub const LINE_SEL_NONE: C2RustUnnamed_44 = 0;
pub type C2RustUnnamed_45 = ::core::ffi::c_uint;
pub const CURSORDRAG_SEL: C2RustUnnamed_45 = 2;
pub const CURSORDRAG_ENDSEL: C2RustUnnamed_45 = 1;
pub const CURSORDRAG_NONE: C2RustUnnamed_45 = 0;
pub const WINDOW_COPY_LINE_NUMBERS_OFF: window_copy_line_numbers = 0;
pub const WINDOW_COPY_LINE_NUMBERS_DEFAULT: window_copy_line_numbers = 1;
pub const WINDOW_COPY_LINE_NUMBERS_HYBRID: window_copy_line_numbers = 4;
pub const WINDOW_COPY_LINE_NUMBERS_ABSOLUTE: window_copy_line_numbers = 2;
pub const WINDOW_COPY_CMD_MOVE: window_copy_cmd_action = 1;
pub type window_copy_cmd_action = ::core::ffi::c_uint;
pub const WINDOW_COPY_CMD_CANCEL: window_copy_cmd_action = 3;
pub const WINDOW_COPY_CMD_REDRAW: window_copy_cmd_action = 2;
pub const WINDOW_COPY_CMD_NOTHING: window_copy_cmd_action = 0;
pub const WINDOW_COPY_CMD_CLEAR_NEVER: window_copy_cmd_clear = 1;
pub type window_copy_cmd_clear = ::core::ffi::c_uint;
pub const WINDOW_COPY_CMD_CLEAR_EMACS_ONLY: window_copy_cmd_clear = 2;
pub const WINDOW_COPY_CMD_CLEAR_ALWAYS: window_copy_cmd_clear = 0;
pub struct window_copy_cmd_state<'a> {
    pub wme: *mut window_mode_entry,
    pub format_search: bool,
    pub wargs: Option<Box<args>>,
    pub m: *mut mouse_event,
    pub c: Option<&'a std::rc::Rc<std::cell::UnsafeCell<client>>>,
    pub s: Option<&'a std::cell::UnsafeCell<session>>,
    pub wl: refbox::Weak<winlink>,
}
impl window_copy_cmd_state<'_> {
    fn s_ptr(&self) -> *mut session {
        self.s
            .map_or(std::ptr::null_mut(), std::cell::UnsafeCell::get)
    }

    /// The caller must not keep this address across removal of the winlink.
    unsafe fn wl_ptr(&self) -> *mut winlink {
        if self.wl.is_alive() {
            self.wl.as_ptr().cast_mut()
        } else {
            std::ptr::null_mut()
        }
    }

    fn parsed_args(&mut self) -> &mut args {
        self.wargs
            .as_deref_mut()
            .expect("parsed copy-mode arguments")
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_46 {
    pub command: &'static CStr,
    pub minargs: u_int,
    pub maxargs: u_int,
    pub args: args_parse,
    pub flags: ::core::ffi::c_int,
    pub clear: window_copy_cmd_clear,
    pub f: Option<unsafe fn(*mut window_copy_cmd_state) -> window_copy_cmd_action>,
}
pub const WINDOW_COPY_REL_POS_ON_SCREEN: C2RustUnnamed_50 = 1;
pub const WINDOW_COPY_REL_POS_BELOW: C2RustUnnamed_50 = 2;
pub const WINDOW_COPY_REL_POS_ABOVE: C2RustUnnamed_50 = 0;
pub const WINDOW_COPY_SEARCHDOWN: C2RustUnnamed_49 = 2;
pub const WINDOW_COPY_SEARCHUP: C2RustUnnamed_49 = 1;
pub const BOTTOM: C2RustUnnamed_48 = 2;
pub const TOP: C2RustUnnamed_48 = 1;
pub const MIDDLE: C2RustUnnamed_48 = 0;
pub type C2RustUnnamed_48 = ::core::ffi::c_uint;
pub const WINDOW_COPY_JUMPTOFORWARD: C2RustUnnamed_49 = 5;
pub const WINDOW_COPY_JUMPTOBACKWARD: C2RustUnnamed_49 = 6;
pub const WINDOW_COPY_JUMPBACKWARD: C2RustUnnamed_49 = 4;
pub const WINDOW_COPY_JUMPFORWARD: C2RustUnnamed_49 = 3;
pub const WINDOW_COPY_OFF: C2RustUnnamed_49 = 0;
pub type C2RustUnnamed_49 = ::core::ffi::c_uint;
pub type C2RustUnnamed_50 = ::core::ffi::c_uint;
pub type window_copy_line_numbers = ::core::ffi::c_uint;
#[inline]
unsafe fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}

pub const REG_NOTBOL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub static window_copy_mode: window_mode = {
    window_mode {
        name: c"copy-mode",
        default_format: None,
        flags: 0,
        init: Some(
            window_copy_init
                as unsafe fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_copy_free as unsafe fn(*mut window_mode_entry) -> ()),
        resize: Some(window_copy_resize as unsafe fn(*mut window_mode_entry, u_int, u_int) -> ()),
        update: None,
        style_changed: Some(window_copy_style_changed as unsafe fn(*mut window_mode_entry) -> ()),
        key: None,
        key_table: Some(
            window_copy_key_table
                as unsafe fn(*mut window_mode_entry) -> *const ::core::ffi::c_char,
        ),
        command: Some(
            window_copy_command
                as unsafe fn(
                    *mut window_mode_entry,
                    Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
                    Option<&std::rc::Rc<std::cell::UnsafeCell<session>>>,
                    *mut winlink,
                    *mut args,
                    *mut mouse_event,
                ) -> (),
        ),
        formats: Some(
            window_copy_formats as unsafe fn(*mut window_mode_entry, *mut format_tree) -> (),
        ),
        get_screen: Some(
            window_copy_get_screen as unsafe fn(*mut window_mode_entry) -> *mut screen,
        ),
    }
};
pub static window_view_mode: window_mode = {
    window_mode {
        name: c"view-mode",
        default_format: None,
        flags: 0,
        init: Some(
            window_copy_view_init
                as unsafe fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_copy_free as unsafe fn(*mut window_mode_entry) -> ()),
        resize: Some(window_copy_resize as unsafe fn(*mut window_mode_entry, u_int, u_int) -> ()),
        update: None,
        style_changed: Some(window_copy_style_changed as unsafe fn(*mut window_mode_entry) -> ()),
        key: None,
        key_table: Some(
            window_copy_key_table
                as unsafe fn(*mut window_mode_entry) -> *const ::core::ffi::c_char,
        ),
        command: Some(
            window_copy_command
                as unsafe fn(
                    *mut window_mode_entry,
                    Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
                    Option<&std::rc::Rc<std::cell::UnsafeCell<session>>>,
                    *mut winlink,
                    *mut args,
                    *mut mouse_event,
                ) -> (),
        ),
        formats: Some(
            window_copy_formats as unsafe fn(*mut window_mode_entry, *mut format_tree) -> (),
        ),
        get_screen: Some(
            window_copy_get_screen as unsafe fn(*mut window_mode_entry) -> *mut screen,
        ),
    }
};
pub const WINDOW_COPY_SEARCH_TIMEOUT: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;
pub const WINDOW_COPY_SEARCH_ALL_TIMEOUT: ::core::ffi::c_int = 200 as ::core::ffi::c_int;
pub const WINDOW_COPY_SEARCH_MAX_LINE: ::core::ffi::c_int = 2000 as ::core::ffi::c_int;
pub const WINDOW_COPY_DRAG_REPEAT_TIME: ::core::ffi::c_int = 50000 as ::core::ffi::c_int;
pub const WINDOW_COPY_REFRESH_INTERVAL: ::core::ffi::c_int = 50000 as ::core::ffi::c_int;
unsafe fn window_copy_scroll_timer(mut arg: *mut ::core::ffi::c_void) {
    let mut wme: *mut window_mode_entry = arg as *mut window_mode_entry;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: WINDOW_COPY_DRAG_REPEAT_TIME as __suseconds_t,
    };
    event_del(&raw mut (*data).dragtimer);
    if (*wp).modes.active != wme {
        return;
    }
    if (*data).cy == 0 as u_int {
        event_add(&raw mut (*data).dragtimer, &raw mut tv);
        window_copy_cursor_up(wme, 1 as ::core::ffi::c_int);
    } else if (*data).cy == (*data).screen.grid().sy.wrapping_sub(1 as u_int) {
        event_add(&raw mut (*data).dragtimer, &raw mut tv);
        window_copy_cursor_down(wme, 1 as ::core::ffi::c_int);
    }
}
unsafe fn window_copy_clone_screen(
    src: &screen,
    hint: &screen,
    mut cursor: Option<(&mut u_int, &mut u_int)>,
    trim: bool,
) -> Box<screen> {
    let mut dst = Box::new(screen::empty());
    let mut sy = src.grid().hsize.wrapping_add(src.grid().sy);
    if trim {
        while sy > src.grid().hsize {
            let Some(gl) = grid_peek_line(src.grid(), sy.wrapping_sub(1)) else {
                break;
            };
            if gl.cellused != 0 {
                break;
            }
            sy = sy.wrapping_sub(1);
        }
    }
    log_debug(format_args!(
        "{}: target screen is {}x{}, source {}x{}",
        "window_copy_clone_screen",
        src.grid().sx,
        sy,
        hint.grid().sx,
        src.grid().hsize.wrapping_add(src.grid().sy)
    ));
    screen_init(&mut dst, src.grid().sx, sy, src.grid().hlimit);
    dst.grid_mut().flags |= GRID_HISTORY;
    grid_duplicate_lines(dst.grid_mut(), 0, src.grid(), 0, sy);
    dst.grid_mut().sy = sy.wrapping_sub(src.grid().hsize);
    dst.grid_mut().hsize = src.grid().hsize;
    dst.grid_mut().hscrolled = src.grid().hscrolled;
    if src.cy > dst.grid().sy.wrapping_sub(1) {
        dst.cx = 0;
        dst.cy = dst.grid().sy.wrapping_sub(1);
    } else {
        dst.cx = src.cx;
        dst.cy = src.cy;
    }
    let reflow = cursor.is_some() && hint.grid().sx != dst.grid().sx;
    let mut wrapped = (0, 0);
    if let Some((cx, cy)) = cursor.as_mut() {
        **cx = dst.cx;
        **cy = dst.grid().hsize.wrapping_add(dst.cy);
        if reflow {
            wrapped = grid_wrap_position(dst.grid(), **cx, **cy);
        }
    }
    screen_resize_cursor(&mut dst, hint.grid().sx, hint.grid().sy, 1, 0, 0);
    if reflow {
        let (cx, cy) = cursor.expect("reflow cursor");
        (*cx, *cy) = grid_unwrap_position(dst.grid(), wrapped.0, wrapped.1);
    }
    dst
}

unsafe fn window_copy_sync_snapshot(mut data: *mut window_copy_mode_data, mut src: *mut grid) {
    (*data).sync_added = (*src).scroll_added;
    (*data).sync_collected = (*src).scroll_collected;
    (*data).sync_generation = (*src).scroll_generation;
}
unsafe fn window_copy_sync_backing(mut wme: *mut window_mode_entry) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let Some(source_owner) = (*wme).swp.upgrade() else {
        return 0;
    };
    let wp = source_owner.get();
    let mut src: *mut screen = &raw mut (*wp).base;
    let mut dst: *mut screen = (*data).backing_mut();
    let mut sg: *mut grid = (*src).grid_mut();
    let mut dg: *mut grid = (*dst).grid_mut();
    let mut sy: u_int = (*sg).sy;
    let mut old_hsize: u_int = (*dg).hsize;
    let mut new_hsize: u_int = (*sg).hsize;
    let mut added: u_int = 0;
    let mut collected: u_int = 0;
    let mut kept: u_int = 0;
    if (*data).viewmode != 0 || !(*wme).swp.ptr_eq(&(*wme).wp) {
        return 0 as ::core::ffi::c_int;
    }
    if (*sg).sx != (*dg).sx
        || (*sg).sy != (*dg).sy
        || (*sg).scroll_generation != (*data).sync_generation
    {
        return 0 as ::core::ffi::c_int;
    }
    added = (*sg).scroll_added.wrapping_sub((*data).sync_added);
    collected = (*sg).scroll_collected.wrapping_sub((*data).sync_collected);
    if added > INT_MAX as u_int
        || collected > INT_MAX as u_int
        || collected > old_hsize
        || old_hsize.wrapping_add(added) < collected
        || old_hsize.wrapping_add(added).wrapping_sub(collected) != new_hsize
    {
        return 0 as ::core::ffi::c_int;
    }
    kept = old_hsize.wrapping_sub(collected);
    if added == 0 as u_int && collected == 0 as u_int {
        grid_duplicate_lines(&mut *dg, old_hsize, &*sg, new_hsize, sy);
    } else {
        if collected > 0 as u_int {
            grid_free_lines(&mut *dg, 0 as u_int, collected);
            let live = old_hsize.wrapping_add(sy) as usize;
            (&mut (*dg).linedata)[..live].rotate_left(collected as usize);
        }
        (*dg)
            .linedata
            .resize_with(new_hsize.wrapping_add(sy) as usize, grid_line::default);
        (*dg).hsize = new_hsize;
        if added > 0 as u_int {
            grid_duplicate_lines(&mut *dg, kept, &*sg, kept, added);
        }
        grid_duplicate_lines(&mut *dg, new_hsize, &*sg, new_hsize, sy);
    }
    (*dg).hscrolled = (*sg).hscrolled;
    if (*src).cy > (*dg).sy.wrapping_sub(1 as u_int) {
        (*dst).cx = 0 as u_int;
        (*dst).cy = (*dg).sy.wrapping_sub(1 as u_int);
    } else {
        (*dst).cx = (*src).cx;
        (*dst).cy = (*src).cy;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_copy_common_init(mut wme: *mut window_mode_entry) -> *mut window_copy_mode_data {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut base: *mut screen = &raw mut (*wp).base;
    let mut data = Box::into_raw(Box::new(window_copy_mode_data::default()));
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).cursordrag = CURSORDRAG_NONE;
    (*data).lineflag = LINE_SEL_NONE;
    (*data).selflag = SEL_CHAR;
    if (*wp).searchstr.is_some() {
        (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
        (*data).searchregex = (*wp).searchregex;
        (*data).searchstr = (*wp).searchstr.clone();
    } else {
        (*data).searchtype = WINDOW_COPY_OFF as ::core::ffi::c_int;
        (*data).searchregex = 0 as ::core::ffi::c_int;
        (*data).searchstr = None;
    }
    (*data).searcho = -(1 as ::core::ffi::c_int);
    (*data).searchy = (*data).searcho;
    (*data).searchx = (*data).searchy;
    (*data).searchall = 1 as ::core::ffi::c_int;
    (*data).jumptype = WINDOW_COPY_OFF as ::core::ffi::c_int;
    (*data).line_numbers = 1 as ::core::ffi::c_int;
    screen_init(
        &mut (*data).screen,
        (*base).grid().sx,
        (*base).grid().sy,
        0 as u_int,
    );
    screen_set_default_cursor(&mut (*data).screen, global_w_options);
    (*data).modekeys = options_get_number(
        options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options),
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    event_set(
        &raw mut (*data).dragtimer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        move |_, _| unsafe { window_copy_scroll_timer(wme as *mut ::core::ffi::c_void) },
    );
    event_set(
        &raw mut (*data).refresh_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        move |_, _| unsafe { window_copy_refresh_timer(wme as *mut ::core::ffi::c_void) },
    );
    return data;
}
unsafe fn window_copy_init(
    mut wme: *mut window_mode_entry,
    _item: *mut cmdq_item,
    _fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let Some(source_owner) = (*wme).swp.upgrade() else {
        return std::ptr::null_mut();
    };
    let wp = source_owner.get();
    let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
    let mut base: *mut screen = &raw mut (*wp).base;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut i: u_int = 0;
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    data = window_copy_common_init(wme);
    (*data).backing = Some(window_copy_clone_screen(
        &*base,
        &(*data).screen,
        Some((&mut cx, &mut cy)),
        !(*wme).swp.ptr_eq(&(*wme).wp),
    ));
    window_copy_sync_snapshot(data, (*base).grid_mut());
    (*data).cx = cx;
    if cy < (*data).backing().grid().hsize {
        (*data).cy = 0 as u_int;
        (*data).oy = (*data).backing().grid().hsize.wrapping_sub(cy);
    } else {
        (*data).cy = cy.wrapping_sub((*data).backing().grid().hsize);
        (*data).oy = 0 as u_int;
    }
    (*data).scroll_exit = args_has(args, 'e' as i32 as u_char);
    (*data).hide_position = args_has(args, 'H' as i32 as u_char);
    if (*base).hyperlinks.is_some() {
        screen_share_hyperlinks(&mut (*data).screen, &*base);
    }
    (*data).screen.cx = window_copy_cursor_offset(wme, (*data).cx, (*data).screen.grid().sx);
    (*data).screen.cy = (*data).cy;
    (*data).mx = (*data).cx;
    (*data).my = (*data).backing()
        .grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).showmark = 0 as ::core::ffi::c_int;
    screen_write_start(&mut ctx, &raw mut (*data).screen);
    i = 0 as u_int;
    while i < (*data).screen.grid().sy {
        window_copy_write_line(wme, &raw mut ctx, i);
        i = i.wrapping_add(1);
    }
    screen_write_cursormove(
        &mut ctx,
        window_copy_cursor_offset(wme, (*data).cx, (*data).screen.grid().sx) as ::core::ffi::c_int,
        (*data).cy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&mut ctx);
    (*data).recentre_state = RECENTRE_MIDDLE;
    (*data).recentre_line = 0 as u_int;
    return &raw mut (*data).screen;
}
unsafe fn window_copy_view_init(
    mut wme: *mut window_mode_entry,
    _item: *mut cmdq_item,
    _fs: *mut cmd_find_state,
    _args: *mut args,
) -> *mut screen {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
    let mut base: *mut screen = &raw mut (*wp).base;
    let mut sx: u_int = (*base).grid().sx;
    data = window_copy_common_init(wme);
    (*data).viewmode = 1 as ::core::ffi::c_int;
    (*data).line_numbers = 0 as ::core::ffi::c_int;
    (*data).backing = Some(Box::new(screen::empty()));
    screen_init((*data).backing_mut(), sx, (*base).grid().sy, UINT_MAX);
    (*data).ictx = Some(input_init(
        None,
        ::core::ptr::null_mut::<bufferevent>(),
        ::core::ptr::null_mut::<colour_palette>(),
        None,
    ));
    (*data).mx = (*data).cx;
    (*data).my = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).showmark = 0 as ::core::ffi::c_int;
    return &raw mut (*data).screen;
}
unsafe fn window_copy_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    event_del(&raw mut (*data).dragtimer);
    event_del(&raw mut (*data).refresh_timer);
    window_copy_clear_searchmark(&mut *data);
    if let Some(ictx) = (*data).ictx.take() {
        input_free(ictx);
    }
    (*data).clear_backing();
    screen_free(&mut (*data).screen);
    drop(Box::from_raw(data));
}
pub unsafe fn window_copy_add(
    pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut parse: ::core::ffi::c_int,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let wp = pane_owner.get();
    let mut wme: *mut window_mode_entry = (*wp).modes.active;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut backing: *mut screen = (*data).backing_mut();
    let mut backing_ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
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
    let mut old_hsize: u_int = 0;
    let mut old_cy: u_int = 0;
    old_hsize = (*data).backing().grid().hsize;
    screen_write_start(&mut backing_ctx, backing);
    if (*data).backing_written != 0 {
        screen_write_carriagereturn(&mut backing_ctx);
        screen_write_linefeed(&mut backing_ctx, 0 as ::core::ffi::c_int, 8 as u_int);
    } else {
        (*data).backing_written = 1 as ::core::ffi::c_int;
    }
    old_cy = (*backing).cy;
    if parse != 0 {
        let text = format_message_with(write);
        input_parse_screen(
            (*data).ictx.as_deref_mut().map_or(std::ptr::null_mut(), |ictx| ictx),
            backing,
            Some(Box::new(|_| {})),
            text.as_ptr() as *const u_char,
            text.as_bytes().len(),
        );
    } else {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
        screen_write_nputs(&mut backing_ctx, 0 as ssize_t, &gc, write);
    }
    screen_write_stop(&mut backing_ctx);
    (*data).oy = (*data)
        .oy
        .wrapping_add((*data).backing().grid().hsize.wrapping_sub(old_hsize));
    screen_write_start_pane(&mut ctx, &(*wp).observer.upgrade().expect("live screen-write pane"), &raw mut (*data).screen);
    if (*data).backing().grid().hsize != 0 {
        window_copy_redraw_lines(wme, 0 as u_int, 1 as u_int);
    }
    window_copy_redraw_lines(
        wme,
        old_cy,
        (*backing).cy.wrapping_sub(old_cy).wrapping_add(1 as u_int),
    );
    screen_write_stop(&mut ctx);
}
pub unsafe fn window_copy_scroll(
    pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut sl_mpos: ::core::ffi::c_int,
    mut my: u_int,
    mut tty_oy: u_int,
    mut scroll_exit: ::core::ffi::c_int,
) {
    let wp = pane_owner.get();
    let mut wme: *mut window_mode_entry = (*wp).modes.active;
    if !wme.is_null() {
        window_set_active_pane((*wp).window as *mut window, wp, 0 as ::core::ffi::c_int);
        window_copy_scroll1(wme, pane_owner, sl_mpos, my, tty_oy, scroll_exit);
    }
}
unsafe fn window_copy_scroll1(
    mut wme: *mut window_mode_entry,
    pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut sl_mpos: ::core::ffi::c_int,
    mut my: u_int,
    mut tty_oy: u_int,
    mut scroll_exit: ::core::ffi::c_int,
) {
    let wp = pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut n: u_int = 0;
    let mut new_offset: u_int = 0;
    let mut slider_height: u_int = (*wp).sb_slider_h;
    let mut sb_height: u_int = (*wp).sy;
    let mut sb_top: u_int = (*wp).yoff as u_int;
    let mut sy: u_int = (*data).backing().grid().sy;
    let mut my_w: u_int = 0;
    let mut new_slider_y: ::core::ffi::c_int = 0;
    let mut delta: ::core::ffi::c_int = 0;
    my_w = my.wrapping_add(tty_oy);
    if my_w <= sb_top.wrapping_add(sl_mpos as u_int) {
        new_slider_y = sb_top.wrapping_sub((*wp).yoff as u_int) as ::core::ffi::c_int;
    } else if my_w.wrapping_sub(sl_mpos as u_int)
        > sb_top.wrapping_add(sb_height).wrapping_sub(slider_height)
    {
        new_slider_y = sb_top
            .wrapping_sub((*wp).yoff as u_int)
            .wrapping_add(sb_height.wrapping_sub(slider_height))
            as ::core::ffi::c_int;
    } else {
        new_slider_y = my_w
            .wrapping_sub((*wp).yoff as u_int)
            .wrapping_sub(sl_mpos as u_int) as ::core::ffi::c_int;
    }
    let Some((offset, size)) = window_copy_get_current_offset(&*wp) else {
        return;
    };
    new_offset = (new_slider_y as ::core::ffi::c_float
        * (size.wrapping_add(sb_height) as ::core::ffi::c_float
            / sb_height as ::core::ffi::c_float)) as u_int;
    delta = (offset as ::core::ffi::c_int as u_int).wrapping_sub(new_offset) as ::core::ffi::c_int;
    oy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    ox = window_copy_find_length(wme, oy);
    if (*data).cx != ox {
        (*data).lastcx = (*data).cx;
        (*data).lastsx = ox;
    }
    (*data).cx = (*data).lastcx;
    if delta >= 0 as ::core::ffi::c_int {
        n = delta as u_int;
        if (*data).oy.wrapping_add(n) > (*data).backing().grid().hsize {
            (*data).oy = (*data).backing().grid().hsize;
            if (*data).cy < n {
                (*data).cy = 0 as u_int;
            } else {
                (*data).cy = (*data).cy.wrapping_sub(n);
            }
        } else {
            (*data).oy = (*data).oy.wrapping_add(n);
        }
    } else {
        n = -delta as u_int;
        if (*data).oy < n {
            (*data).oy = 0 as u_int;
            if (*data).cy.wrapping_add(n.wrapping_sub((*data).oy)) >= sy {
                (*data).cy = sy.wrapping_sub(1 as u_int);
            } else {
                (*data).cy = (*data).cy.wrapping_add(n.wrapping_sub((*data).oy));
            }
        } else {
            (*data).oy = (*data).oy.wrapping_sub(n);
        }
    }
    (*data).cursordrag = CURSORDRAG_NONE;
    if (*data).screen.sel.is_none() || (*data).rectflag == 0 {
        py = (*data).backing().grid()
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        px = window_copy_find_length(wme, py);
        if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
            window_copy_cursor_end_of_line(wme);
        }
    }
    if scroll_exit != 0 && (*data).oy == 0 as u_int && (*data).screen.sel.is_none() {
        window_pane_reset_mode(pane_owner);
        return;
    }
    if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection_view(wme, 1 as ::core::ffi::c_int);
    window_pane_scrollbar_show(pane_owner);
    window_copy_redraw_screen(wme);
}
pub unsafe fn window_copy_pageup(pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    let mut half_page: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    window_copy_pageup1((*wp).modes.active, half_page);
}
unsafe fn window_copy_pageup1(mut wme: *mut window_mode_entry, mut half_page: ::core::ffi::c_int) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut n: u_int = 0;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    oy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    ox = window_copy_find_length(wme, oy);
    if (*data).cx != ox {
        (*data).lastcx = (*data).cx;
        (*data).lastsx = ox;
    }
    (*data).cx = (*data).lastcx;
    n = 1 as u_int;
    if (*s).grid().sy > 2 as u_int {
        if half_page != 0 {
            n = (*s).grid().sy.wrapping_div(2 as u_int);
        } else {
            n = (*s).grid().sy.wrapping_sub(2 as u_int);
        }
    }
    if (*data).oy.wrapping_add(n) > (*data).backing().grid().hsize {
        (*data).oy = (*data).backing().grid().hsize;
        if (*data).cy < n {
            (*data).cy = 0 as u_int;
        } else {
            (*data).cy = (*data).cy.wrapping_sub(n);
        }
    } else {
        (*data).oy = (*data).oy.wrapping_add(n);
    }
    if (*data).screen.sel.is_none() || (*data).rectflag == 0 {
        py = (*data).backing().grid()
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        px = window_copy_find_length(wme, py);
        if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
            window_copy_cursor_end_of_line(wme);
        }
    }
    if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    window_pane_scrollbar_show(&mode_pane_owner);
    window_copy_redraw_screen(wme);
}
pub unsafe fn window_copy_pagedown(pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>, mut scroll_exit: ::core::ffi::c_int) {
    let wp = pane_owner.get();
    let mut half_page: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if window_copy_pagedown1((*wp).modes.active, half_page, scroll_exit) != 0 {
        window_pane_reset_mode(pane_owner);
        return;
    }
}
unsafe fn window_copy_pagedown1(
    mut wme: *mut window_mode_entry,
    mut half_page: ::core::ffi::c_int,
    mut scroll_exit: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut n: u_int = 0;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    oy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    ox = window_copy_find_length(wme, oy);
    if (*data).cx != ox {
        (*data).lastcx = (*data).cx;
        (*data).lastsx = ox;
    }
    (*data).cx = (*data).lastcx;
    n = 1 as u_int;
    if (*s).grid().sy > 2 as u_int {
        if half_page != 0 {
            n = (*s).grid().sy.wrapping_div(2 as u_int);
        } else {
            n = (*s).grid().sy.wrapping_sub(2 as u_int);
        }
    }
    if (*data).oy < n {
        (*data).oy = 0 as u_int;
        if (*data).cy.wrapping_add(n.wrapping_sub((*data).oy)) >= (*data).backing().grid().sy {
            (*data).cy = (*data).backing().grid().sy.wrapping_sub(1 as u_int);
        } else {
            (*data).cy = (*data).cy.wrapping_add(n.wrapping_sub((*data).oy));
        }
    } else {
        (*data).oy = (*data).oy.wrapping_sub(n);
    }
    if (*data).screen.sel.is_none() || (*data).rectflag == 0 {
        py = (*data).backing().grid()
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        px = window_copy_find_length(wme, py);
        if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
            window_copy_cursor_end_of_line(wme);
        }
    }
    if scroll_exit != 0 && (*data).oy == 0 as u_int && (*data).screen.sel.is_none() {
        return 1 as ::core::ffi::c_int;
    }
    if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    window_pane_scrollbar_show(&mode_pane_owner);
    window_copy_redraw_screen(wme);
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_copy_previous_paragraph(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oy: u_int = 0;
    oy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    while oy > 0 as u_int && window_copy_find_length(wme, oy) == 0 as u_int {
        oy = oy.wrapping_sub(1);
    }
    while oy > 0 as u_int && window_copy_find_length(wme, oy) > 0 as u_int {
        oy = oy.wrapping_sub(1);
    }
    window_copy_scroll_to(wme, 0 as u_int, oy, 0 as ::core::ffi::c_int);
}
unsafe fn window_copy_next_paragraph(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut maxy: u_int = 0;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    oy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    maxy = (*data).backing().grid()
        .hsize
        .wrapping_add((*s).grid().sy)
        .wrapping_sub(1 as u_int);
    while oy < maxy && window_copy_find_length(wme, oy) == 0 as u_int {
        oy = oy.wrapping_add(1);
    }
    while oy < maxy && window_copy_find_length(wme, oy) > 0 as u_int {
        oy = oy.wrapping_add(1);
    }
    ox = window_copy_find_length(wme, oy);
    window_copy_scroll_to(wme, ox, oy, 0 as ::core::ffi::c_int);
}
pub(crate) unsafe fn window_copy_get_word_cstring(
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
) -> Option<CString> {
    let mut wme: *mut window_mode_entry = (*wp).modes.active;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*data).backing_mut().grid_mut();
    return crate::src::format::format_grid_word_cstring(
        &*gd,
        x,
        (*gd).hsize.wrapping_add(y).wrapping_sub((*data).oy),
    );
}
pub(crate) unsafe fn window_copy_get_line_cstring(
    mut wp: *mut window_pane,
    mut y: u_int,
) -> Option<CString> {
    let mut wme: *mut window_mode_entry = (*wp).modes.active;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*data).backing_mut().grid_mut();
    return crate::src::format::format_grid_line_cstring(
        &*gd,
        (*gd).hsize.wrapping_add(y).wrapping_sub((*data).oy),
    );
}
pub(crate) unsafe fn window_copy_get_hyperlink_cstring(
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
) -> Option<CString> {
    let mut wme: *mut window_mode_entry = (*wp).modes.active;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*data).screen.grid_mut();
    return crate::src::format::format_grid_hyperlink_cstring(
        &*gd,
        x,
        (*gd).hsize.wrapping_add(y),
        &*(*wp).screen,
    );
}
unsafe fn window_copy_cursor_hyperlink_cb(mut ft: *mut format_tree) -> Option<CString> {
    let pane_owner = format_get_pane(&*ft)?;
    let wp = pane_owner.get();
    let mut wme: *mut window_mode_entry = (*wp).modes.active;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*data).screen.grid_mut();
    return format_grid_hyperlink_cstring(
        &*gd,
        (*data).cx,
        (*gd).hsize.wrapping_add((*data).cy),
        &(*data).screen,
    );
}
unsafe fn window_copy_cursor_word_cb(mut ft: *mut format_tree) -> Option<CString> {
    let pane_owner = format_get_pane(&*ft)?;
    let wp = pane_owner.get();
    let mut wme: *mut window_mode_entry = (*wp).modes.active;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    return window_copy_get_word_cstring(wp, (*data).cx, (*data).cy);
}
unsafe fn window_copy_cursor_line_cb(mut ft: *mut format_tree) -> Option<CString> {
    let pane_owner = format_get_pane(&*ft)?;
    let wp = pane_owner.get();
    let mut wme: *mut window_mode_entry = (*wp).modes.active;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    return window_copy_get_line_cstring(wp, (*data).cy);
}
unsafe fn window_copy_search_match_cb(mut ft: *mut format_tree) -> Option<CString> {
    let pane_owner = format_get_pane(&*ft)?;
    let wp = pane_owner.get();
    let mut wme: *mut window_mode_entry = (*wp).modes.active;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    return window_copy_match_at_cursor_cstring(data);
}
unsafe fn window_copy_formats(mut wme: *mut window_mode_entry, mut ft: *mut format_tree) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut hsize: u_int = (*data).backing().grid().hsize;
    let mut position: u_int = 0;
    let mut limit: u_int = 0;
    let mut t: time_t = 0;
    let gd = (*data).backing().grid();
    let gl = &gd.linedata[hsize.wrapping_sub((*data).oy) as usize];
    t = grid_line_time(gl);
    format_add(
        ft,
        b"top_line_time\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (t as ::core::ffi::c_ulonglong) as u64),
    );
    format_add(
        ft,
        b"scroll_position\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", ((*data).oy) as i32),
    );
    if window_copy_line_number_is_absolute(wme) != 0 {
        position = hsize.wrapping_sub((*data).oy).wrapping_add(1 as u_int);
        limit = hsize.wrapping_add((*data).backing().grid().sy);
    } else {
        position = (*data).oy;
        limit = hsize;
    }
    format_add(
        ft,
        b"copy_position\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (position) as u32),
    );
    format_add(
        ft,
        b"copy_position_limit\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (limit) as u32),
    );
    format_add(
        ft,
        b"copy_line_numbers\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (window_copy_line_numbers_active(wme)) as i32),
    );
    format_add(
        ft,
        b"refresh_active\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", ((*data).refresh_active) as i32),
    );
    format_add(
        ft,
        b"rectangle_toggle\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", ((*data).rectflag) as i32),
    );
    format_add(
        ft,
        b"copy_cursor_x\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", ((*data).cx) as i32),
    );
    format_add(
        ft,
        b"copy_cursor_y\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", ((*data).cy) as i32),
    );
    if !(*data).screen.sel.is_none() {
        format_add(
            ft,
            b"selection_start_x\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", ((*data).selx) as i32),
        );
        format_add(
            ft,
            b"selection_start_y\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", ((*data).sely) as i32),
        );
        format_add(
            ft,
            b"selection_end_x\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", ((*data).endselx) as i32),
        );
        format_add(
            ft,
            b"selection_end_y\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", ((*data).endsely) as i32),
        );
        if (*data).cursordrag as ::core::ffi::c_uint
            != CURSORDRAG_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            format_add(
                ft,
                b"selection_active\0" as *const u8 as *const ::core::ffi::c_char,
                |out| out.write_all(b"1"),
            );
        } else {
            format_add(
                ft,
                b"selection_active\0" as *const u8 as *const ::core::ffi::c_char,
                |out| out.write_all(b"0"),
            );
        }
        if (*data).endselx != (*data).selx || (*data).endsely != (*data).sely {
            format_add(
                ft,
                b"selection_present\0" as *const u8 as *const ::core::ffi::c_char,
                |out| out.write_all(b"1"),
            );
        } else {
            format_add(
                ft,
                b"selection_present\0" as *const u8 as *const ::core::ffi::c_char,
                |out| out.write_all(b"0"),
            );
        }
    } else {
        format_add(
            ft,
            b"selection_active\0" as *const u8 as *const ::core::ffi::c_char,
            |out| out.write_all(b"0"),
        );
        format_add(
            ft,
            b"selection_present\0" as *const u8 as *const ::core::ffi::c_char,
            |out| out.write_all(b"0"),
        );
    }
    match (*data).selflag as ::core::ffi::c_uint {
        0 => {
            format_add(
                ft,
                b"selection_mode\0" as *const u8 as *const ::core::ffi::c_char,
                |out| out.write_all(b"char"),
            );
        }
        1 => {
            format_add(
                ft,
                b"selection_mode\0" as *const u8 as *const ::core::ffi::c_char,
                |out| out.write_all(b"word"),
            );
        }
        2 => {
            format_add(
                ft,
                b"selection_mode\0" as *const u8 as *const ::core::ffi::c_char,
                |out| out.write_all(b"line"),
            );
        }
        _ => {}
    }
    format_add(
        ft,
        b"search_present\0" as *const u8 as *const ::core::ffi::c_char,
        |out| {
            write!(
                out,
                "{}",
                ((!(*data).searchmark.is_empty()) as ::core::ffi::c_int) as i32
            )
        },
    );
    format_add(
        ft,
        b"search_timed_out\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", ((*data).timeout) as i32),
    );
    if (*data).searchcount != -(1 as ::core::ffi::c_int) {
        format_add(
            ft,
            b"search_count\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", ((*data).searchcount) as i32),
        );
        format_add(
            ft,
            b"search_count_partial\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", ((*data).searchmore) as i32),
        );
    }
    format_add_owned_cb(ft, c"search_match", |ft| unsafe {
        window_copy_search_match_cb(ft.as_ptr())
    });
    format_add_owned_cb(ft, c"copy_cursor_word", |ft| unsafe {
        window_copy_cursor_word_cb(ft.as_ptr())
    });
    format_add_owned_cb(ft, c"copy_cursor_line", |ft| unsafe {
        window_copy_cursor_line_cb(ft.as_ptr())
    });
    format_add_owned_cb(ft, c"copy_cursor_hyperlink", |ft| unsafe {
        window_copy_cursor_hyperlink_cb(ft.as_ptr())
    });
}
unsafe fn window_copy_get_screen(mut wme: *mut window_mode_entry) -> *mut screen {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    return (*data).backing_mut();
}
unsafe fn window_copy_size_changed(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut search: ::core::ffi::c_int = (!(*data).searchmark.is_empty()) as ::core::ffi::c_int;
    window_copy_clear_selection(wme);
    window_copy_clear_marks(wme);
    screen_write_start(&mut ctx, s);
    window_copy_write_lines(wme, &raw mut ctx, 0 as u_int, (*s).grid().sy);
    screen_write_stop(&mut ctx);
    if search != 0 && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            0 as ::core::ffi::c_int,
        );
    }
    (*data).searchx = (*data).cx as ::core::ffi::c_int;
    (*data).searchy = (*data).cy as ::core::ffi::c_int;
    (*data).searcho = (*data).oy as ::core::ffi::c_int;
}
unsafe fn window_copy_resize(mut wme: *mut window_mode_entry, mut sx: u_int, mut sy: u_int) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut wx: u_int = 0;
    let mut wy: u_int = 0;
    let mut reflow: ::core::ffi::c_int = 0;
    screen_resize(&mut *s, sx, sy, 0 as ::core::ffi::c_int);
    let gd = (*data).backing().grid();
    cx = (*data).cx;
    if (*data).oy > gd.hsize.wrapping_add((*data).cy) {
        (*data).oy = gd.hsize.wrapping_add((*data).cy);
    }
    cy = gd
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    reflow = (gd.sx != sx) as ::core::ffi::c_int;
    if reflow != 0 {
        (wx, wy) = grid_wrap_position(gd, cx, cy);
    }
    screen_resize_cursor(
        (*data).backing_mut(),
        sx,
        sy,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    let gd = (*data).backing().grid();
    if reflow != 0 {
        (cx, cy) = grid_unwrap_position(gd, wx, wy);
    }
    (*data).cx = cx;
    if cy < gd.hsize {
        (*data).cy = 0 as u_int;
        (*data).oy = gd.hsize.wrapping_sub(cy);
    } else {
        (*data).cy = cy.wrapping_sub(gd.hsize);
        (*data).oy = 0 as u_int;
    }
    window_copy_size_changed(wme);
    window_copy_redraw_screen(wme);
}
unsafe fn window_copy_key_table(mut wme: *mut window_mode_entry) -> *const ::core::ffi::c_char {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    if options_get_number(
        options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options),
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == MODEKEY_VI as ::core::ffi::c_longlong
    {
        return b"copy-mode-vi\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return b"copy-mode\0" as *const u8 as *const ::core::ffi::c_char;
}
unsafe fn window_copy_expand_search_string(
    mut cs: *mut window_copy_cmd_state,
) -> ::core::ffi::c_int {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut ss: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    if ss.is_null() || *ss as ::core::ffi::c_int == '\0' as i32 {
        return 0 as ::core::ffi::c_int;
    }
    if (*cs).format_search {
        let expanded = format_single_cstring(
            ::core::ptr::null_mut::<cmdq_item>(),
            ss,
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            mode_pane,
        );
        if expanded.as_bytes().is_empty() {
            return 0 as ::core::ffi::c_int;
        }
        (*data).searchstr = Some(expanded);
    } else {
        (*data).searchstr = Some(CStr::from_ptr(ss).to_owned());
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_copy_cmd_append_selection(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut s: *mut session = (*cs).s_ptr();
    if !s.is_null() {
        window_copy_append_selection(wme);
    }
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_append_selection_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut s: *mut session = (*cs).s_ptr();
    if !s.is_null() {
        window_copy_append_selection(wme);
    }
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_CANCEL;
}
unsafe fn window_copy_cmd_back_to_indentation(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cursor_back_to_indentation(wme);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_begin_selection(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut m: *mut mouse_event = (*cs).m;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    if !m.is_null() {
        window_copy_start_drag((*cs).c, m);
        return WINDOW_COPY_CMD_MOVE;
    }
    (*data).lineflag = LINE_SEL_NONE;
    (*data).selflag = SEL_CHAR;
    window_copy_start_selection(wme);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_stop_selection(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).cursordrag = CURSORDRAG_NONE;
    (*data).lineflag = LINE_SEL_NONE;
    (*data).selflag = SEL_CHAR;
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_bottom_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).cx = 0 as u_int;
    (*data).cy = (*data).screen.grid().sy.wrapping_sub(1 as u_int);
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_cancel(_cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    return WINDOW_COPY_CMD_CANCEL;
}
unsafe fn window_copy_cmd_clear_selection(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_do_copy_end_of_line(
    mut cs: *mut window_copy_cmd_state,
    mut pipe: ::core::ffi::c_int,
    mut cancel: ::core::ffi::c_int,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let c = (*cs).c.map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut s: *mut session = (*cs).s_ptr();
    let mut wl: *mut winlink = (*cs).wl_ptr();
    let mut wp: *mut window_pane = mode_pane;
    let mut count: u_int = args_count((*cs).parsed_args());
    let mut np: u_int = (*wme).prefix;
    let mut ocx: u_int = 0;
    let mut ocy: u_int = 0;
    let mut ooy: u_int = 0;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut prefix: Option<CString> = None;
    let mut command: Option<CString> = None;
    let mut arg0: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut arg1: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 1 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut set_paste: ::core::ffi::c_int =
        (args_has((*cs).parsed_args(), 'P' as i32 as u_char) == 0) as ::core::ffi::c_int;
    let mut set_clip: ::core::ffi::c_int =
        (args_has((*cs).parsed_args(), 'C' as i32 as u_char) == 0) as ::core::ffi::c_int;
    if pipe != 0 {
        if count == 2 as u_int {
            prefix = Some(format_single_cstring(
                ::core::ptr::null_mut::<cmdq_item>(),
                arg1,
                c,
                s,
                wl,
                wp,
            ));
        }
        if !s.is_null() && count > 0 as u_int && *arg0 as ::core::ffi::c_int != '\0' as i32 {
            command = Some(format_single_cstring(
                ::core::ptr::null_mut::<cmdq_item>(),
                arg0,
                c,
                s,
                wl,
                wp,
            ));
        }
    } else if count == 1 as u_int {
        prefix = Some(format_single_cstring(
            ::core::ptr::null_mut::<cmdq_item>(),
            arg0,
            c,
            s,
            wl,
            wp,
        ));
    }
    ocx = (*data).cx;
    ocy = (*data).cy;
    ooy = (*data).oy;
    window_copy_start_selection(wme);
    while np > 1 as u_int {
        window_copy_cursor_down(wme, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    window_copy_cursor_end_of_line(wme);
    if !s.is_null() {
        if pipe != 0 {
            window_copy_copy_pipe(
                wme,
                s,
                prefix
                    .as_ref()
                    .map_or(::core::ptr::null(), |value| value.as_ptr()),
                command
                    .as_ref()
                    .map_or(::core::ptr::null(), |value| value.as_ptr()),
                set_paste,
                set_clip,
            );
        } else {
            window_copy_copy_selection(
                wme,
                prefix
                    .as_ref()
                    .map_or(::core::ptr::null(), |value| value.as_ptr()),
                set_paste,
                set_clip,
            );
        }
        if cancel != 0 {
            return WINDOW_COPY_CMD_CANCEL;
        }
    }
    window_copy_clear_selection(wme);
    (*data).cx = ocx;
    (*data).cy = ocy;
    (*data).oy = ooy;
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_copy_end_of_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_end_of_line(cs, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
}
unsafe fn window_copy_cmd_copy_end_of_line_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_end_of_line(cs, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
}
unsafe fn window_copy_cmd_copy_pipe_end_of_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_end_of_line(cs, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
}
unsafe fn window_copy_cmd_copy_pipe_end_of_line_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_end_of_line(cs, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
}
unsafe fn window_copy_do_copy_line(
    mut cs: *mut window_copy_cmd_state,
    mut pipe: ::core::ffi::c_int,
    mut cancel: ::core::ffi::c_int,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let c = (*cs).c.map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut s: *mut session = (*cs).s_ptr();
    let mut wl: *mut winlink = (*cs).wl_ptr();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut count: u_int = args_count((*cs).parsed_args());
    let mut np: u_int = (*wme).prefix;
    let mut ocx: u_int = 0;
    let mut ocy: u_int = 0;
    let mut ooy: u_int = 0;
    let mut prefix: Option<CString> = None;
    let mut command: Option<CString> = None;
    let mut arg0: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut arg1: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 1 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut set_paste: ::core::ffi::c_int =
        (args_has((*cs).parsed_args(), 'P' as i32 as u_char) == 0) as ::core::ffi::c_int;
    let mut set_clip: ::core::ffi::c_int =
        (args_has((*cs).parsed_args(), 'C' as i32 as u_char) == 0) as ::core::ffi::c_int;
    if pipe != 0 {
        if count == 2 as u_int {
            prefix = Some(format_single_cstring(
                ::core::ptr::null_mut::<cmdq_item>(),
                arg1,
                c,
                s,
                wl,
                wp,
            ));
        }
        if !s.is_null() && count > 0 as u_int && *arg0 as ::core::ffi::c_int != '\0' as i32 {
            command = Some(format_single_cstring(
                ::core::ptr::null_mut::<cmdq_item>(),
                arg0,
                c,
                s,
                wl,
                wp,
            ));
        }
    } else if count == 1 as u_int {
        prefix = Some(format_single_cstring(
            ::core::ptr::null_mut::<cmdq_item>(),
            arg0,
            c,
            s,
            wl,
            wp,
        ));
    }
    ocx = (*data).cx;
    ocy = (*data).cy;
    ooy = (*data).oy;
    (*data).selflag = SEL_CHAR;
    window_copy_cursor_start_of_line(wme);
    window_copy_start_selection(wme);
    while np > 1 as u_int {
        window_copy_cursor_down(wme, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    window_copy_cursor_end_of_line(wme);
    if !s.is_null() {
        if pipe != 0 {
            window_copy_copy_pipe(
                wme,
                s,
                prefix
                    .as_ref()
                    .map_or(::core::ptr::null(), |value| value.as_ptr()),
                command
                    .as_ref()
                    .map_or(::core::ptr::null(), |value| value.as_ptr()),
                set_paste,
                set_clip,
            );
        } else {
            window_copy_copy_selection(
                wme,
                prefix
                    .as_ref()
                    .map_or(::core::ptr::null(), |value| value.as_ptr()),
                set_paste,
                set_clip,
            );
        }
        if cancel != 0 {
            return WINDOW_COPY_CMD_CANCEL;
        }
    }
    window_copy_clear_selection(wme);
    (*data).cx = ocx;
    (*data).cy = ocy;
    (*data).oy = ooy;
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_copy_line(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    return window_copy_do_copy_line(cs, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
}
unsafe fn window_copy_cmd_copy_line_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_line(cs, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
}
unsafe fn window_copy_cmd_copy_pipe_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_line(cs, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
}
unsafe fn window_copy_cmd_copy_pipe_line_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_line(cs, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
}
unsafe fn window_copy_cmd_copy_selection_no_clear(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let c = (*cs).c.map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut s: *mut session = (*cs).s_ptr();
    let mut wl: *mut winlink = (*cs).wl_ptr();
    let mut wp: *mut window_pane = mode_pane;
    let mut prefix: Option<CString> = None;
    let mut arg0: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut set_paste: ::core::ffi::c_int =
        (args_has((*cs).parsed_args(), 'P' as i32 as u_char) == 0) as ::core::ffi::c_int;
    let mut set_clip: ::core::ffi::c_int =
        (args_has((*cs).parsed_args(), 'C' as i32 as u_char) == 0) as ::core::ffi::c_int;
    if !arg0.is_null() {
        prefix = Some(format_single_cstring(
            ::core::ptr::null_mut::<cmdq_item>(),
            arg0,
            c,
            s,
            wl,
            wp,
        ));
    }
    if !s.is_null() {
        window_copy_copy_selection(
            wme,
            prefix
                .as_ref()
                .map_or(::core::ptr::null(), |value| value.as_ptr()),
            set_paste,
            set_clip,
        );
    }
    return WINDOW_COPY_CMD_NOTHING;
}
unsafe fn window_copy_cmd_copy_selection(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cmd_copy_selection_no_clear(cs);
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_copy_selection_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cmd_copy_selection_no_clear(cs);
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_CANCEL;
}
unsafe fn window_copy_cmd_cursor_down(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_down(wme, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_cursor_down_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut cy: u_int = 0;
    cy = (*data).cy;
    while np != 0 as u_int {
        window_copy_cursor_down(wme, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    if cy == (*data).cy && (*data).oy == 0 as u_int {
        return WINDOW_COPY_CMD_CANCEL;
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_cursor_left(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_left(wme);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_cursor_right(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_right(
            wme,
            (!(*data).screen.sel.is_none() && (*data).rectflag != 0) as ::core::ffi::c_int,
        );
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_scroll_to(
    mut cs: *mut window_copy_cmd_state,
    mut to: u_int,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oy: u_int = 0;
    let mut delta: u_int = 0;
    let mut scroll_up: ::core::ffi::c_int = 0;
    scroll_up = (*data).cy.wrapping_sub(to) as ::core::ffi::c_int;
    delta = abs(scroll_up) as u_int;
    oy = (*data).backing().grid().hsize.wrapping_sub((*data).oy);
    if scroll_up > 0 as ::core::ffi::c_int && (*data).oy >= delta {
        window_copy_scroll_up(wme, delta);
        (*data).cy = (*data).cy.wrapping_sub(delta);
    } else if scroll_up < 0 as ::core::ffi::c_int && oy >= delta {
        window_copy_scroll_down(wme, delta);
        (*data).cy = (*data).cy.wrapping_add(delta);
    }
    window_copy_update_selection_view(wme, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_scroll_bottom(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    let mut bottom: u_int = 0;
    bottom = (*data).screen.grid().sy.wrapping_sub(1 as u_int);
    return window_copy_cmd_scroll_to(cs, bottom);
}
unsafe fn window_copy_cmd_scroll_middle(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    let mut mid_value: u_int = 0;
    mid_value = (*data).screen.grid()
        .sy
        .wrapping_sub(1 as u_int)
        .wrapping_div(2 as u_int);
    return window_copy_cmd_scroll_to(cs, mid_value);
}
unsafe fn window_copy_cmd_scroll_to_mouse(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let c = (*cs).c.map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut m: *mut mouse_event = (*cs).m;
    let mut scroll_exit: ::core::ffi::c_int = args_has((*cs).parsed_args(), 'e' as i32 as u_char);
    let tty_oy = tty_window_offset(&(*c).tty).oy;
    window_copy_scroll(&mode_pane_owner, (*c).tty.mouse_slider_mpos, (*m).y, tty_oy, scroll_exit);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_scroll_top(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    return window_copy_cmd_scroll_to(cs, 0 as u_int);
}
unsafe fn window_copy_cmd_cursor_up(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_up(wme, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_centre_vertical(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    window_copy_update_cursor(wme, (*data).cx, (*mode_pane).sy.wrapping_div(2 as u_int));
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_centre_horizontal(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    window_copy_update_cursor(wme, (*mode_pane).sx.wrapping_div(2 as u_int), (*data).cy);
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_end_of_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cursor_end_of_line(wme);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_halfpage_down(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        if window_copy_pagedown1(wme, 1 as ::core::ffi::c_int, (*data).scroll_exit) != 0 {
            return WINDOW_COPY_CMD_CANCEL;
        }
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_halfpage_down_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        if window_copy_pagedown1(wme, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) != 0 {
            return WINDOW_COPY_CMD_CANCEL;
        }
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_halfpage_up(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_pageup1(wme, 1 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_toggle_position(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).hide_position = ((*data).hide_position == 0) as ::core::ffi::c_int;
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_history_bottom(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let s = (*data).backing();
    let mut oy: u_int = 0;
    let mut old_oy: u_int = (*data).oy;
    oy = (*s).grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        && oy == (*data).endsely
    {
        window_copy_other_end(wme);
    }
    (*data).cy = (*data).screen.grid().sy.wrapping_sub(1 as u_int);
    (*data).cx = window_copy_cursor_limit(
        wme,
        (*s).grid().hsize.wrapping_add((*data).cy),
        0 as ::core::ffi::c_int,
    );
    (*data).oy = 0 as u_int;
    if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    if (*data).oy != old_oy {
        window_pane_scrollbar_show(&mode_pane_owner);
    }
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_history_top(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oy: u_int = 0;
    let mut old_oy: u_int = (*data).oy;
    oy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        && oy == (*data).sely
    {
        window_copy_other_end(wme);
    }
    (*data).cy = 0 as u_int;
    (*data).cx = 0 as u_int;
    (*data).oy = (*data).backing().grid().hsize;
    if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    if (*data).oy != old_oy {
        window_pane_scrollbar_show(&mode_pane_owner);
    }
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_jump_again(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    match (*data).jumptype {
        3 => {
            while np != 0 as u_int {
                window_copy_cursor_jump(wme);
                np = np.wrapping_sub(1);
            }
        }
        4 => {
            while np != 0 as u_int {
                window_copy_cursor_jump_back(wme);
                np = np.wrapping_sub(1);
            }
        }
        5 => {
            while np != 0 as u_int {
                window_copy_cursor_jump_to(wme);
                np = np.wrapping_sub(1);
            }
        }
        6 => {
            while np != 0 as u_int {
                window_copy_cursor_jump_to_back(wme);
                np = np.wrapping_sub(1);
            }
        }
        _ => {}
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_jump_reverse(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    match (*data).jumptype {
        3 => {
            while np != 0 as u_int {
                window_copy_cursor_jump_back(wme);
                np = np.wrapping_sub(1);
            }
        }
        4 => {
            while np != 0 as u_int {
                window_copy_cursor_jump(wme);
                np = np.wrapping_sub(1);
            }
        }
        5 => {
            while np != 0 as u_int {
                window_copy_cursor_jump_to_back(wme);
                np = np.wrapping_sub(1);
            }
        }
        6 => {
            while np != 0 as u_int {
                window_copy_cursor_jump_to(wme);
                np = np.wrapping_sub(1);
            }
        }
        _ => {}
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_middle_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).cx = 0 as u_int;
    (*data).cy = (*data).screen.grid()
        .sy
        .wrapping_sub(1 as u_int)
        .wrapping_div(2 as u_int);
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_previous_matching_bracket(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let s = (*data).backing();
    let mut open: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"{[(\0");
    let mut close: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"}])\0");
    let mut tried: ::core::ffi::c_char = 0;
    let mut found: ::core::ffi::c_char = 0;
    let mut start: ::core::ffi::c_char = 0;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut xx: u_int = 0;
    let mut n: u_int = 0;
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
    let mut failed: ::core::ffi::c_int = 0;
    while np != 0 as u_int {
        px = (*data).cx;
        py = (*s).grid()
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        xx = window_copy_find_length(wme, py);
        if xx == 0 as u_int {
            break;
        }
        tried = 0 as ::core::ffi::c_char;
        loop {
            grid_get_cell((*s).grid(), px, py, &mut gc);
            if gc.data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
                || gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0
            {
                cp = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                found = *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_char;
                cp = strchr(
                    &raw mut close as *mut ::core::ffi::c_char,
                    found as ::core::ffi::c_int,
                );
            }
            if cp.is_null() {
                if !((*data).modekeys == MODEKEY_EMACS) {
                    break;
                }
                if tried == 0 && px > 0 as u_int {
                    px = px.wrapping_sub(1);
                    tried = 1 as ::core::ffi::c_char;
                } else {
                    window_copy_cursor_previous_word(
                        wme,
                        &raw mut close as *mut ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                    );
                    break;
                }
            } else {
                start = open[cp.offset_from(&raw mut close as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_long as usize];
                n = 1 as u_int;
                failed = 0 as ::core::ffi::c_int;
                loop {
                    if px == 0 as u_int {
                        if py == 0 as u_int {
                            failed = 1 as ::core::ffi::c_int;
                            break;
                        } else {
                            loop {
                                py = py.wrapping_sub(1);
                                xx = window_copy_find_length(wme, py);
                                if !(xx == 0 as u_int && py > 0 as u_int) {
                                    break;
                                }
                            }
                            if xx == 0 as u_int && py == 0 as u_int {
                                failed = 1 as ::core::ffi::c_int;
                                break;
                            } else {
                                px = xx.wrapping_sub(1 as u_int);
                            }
                        }
                    } else {
                        px = px.wrapping_sub(1);
                    }
                    grid_get_cell((*s).grid(), px, py, &mut gc);
                    if gc.data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                        && !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0
                    {
                        if *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                            == found as ::core::ffi::c_int
                        {
                            n = n.wrapping_add(1);
                        } else if *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                            == start as ::core::ffi::c_int
                        {
                            n = n.wrapping_sub(1);
                        }
                    }
                    if !(n != 0 as u_int) {
                        break;
                    }
                }
                if failed == 0 {
                    window_copy_scroll_to(wme, px, py, 0 as ::core::ffi::c_int);
                }
                break;
            }
        }
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_next_matching_bracket(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let s = (*data).backing();
    let mut open: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"{[(\0");
    let mut close: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"}])\0");
    let mut tried: ::core::ffi::c_char = 0;
    let mut found: ::core::ffi::c_char = 0;
    let mut end: ::core::ffi::c_char = 0;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut n: u_int = 0;
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
    let mut failed: ::core::ffi::c_int = 0;
    's_22: while np != 0 as u_int {
        px = (*data).cx;
        py = (*s).grid()
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        xx = window_copy_find_length(wme, py);
        yy = (*s).grid()
            .hsize
            .wrapping_add((*s).grid().sy)
            .wrapping_sub(1 as u_int);
        if xx == 0 as u_int {
            break;
        }
        tried = 0 as ::core::ffi::c_char;
        loop {
            grid_get_cell((*s).grid(), px, py, &mut gc);
            if gc.data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
                || gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0
            {
                cp = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                found = *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_char;
                cp = strchr(
                    &raw mut close as *mut ::core::ffi::c_char,
                    found as ::core::ffi::c_int,
                );
                if !cp.is_null() && (*data).modekeys == MODEKEY_VI {
                    sx = (*data).cx;
                    sy = (*s).grid()
                        .hsize
                        .wrapping_add((*data).cy)
                        .wrapping_sub((*data).oy);
                    window_copy_scroll_to(wme, px, py, 0 as ::core::ffi::c_int);
                    window_copy_cmd_previous_matching_bracket(cs);
                    px = (*data).cx;
                    py = (*s).grid()
                        .hsize
                        .wrapping_add((*data).cy)
                        .wrapping_sub((*data).oy);
                    grid_get_cell((*s).grid(), px, py, &mut gc);
                    if gc.data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                        && !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0
                        && !strchr(
                            &raw mut close as *mut ::core::ffi::c_char,
                            *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int,
                        )
                        .is_null()
                    {
                        window_copy_scroll_to(wme, sx, sy, 0 as ::core::ffi::c_int);
                    }
                    break 's_22;
                } else {
                    cp = strchr(
                        &raw mut open as *mut ::core::ffi::c_char,
                        found as ::core::ffi::c_int,
                    );
                }
            }
            if cp.is_null() {
                if (*data).modekeys == MODEKEY_EMACS {
                    if tried == 0 && px <= xx {
                        px = px.wrapping_add(1);
                        tried = 1 as ::core::ffi::c_char;
                    } else {
                        window_copy_cursor_next_word_end(
                            wme,
                            &raw mut open as *mut ::core::ffi::c_char,
                            0 as ::core::ffi::c_int,
                        );
                        break;
                    }
                } else if px > xx {
                    if py == yy {
                        break;
                    }
                    let gl = grid_get_line((*s).grid(), py);
                    if !(gl.flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
                        break;
                    }
                    if gl.cellsize as u_int > (*s).grid().sx {
                        break;
                    }
                    px = 0 as u_int;
                    py = py.wrapping_add(1);
                    xx = window_copy_find_length(wme, py);
                } else {
                    px = px.wrapping_add(1);
                }
            } else {
                end = close[cp.offset_from(&raw mut open as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_long as usize];
                n = 1 as u_int;
                failed = 0 as ::core::ffi::c_int;
                loop {
                    if px > xx {
                        if py == yy {
                            failed = 1 as ::core::ffi::c_int;
                            break;
                        } else {
                            px = 0 as u_int;
                            py = py.wrapping_add(1);
                            xx = window_copy_find_length(wme, py);
                        }
                    } else {
                        px = px.wrapping_add(1);
                    }
                    grid_get_cell((*s).grid(), px, py, &mut gc);
                    if gc.data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                        && !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0
                    {
                        if *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                            == found as ::core::ffi::c_int
                        {
                            n = n.wrapping_add(1);
                        } else if *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                            == end as ::core::ffi::c_int
                        {
                            n = n.wrapping_sub(1);
                        }
                    }
                    if !(n != 0 as u_int) {
                        break;
                    }
                }
                if failed == 0 {
                    window_copy_scroll_to(wme, px, py, 0 as ::core::ffi::c_int);
                }
                break;
            }
        }
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_next_paragraph(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_next_paragraph(wme);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_next_space(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_next_word(wme, b"\0" as *const u8 as *const ::core::ffi::c_char);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_next_space_end(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_next_word_end(
            wme,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_next_word(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    let mut separators: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    separators = options_get_string(
        options_owner_ptr(&mut (*(*cs).s_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
        b"word-separators\0" as *const u8 as *const ::core::ffi::c_char,
    );
    while np != 0 as u_int {
        window_copy_cursor_next_word(wme, separators);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_next_word_end(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    let mut separators: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    separators = options_get_string(
        options_owner_ptr(&mut (*(*cs).s_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
        b"word-separators\0" as *const u8 as *const ::core::ffi::c_char,
    );
    while np != 0 as u_int {
        window_copy_cursor_next_word_end(wme, separators, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_other_end(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).selflag = SEL_CHAR;
    if np.wrapping_rem(2 as u_int) != 0 as u_int {
        window_copy_other_end(wme);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_selection_mode(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut so: *mut options = options_owner_ptr(&mut (*(*cs).s_ptr()).options).map_or(std::ptr::null_mut(), |options| options);
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut ex: u_int = 0;
    let mut ey: u_int = 0;
    let mut fx: u_int = 0;
    let mut fy: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if s.is_null()
        || strcasecmp(s, b"char\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        || strcasecmp(s, b"c\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        (*data).selflag = SEL_CHAR;
    } else if strcasecmp(s, b"word\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcasecmp(s, b"w\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        (*data).separators = window_copy_snapshot_separators(so);
        (*data).selflag = SEL_WORD;
    } else if strcasecmp(s, b"line\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcasecmp(s, b"l\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        (*data).selflag = SEL_LINE;
        if (*data).screen.sel.is_none() {
            return WINDOW_COPY_CMD_MOVE;
        }
        if (*data).cursordrag as ::core::ffi::c_uint
            == CURSORDRAG_SEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            fx = (*data).endselx;
            fy = (*data).endsely;
        } else {
            fx = (*data).selx;
            fy = (*data).sely;
        }
        sx = (*data).selx;
        sy = (*data).sely;
        ex = (*data).endselx;
        ey = (*data).endsely;
        if ey < sy || ey == sy && ex < sx {
            x = sx;
            sx = ex;
            ex = x;
            y = sy;
            sy = ey;
            ey = y;
        }
        let mut gr = grid_reader_start((*data).backing().grid(), sx, sy);
        grid_reader_cursor_start_of_line(&mut gr, 1 as ::core::ffi::c_int);
        (sx, sy) = grid_reader_get_cursor(&gr);
        let mut gr = grid_reader_start((*data).backing().grid(), ex, ey);
        grid_reader_cursor_end_of_line(&mut gr, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        (ex, ey) = grid_reader_get_cursor(&gr);
        (*data).rectflag = 0 as ::core::ffi::c_int;
        (*data).selx = sx;
        (*data).selrx = (*data).selx;
        (*data).sely = sy;
        (*data).selry = (*data).sely;
        (*data).endselx = ex;
        (*data).endselrx = (*data).endselx;
        (*data).endsely = ey;
        (*data).endselry = (*data).endsely;
        x = (*data).cx;
        y = (*data).backing()
            .grid()
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        (*data).dx = fx;
        (*data).dy = fy;
        if (*data).cursordrag as ::core::ffi::c_uint
            != CURSORDRAG_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            && (y < fy || y == fy && x < fx)
        {
            (*data).lineflag = LINE_SEL_RIGHT_LEFT;
            (*data).cursordrag = CURSORDRAG_SEL;
            window_copy_scroll_to(wme, sx, sy, 1 as ::core::ffi::c_int);
        } else {
            (*data).lineflag = LINE_SEL_LEFT_RIGHT;
            if (*data).cursordrag as ::core::ffi::c_uint
                != CURSORDRAG_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*data).cursordrag = CURSORDRAG_ENDSEL;
                x = window_copy_cursor_limit(wme, ey, 0 as ::core::ffi::c_int);
                window_copy_scroll_to(wme, x, ey, 1 as ::core::ffi::c_int);
            }
        }
        if (*data).cursordrag as ::core::ffi::c_uint
            == CURSORDRAG_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_copy_set_selection(wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        }
        return WINDOW_COPY_CMD_REDRAW;
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_page_down(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        if window_copy_pagedown1(wme, 0 as ::core::ffi::c_int, (*data).scroll_exit) != 0 {
            return WINDOW_COPY_CMD_CANCEL;
        }
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_page_down_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        if window_copy_pagedown1(wme, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) != 0 {
            return WINDOW_COPY_CMD_CANCEL;
        }
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_page_up(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_pageup1(wme, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_previous_paragraph(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_previous_paragraph(wme);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_previous_space(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_previous_word(
            wme,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_previous_word(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    let mut separators: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    separators = options_get_string(
        options_owner_ptr(&mut (*(*cs).s_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
        b"word-separators\0" as *const u8 as *const ::core::ffi::c_char,
    );
    while np != 0 as u_int {
        window_copy_cursor_previous_word(wme, separators, 1 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_rectangle_on(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).lineflag = LINE_SEL_NONE;
    window_copy_rectangle_set(wme, 1 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_rectangle_off(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).lineflag = LINE_SEL_NONE;
    window_copy_rectangle_set(wme, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_rectangle_toggle(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).lineflag = LINE_SEL_NONE;
    window_copy_rectangle_set(wme, ((*data).rectflag == 0) as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_scroll_exit_on(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    (*data).scroll_exit = 1 as ::core::ffi::c_int;
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_scroll_exit_off(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    (*data).scroll_exit = 0 as ::core::ffi::c_int;
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_scroll_exit_toggle(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    (*data).scroll_exit = ((*data).scroll_exit == 0) as ::core::ffi::c_int;
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_scroll_down(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut dragging: ::core::ffi::c_int = 0;
    if (*data).oy == 0 as u_int {
        if (*data).scroll_exit != 0 && (*data).screen.sel.is_none() {
            return WINDOW_COPY_CMD_CANCEL;
        }
        return WINDOW_COPY_CMD_NOTHING;
    }
    dragging = (*cs).c.is_some_and(|owner| (*owner.get()).tty.mouse_drag_flag != 0)
        as ::core::ffi::c_int;
    if !(*data).screen.sel.is_none() && dragging == 0 {
        (*data).cursordrag = CURSORDRAG_NONE;
        (*data).lineflag = LINE_SEL_NONE;
        window_copy_scroll_up(wme, np);
        return WINDOW_COPY_CMD_NOTHING;
    }
    while np != 0 as u_int {
        window_copy_cursor_down(wme, 1 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    if (*data).scroll_exit != 0 && (*data).oy == 0 as u_int && (*data).screen.sel.is_none() {
        return WINDOW_COPY_CMD_CANCEL;
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_scroll_down_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut dragging: ::core::ffi::c_int = 0;
    dragging = (*cs).c.is_some_and(|owner| (*owner.get()).tty.mouse_drag_flag != 0)
        as ::core::ffi::c_int;
    if !(*data).screen.sel.is_none() && dragging == 0 {
        (*data).cursordrag = CURSORDRAG_NONE;
        (*data).lineflag = LINE_SEL_NONE;
        window_copy_scroll_up(wme, np);
        return WINDOW_COPY_CMD_NOTHING;
    }
    while np != 0 as u_int {
        window_copy_cursor_down(wme, 1 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    if (*data).oy == 0 as u_int {
        return WINDOW_COPY_CMD_CANCEL;
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_scroll_up(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut dragging: ::core::ffi::c_int = 0;
    if (*data).oy == (*data).backing().grid().hsize {
        return WINDOW_COPY_CMD_NOTHING;
    }
    dragging = (*cs).c.is_some_and(|owner| (*owner.get()).tty.mouse_drag_flag != 0)
        as ::core::ffi::c_int;
    if !(*data).screen.sel.is_none() && dragging == 0 {
        (*data).cursordrag = CURSORDRAG_NONE;
        (*data).lineflag = LINE_SEL_NONE;
        window_copy_scroll_down(wme, np);
        return WINDOW_COPY_CMD_NOTHING;
    }
    while np != 0 as u_int {
        window_copy_cursor_up(wme, 1 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_search_again(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    if (*data).searchtype == WINDOW_COPY_SEARCHUP as ::core::ffi::c_int {
        while np != 0 as u_int {
            window_copy_search_up(wme, (*data).searchregex);
            np = np.wrapping_sub(1);
        }
    } else if (*data).searchtype == WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int {
        while np != 0 as u_int {
            window_copy_search_down(wme, (*data).searchregex);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_search_reverse(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    if (*data).searchtype == WINDOW_COPY_SEARCHUP as ::core::ffi::c_int {
        while np != 0 as u_int {
            window_copy_search_down(wme, (*data).searchregex);
            np = np.wrapping_sub(1);
        }
    } else if (*data).searchtype == WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int {
        while np != 0 as u_int {
            window_copy_search_up(wme, (*data).searchregex);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_select_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    (*data).lineflag = LINE_SEL_LEFT_RIGHT;
    (*data).rectflag = 0 as ::core::ffi::c_int;
    (*data).selflag = SEL_LINE;
    (*data).dx = (*data).cx;
    (*data).dy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    window_copy_cursor_start_of_line(wme);
    (*data).selrx = (*data).cx;
    (*data).selry = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).endselry = (*data).selry;
    window_copy_start_selection(wme);
    window_copy_cursor_end_of_line(wme);
    (*data).endselry = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).endselrx = window_copy_find_length(wme, (*data).endselry);
    while np > 1 as u_int {
        window_copy_cursor_down(wme, 0 as ::core::ffi::c_int);
        window_copy_cursor_end_of_line(wme);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_select_word(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut so: *mut options = options_owner_ptr(&mut (*(*cs).s_ptr()).options).map_or(std::ptr::null_mut(), |options| options);
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut nextx: u_int = 0;
    let mut nexty: u_int = 0;
    (*data).lineflag = LINE_SEL_LEFT_RIGHT;
    (*data).rectflag = 0 as ::core::ffi::c_int;
    (*data).selflag = SEL_WORD;
    (*data).dx = (*data).cx;
    (*data).dy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).separators = window_copy_snapshot_separators(so);
    window_copy_cursor_previous_word(wme, window_copy_separators(&*data), 0 as ::core::ffi::c_int);
    px = (*data).cx;
    py = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).selrx = px;
    (*data).selry = py;
    window_copy_start_selection(wme);
    nextx = px.wrapping_add(1 as u_int);
    nexty = py;
    if (*grid_get_line((*data).backing().grid(), nexty)).flags as ::core::ffi::c_int
        & GRID_LINE_WRAPPED
        != 0
        && nextx > (*data).backing().grid().sx.wrapping_sub(1 as u_int)
    {
        nextx = 0 as u_int;
        nexty = nexty.wrapping_add(1);
    }
    if px >= window_copy_find_length(wme, py)
        || window_copy_in_set(wme, nextx, nexty, WHITESPACE.as_ptr()) == 0
    {
        window_copy_cursor_next_word_end(wme, window_copy_separators(&*data), 1 as ::core::ffi::c_int);
    } else {
        window_copy_update_cursor(wme, px, (*data).cy);
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) != 0
        {
            window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
        }
    }
    (*data).endselrx = (*data).cx;
    (*data).endselry = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    if (*data).dy > (*data).endselry {
        (*data).dy = (*data).endselry;
        (*data).dx = (*data).endselrx;
    } else if (*data).dx > (*data).endselrx {
        (*data).dx = (*data).endselrx;
    }
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_set_mark(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    (*data).mx = (*data).cx;
    (*data).my = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).showmark = 1 as ::core::ffi::c_int;
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_start_of_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cursor_start_of_line(wme);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_top_line(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).cx = 0 as u_int;
    (*data).cy = 0 as u_int;
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_copy_pipe_no_clear(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let c = (*cs).c.map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut s: *mut session = (*cs).s_ptr();
    let mut wl: *mut winlink = (*cs).wl_ptr();
    let mut wp: *mut window_pane = mode_pane;
    let mut command: Option<CString> = None;
    let mut prefix: Option<CString> = None;
    let mut arg0: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut arg1: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 1 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut set_paste: ::core::ffi::c_int =
        (args_has((*cs).parsed_args(), 'P' as i32 as u_char) == 0) as ::core::ffi::c_int;
    let mut set_clip: ::core::ffi::c_int =
        (args_has((*cs).parsed_args(), 'C' as i32 as u_char) == 0) as ::core::ffi::c_int;
    if !arg1.is_null() {
        prefix = Some(format_single_cstring(
            ::core::ptr::null_mut::<cmdq_item>(),
            arg1,
            c,
            s,
            wl,
            wp,
        ));
    }
    if !s.is_null() && !arg0.is_null() && *arg0 as ::core::ffi::c_int != '\0' as i32 {
        command = Some(format_single_cstring(
            ::core::ptr::null_mut::<cmdq_item>(),
            arg0,
            c,
            s,
            wl,
            wp,
        ));
    }
    window_copy_copy_pipe(
        wme,
        s,
        prefix
            .as_ref()
            .map_or(::core::ptr::null(), |value| value.as_ptr()),
        command
            .as_ref()
            .map_or(::core::ptr::null(), |value| value.as_ptr()),
        set_paste,
        set_clip,
    );
    return WINDOW_COPY_CMD_NOTHING;
}
unsafe fn window_copy_cmd_copy_pipe(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cmd_copy_pipe_no_clear(cs);
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_copy_pipe_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cmd_copy_pipe_no_clear(cs);
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_CANCEL;
}
unsafe fn window_copy_cmd_pipe_no_clear(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let c = (*cs).c.map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut s: *mut session = (*cs).s_ptr();
    let mut wl: *mut winlink = (*cs).wl_ptr();
    let mut wp: *mut window_pane = mode_pane;
    let mut command: Option<CString> = None;
    let mut arg0: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    if !s.is_null() && !arg0.is_null() && *arg0 as ::core::ffi::c_int != '\0' as i32 {
        command = Some(format_single_cstring(
            ::core::ptr::null_mut::<cmdq_item>(),
            arg0,
            c,
            s,
            wl,
            wp,
        ));
    }
    window_copy_pipe(
        wme,
        s,
        command
            .as_ref()
            .map_or(::core::ptr::null(), |value| value.as_ptr()),
    );
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_pipe(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cmd_pipe_no_clear(cs);
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_pipe_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cmd_pipe_no_clear(cs);
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_CANCEL;
}
unsafe fn window_copy_cmd_goto_line(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut arg0: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    if *arg0 as ::core::ffi::c_int != '\0' as i32 {
        window_copy_goto_line(wme, arg0);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_jump_backward(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut arg0: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    if *arg0 as ::core::ffi::c_int != '\0' as i32 {
        (*data).jumptype = WINDOW_COPY_JUMPBACKWARD as ::core::ffi::c_int;
        (*data).jumpchar = utf8_fromcstr_vec(CStr::from_ptr(arg0));
        while np != 0 as u_int {
            window_copy_cursor_jump_back(wme);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_jump_forward(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut arg0: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    if *arg0 as ::core::ffi::c_int != '\0' as i32 {
        (*data).jumptype = WINDOW_COPY_JUMPFORWARD as ::core::ffi::c_int;
        (*data).jumpchar = utf8_fromcstr_vec(CStr::from_ptr(arg0));
        while np != 0 as u_int {
            window_copy_cursor_jump(wme);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_jump_to_backward(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut arg0: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    if *arg0 as ::core::ffi::c_int != '\0' as i32 {
        (*data).jumptype = WINDOW_COPY_JUMPTOBACKWARD as ::core::ffi::c_int;
        (*data).jumpchar = utf8_fromcstr_vec(CStr::from_ptr(arg0));
        while np != 0 as u_int {
            window_copy_cursor_jump_to_back(wme);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_jump_to_forward(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut arg0: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    if *arg0 as ::core::ffi::c_int != '\0' as i32 {
        (*data).jumptype = WINDOW_COPY_JUMPTOFORWARD as ::core::ffi::c_int;
        (*data).jumpchar = utf8_fromcstr_vec(CStr::from_ptr(arg0));
        while np != 0 as u_int {
            window_copy_cursor_jump_to(wme);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_jump_to_mark(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_jump_to_mark(wme);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_next_prompt(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cursor_prompt(
        wme,
        1 as ::core::ffi::c_int,
        args_has((*cs).parsed_args(), 'o' as i32 as u_char),
    );
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_previous_prompt(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cursor_prompt(
        wme,
        0 as ::core::ffi::c_int,
        args_has((*cs).parsed_args(), 'o' as i32 as u_char),
    );
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_search_backward(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    if window_copy_expand_search_string(cs) == 0 {
        return WINDOW_COPY_CMD_MOVE;
    }
    if (*data).searchstr.is_some() {
        (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
        (*data).searchregex = 1 as ::core::ffi::c_int;
        (*data).timeout = 0 as ::core::ffi::c_int;
        while np != 0 as u_int {
            window_copy_search_up(wme, 1 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_search_backward_text(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    if window_copy_expand_search_string(cs) == 0 {
        return WINDOW_COPY_CMD_MOVE;
    }
    if (*data).searchstr.is_some() {
        (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
        (*data).searchregex = 0 as ::core::ffi::c_int;
        (*data).timeout = 0 as ::core::ffi::c_int;
        while np != 0 as u_int {
            window_copy_search_up(wme, 0 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_search_forward(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    if window_copy_expand_search_string(cs) == 0 {
        return WINDOW_COPY_CMD_MOVE;
    }
    if (*data).searchstr.is_some() {
        (*data).searchtype = WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int;
        (*data).searchregex = 1 as ::core::ffi::c_int;
        (*data).timeout = 0 as ::core::ffi::c_int;
        while np != 0 as u_int {
            window_copy_search_down(wme, 1 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_search_forward_text(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    if window_copy_expand_search_string(cs) == 0 {
        return WINDOW_COPY_CMD_MOVE;
    }
    if (*data).searchstr.is_some() {
        (*data).searchtype = WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int;
        (*data).searchregex = 0 as ::core::ffi::c_int;
        (*data).timeout = 0 as ::core::ffi::c_int;
        while np != 0 as u_int {
            window_copy_search_down(wme, 0 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_search_backward_incremental(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut arg0: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut ss: *const ::core::ffi::c_char = window_copy_searchstr(&*data);
    let mut prefix: ::core::ffi::c_char = 0;
    let mut action: window_copy_cmd_action = WINDOW_COPY_CMD_MOVE;
    (*data).timeout = 0 as ::core::ffi::c_int;
    log_debug(format_args!(
        "{}: {}",
        "window_copy_cmd_search_backward_incremental",
        log_cstr((arg0) as *const _)
    ));
    let fresh3 = arg0;
    arg0 = arg0.offset(1);
    prefix = *fresh3;
    if (*data).searchx == -(1 as ::core::ffi::c_int)
        || (*data).searchy == -(1 as ::core::ffi::c_int)
    {
        (*data).searchx = (*data).cx as ::core::ffi::c_int;
        (*data).searchy = (*data).cy as ::core::ffi::c_int;
        (*data).searcho = (*data).oy as ::core::ffi::c_int;
    } else if !ss.is_null() && strcmp(arg0, ss) != 0 as ::core::ffi::c_int {
        (*data).cx = (*data).searchx as u_int;
        (*data).cy = (*data).searchy as u_int;
        (*data).oy = (*data).searcho as u_int;
        (*data).cx = window_copy_cursor_limit(
            wme,
            (*data).backing()
                .grid()
                .hsize
                .wrapping_add((*data).cy)
                .wrapping_sub((*data).oy),
            0 as ::core::ffi::c_int,
        );
        action = WINDOW_COPY_CMD_REDRAW;
    }
    if *arg0 as ::core::ffi::c_int == '\0' as i32 {
        window_copy_clear_marks(wme);
        return WINDOW_COPY_CMD_REDRAW;
    }
    match prefix as ::core::ffi::c_int {
        61 | 45 => {
            (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
            (*data).searchregex = 0 as ::core::ffi::c_int;
            (*data).searchstr = Some(CStr::from_ptr(arg0).to_owned());
            if window_copy_search_up(wme, 0 as ::core::ffi::c_int) == 0 {
                window_copy_clear_marks(wme);
                return WINDOW_COPY_CMD_REDRAW;
            }
        }
        43 => {
            (*data).searchtype = WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int;
            (*data).searchregex = 0 as ::core::ffi::c_int;
            (*data).searchstr = Some(CStr::from_ptr(arg0).to_owned());
            if window_copy_search_down(wme, 0 as ::core::ffi::c_int) == 0 {
                window_copy_clear_marks(wme);
                return WINDOW_COPY_CMD_REDRAW;
            }
        }
        _ => {}
    }
    return action;
}
unsafe fn window_copy_cmd_search_forward_incremental(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut arg0: *const ::core::ffi::c_char = args_string(&mut *((*cs).parsed_args()), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut ss: *const ::core::ffi::c_char = window_copy_searchstr(&*data);
    let mut prefix: ::core::ffi::c_char = 0;
    let mut action: window_copy_cmd_action = WINDOW_COPY_CMD_MOVE;
    (*data).timeout = 0 as ::core::ffi::c_int;
    log_debug(format_args!(
        "{}: {}",
        "window_copy_cmd_search_forward_incremental",
        log_cstr((arg0) as *const _)
    ));
    let fresh2 = arg0;
    arg0 = arg0.offset(1);
    prefix = *fresh2;
    if (*data).searchx == -(1 as ::core::ffi::c_int)
        || (*data).searchy == -(1 as ::core::ffi::c_int)
    {
        (*data).searchx = (*data).cx as ::core::ffi::c_int;
        (*data).searchy = (*data).cy as ::core::ffi::c_int;
        (*data).searcho = (*data).oy as ::core::ffi::c_int;
    } else if !ss.is_null() && strcmp(arg0, ss) != 0 as ::core::ffi::c_int {
        (*data).cx = (*data).searchx as u_int;
        (*data).cy = (*data).searchy as u_int;
        (*data).oy = (*data).searcho as u_int;
        (*data).cx = window_copy_cursor_limit(
            wme,
            (*data).backing()
                .grid()
                .hsize
                .wrapping_add((*data).cy)
                .wrapping_sub((*data).oy),
            0 as ::core::ffi::c_int,
        );
        action = WINDOW_COPY_CMD_REDRAW;
    }
    if *arg0 as ::core::ffi::c_int == '\0' as i32 {
        window_copy_clear_marks(wme);
        return WINDOW_COPY_CMD_REDRAW;
    }
    match prefix as ::core::ffi::c_int {
        61 | 43 => {
            (*data).searchtype = WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int;
            (*data).searchregex = 0 as ::core::ffi::c_int;
            (*data).searchstr = Some(CStr::from_ptr(arg0).to_owned());
            if window_copy_search_down(wme, 0 as ::core::ffi::c_int) == 0 {
                window_copy_clear_marks(wme);
                return WINDOW_COPY_CMD_REDRAW;
            }
        }
        45 => {
            (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
            (*data).searchregex = 0 as ::core::ffi::c_int;
            (*data).searchstr = Some(CStr::from_ptr(arg0).to_owned());
            if window_copy_search_up(wme, 0 as ::core::ffi::c_int) == 0 {
                window_copy_clear_marks(wme);
                return WINDOW_COPY_CMD_REDRAW;
            }
        }
        _ => {}
    }
    return action;
}
unsafe fn window_copy_do_refresh(mut wme: *mut window_mode_entry, mut follow: ::core::ffi::c_int) {
    let Some(source_owner) = (*wme).swp.upgrade() else {
        return;
    };
    let wp = source_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oy_from_top: u_int = 0;
    if (*data).oy > (*data).backing().grid().hsize {
        (*data).oy = (*data).backing().grid().hsize;
    }
    oy_from_top = (*data).backing().grid().hsize.wrapping_sub((*data).oy);
    if window_copy_sync_backing(wme) == 0 {
        (*data).clear_backing();
        (*data).backing = Some(window_copy_clone_screen(
            &(*wp).base,
            &(*data).screen,
            None,
            !(*wme).swp.ptr_eq(&(*wme).wp),
        ));
    }
    if follow != 0 {
        (*data).cy = (*data).screen.grid().sy.wrapping_sub(1 as u_int);
        (*data).cx = window_copy_cursor_limit(
            wme,
            (*data).backing().grid().hsize.wrapping_add((*data).cy),
            0 as ::core::ffi::c_int,
        );
        (*data).oy = 0 as u_int;
    } else if oy_from_top <= (*data).backing().grid().hsize {
        (*data).oy = (*data).backing().grid().hsize.wrapping_sub(oy_from_top);
    } else {
        (*data).cy = 0 as u_int;
        (*data).oy = (*data).backing().grid().hsize;
    }
    window_copy_sync_snapshot(data, (*wp).base.grid_mut());
    window_copy_size_changed(wme);
}
unsafe fn window_copy_refresh_arm(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut tv: timeval = timeval {
        tv_sec: (WINDOW_COPY_REFRESH_INTERVAL / 1000000 as ::core::ffi::c_int) as __time_t,
        tv_usec: (WINDOW_COPY_REFRESH_INTERVAL % 1000000 as ::core::ffi::c_int) as __suseconds_t,
    };
    if (*data).refresh_active != 0 {
        event_add(&raw mut (*data).refresh_timer, &raw mut tv);
    }
}
unsafe fn window_copy_refresh_allowed(mut wme: *mut window_mode_entry) -> ::core::ffi::c_int {
    let Some(source_owner) = (*wme).swp.upgrade() else {
        return 0;
    };
    let wp = source_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    if (*data).viewmode != 0 || !(*wme).swp.ptr_eq(&(*wme).wp) {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_copy_refresh_timer(mut arg: *mut ::core::ffi::c_void) {
    let mut wme: *mut window_mode_entry = arg as *mut window_mode_entry;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut follow: ::core::ffi::c_int = 0;
    if (*wp).modes.active != wme || (*data).refresh_active == 0 {
        return;
    }
    if (*wp).flags & PANE_UNSEENCHANGES != 0
        && (*data).screen.sel.is_none()
        && (*data).cursordrag as ::core::ffi::c_uint
            == CURSORDRAG_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        follow = ((*data).oy == 0 as u_int
            && (*data).cy == (*data).screen.grid().sy.wrapping_sub(1 as u_int))
            as ::core::ffi::c_int;
        window_copy_do_refresh(wme, follow);
        window_copy_redraw_screen(wme);
        (*wp).flags |= PANE_REDRAW;
        (*wp).flags &= !PANE_UNSEENCHANGES;
    }
    window_copy_refresh_arm(wme);
}
unsafe fn window_copy_refresh_start(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    if window_copy_refresh_allowed(wme) == 0 || (*data).refresh_active != 0 {
        return;
    }
    (*data).refresh_active = 1 as ::core::ffi::c_int;
    window_copy_refresh_arm(wme);
}
unsafe fn window_copy_refresh_stop(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).refresh_active = 0 as ::core::ffi::c_int;
    event_del(&raw mut (*data).refresh_timer);
}
unsafe fn window_copy_cmd_refresh_now(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut wp: *mut window_pane = mode_pane;
    let mut follow: ::core::ffi::c_int = 0;
    if window_copy_refresh_allowed(wme) == 0 {
        return WINDOW_COPY_CMD_NOTHING;
    }
    follow = ((*data).oy == 0 as u_int
        && (*data).cy == (*data).screen.grid().sy.wrapping_sub(1 as u_int))
        as ::core::ffi::c_int;
    window_copy_do_refresh(wme, follow);
    (*wp).flags &= !PANE_UNSEENCHANGES;
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_refresh_on(mut cs: *mut window_copy_cmd_state) -> window_copy_cmd_action {
    window_copy_refresh_start((*cs).wme);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_refresh_off(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    window_copy_refresh_stop((*cs).wme);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_refresh_toggle(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    if (*data).refresh_active != 0 {
        window_copy_refresh_stop((*cs).wme);
    } else {
        window_copy_refresh_start((*cs).wme);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe fn window_copy_cmd_recentre_top_bottom(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut cy: u_int = (*data).cy;
    let mut oy: u_int = (*data).oy;
    let mut sy: u_int = (*data).screen.grid().sy.wrapping_sub(1 as u_int);
    let mut sm: u_int = sy.wrapping_div(2 as u_int);
    let mut backing_row: u_int = 0;
    let mut target: C2RustUnnamed_48 = MIDDLE;
    backing_row = (*data).backing().grid()
        .hsize
        .wrapping_add(cy)
        .wrapping_sub((*data).oy);
    if (*data).recentre_line != backing_row {
        (*data).recentre_state = RECENTRE_MIDDLE;
        (*data).recentre_line = backing_row;
    }
    match (*data).recentre_state as ::core::ffi::c_uint {
        1 => {
            (*data).recentre_state = RECENTRE_TOP;
            target = MIDDLE;
        }
        0 => {
            (*data).recentre_state = RECENTRE_BOTTOM;
            target = TOP;
        }
        2 | _ => {
            (*data).recentre_state = RECENTRE_MIDDLE;
            target = BOTTOM;
        }
    }
    oy = (*data).oy;
    match target as ::core::ffi::c_uint {
        0 => {
            if cy < sm {
                window_copy_scroll_down(wme, sm.wrapping_sub(cy));
            } else if cy > sm {
                window_copy_scroll_up(wme, cy.wrapping_sub(sm));
            }
            if (*data).oy != oy {
                (*data).cy = cy.wrapping_add((*data).oy.wrapping_sub(oy));
            }
        }
        1 => {
            window_copy_scroll_up(wme, cy);
            (*data).cy = cy.wrapping_sub(oy.wrapping_sub((*data).oy));
        }
        2 => {
            window_copy_scroll_down(wme, sy.wrapping_sub(cy));
            (*data).cy = cy.wrapping_add((*data).oy.wrapping_sub(oy));
        }
        _ => {}
    }
    window_copy_update_selection_view(wme, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe fn window_copy_cmd_line_numbers_on(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    window_copy_set_line_numbers1((*cs).wme, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_NOTHING;
}
unsafe fn window_copy_cmd_line_numbers_off(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    window_copy_set_line_numbers1((*cs).wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_NOTHING;
}
unsafe fn window_copy_cmd_line_numbers_toggle(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    window_copy_set_line_numbers1(
        (*cs).wme,
        (window_copy_line_numbers_active((*cs).wme) == 0) as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    return WINDOW_COPY_CMD_NOTHING;
}
static mut window_copy_cmd_table: [C2RustUnnamed_46; 99] = {
    [
        C2RustUnnamed_46 {
            command: c"append-selection",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_append_selection),
        },
        C2RustUnnamed_46 {
            command: c"append-selection-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_append_selection_and_cancel),
        },
        C2RustUnnamed_46 {
            command: c"back-to-indentation",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_back_to_indentation),
        },
        C2RustUnnamed_46 {
            command: c"begin-selection",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_begin_selection),
        },
        C2RustUnnamed_46 {
            command: c"bottom-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_bottom_line),
        },
        C2RustUnnamed_46 {
            command: c"cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_cancel),
        },
        C2RustUnnamed_46 {
            command: c"clear-selection",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_clear_selection),
        },
        C2RustUnnamed_46 {
            command: c"copy-end-of-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_end_of_line),
        },
        C2RustUnnamed_46 {
            command: c"copy-end-of-line-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_end_of_line_and_cancel),
        },
        C2RustUnnamed_46 {
            command: c"copy-pipe-end-of-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_end_of_line),
        },
        C2RustUnnamed_46 {
            command: c"copy-pipe-end-of-line-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_end_of_line_and_cancel),
        },
        C2RustUnnamed_46 {
            command: c"copy-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_line),
        },
        C2RustUnnamed_46 {
            command: c"copy-line-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_line_and_cancel),
        },
        C2RustUnnamed_46 {
            command: c"copy-pipe-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_line),
        },
        C2RustUnnamed_46 {
            command: c"copy-pipe-line-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_line_and_cancel),
        },
        C2RustUnnamed_46 {
            command: c"copy-pipe-no-clear",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_copy_pipe_no_clear),
        },
        C2RustUnnamed_46 {
            command: c"copy-pipe",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe),
        },
        C2RustUnnamed_46 {
            command: c"copy-pipe-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_and_cancel),
        },
        C2RustUnnamed_46 {
            command: c"copy-selection-no-clear",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_copy_selection_no_clear),
        },
        C2RustUnnamed_46 {
            command: c"copy-selection",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_selection),
        },
        C2RustUnnamed_46 {
            command: c"copy-selection-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_selection_and_cancel),
        },
        C2RustUnnamed_46 {
            command: c"cursor-down",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_cursor_down),
        },
        C2RustUnnamed_46 {
            command: c"cursor-down-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_cursor_down_and_cancel),
        },
        C2RustUnnamed_46 {
            command: c"cursor-left",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_cursor_left),
        },
        C2RustUnnamed_46 {
            command: c"cursor-right",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_cursor_right),
        },
        C2RustUnnamed_46 {
            command: c"cursor-up",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_cursor_up),
        },
        C2RustUnnamed_46 {
            command: c"cursor-centre-vertical",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_centre_vertical),
        },
        C2RustUnnamed_46 {
            command: c"cursor-centre-horizontal",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_centre_horizontal),
        },
        C2RustUnnamed_46 {
            command: c"end-of-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_end_of_line),
        },
        C2RustUnnamed_46 {
            command: c"goto-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_goto_line),
        },
        C2RustUnnamed_46 {
            command: c"halfpage-down",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_halfpage_down),
        },
        C2RustUnnamed_46 {
            command: c"halfpage-down-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_halfpage_down_and_cancel),
        },
        C2RustUnnamed_46 {
            command: c"halfpage-up",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_halfpage_up),
        },
        C2RustUnnamed_46 {
            command: c"history-bottom",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_history_bottom),
        },
        C2RustUnnamed_46 {
            command: c"history-top",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_history_top),
        },
        C2RustUnnamed_46 {
            command: c"jump-again",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_again),
        },
        C2RustUnnamed_46 {
            command: c"jump-backward",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_backward),
        },
        C2RustUnnamed_46 {
            command: c"jump-forward",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_forward),
        },
        C2RustUnnamed_46 {
            command: c"jump-reverse",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_reverse),
        },
        C2RustUnnamed_46 {
            command: c"jump-to-backward",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_to_backward),
        },
        C2RustUnnamed_46 {
            command: c"jump-to-forward",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_to_forward),
        },
        C2RustUnnamed_46 {
            command: c"jump-to-mark",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_jump_to_mark),
        },
        C2RustUnnamed_46 {
            command: c"line-numbers-on",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_line_numbers_on),
        },
        C2RustUnnamed_46 {
            command: c"line-numbers-off",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_line_numbers_off),
        },
        C2RustUnnamed_46 {
            command: c"line-numbers-toggle",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_line_numbers_toggle),
        },
        C2RustUnnamed_46 {
            command: c"next-prompt",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"o",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_next_prompt),
        },
        C2RustUnnamed_46 {
            command: c"previous-prompt",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"o",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_previous_prompt),
        },
        C2RustUnnamed_46 {
            command: c"middle-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_middle_line),
        },
        C2RustUnnamed_46 {
            command: c"next-matching-bracket",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_next_matching_bracket),
        },
        C2RustUnnamed_46 {
            command: c"next-paragraph",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_next_paragraph),
        },
        C2RustUnnamed_46 {
            command: c"next-space",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_next_space),
        },
        C2RustUnnamed_46 {
            command: c"next-space-end",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_next_space_end),
        },
        C2RustUnnamed_46 {
            command: c"next-word",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_next_word),
        },
        C2RustUnnamed_46 {
            command: c"next-word-end",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_next_word_end),
        },
        C2RustUnnamed_46 {
            command: c"other-end",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_other_end),
        },
        C2RustUnnamed_46 {
            command: c"page-down",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_page_down),
        },
        C2RustUnnamed_46 {
            command: c"page-down-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_page_down_and_cancel),
        },
        C2RustUnnamed_46 {
            command: c"page-up",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_page_up),
        },
        C2RustUnnamed_46 {
            command: c"pipe-no-clear",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_pipe_no_clear),
        },
        C2RustUnnamed_46 {
            command: c"pipe",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_pipe),
        },
        C2RustUnnamed_46 {
            command: c"pipe-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_pipe_and_cancel),
        },
        C2RustUnnamed_46 {
            command: c"previous-matching-bracket",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_previous_matching_bracket),
        },
        C2RustUnnamed_46 {
            command: c"previous-paragraph",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_previous_paragraph),
        },
        C2RustUnnamed_46 {
            command: c"previous-space",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_previous_space),
        },
        C2RustUnnamed_46 {
            command: c"previous-word",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_previous_word),
        },
        C2RustUnnamed_46 {
            command: c"recentre-top-bottom",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_recentre_top_bottom),
        },
        C2RustUnnamed_46 {
            command: c"rectangle-on",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_rectangle_on),
        },
        C2RustUnnamed_46 {
            command: c"rectangle-off",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_rectangle_off),
        },
        C2RustUnnamed_46 {
            command: c"rectangle-toggle",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_rectangle_toggle),
        },
        C2RustUnnamed_46 {
            command: c"refresh-on",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_refresh_on),
        },
        C2RustUnnamed_46 {
            command: c"refresh-off",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_refresh_off),
        },
        C2RustUnnamed_46 {
            command: c"refresh-now",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_refresh_now),
        },
        C2RustUnnamed_46 {
            command: c"refresh-toggle",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_refresh_toggle),
        },
        C2RustUnnamed_46 {
            command: c"scroll-bottom",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_bottom),
        },
        C2RustUnnamed_46 {
            command: c"scroll-down",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_scroll_down),
        },
        C2RustUnnamed_46 {
            command: c"scroll-down-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_down_and_cancel),
        },
        C2RustUnnamed_46 {
            command: c"scroll-exit-on",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_exit_on),
        },
        C2RustUnnamed_46 {
            command: c"scroll-exit-off",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_exit_off),
        },
        C2RustUnnamed_46 {
            command: c"scroll-exit-toggle",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_exit_toggle),
        },
        C2RustUnnamed_46 {
            command: c"scroll-middle",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_middle),
        },
        C2RustUnnamed_46 {
            command: c"scroll-to-mouse",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"e",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_scroll_to_mouse),
        },
        C2RustUnnamed_46 {
            command: c"scroll-top",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_top),
        },
        C2RustUnnamed_46 {
            command: c"scroll-up",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_scroll_up),
        },
        C2RustUnnamed_46 {
            command: c"search-again",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_again),
        },
        C2RustUnnamed_46 {
            command: c"search-backward",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_backward),
        },
        C2RustUnnamed_46 {
            command: c"search-backward-text",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_backward_text),
        },
        C2RustUnnamed_46 {
            command: c"search-backward-incremental",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_backward_incremental),
        },
        C2RustUnnamed_46 {
            command: c"search-forward",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_forward),
        },
        C2RustUnnamed_46 {
            command: c"search-forward-text",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_forward_text),
        },
        C2RustUnnamed_46 {
            command: c"search-forward-incremental",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_forward_incremental),
        },
        C2RustUnnamed_46 {
            command: c"search-reverse",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_reverse),
        },
        C2RustUnnamed_46 {
            command: c"select-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_select_line),
        },
        C2RustUnnamed_46 {
            command: c"select-word",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_select_word),
        },
        C2RustUnnamed_46 {
            command: c"selection-mode",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_selection_mode),
        },
        C2RustUnnamed_46 {
            command: c"set-mark",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_set_mark),
        },
        C2RustUnnamed_46 {
            command: c"start-of-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_start_of_line),
        },
        C2RustUnnamed_46 {
            command: c"stop-selection",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_stop_selection),
        },
        C2RustUnnamed_46 {
            command: c"toggle-position",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_toggle_position),
        },
        C2RustUnnamed_46 {
            command: c"top-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_top_line),
        },
    ]
};
pub const WINDOW_COPY_CMD_FLAG_READONLY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
unsafe fn window_copy_command(
    wme: *mut window_mode_entry,
    client_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
    session: Option<&std::rc::Rc<std::cell::UnsafeCell<session>>>,
    wl: *mut winlink,
    args: *mut args,
    m: *mut mouse_event,
) {
    let session_owner = session.cloned();
    window_copy_command_with_session(wme, client_owner, session_owner.as_deref(), wl, args, m);
    if let Some(owner) = session_owner {
        crate::src::session::session_remove_ref(owner, c"window_copy_command");
    }
}

unsafe fn window_copy_command_with_session(
    wme: *mut window_mode_entry,
    client_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
    s: Option<&std::cell::UnsafeCell<session>>,
    wl: *mut winlink,
    args: *mut args,
    m: *mut mouse_event,
) {
    let c = client_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut wp: *mut window_pane = mode_pane;
    let mut cs: window_copy_cmd_state = window_copy_cmd_state {
        wme: ::core::ptr::null_mut::<window_mode_entry>(),
        format_search: false,
        wargs: None,
        m: ::core::ptr::null_mut::<mouse_event>(),
        c: None,
        s: None,
        wl: refbox::Weak::new(),
    };
    let mut action: window_copy_cmd_action = WINDOW_COPY_CMD_NOTHING;
    let mut clear: window_copy_cmd_clear = WINDOW_COPY_CMD_CLEAR_NEVER;
    let mut command: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut count: u_int = args_count(args);
    let mut keys: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0;
    if count == 0 as u_int {
        return;
    }
    command = args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
    if !m.is_null()
        && (*m).valid != 0
        && !((*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
            || (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int)
    {
        window_copy_move_mouse(m);
    }
    cs.wme = wme;
    cs.format_search = args_has(args, 'F' as i32 as u_char) != 0;
    cs.wargs = None;
    cs.m = m;
    cs.c = client_owner;
    cs.s = s;
    cs.wl = wl.as_ref().map_or_else(refbox::Weak::new, |wl| wl.observer.clone());
    action = WINDOW_COPY_CMD_MOVE;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_46; 99]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_46>() as usize)
    {
        if strcmp(window_copy_cmd_table[i as usize].command.as_ptr(), command) == 0 as ::core::ffi::c_int {
            flags = window_copy_cmd_table[i as usize].flags;
            if !c.is_null()
                && (*c).flags & CLIENT_READONLY as uint64_t != 0
                && !flags & WINDOW_COPY_CMD_FLAG_READONLY != 0
            {
                status_message_set(
                    c,
                    -(1 as ::core::ffi::c_int),
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    |out| out.write_all(b"client is read-only"),
                );
                return;
            }
            cs.wargs =
                args_parse(&window_copy_cmd_table[i as usize].args, &(*args).values).ok();
            if cs.wargs.is_none() {
                break;
            }
            clear = window_copy_cmd_table[i as usize].clear;
            action = window_copy_cmd_table[i as usize]
                .f
                .expect("non-null function pointer")(&raw mut cs);
            drop(cs.wargs.take());
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if strncmp(
        command,
        b"search-\0" as *const u8 as *const ::core::ffi::c_char,
        7 as size_t,
    ) != 0 as ::core::ffi::c_int
        && !(*data).searchmark.is_empty()
    {
        keys = options_get_number(
            options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options),
            b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int;
        if clear as ::core::ffi::c_uint
            == WINDOW_COPY_CMD_CLEAR_EMACS_ONLY as ::core::ffi::c_int as ::core::ffi::c_uint
            && keys == MODEKEY_VI
        {
            clear = WINDOW_COPY_CMD_CLEAR_NEVER;
        }
        if clear as ::core::ffi::c_uint
            != WINDOW_COPY_CMD_CLEAR_NEVER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_copy_clear_marks(wme);
            (*data).searchy = -(1 as ::core::ffi::c_int);
            (*data).searchx = (*data).searchy;
        }
        if action as ::core::ffi::c_uint
            == WINDOW_COPY_CMD_MOVE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            action = WINDOW_COPY_CMD_REDRAW;
        }
    }
    (*wme).prefix = 1 as u_int;
    if action as ::core::ffi::c_uint
        == WINDOW_COPY_CMD_CANCEL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_pane_reset_mode(&mode_pane_owner);
    } else if action as ::core::ffi::c_uint
        == WINDOW_COPY_CMD_REDRAW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_copy_redraw_screen(wme);
    } else if action as ::core::ffi::c_uint
        == WINDOW_COPY_CMD_MOVE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_copy_redraw_lines(wme, 0 as u_int, 1 as u_int);
    }
}
unsafe fn window_copy_scroll_to(
    mut wme: *mut window_mode_entry,
    mut px: u_int,
    mut py: u_int,
    mut no_redraw: ::core::ffi::c_int,
) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*data).backing_mut().grid_mut();
    let mut offset: u_int = 0;
    let mut gap: u_int = 0;
    let mut old_oy: u_int = (*data).oy;
    (*data).cx = px;
    if py >= (*gd).hsize.wrapping_sub((*data).oy)
        && py < (*gd).hsize.wrapping_sub((*data).oy).wrapping_add((*gd).sy)
    {
        (*data).cy = py.wrapping_sub((*gd).hsize.wrapping_sub((*data).oy));
    } else {
        gap = (*gd).sy.wrapping_div(4 as u_int);
        if py < (*gd).sy {
            offset = 0 as u_int;
            (*data).cy = py;
        } else if py > (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(gap) {
            offset = (*gd).hsize;
            (*data).cy = py.wrapping_sub((*gd).hsize);
        } else {
            offset = py.wrapping_add(gap).wrapping_sub((*gd).sy);
            (*data).cy = py.wrapping_sub(offset);
        }
        (*data).oy = (*gd).hsize.wrapping_sub(offset);
    }
    if no_redraw == 0 && !(*data).searchmark.is_empty() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    if (*data).oy != old_oy {
        window_pane_scrollbar_show(&mode_pane_owner);
    }
    if no_redraw == 0 {
        window_copy_redraw_screen(wme);
    }
}
unsafe fn window_copy_search_compare(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut sgd: *mut grid,
    mut spx: u_int,
    mut cis: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
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
    let mut ud: *const utf8_data = ::core::ptr::null::<utf8_data>();
    let mut sud: *const utf8_data = ::core::ptr::null::<utf8_data>();
    grid_get_cell(&*gd, px, py, &mut gc);
    ud = &raw mut gc.data;
    grid_get_cell(&*sgd, spx, 0 as u_int, &mut sgc);
    sud = &raw mut sgc.data;
    if *(&raw const (*sud).data as *const u_char) as ::core::ffi::c_int == '\t' as i32
        && (*sud).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*ud).size as ::core::ffi::c_int != (*sud).size as ::core::ffi::c_int
        || (*ud).width as ::core::ffi::c_int != (*sud).width as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if cis != 0 && (*ud).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        return (({
            let mut __res: ::core::ffi::c_int = 0;
            if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                if 0 != 0 {
                    let mut __c: ::core::ffi::c_int =
                        (*ud).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int;
                    __res =
                        (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                            __c as __int32_t
                        } else {
                            *(*__ctype_tolower_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                } else {
                    __res =
                        tolower((*ud).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int);
                }
            } else {
                __res = *(*__ctype_tolower_loc()).offset(
                    (*ud).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int as isize,
                ) as ::core::ffi::c_int;
            }
            __res
        }) == (*sud).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            as ::core::ffi::c_int;
    }
    return (memcmp(
        &raw const (*ud).data as *const u_char as *const ::core::ffi::c_void,
        &raw const (*sud).data as *const u_char as *const ::core::ffi::c_void,
        (*ud).size as size_t,
    ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe fn window_copy_search_lr(
    mut gd: *mut grid,
    mut sgd: *mut grid,
    mut ppx: *mut u_int,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    mut cis: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ax: u_int = 0;
    let mut bx: u_int = 0;
    let mut px: u_int = 0;
    let mut pywrap: u_int = 0;
    let mut endline: u_int = 0;
    let mut padding: u_int = 0;
    let mut matched: ::core::ffi::c_int = 0;
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
    endline = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    ax = first;
    while ax < last {
        padding = 0 as u_int;
        bx = 0 as u_int;
        while bx < (*sgd).sx {
            px = ax.wrapping_add(bx).wrapping_add(padding);
            pywrap = py;
            while px >= (*gd).sx && pywrap < endline {
                let gl = grid_get_line(&*gd, pywrap);
                if !(gl.flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
                    break;
                }
                px = px.wrapping_sub((*gd).sx);
                pywrap = pywrap.wrapping_add(1);
            }
            if px.wrapping_sub(padding) >= (*gd).sx {
                break;
            }
            grid_get_cell(&*gd, px, pywrap, &mut gc);
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                padding = padding.wrapping_add(
                    (gc.data.width as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u_int,
                );
            }
            matched = window_copy_search_compare(gd, px, pywrap, sgd, bx, cis);
            if matched == 0 {
                break;
            }
            bx = bx.wrapping_add(1);
        }
        if bx == (*sgd).sx {
            *ppx = ax;
            return 1 as ::core::ffi::c_int;
        }
        ax = ax.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_copy_search_rl(
    mut gd: *mut grid,
    mut sgd: *mut grid,
    mut ppx: *mut u_int,
    mut py: u_int,
    mut last: u_int,
    mut cis: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut first: u_int = 0 as u_int;
    let mut ax: u_int = 0;
    let mut bx: u_int = 0;
    let mut px: u_int = 0;
    let mut pywrap: u_int = 0;
    let mut endline: u_int = 0;
    let mut padding: u_int = 0;
    let mut matched: ::core::ffi::c_int = 0;
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
    endline = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    ax = last;
    while ax > first {
        padding = 0 as u_int;
        bx = 0 as u_int;
        while bx < (*sgd).sx {
            px = ax
                .wrapping_sub(1 as u_int)
                .wrapping_add(bx)
                .wrapping_add(padding);
            pywrap = py;
            while px >= (*gd).sx && pywrap < endline {
                let gl = grid_get_line(&*gd, pywrap);
                if !(gl.flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
                    break;
                }
                px = px.wrapping_sub((*gd).sx);
                pywrap = pywrap.wrapping_add(1);
            }
            if px.wrapping_sub(padding) >= (*gd).sx {
                break;
            }
            grid_get_cell(&*gd, px, pywrap, &mut gc);
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                padding = padding.wrapping_add(
                    (gc.data.width as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u_int,
                );
            }
            matched = window_copy_search_compare(gd, px, pywrap, sgd, bx, cis);
            if matched == 0 {
                break;
            }
            bx = bx.wrapping_add(1);
        }
        if bx == (*sgd).sx {
            *ppx = ax.wrapping_sub(1 as u_int);
            return 1 as ::core::ffi::c_int;
        }
        ax = ax.wrapping_sub(1);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_copy_search_lr_regex(
    gd: &grid,
    ppx: &mut u_int,
    psx: &mut u_int,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    reg: &CompiledRegex,
) -> ::core::ffi::c_int {
    let mut eflags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut endline: u_int = 0;
    let mut foundx: u_int = 0;
    let mut foundy: u_int = 0;
    let mut len: u_int = 0;
    let mut pywrap: u_int = 0;
    let mut regmatch: RegexMatch = RegexMatch { rm_so: 0, rm_eo: 0 };
    if first >= last {
        return 0 as ::core::ffi::c_int;
    }
    if first != 0 as u_int {
        eflags |= REG_NOTBOL;
    }
    let mut buf = vec![0u8];
    window_copy_stringify(gd, py, first, gd.sx, &mut buf);
    len = gd.sx.wrapping_sub(first);
    endline = gd.hsize.wrapping_add(gd.sy).wrapping_sub(1 as u_int);
    pywrap = py;
    while pywrap < endline && len < WINDOW_COPY_SEARCH_MAX_LINE as u_int {
        let gl = &gd.linedata[pywrap as usize];
        if !(gl.flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
            break;
        }
        pywrap = pywrap.wrapping_add(1);
        window_copy_stringify(gd, pywrap, 0 as u_int, gd.sx, &mut buf);
        len = len.wrapping_add(gd.sx);
    }
    if reg.execute_at(
        CStr::from_bytes_until_nul(&buf).expect("search line is terminated"),
        0,
        std::slice::from_mut(&mut regmatch),
        eflags,
    ) && regmatch.rm_so != regmatch.rm_eo
    {
        foundx = first;
        foundy = py;
        window_copy_cstrtocellpos(
            gd,
            len,
            &mut foundx,
            &mut foundy,
            &buf[regmatch.rm_so as usize..],
        );
        if foundy == py && foundx < last {
            *ppx = foundx;
            len = len.wrapping_sub(foundx.wrapping_sub(first));
            window_copy_cstrtocellpos(
                gd,
                len,
                &mut foundx,
                &mut foundy,
                &buf[regmatch.rm_eo as usize..],
            );
            *psx = foundx;
            while foundy > py {
                *psx = (*psx).wrapping_add(gd.sx);
                foundy = foundy.wrapping_sub(1);
            }
            *psx = (*psx).wrapping_sub(*ppx);
            return 1 as ::core::ffi::c_int;
        }
    }
    *ppx = 0 as u_int;
    *psx = 0 as u_int;
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_copy_search_rl_regex(
    gd: &grid,
    ppx: &mut u_int,
    psx: &mut u_int,
    mut py: u_int,
    mut last: u_int,
    reg: &CompiledRegex,
) -> ::core::ffi::c_int {
    let mut first: u_int = 0 as u_int;
    let mut eflags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut endline: u_int = 0;
    let mut len: u_int = 0;
    let mut pywrap: u_int = 0;
    let mut buf = vec![0u8];

    window_copy_stringify(gd, py, first, gd.sx, &mut buf);
    len = gd.sx.wrapping_sub(first);
    endline = gd.hsize.wrapping_add(gd.sy).wrapping_sub(1 as u_int);
    pywrap = py;
    while pywrap < endline && len < WINDOW_COPY_SEARCH_MAX_LINE as u_int {
        let gl = &gd.linedata[pywrap as usize];
        if !(gl.flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
            break;
        }
        pywrap = pywrap.wrapping_add(1);
        window_copy_stringify(gd, pywrap, 0 as u_int, gd.sx, &mut buf);
        len = len.wrapping_add(gd.sx);
    }
    if window_copy_last_regex(
        gd,
        py,
        first,
        last,
        len,
        ppx,
        psx,
        CStr::from_bytes_until_nul(&buf).expect("search line is terminated"),
        reg,
        eflags,
    ) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    *ppx = 0 as u_int;
    *psx = 0 as u_int;
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_copy_cellstring(gl: &grid_line, px: u_int) -> Cow<'_, [u8]> {
    let mut ud = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    if px >= gl.cellsize as u_int {
        return Cow::Borrowed(b" ");
    }
    let gce = &gl.celldata[px as usize];
    if gce.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        return Cow::Borrowed(b"");
    }
    if !(gce.flags as ::core::ffi::c_int) & GRID_FLAG_EXTENDED != 0 {
        return Cow::Borrowed(std::slice::from_ref(&gce.c2rust_unnamed.data.data));
    }
    if gce.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        return Cow::Borrowed(b"\t");
    }
    utf8_to_data(
        gl.extddata[gce.c2rust_unnamed.offset as usize].data,
        &mut ud,
    );
    if ud.size == 0 {
        return Cow::Borrowed(b"");
    }
    Cow::Owned(ud.data[..ud.size as usize].to_vec())
}
unsafe fn window_copy_last_regex(
    gd: &grid,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    mut len: u_int,
    ppx: &mut u_int,
    psx: &mut u_int,
    buf: &CStr,
    preg: &CompiledRegex,
    mut eflags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut foundx: u_int = 0;
    let mut foundy: u_int = 0;
    let mut oldx: u_int = 0;
    let mut px: u_int = 0 as u_int;
    let mut savepx: u_int = 0;
    let mut savesx: u_int = 0 as u_int;
    let mut regmatch: RegexMatch = RegexMatch { rm_so: 0, rm_eo: 0 };
    foundx = first;
    foundy = py;
    oldx = first;
    while preg.execute_at(
        buf,
        px as usize,
        std::slice::from_mut(&mut regmatch),
        eflags,
    ) {
        if regmatch.rm_so == regmatch.rm_eo {
            break;
        }
        window_copy_cstrtocellpos(
            gd,
            len,
            &mut foundx,
            &mut foundy,
            &buf.to_bytes()[px as usize + regmatch.rm_so as usize..],
        );
        if foundy > py || foundx >= last {
            break;
        }
        len = len.wrapping_sub(foundx.wrapping_sub(oldx));
        savepx = foundx;
        window_copy_cstrtocellpos(
            gd,
            len,
            &mut foundx,
            &mut foundy,
            &buf.to_bytes()[px as usize + regmatch.rm_eo as usize..],
        );
        if foundy > py || foundx >= last {
            *ppx = savepx;
            *psx = foundx;
            while foundy > py {
                *psx = (*psx).wrapping_add(gd.sx);
                foundy = foundy.wrapping_sub(1);
            }
            *psx = (*psx).wrapping_sub(*ppx);
            return 1 as ::core::ffi::c_int;
        } else {
            savesx = foundx.wrapping_sub(savepx);
            len = len.wrapping_sub(savesx);
            oldx = foundx;
        }
        px = px.wrapping_add(regmatch.rm_eo as u_int);
    }
    if savesx > 0 as u_int {
        *ppx = savepx;
        *psx = savesx;
        return 1 as ::core::ffi::c_int;
    } else {
        *ppx = 0 as u_int;
        *psx = 0 as u_int;
        return 0 as ::core::ffi::c_int;
    };
}
unsafe fn window_copy_search_line(gd: &grid, py: u_int) -> Option<&grid_line> {
    if py >= gd.hsize.wrapping_add(gd.sy) {
        log_debug(format_args!("grid_peek_line: y out of range: {}", py));
        return None;
    }
    gd.linedata.get(py as usize)
}

unsafe fn window_copy_stringify(
    gd: &grid,
    py: u_int,
    first: u_int,
    last: u_int,
    buf: &mut Vec<u8>,
) {
    let Some(gl) = window_copy_search_line(gd, py) else {
        return;
    };
    buf.pop(); // Remove the previous terminator before appending another line.
    for ax in first..last {
        // Keep interior NUL bytes in the buffer; only the final byte terminates it.
        buf.extend_from_slice(&window_copy_cellstring(gl, ax));
    }
    buf.push(0);
}
unsafe fn window_copy_cstrtocellpos(
    gd: &grid,
    ncells: u_int,
    ppx: &mut u_int,
    ppy: &mut u_int,
    text: &[u8],
) {
    let mut cells: Vec<Cow<'_, [u8]>> = Vec::with_capacity(ncells as usize);
    let mut px = *ppx;
    let mut py = *ppy;
    let Some(mut line) = window_copy_search_line(gd, py) else {
        return;
    };
    for _ in 0..ncells {
        cells.push(window_copy_cellstring(line, px));
        px = px.wrapping_add(1);
        if px == gd.sx {
            px = 0;
            py = py.wrapping_add(1);
            let Some(next) = window_copy_search_line(gd, py) else {
                break;
            };
            line = next;
        }
    }

    let text = &text[..text
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(text.len())];
    let cell = window_copy_find_cell_suffix(&cells, text);
    px = ppx.wrapping_add(cell as u_int);
    py = *ppy;
    while px >= gd.sx {
        px = px.wrapping_sub(gd.sx);
        py = py.wrapping_add(1);
    }
    *ppx = px;
    *ppy = py;
}

fn window_copy_find_cell_suffix(cells: &[Cow<'_, [u8]>], text: &[u8]) -> usize {
    for first in 0..cells.len() {
        let mut remaining = text;
        let matches = cells[first..].iter().all(|cell| {
            // tmux rejects an exhausted string even if the next cell is padding.
            if remaining.is_empty() {
                return false;
            }
            let len = cell.len().min(remaining.len());
            if remaining[..len] != cell[..len] {
                return false;
            }
            remaining = &remaining[len..];
            true
        });
        if matches {
            return first;
        }
    }
    // A missing suffix maps to one cell past the scanned range.
    cells.len()
}
unsafe fn window_copy_move_left(
    mut s: *mut screen,
    mut fx: *mut u_int,
    mut fy: *mut u_int,
    mut wrapflag: ::core::ffi::c_int,
) {
    if *fx == 0 as u_int {
        if *fy == 0 as u_int {
            if wrapflag != 0 {
                *fx = (*s).grid().sx.wrapping_sub(1 as u_int);
                *fy = (*s).grid()
                    .hsize
                    .wrapping_add((*s).grid().sy)
                    .wrapping_sub(1 as u_int);
            }
            return;
        }
        *fx = (*s).grid().sx.wrapping_sub(1 as u_int);
        *fy = (*fy).wrapping_sub(1 as u_int);
    } else {
        *fx = (*fx).wrapping_sub(1 as u_int);
    };
}
unsafe fn window_copy_move_right(
    mut s: *mut screen,
    mut fx: *mut u_int,
    mut fy: *mut u_int,
    mut wrapflag: ::core::ffi::c_int,
) {
    if *fx == (*s).grid().sx.wrapping_sub(1 as u_int) {
        if *fy
            == (*s).grid()
                .hsize
                .wrapping_add((*s).grid().sy)
                .wrapping_sub(1 as u_int)
        {
            if wrapflag != 0 {
                *fx = 0 as u_int;
                *fy = 0 as u_int;
            }
            return;
        }
        *fx = 0 as u_int;
        *fy = (*fy).wrapping_add(1 as u_int);
    } else {
        *fx = (*fx).wrapping_add(1 as u_int);
    };
}
unsafe fn window_copy_is_lowercase(mut ptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        if *ptr as ::core::ffi::c_int
            != ({
                let mut __res: ::core::ffi::c_int = 0;
                if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                    if 0 != 0 {
                        let mut __c: ::core::ffi::c_int = *ptr as u_char as ::core::ffi::c_int;
                        __res = (if __c < -(128 as ::core::ffi::c_int)
                            || __c > 255 as ::core::ffi::c_int
                        {
                            __c as __int32_t
                        } else {
                            *(*__ctype_tolower_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                    } else {
                        __res = tolower(*ptr as u_char as ::core::ffi::c_int);
                    }
                } else {
                    __res = *(*__ctype_tolower_loc())
                        .offset(*ptr as u_char as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int;
                }
                __res
            })
        {
            return 0 as ::core::ffi::c_int;
        }
        ptr = ptr.offset(1);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_copy_search_back_overlap(
    gd: &grid,
    preg: &CompiledRegex,
    ppx: &mut u_int,
    psx: &mut u_int,
    ppy: &mut u_int,
    mut endline: u_int,
) {
    let mut endx: u_int = 0;
    let mut endy: u_int = 0;
    let mut oldendx: u_int = 0;
    let mut oldendy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut sx: u_int = 0;
    let mut found: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    oldendx = (*ppx).wrapping_add(*psx);
    oldendy = (*ppy).wrapping_sub(1 as u_int);
    while oldendx > gd.sx.wrapping_sub(1 as u_int) {
        oldendx = oldendx.wrapping_sub(gd.sx);
        oldendy = oldendy.wrapping_add(1);
    }
    endx = oldendx;
    endy = oldendy;
    px = *ppx;
    py = *ppy;
    while found != 0
        && px == 0 as u_int
        && py.wrapping_sub(1 as u_int) > endline
        && gd.linedata[py.wrapping_sub(2) as usize].flags as ::core::ffi::c_int & GRID_LINE_WRAPPED
            != 0
        && endx == oldendx
        && endy == oldendy
    {
        py = py.wrapping_sub(1);
        found = window_copy_search_rl_regex(
            gd,
            &mut px,
            &mut sx,
            py.wrapping_sub(1 as u_int),
            gd.sx,
            preg,
        );
        if found != 0 {
            endx = px.wrapping_add(sx);
            endy = py.wrapping_sub(1 as u_int);
            while endx > gd.sx.wrapping_sub(1 as u_int) {
                endx = endx.wrapping_sub(gd.sx);
                endy = endy.wrapping_add(1);
            }
            if endx == oldendx && endy == oldendy {
                *ppx = px;
                *ppy = py;
            }
        }
    }
}
unsafe fn window_copy_search_jump(
    mut wme: *mut window_mode_entry,
    mut gd: *mut grid,
    mut sgd: *mut grid,
    mut fx: u_int,
    mut fy: u_int,
    mut endline: u_int,
    mut cis: ::core::ffi::c_int,
    mut wrap: ::core::ffi::c_int,
    mut direction: ::core::ffi::c_int,
    mut regex: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    let mut px: u_int = 0;
    let mut sx: u_int = 0;
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cflags: ::core::ffi::c_int = REG_EXTENDED;
    let mut regex_storage = RegexStorage::default();
    let regex_owner = if regex != 0 {
        let mut sbuf = vec![0u8];
        window_copy_stringify(&*sgd, 0 as u_int, 0 as u_int, (*sgd).sx, &mut sbuf);
        if cis != 0 {
            cflags |= REG_ICASE;
        }
        let Ok(compiled) = regex_storage.compile(
            CStr::from_bytes_until_nul(&sbuf).expect("search pattern is terminated"),
            cflags,
        ) else {
            return 0 as ::core::ffi::c_int;
        };
        Some(compiled)
    } else {
        None
    };
    if direction != 0 {
        i = fy;
        while i <= endline {
            if regex != 0 {
                found = window_copy_search_lr_regex(
                    &*gd,
                    &mut px,
                    &mut sx,
                    i,
                    fx,
                    (*gd).sx,
                    regex_owner
                        .as_ref()
                        .expect("regex search has a compiled pattern"),
                );
            } else {
                found = window_copy_search_lr(gd, sgd, &raw mut px, i, fx, (*gd).sx, cis);
            }
            if found != 0 {
                break;
            }
            fx = 0 as u_int;
            i = i.wrapping_add(1);
        }
    } else {
        i = fy.wrapping_add(1 as u_int);
        while endline < i {
            if regex != 0 {
                found = window_copy_search_rl_regex(
                    &*gd,
                    &mut px,
                    &mut sx,
                    i.wrapping_sub(1 as u_int),
                    fx.wrapping_add(1 as u_int),
                    regex_owner
                        .as_ref()
                        .expect("regex search has a compiled pattern"),
                );
                if found != 0 {
                    window_copy_search_back_overlap(
                        &*gd,
                        regex_owner
                            .as_ref()
                            .expect("regex search has a compiled pattern"),
                        &mut px,
                        &mut sx,
                        &mut i,
                        endline,
                    );
                }
            } else {
                found = window_copy_search_rl(
                    gd,
                    sgd,
                    &raw mut px,
                    i.wrapping_sub(1 as u_int),
                    fx.wrapping_add(1 as u_int),
                    cis,
                );
            }
            if found != 0 {
                i = i.wrapping_sub(1);
                break;
            } else {
                fx = (*gd).sx.wrapping_sub(1 as u_int);
                i = i.wrapping_sub(1);
            }
        }
    }
    drop(regex_owner);
    if found != 0 {
        window_copy_scroll_to(wme, px, i, 1 as ::core::ffi::c_int);
        return 1 as ::core::ffi::c_int;
    }
    if wrap != 0 {
        return window_copy_search_jump(
            wme,
            gd,
            sgd,
            if direction != 0 {
                0 as u_int
            } else {
                (*gd).sx.wrapping_sub(1 as u_int)
            },
            if direction != 0 {
                0 as u_int
            } else {
                (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int)
            },
            fy,
            cis,
            0 as ::core::ffi::c_int,
            direction,
            regex,
        );
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_copy_move_after_search_mark(
    mut data: *mut window_copy_mode_data,
    mut fx: *mut u_int,
    mut fy: *mut u_int,
    mut wrapflag: ::core::ffi::c_int,
) {
    let mut s: *mut screen = (*data).backing_mut();
    let mut at: u_int = 0;
    let mut start: u_int = 0;
    if window_copy_search_mark_at(data, *fx, *fy, &raw mut start) == 0 as ::core::ffi::c_int
        && (&(*data).searchmark)[start as usize] as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        while window_copy_search_mark_at(data, *fx, *fy, &raw mut at) == 0 as ::core::ffi::c_int {
            if (&(*data).searchmark)[at as usize] as ::core::ffi::c_int
                != (&(*data).searchmark)[start as usize] as ::core::ffi::c_int
            {
                break;
            }
            if wrapflag == 0
                && *fx == (*s).grid().sx.wrapping_sub(1 as u_int)
                && *fy
                    == (*s).grid()
                        .hsize
                        .wrapping_add((*s).grid().sy)
                        .wrapping_sub(1 as u_int)
            {
                break;
            }
            window_copy_move_right(s, fx, fy, wrapflag);
        }
    }
}
unsafe fn window_copy_search(
    mut wme: *mut window_mode_entry,
    mut direction: ::core::ffi::c_int,
    mut regex: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = (*data).backing_mut();
    let mut ss: screen = screen::empty();
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut gd: *mut grid = (*s).grid_mut();
    let mut str: *const ::core::ffi::c_char = window_copy_searchstr(&*data);
    let mut at: u_int = 0;
    let mut endline: u_int = 0;
    let mut fx: u_int = 0;
    let mut fy: u_int = 0;
    let mut start: u_int = 0;
    let mut ssx: u_int = 0;
    let mut cis: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut keys: ::core::ffi::c_int = 0;
    let mut visible_only: ::core::ffi::c_int = 0;
    let mut wrapflag: ::core::ffi::c_int = 0;
    if regex != 0
        && *str.offset(strcspn(
            str,
            b"^$*+()?[].\\\0" as *const u8 as *const ::core::ffi::c_char,
        ) as isize) as ::core::ffi::c_int
            == '\0' as i32
    {
        regex = 0 as ::core::ffi::c_int;
    }
    (*data).searchdirection = direction;
    if (*data).timeout != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*data).searchall != 0 || (*wp).searchstr.is_none() || (*wp).searchregex != regex {
        visible_only = 0 as ::core::ffi::c_int;
        (*data).searchall = 0 as ::core::ffi::c_int;
    } else {
        visible_only = (strcmp(
            (*wp)
                .searchstr
                .as_ref()
                .map_or(::core::ptr::null(), |value| value.as_ptr()),
            str,
        ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    if visible_only == 0 as ::core::ffi::c_int && !(*data).searchmark.is_empty() {
        window_copy_clear_marks(wme);
    }
    window_pane_set_searchstr(&mut *wp, Some(CStr::from_ptr(str).to_owned()));
    (*wp).searchregex = regex;
    fx = (*data).cx;
    fy = (*data).backing().grid()
        .hsize
        .wrapping_sub((*data).oy)
        .wrapping_add((*data).cy);
    ssx = screen_write_strlen(CStr::from_ptr(str)) as u_int;
    if ssx == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    screen_init(&mut ss, ssx, 1 as u_int, 0 as u_int);
    screen_write_start(&mut ctx, &raw mut ss);
    screen_write_nputs(
        &mut ctx,
        -(1 as ::core::ffi::c_int) as ssize_t,
        &grid_default_cell,
        |out| write_cstr(out, str),
    );
    screen_write_stop(&mut ctx);
    wrapflag = options_get_number(
        options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options),
        b"wrap-search\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    cis = window_copy_is_lowercase(str);
    keys = options_get_number(
        options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options),
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if direction != 0 {
        if keys == MODEKEY_VI {
            if !(*data).searchmark.is_empty() {
                window_copy_move_after_search_mark(data, &raw mut fx, &raw mut fy, wrapflag);
            } else {
                window_copy_move_right(s, &raw mut fx, &raw mut fy, wrapflag);
            }
        }
        endline = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    } else {
        window_copy_move_left(s, &raw mut fx, &raw mut fy, wrapflag);
        endline = 0 as u_int;
    }
    found = window_copy_search_jump(
        wme, gd, ss.grid_mut(), fx, fy, endline, cis, wrapflag, direction, regex,
    );
    if found != 0 {
        window_copy_search_marks(wme, &raw mut ss, regex, visible_only);
        fx = (*data).cx;
        fy = (*data).backing().grid()
            .hsize
            .wrapping_sub((*data).oy)
            .wrapping_add((*data).cy);
        if direction != 0
            && window_copy_search_mark_at(data, fx, fy, &raw mut at) == 0 as ::core::ffi::c_int
            && at > 0 as u_int
            && !(*data).searchmark.is_empty()
            && (&(*data).searchmark)[at as usize] as ::core::ffi::c_int
                == (&(*data).searchmark)[at.wrapping_sub(1 as u_int) as usize] as ::core::ffi::c_int
        {
            window_copy_move_after_search_mark(data, &raw mut fx, &raw mut fy, wrapflag);
            window_copy_search_jump(
                wme, gd, ss.grid_mut(), fx, fy, endline, cis, wrapflag, direction, regex,
            );
            fx = (*data).cx;
            fy = (*data).backing().grid()
                .hsize
                .wrapping_sub((*data).oy)
                .wrapping_add((*data).cy);
        }
        if direction != 0 {
            if keys == MODEKEY_EMACS {
                window_copy_move_after_search_mark(data, &raw mut fx, &raw mut fy, wrapflag);
                (*data).cx = fx;
                (*data).cy = fy
                    .wrapping_sub((*data).backing().grid().hsize)
                    .wrapping_add((*data).oy);
            }
        } else if window_copy_search_mark_at(data, fx, fy, &raw mut start)
            == 0 as ::core::ffi::c_int
        {
            while window_copy_search_mark_at(data, fx, fy, &raw mut at) == 0 as ::core::ffi::c_int
                && !(*data).searchmark.is_empty()
                && (&(*data).searchmark)[at as usize] as ::core::ffi::c_int
                    == (&(*data).searchmark)[start as usize] as ::core::ffi::c_int
            {
                (*data).cx = fx;
                (*data).cy = fy
                    .wrapping_sub((*data).backing().grid().hsize)
                    .wrapping_add((*data).oy);
                if at == 0 as u_int {
                    break;
                }
                window_copy_move_left(s, &raw mut fx, &raw mut fy, 0 as ::core::ffi::c_int);
            }
        }
    }
    window_copy_redraw_screen(wme);
    screen_free(&mut ss);
    return found;
}
unsafe fn window_copy_visible_lines(
    mut data: *mut window_copy_mode_data,
    mut start: *mut u_int,
    mut end: *mut u_int,
) {
    let mut gd: *mut grid = (*data).backing_mut().grid_mut();
    *start = (*gd).hsize.wrapping_sub((*data).oy);
    while *start > 0 as u_int {
        let Some(gl) = grid_peek_line(&*gd, (*start).wrapping_sub(1 as u_int)) else {
            break;
        };
        if !(gl.flags as i32) & GRID_LINE_WRAPPED != 0 {
            break;
        }
        *start = (*start).wrapping_sub(1);
    }
    *end = (*gd).hsize.wrapping_sub((*data).oy).wrapping_add((*gd).sy);
}
unsafe fn window_copy_search_mark_at(
    mut data: *mut window_copy_mode_data,
    mut px: u_int,
    mut py: u_int,
    mut at: *mut u_int,
) -> ::core::ffi::c_int {
    let mut s: *mut screen = (*data).backing_mut();
    let mut gd: *mut grid = (*s).grid_mut();
    if py < (*gd).hsize.wrapping_sub((*data).oy) {
        return -(1 as ::core::ffi::c_int);
    }
    if py
        > (*gd)
            .hsize
            .wrapping_sub((*data).oy)
            .wrapping_add((*gd).sy)
            .wrapping_sub(1 as u_int)
    {
        return -(1 as ::core::ffi::c_int);
    }
    *at = py
        .wrapping_sub((*gd).hsize.wrapping_sub((*data).oy))
        .wrapping_mul((*gd).sx)
        .wrapping_add(px);
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_copy_clip_width(
    mut width: u_int,
    mut b: u_int,
    mut sx: u_int,
    mut sy: u_int,
) -> u_int {
    return if b.wrapping_add(width) > sx.wrapping_mul(sy) {
        sx.wrapping_mul(sy).wrapping_sub(b)
    } else {
        width
    };
}
unsafe fn window_copy_search_mark_match(
    mut data: *mut window_copy_mode_data,
    mut px: u_int,
    mut py: u_int,
    mut width: u_int,
    mut regex: ::core::ffi::c_int,
) -> u_int {
    let mut gd: *mut grid = (*data).backing_mut().grid_mut();
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
    let mut i: u_int = 0;
    let mut b: u_int = 0;
    let mut w: u_int = width;
    let mut sx: u_int = (*gd).sx;
    let mut sy: u_int = (*gd).sy;
    if window_copy_search_mark_at(data, px, py, &raw mut b) == 0 as ::core::ffi::c_int {
        width = window_copy_clip_width(width, b, sx, sy);
        w = width;
        i = b;
        while i < b.wrapping_add(w) {
            if regex == 0 {
                grid_get_cell(&*gd, px.wrapping_add(i.wrapping_sub(b)), py, &mut gc);
                if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                    w = w.wrapping_add(
                        (gc.data.width as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u_int,
                    );
                }
                w = window_copy_clip_width(w, b, sx, sy);
            }
            if !((&(*data).searchmark)[i as usize] as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
            {
                (&mut (*data).searchmark)[i as usize] = (*data).searchgen;
            }
            i = i.wrapping_add(1);
        }
        if (*data).searchgen as ::core::ffi::c_int == UCHAR_MAX {
            (*data).searchgen = 1 as u_char;
        } else {
            (*data).searchgen = (*data).searchgen.wrapping_add(1);
        }
    }
    return w;
}
fn window_copy_clear_searchmark(data: &mut window_copy_mode_data) {
    data.searchmark.clear();
}

unsafe fn window_copy_replace_searchmark(data: &mut window_copy_mode_data, sx: u_int, sy: u_int) {
    window_copy_clear_searchmark(data);
    if sx == 0 || sy == 0 {
        fatalx(|out| out.write_all(b"xcalloc: zero size"));
    }
    let Some(len) = (sx as usize).checked_mul(sy as usize) else {
        fatalx(|out| out.write_all(b"xcalloc: nmemb * size > SIZE_MAX"));
    };
    let marks = &mut data.searchmark;
    if marks.try_reserve_exact(len).is_err() {
        fatal(|out| {
            write!(
                out,
                "xcalloc: allocating {} bytes",
                (len as size_t) as usize
            )
        });
    }
    marks.resize(len, 0);
}

unsafe fn window_copy_search_marks(
    mut wme: *mut window_mode_entry,
    mut ssp: *mut screen,
    mut regex: ::core::ffi::c_int,
    mut visible_only: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = (*data).backing_mut();
    let mut ss: screen = screen::empty();
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut gd: *mut grid = (*s).grid_mut();
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
    let mut found: ::core::ffi::c_int = 0;
    let mut cis: ::core::ffi::c_int = 0;
    let mut stopped: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cflags: ::core::ffi::c_int = REG_EXTENDED;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut nfound: u_int = 0 as u_int;
    let mut width: u_int = 0;
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    let mut sx: u_int = (*gd).sx;
    let mut sy: u_int = (*gd).sy;
    let mut regex_storage = RegexStorage::default();
    let mut stop: uint64_t = 0 as uint64_t;
    let mut tstart: uint64_t = 0;
    let mut t: uint64_t = 0;
    if ssp.is_null() {
        // Preserve the previous libc %s rendering for an absent search string.
        width = screen_write_strlen((*data).searchstr.as_deref().unwrap_or(c"(null)")) as u_int;
        screen_init(&mut ss, width, 1 as u_int, 0 as u_int);
        screen_write_start(&mut ctx, &raw mut ss);
        screen_write_nputs(
            &mut ctx,
            -(1 as ::core::ffi::c_int) as ssize_t,
            &grid_default_cell,
            |out| write_cstr(out, window_copy_searchstr(&*data)),
        );
        screen_write_stop(&mut ctx);
        ssp = &raw mut ss;
    } else {
        width = (*ssp).grid().sx;
    }
    cis = window_copy_is_lowercase(window_copy_searchstr(&*data));
    let regex_owner = if regex != 0 {
        let mut sbuf = vec![0u8];
        window_copy_stringify(
            (*ssp).grid(),
            0 as u_int,
            0 as u_int,
            (*ssp).grid().sx,
            &mut sbuf,
        );
        if cis != 0 {
            cflags |= REG_ICASE;
        }
        let Ok(compiled) = regex_storage.compile(
            CStr::from_bytes_until_nul(&sbuf).expect("search pattern is terminated"),
            cflags,
        ) else {
            window_copy_clear_searchmark(&mut *data);
            return 0 as ::core::ffi::c_int;
        };
        Some(compiled)
    } else {
        None
    };
    tstart = get_timer();
    if visible_only != 0 {
        window_copy_visible_lines(data, &raw mut start, &raw mut end);
    } else {
        start = 0 as u_int;
        end = (*gd).hsize.wrapping_add(sy);
        stop = get_timer().wrapping_add(WINDOW_COPY_SEARCH_ALL_TIMEOUT as uint64_t);
    }
    loop {
        window_copy_replace_searchmark(&mut *data, sx, sy);
        (*data).searchgen = 1 as u_char;
        py = start;
        while py < end {
            px = 0 as u_int;
            loop {
                if regex != 0 {
                    let first = px;
                    found = window_copy_search_lr_regex(
                        &*gd,
                        &mut px,
                        &mut width,
                        py,
                        first,
                        sx,
                        regex_owner
                            .as_ref()
                            .expect("regex search has a compiled pattern"),
                    );
                    grid_get_cell(
                        &*gd,
                        px.wrapping_add(width).wrapping_sub(1 as u_int),
                        py,
                        &mut gc,
                    );
                    if gc.data.width as ::core::ffi::c_int > 2 as ::core::ffi::c_int {
                        width = width.wrapping_add(
                            (gc.data.width as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                as u_int,
                        );
                    }
                    if found == 0 {
                        break;
                    }
                } else {
                    found = window_copy_search_lr(gd, (*ssp).grid_mut(), &raw mut px, py, px, sx, cis);
                    if found == 0 {
                        break;
                    }
                }
                nfound = nfound.wrapping_add(1);
                px = px.wrapping_add(window_copy_search_mark_match(data, px, py, width, regex));
            }
            t = get_timer();
            if t.wrapping_sub(tstart) > WINDOW_COPY_SEARCH_TIMEOUT as uint64_t {
                (*data).timeout = 1 as ::core::ffi::c_int;
                break;
            } else if stop != 0 as uint64_t && t > stop {
                stopped = 1 as ::core::ffi::c_int;
                break;
            } else {
                py = py.wrapping_add(1);
            }
        }
        if (*data).timeout != 0 {
            window_copy_clear_marks(wme);
            break;
        } else if stopped != 0 && stop != 0 as uint64_t {
            window_copy_visible_lines(data, &raw mut start, &raw mut end);
            stop = 0 as uint64_t;
        } else {
            if visible_only == 0 {
                if stopped != 0 {
                    if nfound > 1000 as u_int {
                        (*data).searchcount = 1000 as ::core::ffi::c_int;
                    } else if nfound > 100 as u_int {
                        (*data).searchcount = 100 as ::core::ffi::c_int;
                    } else if nfound > 10 as u_int {
                        (*data).searchcount = 10 as ::core::ffi::c_int;
                    } else {
                        (*data).searchcount = -(1 as ::core::ffi::c_int);
                    }
                    (*data).searchmore = 1 as ::core::ffi::c_int;
                } else {
                    (*data).searchcount = nfound as ::core::ffi::c_int;
                    (*data).searchmore = 0 as ::core::ffi::c_int;
                }
            }
            break;
        }
    }
    if ssp == &raw mut ss {
        screen_free(&mut ss);
    }
    drop(regex_owner);
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_copy_clear_marks(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).searchcount = -(1 as ::core::ffi::c_int);
    (*data).searchmore = 0 as ::core::ffi::c_int;
    window_copy_clear_searchmark(&mut *data);
}
unsafe fn window_copy_search_up(
    mut wme: *mut window_mode_entry,
    mut regex: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return window_copy_search(wme, 0 as ::core::ffi::c_int, regex);
}
unsafe fn window_copy_search_down(
    mut wme: *mut window_mode_entry,
    mut regex: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return window_copy_search(wme, 1 as ::core::ffi::c_int, regex);
}
unsafe fn window_copy_goto_line(
    mut wme: *mut window_mode_entry,
    mut linestr: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut hsize: u_int = (*data).backing().grid().hsize;
    let mut line: u_int = 0;
    let mut lineno: ::core::ffi::c_int = 0;
    lineno = strtonum(
        linestr,
        -(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong,
        INT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as ::core::ffi::c_int;
    if !errstr.is_null() {
        return;
    }
    if window_copy_line_number_is_absolute(wme) != 0 {
        if lineno <= 0 as ::core::ffi::c_int {
            line = 1 as u_int;
        } else if lineno as u_int > hsize.wrapping_add(1 as u_int) {
            line = hsize.wrapping_add(1 as u_int);
        } else {
            line = lineno as u_int;
        }
        (*data).oy = hsize.wrapping_sub(line.wrapping_sub(1 as u_int));
    } else {
        if lineno < 0 as ::core::ffi::c_int || lineno as u_int > hsize {
            lineno = hsize as ::core::ffi::c_int;
        }
        (*data).oy = lineno as u_int;
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    window_copy_redraw_screen(wme);
}
unsafe fn window_copy_match_start_end(
    mut data: *mut window_copy_mode_data,
    mut at: u_int,
    mut start: *mut u_int,
    mut end: *mut u_int,
) {
    let mut gd: *mut grid = (*data).backing_mut().grid_mut();
    let mut last: u_int = (*gd).sy.wrapping_mul((*gd).sx).wrapping_sub(1 as u_int);
    let mut mark: u_char = (&(*data).searchmark)[at as usize];
    *end = at;
    *start = *end;
    while *start != 0 as u_int
        && (&(*data).searchmark)[*start as usize] as ::core::ffi::c_int
            == mark as ::core::ffi::c_int
    {
        *start = (*start).wrapping_sub(1);
    }
    if (&(*data).searchmark)[*start as usize] as ::core::ffi::c_int != mark as ::core::ffi::c_int {
        *start = (*start).wrapping_add(1);
    }
    while *end != last
        && (&(*data).searchmark)[*end as usize] as ::core::ffi::c_int == mark as ::core::ffi::c_int
    {
        *end = (*end).wrapping_add(1);
    }
    if (&(*data).searchmark)[*end as usize] as ::core::ffi::c_int != mark as ::core::ffi::c_int {
        *end = (*end).wrapping_sub(1);
    }
}
unsafe fn window_copy_match_at_cursor_bytes(
    mut data: *mut window_copy_mode_data,
) -> Option<Vec<u8>> {
    let mut gd: *mut grid = (*data).backing_mut().grid_mut();
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
    let mut at: u_int = 0;
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    let mut cy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut sx: u_int = (*data).backing().grid().sx;
    let mut output = Vec::<u8>::new();
    if (*data).searchmark.is_empty() {
        return None;
    }
    cy = (*data).backing().grid()
        .hsize
        .wrapping_sub((*data).oy)
        .wrapping_add((*data).cy);
    if window_copy_search_mark_at(data, (*data).cx, cy, &raw mut at) != 0 as ::core::ffi::c_int {
        return None;
    }
    if (&(*data).searchmark)[at as usize] as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        if at == 0 as u_int || {
            at = at.wrapping_sub(1);
            (&(*data).searchmark)[at as usize] as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        } {
            return None;
        }
    }
    window_copy_match_start_end(data, at, &raw mut start, &raw mut end);
    at = start;
    while at <= end {
        py = at.wrapping_div(sx);
        px = at.wrapping_sub(py.wrapping_mul(sx));
        grid_get_cell(
            &*gd,
            px,
            (*gd).hsize.wrapping_add(py).wrapping_sub((*data).oy),
            &mut gc,
        );
        if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
            output.push(b'\t');
        } else if !(gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
            output.extend_from_slice(std::slice::from_raw_parts(
                gc.data.data.as_ptr(),
                gc.data.size as usize,
            ));
        }
        at = at.wrapping_add(1);
    }
    if output.is_empty() {
        return None;
    }
    Some(output)
}
unsafe fn window_copy_match_at_cursor_cstring(data: *mut window_copy_mode_data) -> Option<CString> {
    let mut output = window_copy_match_at_cursor_bytes(data)?;
    // Format strings use the first-NUL view; copy-selection consumers still
    // use the full byte result from window_copy_match_at_cursor_bytes.
    if let Some(end) = output.iter().position(|&byte| byte == 0) {
        output.truncate(end);
    }
    Some(CString::new(output).expect("format match contains no NUL"))
}
unsafe fn window_copy_update_style(
    mut wme: *mut window_mode_entry,
    mut fx: u_int,
    mut fy: u_int,
    mut gc: *mut grid_cell,
    mut mgc: *const grid_cell,
    mut cgc: *const grid_cell,
    mut mkgc: *const grid_cell,
    mut clgc: *const grid_cell,
) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut mark: u_int = 0;
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    let mut cy: u_int = 0;
    let mut cursor: u_int = 0;
    let mut current: u_int = 0;
    let mut inv: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut keys: ::core::ffi::c_int = 0;
    cy = (*data).backing().grid()
        .hsize
        .wrapping_sub((*data).oy)
        .wrapping_add((*data).cy);
    if fy == cy {
        if (*clgc).fg != 8 as ::core::ffi::c_int {
            (*gc).fg = (*clgc).fg;
        }
        if (*clgc).bg != 8 as ::core::ffi::c_int {
            (*gc).bg = (*clgc).bg;
        }
        (*gc).attr =
            ((*gc).attr as ::core::ffi::c_int | (*clgc).attr as ::core::ffi::c_int) as u_short;
    }
    if (*data).showmark != 0 && fy == (*data).my {
        (*gc).attr = (*mkgc).attr;
        if fx == (*data).mx {
            inv = 1 as ::core::ffi::c_int;
        }
        if inv != 0 {
            (*gc).fg = (*mkgc).bg;
            (*gc).bg = (*mkgc).fg;
        } else {
            (*gc).fg = (*mkgc).fg;
            (*gc).bg = (*mkgc).bg;
        }
    }
    if (*data).searchmark.is_empty() {
        return;
    }
    if window_copy_search_mark_at(data, fx, fy, &raw mut current) != 0 as ::core::ffi::c_int {
        return;
    }
    mark = (&(*data).searchmark)[current as usize] as u_int;
    if mark == 0 as u_int {
        return;
    }
    if window_copy_search_mark_at(data, (*data).cx, cy, &raw mut cursor) == 0 as ::core::ffi::c_int
    {
        keys = options_get_number(
            options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options),
            b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int;
        if cursor != 0 as u_int && keys == MODEKEY_EMACS && (*data).searchdirection != 0 {
            if (&(*data).searchmark)[cursor.wrapping_sub(1 as u_int) as usize] as u_int == mark {
                cursor = cursor.wrapping_sub(1);
                found = 1 as ::core::ffi::c_int;
            }
        } else if (&(*data).searchmark)[cursor as usize] as u_int == mark {
            found = 1 as ::core::ffi::c_int;
        }
        if found != 0 {
            window_copy_match_start_end(data, cursor, &raw mut start, &raw mut end);
            if current >= start && current <= end {
                (*gc).attr = (*cgc).attr;
                if inv != 0 {
                    (*gc).fg = (*cgc).bg;
                    (*gc).bg = (*cgc).fg;
                } else {
                    (*gc).fg = (*cgc).fg;
                    (*gc).bg = (*cgc).bg;
                }
                return;
            }
        }
    }
    (*gc).attr = (*mgc).attr;
    if inv != 0 {
        (*gc).fg = (*mgc).bg;
        (*gc).bg = (*mgc).fg;
    } else {
        (*gc).fg = (*mgc).fg;
        (*gc).bg = (*mgc).bg;
    };
}
unsafe fn window_copy_write_one(
    mut wme: *mut window_mode_entry,
    mut ctx: *mut screen_write_ctx,
    mut px: u_int,
    mut py: u_int,
    mut fy: u_int,
    mut nx: u_int,
    mut mgc: *const grid_cell,
    mut cgc: *const grid_cell,
    mut mkgc: *const grid_cell,
    mut clgc: *const grid_cell,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*data).backing_mut().grid_mut();
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
    let mut fx: u_int = 0;
    let mut i: u_int = 0;
    let mut width: u_int = 0;
    screen_write_cursormove(
        &mut *ctx,
        px as ::core::ffi::c_int,
        py as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    fx = 0 as u_int;
    while fx < nx {
        grid_get_cell(&*gd, fx, fy, &mut gc);
        if fx.wrapping_add(gc.data.width as u_int) <= nx {
            window_copy_update_style(wme, fx, fy, &raw mut gc, mgc, cgc, mkgc, clgc);
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
                if (*(*ctx).s).cy == py && (*(*ctx).s).cx <= px.wrapping_add(fx) {
                    gc.flags = (gc.flags as ::core::ffi::c_int & !GRID_FLAG_PADDING) as u_char;
                    screen_write_cursormove(
                        &mut *ctx,
                        px.wrapping_add(fx) as ::core::ffi::c_int,
                        py as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    screen_write_putc(&mut *ctx, &gc, ' ' as i32 as u_char);
                }
            } else if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                width = gc.data.width as u_int;
                gc.flags = (gc.flags as ::core::ffi::c_int & !GRID_FLAG_TAB) as u_char;
                i = 0 as u_int;
                while i < width {
                    screen_write_putc(&mut *ctx, &gc, ' ' as i32 as u_char);
                    i = i.wrapping_add(1);
                }
            } else {
                screen_write_cell(&mut *ctx, &gc);
            }
        } else {
            screen_write_putc(&mut *ctx, &grid_default_cell, ' ' as i32 as u_char);
        }
        fx = fx.wrapping_add(1);
    }
}
unsafe fn window_copy_line_number_mode(mut wme: *mut window_mode_entry) -> ::core::ffi::c_int {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options);
    let mut mode: ::core::ffi::c_int = 0;
    if (*data).line_numbers == 0 {
        return WINDOW_COPY_LINE_NUMBERS_OFF as ::core::ffi::c_int;
    }
    mode = options_get_number(
        oo,
        b"copy-mode-line-numbers\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if (*data).line_numbers == 2 as ::core::ffi::c_int
        && mode == WINDOW_COPY_LINE_NUMBERS_OFF as ::core::ffi::c_int
    {
        return WINDOW_COPY_LINE_NUMBERS_DEFAULT as ::core::ffi::c_int;
    }
    return mode;
}
unsafe fn window_copy_line_number_is_absolute(
    mut wme: *mut window_mode_entry,
) -> ::core::ffi::c_int {
    match window_copy_line_number_mode(wme) {
        2..=4 => return 1 as ::core::ffi::c_int,
        0 | 1 => return 0 as ::core::ffi::c_int,
        _ => {}
    }
    fatalx(|out| out.write_all(b"bad line number mode"));
}
unsafe fn window_copy_line_numbers_active(mut wme: *mut window_mode_entry) -> ::core::ffi::c_int {
    return (window_copy_line_number_mode(wme) != WINDOW_COPY_LINE_NUMBERS_OFF as ::core::ffi::c_int)
        as ::core::ffi::c_int;
}
unsafe fn window_copy_cursor_line_active(mut wme: *mut window_mode_entry) -> ::core::ffi::c_int {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut oo: *mut options = options_owner_ptr(&mut (*(*mode_pane).window).options).map_or(std::ptr::null_mut(), |options| options);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = options_get_string(
        oo,
        b"copy-mode-current-line-style\0" as *const u8 as *const ::core::ffi::c_char,
    );
    return (strcmp(s, b"default\0" as *const u8 as *const ::core::ffi::c_char)
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe fn window_copy_line_number_width(mut wme: *mut window_mode_entry) -> u_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut lines: u_int = 0;
    let mut digits: u_int = 0;
    if window_copy_line_numbers_active(wme) == 0 {
        return 0 as u_int;
    }
    lines = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).backing().grid().sy)
        .wrapping_add(1 as u_int);
    digits = 1 as u_int;
    while lines >= 10 as u_int {
        lines = lines.wrapping_div(10 as u_int);
        digits = digits.wrapping_add(1);
    }
    if digits < 3 as u_int {
        digits = 3 as u_int;
    }
    return digits.wrapping_add(1 as u_int);
}
unsafe fn window_copy_cursor_offset(
    mut wme: *mut window_mode_entry,
    mut cx: u_int,
    mut sx: u_int,
) -> u_int {
    let mut width: u_int = window_copy_line_number_width(wme);
    let mut content: u_int = 0;
    if width == 0 as u_int {
        return cx;
    }
    if width >= sx {
        content = 1 as u_int;
    } else {
        content = sx.wrapping_sub(width);
    }
    if cx >= content {
        return sx.wrapping_sub(1 as u_int);
    }
    return width.wrapping_add(cx);
}
unsafe fn window_copy_cursor_unoffset(
    mut wme: *mut window_mode_entry,
    mut vx: u_int,
    mut sx: u_int,
) -> u_int {
    let mut width: u_int = window_copy_line_number_width(wme);
    let mut content: u_int = 0;
    if width == 0 as u_int {
        return vx;
    }
    if width >= sx {
        content = 1 as u_int;
    } else {
        content = sx.wrapping_sub(width);
    }
    if vx < width {
        return 0 as u_int;
    }
    vx = vx.wrapping_sub(width);
    if vx >= content {
        return content.wrapping_sub(1 as u_int);
    }
    return vx;
}
pub unsafe fn window_copy_set_line_numbers(
    pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut enabled: ::core::ffi::c_int,
) {
    let wp = pane_owner.get();
    let mut wme: *mut window_mode_entry = (*wp).modes.active;
    if wme.is_null() || !std::ptr::eq((*wme).mode, &window_copy_mode) {
        return;
    }
    window_copy_set_line_numbers1(wme, enabled, 0 as ::core::ffi::c_int);
}
unsafe fn window_copy_set_line_numbers1(
    mut wme: *mut window_mode_entry,
    mut enabled: ::core::ffi::c_int,
    mut force: ::core::ffi::c_int,
) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = options_owner_ptr(&mut (*(*mode_pane).window).options).map_or(std::ptr::null_mut(), |options| options);
    let mut active: ::core::ffi::c_int = 0;
    let mut line_numbers: ::core::ffi::c_int = 0;
    if data.is_null() {
        return;
    }
    active = window_copy_line_numbers_active(wme);
    if enabled == 0 {
        line_numbers = 0 as ::core::ffi::c_int;
    } else if force != 0
        && options_get_number(
            oo,
            b"copy-mode-line-numbers\0" as *const u8 as *const ::core::ffi::c_char,
        ) == WINDOW_COPY_LINE_NUMBERS_OFF as ::core::ffi::c_int as ::core::ffi::c_longlong
    {
        line_numbers = 2 as ::core::ffi::c_int;
    } else {
        line_numbers = 1 as ::core::ffi::c_int;
    }
    if (*data).line_numbers == line_numbers && active == enabled {
        return;
    }
    (*data).line_numbers = line_numbers;
    window_copy_redraw_screen(wme);
}
pub unsafe fn window_copy_get_current_offset(pane: &window_pane) -> Option<(u_int, u_int)> {
    let mode = pane.modes.active.as_ref()?;
    if !std::ptr::eq(mode.mode, &window_copy_mode) && !std::ptr::eq(mode.mode, &window_view_mode) {
        return None;
    }
    let data = (mode.data as *const window_copy_mode_data).as_ref()?;
    let hsize = data.backing().grid().hsize;
    Some((hsize.wrapping_sub(data.oy), hsize))
}

unsafe fn window_copy_write_line(
    mut wme: *mut window_mode_entry,
    mut ctx: *mut screen_write_ctx,
    mut py: u_int,
) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut oo: *mut options = options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options);
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
    let mut cgc: grid_cell = grid_cell {
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
    let mut mkgc: grid_cell = grid_cell {
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
    let mut clgc: grid_cell = grid_cell {
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
    let mut ln_gc: grid_cell = grid_cell {
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
    let mut cur_ln_gc: grid_cell = grid_cell {
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
    let mut sx: u_int = (*s).grid().sx;
    let mut hsize: u_int = (*data).backing().grid().hsize;
    let mut width: u_int = 0;
    let mut absolute: u_int = 0;
    let mut line_number: u_int = 0;
    let mut content_sx: u_int = 0;
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut current: ::core::ffi::c_int = 0;
    let mut mode: ::core::ffi::c_int = 0;
    width = window_copy_line_number_width(wme);
    if width >= sx {
        content_sx = 1 as u_int;
    } else if width != 0 as u_int {
        content_sx = sx.wrapping_sub(width);
    } else {
        content_sx = sx;
    }
    screen_write_cursormove(
        &mut *ctx,
        0 as ::core::ffi::c_int,
        py as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        wp,
    );
    style_apply(
        &raw mut gc,
        oo,
        b"copy-mode-position-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    style_apply(
        &raw mut mgc,
        oo,
        b"copy-mode-match-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    mgc.flags = (mgc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    style_apply(
        &raw mut cgc,
        oo,
        b"copy-mode-current-match-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    cgc.flags = (cgc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    style_apply(
        &raw mut mkgc,
        oo,
        b"copy-mode-mark-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    mkgc.flags = (mkgc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    style_apply(
        &raw mut clgc,
        oo,
        b"copy-mode-current-line-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    clgc.flags = (clgc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    if width != 0 as u_int {
        style_apply(
            &raw mut ln_gc,
            oo,
            b"copy-mode-line-number-style\0" as *const u8 as *const ::core::ffi::c_char,
            ft,
        );
        ln_gc.flags = (ln_gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        style_apply(
            &raw mut cur_ln_gc,
            oo,
            b"copy-mode-current-line-number-style\0" as *const u8 as *const ::core::ffi::c_char,
            ft,
        );
        cur_ln_gc.flags = (cur_ln_gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        current = (py == (*data).cy) as ::core::ffi::c_int;
        absolute = hsize
            .wrapping_sub((*data).oy)
            .wrapping_add(py)
            .wrapping_add(1 as u_int);
        mode = window_copy_line_number_mode(wme);
        if mode == WINDOW_COPY_LINE_NUMBERS_DEFAULT as ::core::ffi::c_int {
            if py < (*data).oy {
                line_number = (*data).oy.wrapping_sub(py);
            } else {
                line_number = py.wrapping_sub((*data).oy);
            }
        } else if mode == WINDOW_COPY_LINE_NUMBERS_ABSOLUTE as ::core::ffi::c_int {
            line_number = absolute;
        } else if mode == WINDOW_COPY_LINE_NUMBERS_HYBRID as ::core::ffi::c_int && current != 0 {
            line_number = absolute;
        } else if py > (*data).cy {
            line_number = py.wrapping_sub((*data).cy);
        } else {
            line_number = (*data).cy.wrapping_sub(py);
        }
        screen_write_cursormove(
            &mut *ctx,
            0 as ::core::ffi::c_int,
            py as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_nputs(
            &mut *ctx,
            width as ssize_t,
            &*if current != 0 {
                &raw mut cur_ln_gc
            } else {
                &raw mut ln_gc
            },
            |out| {
                let width = (width as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as i32;
                if width < 0 {
                    write!(
                        out,
                        "{:<width$}",
                        (line_number) as u32,
                        width = width.unsigned_abs() as usize
                    )
                } else {
                    write!(
                        out,
                        "{:>width$}",
                        (line_number) as u32,
                        width = width as usize
                    )
                }?;
                out.write_all(b" ")
            },
        );
    }
    window_copy_write_one(
        wme,
        ctx,
        width,
        py,
        hsize.wrapping_sub((*data).oy).wrapping_add(py),
        content_sx,
        &raw mut mgc,
        &raw mut cgc,
        &raw mut mkgc,
        &raw mut clgc,
    );
    if py == 0 as u_int && (*s).rupper < (*s).rlower && (*data).hide_position == 0 {
        value = options_get_string(
            oo,
            b"copy-mode-position-format\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if *value as ::core::ffi::c_int != '\0' as i32 {
            let expanded = format_expand_cstring(ft, value);
            if !expanded.is_empty() {
                screen_write_cursormove(
                    &mut *ctx,
                    width as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                format_draw(
                    ctx,
                    &raw mut gc,
                    content_sx,
                    expanded.as_ptr(),
                    ::core::ptr::null_mut::<style_ranges>(),
                    0 as ::core::ffi::c_int,
                );
            }
        }
    }
    if py == (*data).cy && (*data).cx >= content_sx {
        screen_write_cursormove(
            &mut *ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*s).grid().sx) as ::core::ffi::c_int,
            py as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_putc(&mut *ctx, &grid_default_cell, '$' as i32 as u_char);
    }
    format_free(ft);
}
unsafe fn window_copy_write_lines(
    mut wme: *mut window_mode_entry,
    mut ctx: *mut screen_write_ctx,
    mut py: u_int,
    mut ny: u_int,
) {
    let mut yy: u_int = 0;
    yy = py;
    while yy < py.wrapping_add(ny) {
        window_copy_write_line(wme, ctx, yy);
        yy = yy.wrapping_add(1);
    }
}
unsafe fn window_copy_redraw_selection(mut wme: *mut window_mode_entry, mut old_y: u_int) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*data).backing_mut().grid_mut();
    let mut new_y: u_int = 0;
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    new_y = (*data).cy;
    if old_y <= new_y {
        start = old_y;
        end = new_y;
    } else {
        start = new_y;
        end = old_y;
    }
    if (*data).selflag as ::core::ffi::c_uint
        == SEL_WORD as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if end < (*gd).sy.wrapping_add((*data).oy).wrapping_sub(1 as u_int) {
            end = end.wrapping_add(1);
        }
    }
    window_copy_redraw_lines(wme, start, end.wrapping_sub(start).wrapping_add(1 as u_int));
}
unsafe fn window_copy_redraw_lines(mut wme: *mut window_mode_entry, mut py: u_int, mut ny: u_int) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut i: u_int = 0;
    if window_copy_line_number_width(wme) != 0 as u_int {
        screen_write_start(&mut ctx, &raw mut (*data).screen);
        i = py;
        while i < py.wrapping_add(ny) {
            window_copy_write_line(wme, &raw mut ctx, i);
            i = i.wrapping_add(1);
        }
        screen_write_cursormove(
            &mut ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*s).grid().sx) as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&mut ctx);
        (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR;
        return;
    }
    if window_pane_scrollbar_overlay_visible(&*wp) != 0 {
        screen_write_start(&mut ctx, &raw mut (*data).screen);
    } else {
        screen_write_start_pane(&mut ctx, &(*wp).observer.upgrade().expect("live screen-write pane"), ::core::ptr::null_mut::<screen>());
    }
    i = py;
    while i < py.wrapping_add(ny) {
        window_copy_write_line(wme, &raw mut ctx, i);
        i = i.wrapping_add(1);
    }
    screen_write_cursormove(
        &mut ctx,
        window_copy_cursor_offset(wme, (*data).cx, (*s).grid().sx) as ::core::ffi::c_int,
        (*data).cy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&mut ctx);
    window_pane_scrollbar_redraw(&mode_pane_owner);
}
unsafe fn window_copy_redraw_screen(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    window_copy_redraw_lines(wme, 0 as u_int, (*data).screen.grid().sy);
}
unsafe fn window_copy_style_changed(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    if !(*data).screen.sel.is_none() {
        window_copy_set_selection(wme, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    }
    window_copy_redraw_screen(wme);
}
unsafe fn window_copy_synchronize_cursor_end(
    mut wme: *mut window_mode_entry,
    mut begin: ::core::ffi::c_int,
    mut no_reset: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    xx = (*data).cx;
    yy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    match (*data).selflag as ::core::ffi::c_uint {
        1 => {
            if !(no_reset != 0) {
                begin = 0 as ::core::ffi::c_int;
                if (*data).dy > yy || (*data).dy == yy && (*data).dx > xx {
                    window_copy_cursor_previous_word_pos(
                        wme,
                        window_copy_separators(&*data),
                        &raw mut xx,
                        &raw mut yy,
                    );
                    begin = 1 as ::core::ffi::c_int;
                    (*data).endselx = (*data).endselrx;
                    (*data).endsely = (*data).endselry;
                } else {
                    if xx >= window_copy_find_length(wme, yy)
                        || window_copy_in_set(
                            wme,
                            xx.wrapping_add(1 as u_int),
                            yy,
                            WHITESPACE.as_ptr(),
                        ) == 0
                    {
                        window_copy_cursor_next_word_end_pos(
                            wme,
                            window_copy_separators(&*data),
                            &raw mut xx,
                            &raw mut yy,
                        );
                    }
                    (*data).selx = (*data).selrx;
                    (*data).sely = (*data).selry;
                }
            }
        }
        2 => {
            if !(no_reset != 0) {
                begin = 0 as ::core::ffi::c_int;
                if (*data).dy > yy {
                    xx = 0 as u_int;
                    begin = 1 as ::core::ffi::c_int;
                    (*data).endselx = (*data).endselrx;
                    (*data).endsely = (*data).endselry;
                } else {
                    if yy < (*data).endselry {
                        yy = (*data).endselry;
                    }
                    xx = window_copy_find_length(wme, yy);
                    (*data).selx = (*data).selrx;
                    (*data).sely = (*data).selry;
                }
            }
        }
        0 | _ => {}
    }
    if begin != 0 {
        (*data).selx = xx;
        (*data).sely = yy;
    } else {
        (*data).endselx = xx;
        (*data).endsely = yy;
    };
}
unsafe fn window_copy_synchronize_cursor(
    mut wme: *mut window_mode_entry,
    mut no_reset: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    match (*data).cursordrag as ::core::ffi::c_uint {
        1 => {
            window_copy_synchronize_cursor_end(wme, 0 as ::core::ffi::c_int, no_reset);
        }
        2 => {
            window_copy_synchronize_cursor_end(wme, 1 as ::core::ffi::c_int, no_reset);
        }
        0 | _ => {}
    };
}
unsafe fn window_copy_update_cursor(mut wme: *mut window_mode_entry, mut cx: u_int, mut cy: u_int) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut old_cx: u_int = 0;
    let mut old_cy: u_int = 0;
    let mut py: u_int = 0;
    let mut width: u_int = 0;
    let mut content_sx: u_int = 0;
    let mut maxx: u_int = 0;
    let mut allow_onemore: ::core::ffi::c_int = 0;
    if (*data).rectflag == 0 && cy < (*s).grid().sy {
        allow_onemore =
            (!(*data).screen.sel.is_none() && (*data).rectflag != 0) as ::core::ffi::c_int;
        py = (*data).backing().grid()
            .hsize
            .wrapping_add(cy)
            .wrapping_sub((*data).oy);
        maxx = window_copy_cursor_limit(wme, py, allow_onemore);
        if cx > maxx {
            cx = maxx;
        }
    }
    old_cx = (*data).cx;
    old_cy = (*data).cy;
    (*data).cx = cx;
    (*data).cy = cy;
    if window_copy_line_numbers_active(wme) != 0 {
        width = window_copy_line_number_width(wme);
        if !(*s).sel.is_none()
            || (*data).lineflag as ::core::ffi::c_uint
                != LINE_SEL_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            || old_cy != (*data).cy
        {
            window_copy_redraw_screen(wme);
            return;
        }
        if width >= (*s).grid().sx {
            content_sx = 1 as u_int;
        } else {
            content_sx = (*s).grid().sx.wrapping_sub(width);
        }
        if old_cx >= content_sx || (*data).cx >= content_sx {
            window_copy_redraw_screen(wme);
            return;
        }
        screen_write_start_pane(&mut ctx, &(*wp).observer.upgrade().expect("live screen-write pane"), ::core::ptr::null_mut::<screen>());
        screen_write_cursormove(
            &mut ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*s).grid().sx) as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&mut ctx);
        return;
    }
    if old_cy != (*data).cy && window_copy_cursor_line_active(wme) != 0 {
        window_copy_redraw_lines(wme, old_cy, 1 as u_int);
        window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
        return;
    }
    if old_cx == (*s).grid().sx {
        window_copy_redraw_lines(wme, old_cy, 1 as u_int);
    }
    if (*data).cx == (*s).grid().sx {
        window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
    } else {
        screen_write_start_pane(&mut ctx, &(*wp).observer.upgrade().expect("live screen-write pane"), ::core::ptr::null_mut::<screen>());
        screen_write_cursormove(
            &mut ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*s).grid().sx) as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&mut ctx);
    };
}
unsafe fn window_copy_start_selection(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).selx = (*data).cx;
    (*data).sely = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).endselx = (*data).selx;
    (*data).endsely = (*data).sely;
    (*data).cursordrag = CURSORDRAG_ENDSEL;
    window_copy_set_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
}
unsafe fn window_copy_mouse_in_selection(
    mut wme: *mut window_mode_entry,
    mut x: u_int,
    mut y: u_int,
    mut on_start: *mut ::core::ffi::c_int,
    mut on_end: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut sx: u_int = (*data).screen.grid().sx;
    let mut sy: u_int = (*data).screen.grid().sy;
    let mut hsize: u_int = 0;
    let mut screeny: u_int = 0;
    let mut mx: u_int = 0;
    let mut my: u_int = 0;
    let mut selx: u_int = 0;
    let mut sely: u_int = 0;
    let mut endselx: u_int = 0;
    let mut endsely: u_int = 0;
    let mut cursorx: u_int = 0;
    let mut cursory: u_int = 0;
    let mut mpos: ::core::ffi::c_longlong = 0;
    let mut spos: ::core::ffi::c_longlong = 0;
    let mut epos: ::core::ffi::c_longlong = 0;
    let mut dstart: ::core::ffi::c_longlong = 0;
    let mut dend: ::core::ffi::c_longlong = 0;
    if !on_start.is_null() {
        *on_start = 0 as ::core::ffi::c_int;
    }
    if !on_end.is_null() {
        *on_end = 0 as ::core::ffi::c_int;
    }
    if (*data).screen.sel.is_none() {
        return 0 as ::core::ffi::c_int;
    }
    hsize = (*data).backing().grid().hsize;
    screeny = hsize.wrapping_sub((*data).oy);
    selx = window_copy_cursor_offset(wme, (*data).selx, sx);
    sely = (*data).sely.wrapping_sub(screeny);
    if (*data).sely >= screeny && sely < sy && x == selx && y == sely {
        if !on_start.is_null() {
            *on_start = 1 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    endselx = window_copy_cursor_offset(wme, (*data).endselx, sx);
    endsely = (*data).endsely.wrapping_sub(screeny);
    if (*data).endsely >= screeny && endsely < sy && x == endselx && y == endsely {
        if !on_end.is_null() {
            *on_end = 1 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    cursorx = window_copy_cursor_offset(wme, (*data).cx, sx);
    cursory = (*data).cy;
    if x != cursorx || y != cursory {
        if screen_check_selection(&(*data).screen, x, y) == 0 {
            return 0 as ::core::ffi::c_int;
        }
    }
    if !on_start.is_null() || !on_end.is_null() {
        mx = window_copy_cursor_unoffset(wme, x, sx);
        my = screeny.wrapping_add(y);
        mpos = my as ::core::ffi::c_longlong
            * sx.wrapping_add(1 as u_int) as ::core::ffi::c_longlong
            + mx as ::core::ffi::c_longlong;
        spos = (*data).sely as ::core::ffi::c_longlong
            * sx.wrapping_add(1 as u_int) as ::core::ffi::c_longlong
            + (*data).selx as ::core::ffi::c_longlong;
        epos = (*data).endsely as ::core::ffi::c_longlong
            * sx.wrapping_add(1 as u_int) as ::core::ffi::c_longlong
            + (*data).endselx as ::core::ffi::c_longlong;
        dstart = llabs(mpos - spos);
        dend = llabs(mpos - epos);
        if dstart <= dend {
            if !on_start.is_null() {
                *on_start = 1 as ::core::ffi::c_int;
            }
        } else if !on_end.is_null() {
            *on_end = 1 as ::core::ffi::c_int;
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_copy_adjust_selection(
    mut wme: *mut window_mode_entry,
    mut selx: *mut u_int,
    mut sely: *mut u_int,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut ty: u_int = 0;
    let mut relpos: ::core::ffi::c_int = 0;
    sx = *selx;
    sy = *sely;
    ty = (*data).backing().grid().hsize.wrapping_sub((*data).oy);
    if sy < ty {
        relpos = WINDOW_COPY_REL_POS_ABOVE as ::core::ffi::c_int;
        if (*data).rectflag == 0 {
            sx = 0 as u_int;
        }
        sy = 0 as u_int;
    } else if sy > ty.wrapping_add((*s).grid().sy).wrapping_sub(1 as u_int) {
        relpos = WINDOW_COPY_REL_POS_BELOW as ::core::ffi::c_int;
        if (*data).rectflag == 0 {
            sx = (*s).grid().sx.wrapping_sub(1 as u_int);
        }
        sy = (*s).grid().sy.wrapping_sub(1 as u_int);
    } else {
        relpos = WINDOW_COPY_REL_POS_ON_SCREEN as ::core::ffi::c_int;
        sy = sy.wrapping_sub(ty);
    }
    *selx = sx;
    *sely = sy;
    return relpos;
}
unsafe fn window_copy_update_selection(
    mut wme: *mut window_mode_entry,
    mut may_redraw: ::core::ffi::c_int,
    mut no_reset: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    if (*s).sel.is_none()
        && (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    return window_copy_set_selection(wme, may_redraw, no_reset);
}
unsafe fn window_copy_update_selection_view(
    mut wme: *mut window_mode_entry,
    mut may_redraw: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut no_reset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut selx: u_int = 0;
    let mut sely: u_int = 0;
    let mut endselx: u_int = 0;
    let mut endsely: u_int = 0;
    let mut changed: ::core::ffi::c_int = 0;
    if (*data).cursordrag as ::core::ffi::c_uint
        != CURSORDRAG_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return window_copy_update_selection(wme, may_redraw, no_reset);
    }
    selx = (*data).selx;
    sely = (*data).sely;
    endselx = (*data).endselx;
    endsely = (*data).endsely;
    changed = window_copy_update_selection(wme, may_redraw, 1 as ::core::ffi::c_int);
    (*data).selx = selx;
    (*data).sely = sely;
    (*data).endselx = endselx;
    (*data).endsely = endsely;
    return changed;
}
unsafe fn window_copy_set_selection(
    mut wme: *mut window_mode_entry,
    mut may_redraw: ::core::ffi::c_int,
    mut no_reset: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut oo: *mut options = options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options);
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
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut cy: u_int = 0;
    let mut endsx: u_int = 0;
    let mut endsy: u_int = 0;
    let mut clipx: u_int = 0;
    let mut startrelpos: ::core::ffi::c_int = 0;
    let mut endrelpos: ::core::ffi::c_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    window_copy_synchronize_cursor(wme, no_reset);
    sx = (*data).selx;
    sy = (*data).sely;
    startrelpos = window_copy_adjust_selection(wme, &raw mut sx, &raw mut sy);
    endsx = (*data).endselx;
    endsy = (*data).endsely;
    endrelpos = window_copy_adjust_selection(wme, &raw mut endsx, &raw mut endsy);
    if startrelpos == endrelpos
        && startrelpos != WINDOW_COPY_REL_POS_ON_SCREEN as ::core::ffi::c_int
    {
        screen_hide_selection(&mut *s);
        return 0 as ::core::ffi::c_int;
    }
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        wp,
    );
    style_apply(
        &raw mut gc,
        oo,
        b"copy-mode-selection-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    format_free(ft);
    clipx = window_copy_line_number_width(wme);
    if clipx >= (*s).grid().sx {
        clipx = (*s).grid().sx.wrapping_sub(1 as u_int);
    }
    if window_copy_line_numbers_active(wme) != 0 {
        sx = window_copy_cursor_offset(wme, sx, (*s).grid().sx);
        endsx = window_copy_cursor_offset(wme, endsx, (*s).grid().sx);
    }
    screen_set_selection(
        &mut *s,
        sx,
        sy,
        endsx,
        endsy,
        (*data).rectflag as u_int,
        clipx,
        (*data).modekeys,
        &gc,
    );
    if (*data).rectflag != 0 && may_redraw != 0 {
        cy = (*data).cy;
        if (*data).cursordrag as ::core::ffi::c_uint
            == CURSORDRAG_ENDSEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if sy < cy {
                window_copy_redraw_lines(wme, sy, cy.wrapping_sub(sy).wrapping_add(1 as u_int));
            } else {
                window_copy_redraw_lines(wme, cy, sy.wrapping_sub(cy).wrapping_add(1 as u_int));
            }
        } else if endsy < cy {
            window_copy_redraw_lines(wme, endsy, cy.wrapping_sub(endsy).wrapping_add(1 as u_int));
        } else {
            window_copy_redraw_lines(wme, cy, endsy.wrapping_sub(cy).wrapping_add(1 as u_int));
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_copy_get_selection(mut wme: *mut window_mode_entry) -> Option<Vec<u8>> {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut buf = Vec::<u8>::new();
    let mut i: u_int = 0;
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut ex: u_int = 0;
    let mut ey: u_int = 0;
    let mut ey_last: u_int = 0;
    let mut firstsx: u_int = 0;
    let mut lastex: u_int = 0;
    let mut restex: u_int = 0;
    let mut restsx: u_int = 0;
    let mut selx: u_int = 0;
    let mut keys: ::core::ffi::c_int = 0;
    if (*data).screen.sel.is_none()
        && (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return window_copy_match_at_cursor_bytes(data).map(|mut matched| {
            // This fallback used strlen, so an embedded NUL ends the copied match.
            if let Some(first_nul) = matched.iter().position(|&byte| byte == 0) {
                matched.truncate(first_nul);
            }
            matched
        });
    }
    xx = (*data).endselx;
    yy = (*data).endsely;
    if yy < (*data).sely || yy == (*data).sely && xx < (*data).selx {
        sx = xx;
        sy = yy;
        ex = (*data).selx;
        ey = (*data).sely;
    } else {
        sx = (*data).selx;
        sy = (*data).sely;
        ex = xx;
        ey = yy;
    }
    ey_last = window_copy_find_length(wme, ey);
    if ex > ey_last {
        ex = ey_last;
    }
    xx = (*s).grid().sx;
    keys = options_get_number(
        options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options),
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if (*data).rectflag != 0 {
        if (*data).cursordrag as ::core::ffi::c_uint
            == CURSORDRAG_ENDSEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            selx = (*data).selx;
        } else {
            selx = (*data).endselx;
        }
        if selx < (*data).cx {
            if keys == MODEKEY_EMACS {
                lastex = (*data).cx;
                restex = (*data).cx;
            } else {
                lastex = (*data).cx.wrapping_add(1 as u_int);
                restex = (*data).cx.wrapping_add(1 as u_int);
            }
            firstsx = selx;
            restsx = selx;
        } else {
            lastex = selx.wrapping_add(1 as u_int);
            restex = selx.wrapping_add(1 as u_int);
            firstsx = (*data).cx;
            restsx = (*data).cx;
        }
    } else {
        if keys == MODEKEY_EMACS {
            lastex = ex;
        } else {
            lastex = ex.wrapping_add(1 as u_int);
        }
        restex = xx;
        firstsx = sx;
        restsx = 0 as u_int;
    }
    i = sy;
    while i <= ey {
        window_copy_copy_line(
            wme,
            &mut buf,
            i,
            if i == sy { firstsx } else { restsx },
            if i == ey { lastex } else { restex },
        );
        i = i.wrapping_add(1);
    }
    if buf.is_empty() {
        return None;
    }
    if keys == MODEKEY_EMACS || lastex <= ey_last {
        if !((*grid_get_line((*data).backing().grid(), ey)).flags as ::core::ffi::c_int)
            & GRID_LINE_WRAPPED
            != 0
            || lastex != ey_last
        {
            buf.pop();
        }
    }
    return Some(buf);
}
unsafe fn window_copy_copy_buffer(
    mut wme: *mut window_mode_entry,
    prefix: *const ::core::ffi::c_char,
    buf: Vec<u8>,
    mut set_paste: ::core::ffi::c_int,
    mut set_clip: ::core::ffi::c_int,
) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let prefix = if set_paste != 0 && !buf.is_empty() && !prefix.is_null() {
        Some(CStr::from_ptr(prefix).to_owned())
    } else {
        None
    };
    let mut wp: *mut window_pane = mode_pane;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if set_clip != 0
        && options_get_number(
            global_options,
            b"set-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0 as ::core::ffi::c_longlong
    {
        if window_copy_line_numbers_active(wme) != 0 && (*wp).flags & PANE_REDRAW != 0 {
            redraw = PANE_REDRAW;
            (*wp).flags &= !PANE_REDRAW;
        }
        screen_write_start_pane(&mut ctx, &(*wp).observer.upgrade().expect("live screen-write pane"), ::core::ptr::null_mut::<screen>());
        screen_write_setselection(
            &mut ctx,
            c"",
            &buf,
        );
        screen_write_stop(&mut ctx);
        (*wp).flags |= redraw;
        events_fire_pane(
            b"pane-set-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
            wp,
        );
    }
    if set_paste != 0 {
        paste_add_owned(prefix, buf.into_boxed_slice());
    };
}
unsafe fn window_copy_pipe_run(
    mut wme: *mut window_mode_entry,
    mut s: *mut session,
    mut cmd: *const ::core::ffi::c_char,
) -> Option<Vec<u8>> {
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    let buf = window_copy_get_selection(wme);
    if cmd.is_null() || *cmd as ::core::ffi::c_int == '\0' as i32 {
        cmd = options_get_string(
            global_options,
            b"copy-command\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !cmd.is_null() && *cmd as ::core::ffi::c_int != '\0' as i32 {
        job = job_run(
            Some(CStr::from_ptr(cmd)),
            &Vec::new(),
            None,
            s,
            None,
            None,
            None,
            None,
            JOB_NOWAIT,
            -(1 as ::core::ffi::c_int),
            -(1 as ::core::ffi::c_int),
        );
        if !job.is_null() {
            bufferevent_write(
                job_get_event(job),
                buf.as_ref()
                    .map_or(::core::ptr::null(), |buf| buf.as_ptr())
                    .cast(),
                buf.as_ref().map_or(0, Vec::len),
            );
        }
    }
    buf
}
unsafe fn window_copy_pipe(
    mut wme: *mut window_mode_entry,
    mut s: *mut session,
    mut cmd: *const ::core::ffi::c_char,
) {
    let _ = window_copy_pipe_run(wme, s, cmd);
}
unsafe fn window_copy_copy_pipe(
    mut wme: *mut window_mode_entry,
    mut s: *mut session,
    mut prefix: *const ::core::ffi::c_char,
    mut cmd: *const ::core::ffi::c_char,
    mut set_paste: ::core::ffi::c_int,
    mut set_clip: ::core::ffi::c_int,
) {
    if let Some(buf) = window_copy_pipe_run(wme, s, cmd) {
        window_copy_copy_buffer(wme, prefix, buf, set_paste, set_clip);
    }
}
unsafe fn window_copy_copy_selection(
    mut wme: *mut window_mode_entry,
    mut prefix: *const ::core::ffi::c_char,
    mut set_paste: ::core::ffi::c_int,
    mut set_clip: ::core::ffi::c_int,
) {
    if let Some(buf) = window_copy_get_selection(wme) {
        window_copy_copy_buffer(wme, prefix, buf, set_paste, set_clip);
    }
}
unsafe fn window_copy_append_selection(mut wme: *mut window_mode_entry) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut buf: Vec<u8>;
    let mut bufname: Option<CString> = None;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    buf = match window_copy_get_selection(wme) {
        Some(buf) => buf,
        None => return,
    };
    if options_get_number(
        global_options,
        b"set-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0 as ::core::ffi::c_longlong
    {
        screen_write_start_pane(&mut ctx, &(*wp).observer.upgrade().expect("live screen-write pane"), ::core::ptr::null_mut::<screen>());
        screen_write_setselection(
            &mut ctx,
            c"",
            &buf,
        );
        screen_write_stop(&mut ctx);
        events_fire_pane(
            b"pane-set-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
            wp,
        );
    }
    if let Some(pb) = paste_get_top(Some(&mut bufname)) {
        let buffer = pb.borrow();
        let bufdata = paste_buffer_data(&buffer).unwrap_or_default();
        let mut appended = bufdata.to_vec();
        appended.extend_from_slice(&buf);
        buf = appended;
    }
    let _ = paste_set_owned(buf.into_boxed_slice(), bufname.as_deref(), None);
}
unsafe fn window_copy_copy_line(
    mut wme: *mut window_mode_entry,
    buf: &mut Vec<u8>,
    mut sy: u_int,
    mut sx: u_int,
    mut ex: u_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*data).backing_mut().grid_mut();
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
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut i: u_int = 0;
    let mut xx: u_int = 0;
    let mut wrapped: u_int = 0 as u_int;
    if sx > ex {
        return;
    }
    let gl = grid_get_line(&*gd, sy);
    if gl.flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0
        && gl.cellsize as u_int <= (*gd).sx
    {
        wrapped = 1 as u_int;
    }
    if wrapped != 0 {
        xx = gl.cellsize as u_int;
    } else {
        xx = window_copy_find_length(wme, sy);
    }
    if ex > xx {
        ex = xx;
    }
    if sx > xx {
        sx = xx;
    }
    if sx < ex {
        i = sx;
        while i < ex {
            grid_get_cell(&*gd, i, sy, &mut gc);
            if !(gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
                if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                    utf8_set(&mut ud, '\t' as i32 as u_char);
                } else {
                    ud = utf8_copy(&gc.data);
                }
                if ud.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                    && gc.attr as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0
                {
                    if let Some(acs) = tty_acs_get(None, ud.data[0]) {
                        let bytes = acs.to_bytes();
                        if bytes.len() <= ud.data.len() {
                            ud.size = bytes.len() as u_char;
                            ud.data[..bytes.len()].copy_from_slice(bytes);
                        }
                    }
                }
                buf.extend_from_slice(&ud.data[..ud.size as usize]);
            }
            i = i.wrapping_add(1);
        }
    }
    if wrapped == 0 || ex != xx {
        buf.push(b'\n');
    }
}
unsafe fn window_copy_clear_selection(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    screen_clear_selection(&mut (*data).screen);
    (*data).cursordrag = CURSORDRAG_NONE;
    (*data).lineflag = LINE_SEL_NONE;
    (*data).selflag = SEL_CHAR;
    py = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    px = window_copy_cursor_limit(wme, py, (*data).rectflag);
    if (*data).cx > px {
        window_copy_update_cursor(wme, px, (*data).cy);
    }
}
unsafe fn window_copy_in_set(
    mut wme: *mut window_mode_entry,
    mut px: u_int,
    mut py: u_int,
    mut set: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    return grid_in_set(&*((*data).backing().grid()), px, py, CStr::from_ptr(set));
}
unsafe fn window_copy_find_length(mut wme: *mut window_mode_entry, mut py: u_int) -> u_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    return grid_line_length(&*((*data).backing().grid()), py);
}
unsafe fn window_copy_cursor_limit(
    mut wme: *mut window_mode_entry,
    mut py: u_int,
    mut allow_onemore: ::core::ffi::c_int,
) -> u_int {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = options_owner_ptr(&mut (*(*mode_pane).window).options).map_or(std::ptr::null_mut(), |options| options);
    if allow_onemore != 0
        || options_get_number(
            oo,
            b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
        ) != MODEKEY_VI as ::core::ffi::c_longlong
    {
        return window_copy_find_length(wme, py);
    }
    return grid_line_limit(&*((*data).backing().grid()), py);
}
unsafe fn window_copy_cursor_start_of_line(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    grid_reader_cursor_start_of_line(&mut gr, 1 as ::core::ffi::c_int);
    (px, py) = grid_reader_get_cursor(&gr);
    window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
}
unsafe fn window_copy_cursor_back_to_indentation(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    grid_reader_cursor_back_to_indentation(&mut gr);
    (px, py) = grid_reader_get_cursor(&gr);
    window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
}
unsafe fn window_copy_cursor_end_of_line(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    if !(*data).screen.sel.is_none() && (*data).rectflag != 0 {
        grid_reader_cursor_end_of_line(
            &mut gr,
            1 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
    } else {
        grid_reader_cursor_end_of_line(
            &mut gr,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    (px, py) = grid_reader_get_cursor(&gr);
    if (*data).screen.sel.is_none() || (*data).rectflag == 0 {
        px = window_copy_cursor_limit(wme, py, 0 as ::core::ffi::c_int);
    }
    window_copy_acquire_cursor_down(
        wme,
        hsize,
        (*back_s).grid().sy,
        (*data).oy,
        oldy,
        px,
        py,
        0 as ::core::ffi::c_int,
    );
}
unsafe fn window_copy_other_end(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut selx: u_int = 0;
    let mut sely: u_int = 0;
    let mut cy: u_int = 0;
    let mut yy: u_int = 0;
    let mut hsize: u_int = 0;
    if (*s).sel.is_none()
        && (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*data).lineflag = LINE_SEL_RIGHT_LEFT;
    } else if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*data).lineflag = LINE_SEL_LEFT_RIGHT;
    }
    match (*data).cursordrag as ::core::ffi::c_uint {
        0 | 2 => {
            (*data).cursordrag = CURSORDRAG_ENDSEL;
        }
        1 => {
            (*data).cursordrag = CURSORDRAG_SEL;
        }
        _ => {}
    }
    selx = (*data).endselx;
    sely = (*data).endsely;
    if (*data).cursordrag as ::core::ffi::c_uint
        == CURSORDRAG_SEL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        selx = (*data).selx;
        sely = (*data).sely;
    }
    cy = (*data).cy;
    yy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).cx = selx;
    hsize = (*data).backing().grid().hsize;
    if sely < hsize.wrapping_sub((*data).oy) {
        (*data).oy = hsize.wrapping_sub(sely);
        (*data).cy = 0 as u_int;
    } else if sely > hsize.wrapping_sub((*data).oy).wrapping_add((*s).grid().sy) {
        (*data).oy = hsize
            .wrapping_sub(sely)
            .wrapping_add((*s).grid().sy)
            .wrapping_sub(1 as u_int);
        (*data).cy = (*s).grid().sy.wrapping_sub(1 as u_int);
    } else {
        (*data).cy = cy.wrapping_add(sely).wrapping_sub(yy);
    }
    yy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    hsize = window_copy_cursor_limit(wme, yy, (*data).rectflag);
    if (*data).cx > hsize {
        (*data).cx = hsize;
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    window_copy_redraw_screen(wme);
}
unsafe fn window_copy_cursor_left(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    grid_reader_cursor_left(&mut gr, 1 as ::core::ffi::c_int);
    (px, py) = grid_reader_get_cursor(&gr);
    window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
}
unsafe fn window_copy_cursor_right(mut wme: *mut window_mode_entry, mut all: ::core::ffi::c_int) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options);
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    let mut onemore: ::core::ffi::c_int = 0;
    px = (*data).cx;
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    onemore = (options_get_number(
        oo,
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) != MODEKEY_VI as ::core::ffi::c_longlong) as ::core::ffi::c_int;
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    grid_reader_cursor_right(&mut gr, 1 as ::core::ffi::c_int, all, onemore);
    (px, py) = grid_reader_get_cursor(&gr);
    window_copy_acquire_cursor_down(
        wme,
        hsize,
        (*back_s).grid().sy,
        (*data).oy,
        oldy,
        px,
        py,
        0 as ::core::ffi::c_int,
    );
}
unsafe fn window_copy_cursor_up(
    mut wme: *mut window_mode_entry,
    mut scroll_only: ::core::ffi::c_int,
) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = options_owner_ptr(&mut (*(*mode_pane).window).options).map_or(std::ptr::null_mut(), |options| options);
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut norectsel: ::core::ffi::c_int = 0;
    norectsel = ((*data).screen.sel.is_none() || (*data).rectflag == 0) as ::core::ffi::c_int;
    oy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    ox = window_copy_find_length(wme, oy);
    if norectsel != 0 && (*data).cx != ox {
        (*data).lastcx = (*data).cx;
        (*data).lastsx = ox;
    }
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        && oy == (*data).sely
    {
        window_copy_other_end(wme);
    }
    if scroll_only != 0
        && options_get_number(
            oo,
            b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
        ) == MODEKEY_VI as ::core::ffi::c_longlong
    {
        if (*data).cy < (*s).grid().sy.wrapping_sub(1 as u_int) {
            window_copy_update_cursor(wme, (*data).cx, (*data).cy.wrapping_add(1 as u_int));
        }
    }
    if scroll_only != 0 || (*data).cy == 0 as u_int {
        if norectsel != 0 {
            (*data).cx = (*data).lastcx;
        }
        window_copy_scroll_down(wme, 1 as u_int);
        if scroll_only != 0 {
            if (*data).cy == (*s).grid().sy.wrapping_sub(1 as u_int) {
                window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
            } else {
                window_copy_redraw_lines(wme, (*data).cy, 2 as u_int);
            }
        }
    } else {
        if norectsel != 0 {
            window_copy_update_cursor(wme, (*data).lastcx, (*data).cy.wrapping_sub(1 as u_int));
        } else {
            window_copy_update_cursor(wme, (*data).cx, (*data).cy.wrapping_sub(1 as u_int));
        }
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0
        {
            if (*data).cy == (*s).grid().sy.wrapping_sub(1 as u_int) {
                window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
            } else {
                window_copy_redraw_lines(wme, (*data).cy, 2 as u_int);
            }
        }
    }
    if norectsel != 0 {
        py = (*data).backing().grid()
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        px = window_copy_find_length(wme, py);
        if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
            window_copy_update_cursor(wme, px, (*data).cy);
            if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int)
                != 0
            {
                window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
            }
        }
    }
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        py = (*data).backing().grid()
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        if (*data).rectflag != 0 {
            px = (*data).backing().grid().sx;
        } else {
            px = window_copy_find_length(wme, py);
        }
        window_copy_update_cursor(wme, px, (*data).cy);
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0
        {
            window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
        }
    } else if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_copy_update_cursor(wme, 0 as u_int, (*data).cy);
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0
        {
            window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
        }
    }
}
unsafe fn window_copy_cursor_down(
    mut wme: *mut window_mode_entry,
    mut scroll_only: ::core::ffi::c_int,
) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = options_owner_ptr(&mut (*(*mode_pane).window).options).map_or(std::ptr::null_mut(), |options| options);
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut norectsel: ::core::ffi::c_int = 0;
    norectsel = ((*data).screen.sel.is_none() || (*data).rectflag == 0) as ::core::ffi::c_int;
    oy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    ox = window_copy_find_length(wme, oy);
    if norectsel != 0 && (*data).cx != ox {
        (*data).lastcx = (*data).cx;
        (*data).lastsx = ox;
    }
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        && oy == (*data).endsely
    {
        window_copy_other_end(wme);
    }
    if scroll_only != 0
        && options_get_number(
            oo,
            b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
        ) == MODEKEY_VI as ::core::ffi::c_longlong
    {
        if (*data).cy > 0 as u_int {
            window_copy_update_cursor(wme, (*data).cx, (*data).cy.wrapping_sub(1 as u_int));
        }
    }
    if scroll_only != 0 || (*data).cy == (*s).grid().sy.wrapping_sub(1 as u_int) {
        if norectsel != 0 {
            (*data).cx = (*data).lastcx;
        }
        window_copy_scroll_up(wme, 1 as u_int);
        if scroll_only != 0 && (*data).cy > 0 as u_int {
            window_copy_redraw_lines(wme, (*data).cy.wrapping_sub(1 as u_int), 2 as u_int);
        }
    } else {
        if norectsel != 0 {
            window_copy_update_cursor(wme, (*data).lastcx, (*data).cy.wrapping_add(1 as u_int));
        } else {
            window_copy_update_cursor(wme, (*data).cx, (*data).cy.wrapping_add(1 as u_int));
        }
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0
        {
            window_copy_redraw_lines(wme, (*data).cy.wrapping_sub(1 as u_int), 2 as u_int);
        }
    }
    if norectsel != 0 {
        py = (*data).backing().grid()
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        px = window_copy_find_length(wme, py);
        if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
            window_copy_update_cursor(wme, px, (*data).cy);
            if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int)
                != 0
            {
                window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
            }
        }
    }
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        py = (*data).backing().grid()
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        if (*data).rectflag != 0 {
            px = (*data).backing().grid().sx;
        } else {
            px = window_copy_find_length(wme, py);
        }
        window_copy_update_cursor(wme, px, (*data).cy);
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0
        {
            window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
        }
    } else if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_copy_update_cursor(wme, 0 as u_int, (*data).cy);
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0
        {
            window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
        }
    }
}
unsafe fn window_copy_cursor_jump(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx.wrapping_add(1 as u_int);
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    if grid_reader_cursor_jump(&mut gr, &(&(*data).jumpchar)[0]) != 0 {
        (px, py) = grid_reader_get_cursor(&gr);
        window_copy_acquire_cursor_down(
            wme,
            hsize,
            (*back_s).grid().sy,
            (*data).oy,
            oldy,
            px,
            py,
            0 as ::core::ffi::c_int,
        );
    }
}
unsafe fn window_copy_cursor_jump_back(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    grid_reader_cursor_left(&mut gr, 0 as ::core::ffi::c_int);
    if grid_reader_cursor_jump_back(&mut gr, &(&(*data).jumpchar)[0]) != 0 {
        (px, py) = grid_reader_get_cursor(&gr);
        window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
    }
}
unsafe fn window_copy_cursor_jump_to(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx.wrapping_add(2 as u_int);
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    if grid_reader_cursor_jump(&mut gr, &(&(*data).jumpchar)[0]) != 0 {
        grid_reader_cursor_left(&mut gr, 1 as ::core::ffi::c_int);
        (px, py) = grid_reader_get_cursor(&gr);
        window_copy_acquire_cursor_down(
            wme,
            hsize,
            (*back_s).grid().sy,
            (*data).oy,
            oldy,
            px,
            py,
            0 as ::core::ffi::c_int,
        );
    }
}
unsafe fn window_copy_cursor_jump_to_back(mut wme: *mut window_mode_entry) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = options_owner_ptr(&mut (*(*mode_pane).window).options).map_or(std::ptr::null_mut(), |options| options);
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    let mut onemore: ::core::ffi::c_int = 0;
    px = (*data).cx;
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    onemore = (options_get_number(
        oo,
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) != MODEKEY_VI as ::core::ffi::c_longlong) as ::core::ffi::c_int;
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    grid_reader_cursor_left(&mut gr, 0 as ::core::ffi::c_int);
    grid_reader_cursor_left(&mut gr, 0 as ::core::ffi::c_int);
    if grid_reader_cursor_jump_back(&mut gr, &(&(*data).jumpchar)[0]) != 0 {
        grid_reader_cursor_right(
            &mut gr,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            onemore,
        );
        (px, py) = grid_reader_get_cursor(&gr);
        window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
    }
}
unsafe fn window_copy_cursor_next_word(
    mut wme: *mut window_mode_entry,
    mut separators: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    grid_reader_cursor_next_word(&mut gr, CStr::from_ptr(separators));
    (px, py) = grid_reader_get_cursor(&gr);
    window_copy_acquire_cursor_down(
        wme,
        hsize,
        (*back_s).grid().sy,
        (*data).oy,
        oldy,
        px,
        py,
        0 as ::core::ffi::c_int,
    );
}
unsafe fn window_copy_cursor_next_word_end_pos(
    mut wme: *mut window_mode_entry,
    mut separators: *const ::core::ffi::c_char,
    mut ppx: *mut u_int,
    mut ppy: *mut u_int,
) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options);
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    if options_get_number(
        oo,
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == MODEKEY_VI as ::core::ffi::c_longlong
    {
        if grid_reader_in_set(&gr, c"\t ") == 0 {
            grid_reader_cursor_right(
                &mut gr,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
        grid_reader_cursor_next_word_end(&mut gr, CStr::from_ptr(separators));
        grid_reader_cursor_left(&mut gr, 1 as ::core::ffi::c_int);
    } else {
        grid_reader_cursor_next_word_end(&mut gr, CStr::from_ptr(separators));
    }
    (px, py) = grid_reader_get_cursor(&gr);
    *ppx = px;
    *ppy = py;
}
unsafe fn window_copy_cursor_next_word_end(
    mut wme: *mut window_mode_entry,
    mut separators: *const ::core::ffi::c_char,
    mut no_reset: ::core::ffi::c_int,
) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options);
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    if options_get_number(
        oo,
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == MODEKEY_VI as ::core::ffi::c_longlong
    {
        if grid_reader_in_set(&gr, c"\t ") == 0 {
            grid_reader_cursor_right(
                &mut gr,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
        grid_reader_cursor_next_word_end(&mut gr, CStr::from_ptr(separators));
        grid_reader_cursor_left(&mut gr, 1 as ::core::ffi::c_int);
    } else {
        grid_reader_cursor_next_word_end(&mut gr, CStr::from_ptr(separators));
    }
    (px, py) = grid_reader_get_cursor(&gr);
    window_copy_acquire_cursor_down(
        wme,
        hsize,
        (*back_s).grid().sy,
        (*data).oy,
        oldy,
        px,
        py,
        no_reset,
    );
}
unsafe fn window_copy_cursor_previous_word_pos(
    mut wme: *mut window_mode_entry,
    mut separators: *const ::core::ffi::c_char,
    mut ppx: *mut u_int,
    mut ppy: *mut u_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    grid_reader_cursor_previous_word(
        &mut gr,
        CStr::from_ptr(separators),
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    (px, py) = grid_reader_get_cursor(&gr);
    *ppx = px;
    *ppy = py;
}
unsafe fn window_copy_cursor_previous_word(
    mut wme: *mut window_mode_entry,
    mut separators: *const ::core::ffi::c_char,
    mut already: ::core::ffi::c_int,
) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut w: *mut window = (*mode_pane).window as *mut window;
    let back_s = (*data).backing();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    let mut stop_at_eol: ::core::ffi::c_int = 0;
    if options_get_number(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == MODEKEY_EMACS as ::core::ffi::c_longlong
    {
        stop_at_eol = 1 as ::core::ffi::c_int;
    } else {
        stop_at_eol = 0 as ::core::ffi::c_int;
    }
    px = (*data).cx;
    hsize = (*back_s).grid().hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    let mut gr = grid_reader_start((*back_s).grid(), px, py);
    grid_reader_cursor_previous_word(&mut gr, CStr::from_ptr(separators), already, stop_at_eol);
    (px, py) = grid_reader_get_cursor(&gr);
    window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
}
unsafe fn window_copy_cursor_prompt(
    mut wme: *mut window_mode_entry,
    mut direction: ::core::ffi::c_int,
    mut start_output: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = (*data).backing_mut();
    let mut gd: *mut grid = (*s).grid_mut();
    let mut end_line: u_int = 0;
    let mut line: u_int = (*gd)
        .hsize
        .wrapping_sub((*data).oy)
        .wrapping_add((*data).cy);
    let mut add: ::core::ffi::c_int = 0;
    let mut line_flag: ::core::ffi::c_int = 0;
    if start_output != 0 {
        line_flag = GRID_LINE_START_OUTPUT;
    } else {
        line_flag = GRID_LINE_START_PROMPT;
    }
    if direction == 0 as ::core::ffi::c_int {
        add = -(1 as ::core::ffi::c_int);
        end_line = 0 as u_int;
    } else {
        add = 1 as ::core::ffi::c_int;
        end_line = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    }
    if line == end_line {
        return;
    }
    loop {
        if line == end_line {
            return;
        }
        line = line.wrapping_add(add as u_int);
        if (*grid_get_line(&*gd, line)).flags as ::core::ffi::c_int & line_flag != 0 {
            break;
        }
    }
    (*data).cx = 0 as u_int;
    if line > (*gd).hsize {
        (*data).cy = line.wrapping_sub((*gd).hsize);
        (*data).oy = 0 as u_int;
    } else {
        (*data).cy = 0 as u_int;
        (*data).oy = (*gd).hsize.wrapping_sub(line);
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    window_copy_redraw_screen(wme);
}
unsafe fn window_copy_scroll_up(mut wme: *mut window_mode_entry, mut ny: u_int) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    if (*data).oy < ny {
        ny = (*data).oy;
    }
    if ny == 0 as u_int {
        return;
    }
    (*data).oy = (*data).oy.wrapping_sub(ny);
    window_pane_scrollbar_show(&mode_pane_owner);
    if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection_view(wme, 0 as ::core::ffi::c_int);
    if window_copy_cursor_line_active(wme) != 0 {
        window_copy_redraw_screen(wme);
        return;
    }
    if window_copy_line_numbers_active(wme) != 0 {
        if window_copy_line_number_mode(wme)
            != WINDOW_COPY_LINE_NUMBERS_ABSOLUTE as ::core::ffi::c_int
        {
            window_copy_redraw_screen(wme);
            return;
        }
        screen_write_start(&mut ctx, &raw mut (*data).screen);
        screen_write_cursormove(
            &mut ctx,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_deleteline(&mut ctx, ny, 8 as u_int);
        window_copy_write_lines(wme, &raw mut ctx, (*s).grid().sy.wrapping_sub(ny), ny);
        window_copy_write_line(wme, &raw mut ctx, 0 as u_int);
        if (*s).grid().sy > 1 as u_int {
            window_copy_write_line(wme, &raw mut ctx, 1 as u_int);
        }
        if (*s).grid().sy > 3 as u_int {
            window_copy_write_line(wme, &raw mut ctx, (*s).grid().sy.wrapping_sub(2 as u_int));
        }
        if !(*s).sel.is_none() && (*s).grid().sy > ny {
            window_copy_write_line(
                wme,
                &raw mut ctx,
                (*s).grid().sy.wrapping_sub(ny).wrapping_sub(1 as u_int),
            );
        }
        screen_write_cursormove(
            &mut ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*s).grid().sx) as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&mut ctx);
        (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR;
        return;
    }
    if window_pane_scrollbar_overlay_visible(&*wp) != 0 {
        screen_write_start(&mut ctx, &raw mut (*data).screen);
    } else {
        screen_write_start_pane(&mut ctx, &(*wp).observer.upgrade().expect("live screen-write pane"), ::core::ptr::null_mut::<screen>());
    }
    screen_write_cursormove(
        &mut ctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_deleteline(&mut ctx, ny, 8 as u_int);
    window_copy_write_lines(wme, &raw mut ctx, (*s).grid().sy.wrapping_sub(ny), ny);
    window_copy_write_line(wme, &raw mut ctx, 0 as u_int);
    if (*s).grid().sy > 1 as u_int {
        window_copy_write_line(wme, &raw mut ctx, 1 as u_int);
    }
    if (*s).grid().sy > 3 as u_int {
        window_copy_write_line(wme, &raw mut ctx, (*s).grid().sy.wrapping_sub(2 as u_int));
    }
    if !(*s).sel.is_none() && (*s).grid().sy > ny {
        window_copy_write_line(
            wme,
            &raw mut ctx,
            (*s).grid().sy.wrapping_sub(ny).wrapping_sub(1 as u_int),
        );
    }
    screen_write_cursormove(
        &mut ctx,
        window_copy_cursor_offset(wme, (*data).cx, (*s).grid().sx) as ::core::ffi::c_int,
        (*data).cy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&mut ctx);
    window_pane_scrollbar_redraw(&mode_pane_owner);
}
unsafe fn window_copy_scroll_down(mut wme: *mut window_mode_entry, mut ny: u_int) {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    if ny > (*data).backing().grid().hsize {
        return;
    }
    if (*data).oy > (*data).backing().grid().hsize.wrapping_sub(ny) {
        ny = (*data).backing().grid().hsize.wrapping_sub((*data).oy);
    }
    if ny == 0 as u_int {
        return;
    }
    (*data).oy = (*data).oy.wrapping_add(ny);
    window_pane_scrollbar_show(&mode_pane_owner);
    if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection_view(wme, 0 as ::core::ffi::c_int);
    if window_copy_cursor_line_active(wme) != 0 {
        window_copy_redraw_screen(wme);
        return;
    }
    if window_copy_line_numbers_active(wme) != 0 {
        if window_copy_line_number_mode(wme)
            != WINDOW_COPY_LINE_NUMBERS_ABSOLUTE as ::core::ffi::c_int
        {
            window_copy_redraw_screen(wme);
            return;
        }
        screen_write_start(&mut ctx, &raw mut (*data).screen);
        screen_write_cursormove(
            &mut ctx,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_insertline(&mut ctx, ny, 8 as u_int);
        window_copy_write_lines(wme, &raw mut ctx, 0 as u_int, ny);
        if !(*s).sel.is_none() && (*s).grid().sy > ny {
            window_copy_write_line(wme, &raw mut ctx, ny);
        } else if ny == 1 as u_int {
            window_copy_write_line(wme, &raw mut ctx, 1 as u_int);
        }
        screen_write_cursormove(
            &mut ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*s).grid().sx) as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&mut ctx);
        (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR;
        return;
    }
    if window_pane_scrollbar_overlay_visible(&*wp) != 0 {
        screen_write_start(&mut ctx, &raw mut (*data).screen);
    } else {
        screen_write_start_pane(&mut ctx, &(*wp).observer.upgrade().expect("live screen-write pane"), ::core::ptr::null_mut::<screen>());
    }
    screen_write_cursormove(
        &mut ctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_insertline(&mut ctx, ny, 8 as u_int);
    window_copy_write_lines(wme, &raw mut ctx, 0 as u_int, ny);
    if !(*s).sel.is_none() && (*s).grid().sy > ny {
        window_copy_write_line(wme, &raw mut ctx, ny);
    } else if ny == 1 as u_int {
        window_copy_write_line(wme, &raw mut ctx, 1 as u_int);
    }
    screen_write_cursormove(
        &mut ctx,
        window_copy_cursor_offset(wme, (*data).cx, (*s).grid().sx) as ::core::ffi::c_int,
        (*data).cy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&mut ctx);
    window_pane_scrollbar_redraw(&mode_pane_owner);
}
unsafe fn window_copy_rectangle_set(
    mut wme: *mut window_mode_entry,
    mut rectflag: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    (*data).rectflag = rectflag;
    py = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    px = window_copy_cursor_limit(wme, py, (*data).rectflag);
    if (*data).cx > px {
        window_copy_update_cursor(wme, px, (*data).cy);
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    window_copy_redraw_screen(wme);
}
unsafe fn window_copy_move_mouse(mut m: *mut mouse_event) {
    let mouse_pane_owner;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
    mouse_pane_owner = cmd_mouse_pane(
        m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    wp = mouse_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() {
        return;
    }
    wme = (*wp).modes.active;
    if wme.is_null() {
        return;
    }
    if !std::ptr::eq((*wme).mode, &window_copy_mode) && !std::ptr::eq((*wme).mode, &window_view_mode) {
        return;
    }
    if cmd_mouse_at(wp, m, &raw mut x, &raw mut y, 0 as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        return;
    }
    data = (*wme).data as *mut window_copy_mode_data;
    x = window_copy_cursor_unoffset(wme, x, (*data).screen.grid().sx);
    window_copy_update_cursor(wme, x, y);
}
unsafe fn window_copy_install_drag_callbacks(client_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) {
    let c = client_owner.get();
    let drag_client = std::rc::Rc::downgrade(client_owner);
    (*c).tty.mouse_drag_update = Some(Box::new(move |m| {
        if let Some(owner) = drag_client.upgrade() {
            unsafe { window_copy_drag_update(&owner, m) }
        }
    }));
    let drag_client = std::rc::Rc::downgrade(client_owner);
    (*c).tty.mouse_drag_release = Some(Box::new(move |m| {
        if let Some(owner) = drag_client.upgrade() {
            unsafe { window_copy_drag_release(&owner, m) }
        }
    }));
}

pub unsafe fn window_copy_start_drag(client_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>, mut m: *mut mouse_event) {
    let Some(client_owner) = client_owner else { return; };
    let mouse_pane_owner;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut yg: u_int = 0;
    let mut inside_selection: ::core::ffi::c_int = 0;
    let mut on_start: ::core::ffi::c_int = 0;
    let mut on_end: ::core::ffi::c_int = 0;
    mouse_pane_owner = cmd_mouse_pane(
        m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    wp = mouse_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() {
        return;
    }
    wme = (*wp).modes.active;
    if wme.is_null() {
        return;
    }
    if !std::ptr::eq((*wme).mode, &window_copy_mode) && !std::ptr::eq((*wme).mode, &window_view_mode) {
        return;
    }
    if cmd_mouse_at(wp, m, &raw mut x, &raw mut y, 1 as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        return;
    }
    window_copy_install_drag_callbacks(client_owner);
    data = (*wme).data as *mut window_copy_mode_data;
    on_end = 0 as ::core::ffi::c_int;
    on_start = on_end;
    inside_selection =
        window_copy_mouse_in_selection(wme, x, y, &raw mut on_start, &raw mut on_end);
    x = window_copy_cursor_unoffset(wme, x, (*data).screen.grid().sx);
    yg = (*data).backing().grid()
        .hsize
        .wrapping_add(y)
        .wrapping_sub((*data).oy);
    if on_start != 0
        || on_end != 0
        || inside_selection == 0
        || x < (*data).selrx
        || x > (*data).endselrx
        || yg != (*data).selry
    {
        (*data).lineflag = LINE_SEL_NONE;
        (*data).selflag = SEL_CHAR;
    }
    match (*data).selflag as ::core::ffi::c_uint {
        1 => {
            if (*data).separators.is_some() {
                window_copy_update_cursor(wme, x, y);
                window_copy_cursor_previous_word_pos(
                    wme,
                    window_copy_separators(&*data),
                    &raw mut x,
                    &raw mut y,
                );
                y = y.wrapping_sub((*data).backing().grid().hsize.wrapping_sub((*data).oy));
            }
            window_copy_update_cursor(wme, x, y);
        }
        2 => {
            window_copy_update_cursor(wme, 0 as u_int, y);
        }
        0 => {
            window_copy_update_cursor(wme, x, y);
            if inside_selection == 0 {
                window_copy_start_selection(wme);
            } else {
                if on_start != 0 {
                    (*data).cursordrag = CURSORDRAG_SEL;
                } else if on_end != 0 {
                    (*data).cursordrag = CURSORDRAG_ENDSEL;
                }
                window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            }
        }
        _ => {}
    }
    window_copy_redraw_screen(wme);
    window_copy_drag_update(client_owner, m);
}
unsafe fn window_copy_drag_update(_client_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>, mut m: *mut mouse_event) {
    let mouse_pane_owner;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut old_cx: u_int = 0;
    let mut old_cy: u_int = 0;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: WINDOW_COPY_DRAG_REPEAT_TIME as __suseconds_t,
    };
    mouse_pane_owner = cmd_mouse_pane(
        m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    wp = mouse_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() {
        return;
    }
    wme = (*wp).modes.active;
    if wme.is_null() {
        return;
    }
    if !std::ptr::eq((*wme).mode, &window_copy_mode) && !std::ptr::eq((*wme).mode, &window_view_mode) {
        return;
    }
    data = (*wme).data as *mut window_copy_mode_data;
    event_del(&raw mut (*data).dragtimer);
    if cmd_mouse_at(wp, m, &raw mut x, &raw mut y, 0 as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        return;
    }
    x = window_copy_cursor_unoffset(wme, x, (*data).screen.grid().sx);
    old_cx = (*data).cx;
    old_cy = (*data).cy;
    window_copy_update_cursor(wme, x, y);
    if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
        window_copy_redraw_selection(wme, old_cy);
    }
    if old_cy != (*data).cy || old_cx == (*data).cx {
        if y == 0 as u_int {
            event_add(&raw mut (*data).dragtimer, &raw mut tv);
            window_copy_cursor_up(wme, 1 as ::core::ffi::c_int);
        } else if y == (*data).screen.grid().sy.wrapping_sub(1 as u_int) {
            event_add(&raw mut (*data).dragtimer, &raw mut tv);
            window_copy_cursor_down(wme, 1 as ::core::ffi::c_int);
        }
    }
}
unsafe fn window_copy_drag_release(client_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>, mut m: *mut mouse_event) {
    let mouse_pane_owner;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
    mouse_pane_owner = cmd_mouse_pane(
        m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    wp = mouse_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() {
        return;
    }
    wme = (*wp).modes.active;
    if wme.is_null() {
        return;
    }
    if !std::ptr::eq((*wme).mode, &window_copy_mode) && !std::ptr::eq((*wme).mode, &window_view_mode) {
        return;
    }
    data = (*wme).data as *mut window_copy_mode_data;
    if window_copy_line_numbers_active(wme) != 0 {
        window_copy_drag_update(client_owner, m);
    }
    (*data).cursordrag = CURSORDRAG_NONE;
    event_del(&raw mut (*data).dragtimer);
}
unsafe fn window_copy_jump_to_mark(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut tmx: u_int = 0;
    let mut tmy: u_int = 0;
    tmx = (*data).cx;
    tmy = (*data).backing().grid()
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).cx = (*data).mx;
    if (*data).my < (*data).backing().grid().hsize {
        (*data).cy = 0 as u_int;
        (*data).oy = (*data).backing().grid().hsize.wrapping_sub((*data).my);
    } else {
        (*data).cy = (*data).my.wrapping_sub((*data).backing().grid().hsize);
        (*data).oy = 0 as u_int;
    }
    (*data).mx = tmx;
    (*data).my = tmy;
    (*data).showmark = 1 as ::core::ffi::c_int;
    window_copy_update_selection(wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    window_copy_redraw_screen(wme);
}
unsafe fn window_copy_acquire_cursor_up(
    mut wme: *mut window_mode_entry,
    mut hsize: u_int,
    mut oy: u_int,
    mut oldy: u_int,
    mut px: u_int,
    mut py: u_int,
) {
    let mut cy: u_int = 0;
    let mut yy: u_int = 0;
    let mut ny: u_int = 0;
    let mut nd: u_int = 0;
    yy = hsize.wrapping_sub(oy);
    if py < yy {
        ny = yy.wrapping_sub(py);
        cy = 0 as u_int;
        nd = 1 as u_int;
    } else {
        ny = 0 as u_int;
        cy = py.wrapping_sub(yy);
        nd = oldy.wrapping_sub(cy).wrapping_add(1 as u_int);
    }
    while ny > 0 as u_int {
        window_copy_cursor_up(wme, 1 as ::core::ffi::c_int);
        ny = ny.wrapping_sub(1);
    }
    window_copy_update_cursor(wme, px, cy);
    if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
        window_copy_redraw_lines(wme, cy, nd);
    }
}
unsafe fn window_copy_acquire_cursor_down(
    mut wme: *mut window_mode_entry,
    mut hsize: u_int,
    mut sy: u_int,
    mut oy: u_int,
    mut oldy: u_int,
    mut px: u_int,
    mut py: u_int,
    mut no_reset: ::core::ffi::c_int,
) {
    let mut cy: u_int = 0;
    let mut yy: u_int = 0;
    let mut ny: u_int = 0;
    let mut nd: u_int = 0;
    cy = py.wrapping_sub(hsize).wrapping_add(oy);
    yy = sy.wrapping_sub(1 as u_int);
    if cy > yy {
        ny = cy.wrapping_sub(yy);
        oldy = yy;
        nd = 1 as u_int;
    } else {
        ny = 0 as u_int;
        nd = cy.wrapping_sub(oldy).wrapping_add(1 as u_int);
    }
    while ny > 0 as u_int {
        window_copy_cursor_down(wme, 1 as ::core::ffi::c_int);
        ny = ny.wrapping_sub(1);
    }
    if cy > yy {
        window_copy_update_cursor(wme, px, yy);
    } else {
        window_copy_update_cursor(wme, px, cy);
    }
    if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, no_reset) != 0 {
        window_copy_redraw_lines(wme, oldy, nd);
    }
}

#[cfg(test)]
mod drag_client_tests {
    use super::*;

    #[test]
    fn detached_drag_callbacks_do_not_keep_client_alive() {
        unsafe {
            let owner = client::new();
            let weak = std::rc::Rc::downgrade(&owner);
            window_copy_install_drag_callbacks(&owner);
            assert_eq!(std::rc::Rc::strong_count(&owner), 1);
            let mut update = (*owner.get()).tty.mouse_drag_update.take().unwrap();
            let release = (*owner.get()).tty.mouse_drag_release.take().unwrap();
            drop(owner);
            assert!(weak.upgrade().is_none());

            let mut mouse = mouse_event::default();
            update(&mut mouse);
            update(&mut mouse);
            release(&mut mouse);
            assert!(weak.upgrade().is_none());
        }
    }
}

#[cfg(test)]
mod regex_cell_tests {
    use super::*;

    #[test]
    fn suffix_mapping_preserves_padding_and_partial_cells() {
        let cells = [
            Cow::Borrowed(&b"a"[..]),
            Cow::Borrowed(&b""[..]),
            Cow::Borrowed(&b"bc"[..]),
        ];
        for (text, expected) in [
            (&b"abc"[..], 0),
            (&b"bc"[..], 1),
            (&b"b"[..], 1),
            (&b"c"[..], 3),
            (&b""[..], 3),
            (&b"abcd"[..], 0),
        ] {
            assert_eq!(window_copy_find_cell_suffix(&cells, text), expected);
        }
        let wide = [Cow::Borrowed("漢".as_bytes()), Cow::Borrowed(&b""[..])];
        // An exhausted suffix rejects trailing padding, as in tmux.
        assert_eq!(window_copy_find_cell_suffix(&wide, "漢".as_bytes()), 1);
        assert_eq!(window_copy_find_cell_suffix(&wide[..1], &[0xe6]), 0);
        assert_eq!(window_copy_find_cell_suffix(&wide[..1], &[0xff]), 1);
    }

    #[test]
    fn cell_mapping_bounds_lines_and_stops_at_nul() {
        let mut gd = grid {
            sx: 3,
            sy: 2,
            ..grid::default()
        };
        gd.linedata.resize_with(2, grid_line::default);
        for (line, bytes) in gd.linedata.iter_mut().zip([b"abc", b"def"]) {
            line.cellsize = 3;
            line.celldata.resize_with(3, grid_cell_entry::default);
            for (cell, &byte) in line.celldata.iter_mut().zip(bytes) {
                cell.c2rust_unnamed = crate::src::shared::grid::grid_cell_entry_storage {
                    data: crate::src::shared::grid::grid_cell_entry_data {
                        attr: 0,
                        fg: 0,
                        bg: 0,
                        data: byte,
                    },
                };
            }
        }
        unsafe {
            let (mut x, mut y) = (1, 0);
            window_copy_cstrtocellpos(&gd, 5, &mut x, &mut y, b"def\0ignored");
            assert_eq!((x, y), (0, 1));
            window_copy_cstrtocellpos(&gd, 3, &mut x, &mut y, b"");
            assert_eq!((x, y), (0, 2));
            window_copy_cstrtocellpos(&gd, 3, &mut x, &mut y, b"a");
            assert_eq!((x, y), (0, 2));
        }
    }
}

#[cfg(test)]
mod backing_owner_tests {
    use super::*;
    use crate::src::grid::grid_set_cell;
    use crate::src::options::{options_create, options_default, options_free};
    use crate::src::options_table::options_table;

    struct ScreenOptions(*mut options);

    impl ScreenOptions {
        unsafe fn new() -> Self {
            let previous = global_options;
            global_options = options_create(std::ptr::null_mut());
            let entry = options_table
                .iter()
                .find(|entry| {
                    entry.name == Some(c"extended-keys")
                })
                .unwrap();
            options_default(global_options, entry);
            Self(previous)
        }
    }

    impl Drop for ScreenOptions {
        fn drop(&mut self) {
            unsafe {
                options_free(global_options);
                global_options = self.0;
            }
        }
    }

    unsafe fn byte_at(s: &screen, x: u_int, y: u_int) -> u8 {
        let mut cell = grid_default_cell;
        grid_get_cell(s.grid(), x, y, &mut cell);
        cell.data.data[0]
    }

    #[test]
    fn reflowed_snapshot_owns_cells_independently_of_the_source() {
        unsafe {
            let _options = ScreenOptions::new();
            let mut source = screen::empty();
            let mut hint = screen::empty();
            screen_init(&mut source, 8, 3, 10);
            screen_init(&mut hint, 4, 3, 0);
            for (x, byte) in b"ABCDEFGH".iter().enumerate() {
                let mut cell = grid_default_cell;
                cell.data.data[0] = *byte;
                grid_set_cell(source.grid_mut(), x as u_int, 0, &cell);
            }
            source.cx = 6;
            let (mut cx, mut cy) = (0, 0);
            let mut data = window_copy_mode_data::default();
            data.backing = Some(window_copy_clone_screen(
                &source,
                &hint,
                Some((&mut cx, &mut cy)),
                false,
            ));
            assert_eq!((cx, cy), (2, 1));
            assert_eq!(byte_at(data.backing(), 0, 0), b'A');
            assert_eq!(byte_at(data.backing(), 0, 1), b'E');
            assert_eq!((source.grid().sx, source.cx, source.cy), (8, 6, 0));
            assert_eq!(byte_at(&source, 4, 0), b'E');

            // The borrowed source can be freed before its independent snapshot.
            screen_free(&mut source);
            assert_eq!(byte_at(data.backing(), 3, 1), b'H');
            screen_free(&mut hint);
            drop(data);
        }
    }

    #[test]
    fn incremental_sync_reuses_backing_and_replacement_preserves_the_source() {
        unsafe {
            let _options = ScreenOptions::new();
            let pane_owner = window_pane::new();
            let pane = &mut *pane_owner.get();
            screen_init(&mut pane.base, 8, 3, 10);
            let mut data = window_copy_mode_data::default();
            data.backing = Some(window_copy_clone_screen(
                &pane.base, &pane.base, None, false,
            ));
            window_copy_sync_snapshot(&mut data, pane.base.grid_mut());
            let original = data.backing() as *const screen;
            let mut mode = window_mode_entry {
                wp: std::rc::Rc::downgrade(&pane_owner),
                swp: std::rc::Rc::downgrade(&pane_owner),
                mode: &window_copy_mode,
                data: (&mut data as *mut window_copy_mode_data).cast(),
            data_owner: None,
                screen: std::ptr::null_mut(),
                prefix: 0,
                kill: 0,
            };
            assert_eq!(window_copy_get_current_offset(pane), None);
            pane.modes.active = &mut mode;
            let hsize = data.backing().grid().hsize;
            assert_eq!(window_copy_get_current_offset(pane), Some((hsize, hsize)));
            mode.mode = &window_view_mode;
            assert_eq!(window_copy_get_current_offset(pane), Some((hsize, hsize)));
            mode.mode = &crate::src::window_clock::window_clock_mode;
            assert_eq!(window_copy_get_current_offset(pane), None);
            mode.mode = &window_copy_mode;
            pane.modes.active = std::ptr::null_mut();
            let mut cell = grid_default_cell;
            cell.data.data[0] = b'B';
            grid_set_cell(pane.base.grid_mut(), 0, 0, &cell);
            assert_eq!(byte_at(data.backing(), 0, 0), b' ');
            assert_eq!(window_copy_sync_backing(&mut mode), 1);
            assert_eq!(data.backing() as *const screen, original);
            assert_eq!(byte_at(data.backing(), 0, 0), b'B');

            pane.base.grid_mut().scroll_generation += 1;
            assert_eq!(window_copy_sync_backing(&mut mode), 0);
            data.clear_backing();
            assert!(data.backing.is_none());
            data.backing = Some(window_copy_clone_screen(&pane.base, &pane.base, None, true));
            assert_eq!(byte_at(data.backing(), 0, 0), b'B');
            assert_eq!(byte_at(&pane.base, 0, 0), b'B');
            drop(pane_owner);
            assert!(mode.swp.upgrade().is_none());
            // An expired source leaves the independently owned snapshot intact.
            assert_eq!(window_copy_sync_backing(&mut mode), 0);
            assert_eq!(window_copy_refresh_allowed(&mut mode), 0);
            window_copy_do_refresh(&mut mode, 0);
            assert_eq!(byte_at(data.backing(), 0, 0), b'B');
            drop(data);
            // An empty backing during initialization or repeated cleanup is valid.
            let mut empty = window_copy_mode_data::default();
            empty.clear_backing();
        }
    }
}
