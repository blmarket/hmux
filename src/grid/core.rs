use crate::src::colour::{colour_split_rgb, colour_theme_terminal_colour};
use crate::src::ffi::libc::{free, memcmp, memcpy, memmove, memset, strchr, strlcat, strlen};
use crate::src::hyperlinks::hyperlinks_get;
use crate::src::log::{fatalx, log_debug};
use crate::src::server::current_time;
use crate::src::shared::abi::*;
pub use crate::src::shared::colour::{COLOUR_FLAG_256, COLOUR_FLAG_RGB, COLOUR_FLAG_THEME};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::limits::{__INT_MAX__, UINT_MAX};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
use crate::src::tmux::start_time;
use crate::src::utf8::{
    utf8_build_one, utf8_cstrhas, utf8_from_data, utf8_has_whitespace, utf8_set, utf8_to_data,
};
use crate::src::xmalloc::{xcalloc, xmalloc, xreallocarray, xsnprintf};
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_0;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {
    pub mask: u_int,
    pub code: u_int,
}

#[no_mangle]
pub static mut grid_default_cell: grid_cell = grid_cell {
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
    us: 8 as ::core::ffi::c_int,
    link: 0 as u_int,
};
static mut grid_padding_cell: grid_cell = grid_cell {
    data: utf8_data {
        data: [
            '!' as i32 as u_char,
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
        size: 0 as u_char,
        width: 0 as u_char,
    },
    attr: 0 as u_short,
    flags: GRID_FLAG_PADDING as u_char,
    fg: 8 as ::core::ffi::c_int,
    bg: 8 as ::core::ffi::c_int,
    us: 8 as ::core::ffi::c_int,
    link: 0 as u_int,
};
static mut grid_cleared_cell: grid_cell = grid_cell {
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
    flags: GRID_FLAG_CLEARED as u_char,
    fg: 8 as ::core::ffi::c_int,
    bg: 8 as ::core::ffi::c_int,
    us: 8 as ::core::ffi::c_int,
    link: 0 as u_int,
};
static mut grid_cleared_entry: grid_cell_entry = grid_cell_entry {
    c2rust_unnamed: grid_cell_entry_storage {
        data: grid_cell_entry_data {
            attr: 0 as u_char,
            fg: 8 as u_char,
            bg: 8 as u_char,
            data: ' ' as i32 as u_char,
        },
    },
    flags: GRID_FLAG_CLEARED as u_char,
};
#[no_mangle]
pub unsafe extern "C" fn grid_check_is_clear(mut gd: *mut grid) {}
unsafe extern "C" fn grid_store_cell(
    mut gce: *mut grid_cell_entry,
    mut gc: *const grid_cell,
    mut c: u_char,
) {
    (*gce).flags = ((*gc).flags as ::core::ffi::c_int & !GRID_FLAG_CLEARED) as u_char;
    (*gce).c2rust_unnamed.data.fg = ((*gc).fg & 0xff as ::core::ffi::c_int) as u_char;
    if (*gc).fg & COLOUR_FLAG_256 != 0 {
        (*gce).flags = ((*gce).flags as ::core::ffi::c_int | GRID_FLAG_FG256) as u_char;
    }
    (*gce).c2rust_unnamed.data.bg = ((*gc).bg & 0xff as ::core::ffi::c_int) as u_char;
    if (*gc).bg & COLOUR_FLAG_256 != 0 {
        (*gce).flags = ((*gce).flags as ::core::ffi::c_int | GRID_FLAG_BG256) as u_char;
    }
    (*gce).c2rust_unnamed.data.attr = (*gc).attr as u_char;
    (*gce).c2rust_unnamed.data.data = c;
}
unsafe extern "C" fn grid_need_extended_cell(
    mut gce: *const grid_cell_entry,
    mut gc: *const grid_cell,
) -> ::core::ffi::c_int {
    if (*gce).flags as ::core::ffi::c_int & GRID_FLAG_EXTENDED != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if (*gc).attr as ::core::ffi::c_int > 0xff as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    if (*gc).data.size as ::core::ffi::c_int > 1 as ::core::ffi::c_int
        || (*gc).data.width as ::core::ffi::c_int > 1 as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*gc).fg & (COLOUR_FLAG_RGB | COLOUR_FLAG_THEME) != 0
        || (*gc).bg & (COLOUR_FLAG_RGB | COLOUR_FLAG_THEME) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*gc).us != 8 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    if (*gc).link != 0 as u_int {
        return 1 as ::core::ffi::c_int;
    }
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn grid_get_extended_cell(
    mut gl: *mut grid_line,
    mut gce: *mut grid_cell_entry,
    mut flags: ::core::ffi::c_int,
) {
    let mut at: u_int = (*gl).extdsize.wrapping_add(1 as u_int);
    (*gl).extddata = xreallocarray(
        (*gl).extddata as *mut ::core::ffi::c_void,
        at as size_t,
        ::core::mem::size_of::<grid_extd_entry>() as size_t,
    ) as *mut grid_extd_entry;
    (*gl).extdsize = at;
    (*gce).c2rust_unnamed.offset = at.wrapping_sub(1 as u_int);
    (*gce).flags = (flags | GRID_FLAG_EXTENDED) as u_char;
}
unsafe extern "C" fn grid_extended_cell(
    mut gl: *mut grid_line,
    mut gce: *mut grid_cell_entry,
    mut gc: *const grid_cell,
) -> *mut grid_extd_entry {
    let mut gee: *mut grid_extd_entry = ::core::ptr::null_mut::<grid_extd_entry>();
    let mut flags: ::core::ffi::c_int = (*gc).flags as ::core::ffi::c_int & !GRID_FLAG_CLEARED;
    let mut uc: utf8_char = 0;
    if !((*gce).flags as ::core::ffi::c_int) & GRID_FLAG_EXTENDED != 0 {
        grid_get_extended_cell(gl, gce, flags);
    } else if (*gce).c2rust_unnamed.offset >= (*gl).extdsize {
        fatalx(b"offset too big\0" as *const u8 as *const ::core::ffi::c_char);
    }
    (*gl).flags = ((*gl).flags as ::core::ffi::c_int | GRID_LINE_EXTENDED) as u_short;
    if (*gc).link != 0 as u_int {
        (*gl).flags = ((*gl).flags as ::core::ffi::c_int | GRID_LINE_HYPERLINK) as u_short;
    }
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        uc = (*gc).data.width as utf8_char;
    } else {
        utf8_from_data(&raw const (*gc).data, &raw mut uc);
    }
    gee = (*gl).extddata.offset((*gce).c2rust_unnamed.offset as isize) as *mut grid_extd_entry;
    (*gee).data = uc;
    (*gee).attr = (*gc).attr;
    (*gee).flags = flags as u_char;
    (*gee).fg = (*gc).fg;
    (*gee).bg = (*gc).bg;
    (*gee).us = (*gc).us;
    (*gee).link = (*gc).link;
    return gee;
}
unsafe extern "C" fn grid_compact_line(mut gl: *mut grid_line) {
    let mut new_extdsize: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut new_extddata: *mut grid_extd_entry = ::core::ptr::null_mut::<grid_extd_entry>();
    let mut gce: *mut grid_cell_entry = ::core::ptr::null_mut::<grid_cell_entry>();
    let mut gee: *mut grid_extd_entry = ::core::ptr::null_mut::<grid_extd_entry>();
    let mut px: u_int = 0;
    let mut idx: u_int = 0;
    if (*gl).extdsize == 0 as u_int {
        return;
    }
    px = 0 as u_int;
    while px < (*gl).cellsize as u_int {
        gce = (*gl).celldata.offset(px as isize) as *mut grid_cell_entry;
        if (*gce).flags as ::core::ffi::c_int & GRID_FLAG_EXTENDED != 0 {
            new_extdsize += 1;
        }
        px = px.wrapping_add(1);
    }
    if new_extdsize == 0 as ::core::ffi::c_int {
        free((*gl).extddata as *mut ::core::ffi::c_void);
        (*gl).extddata = ::core::ptr::null_mut::<grid_extd_entry>();
        (*gl).extdsize = 0 as u_int;
        return;
    }
    new_extddata = xreallocarray(
        NULL,
        new_extdsize as size_t,
        ::core::mem::size_of::<grid_extd_entry>() as size_t,
    ) as *mut grid_extd_entry;
    idx = 0 as u_int;
    px = 0 as u_int;
    while px < (*gl).cellsize as u_int {
        gce = (*gl).celldata.offset(px as isize) as *mut grid_cell_entry;
        if (*gce).flags as ::core::ffi::c_int & GRID_FLAG_EXTENDED != 0 {
            gee = (*gl).extddata.offset((*gce).c2rust_unnamed.offset as isize)
                as *mut grid_extd_entry;
            memcpy(
                new_extddata.offset(idx as isize) as *mut grid_extd_entry
                    as *mut ::core::ffi::c_void,
                gee as *const ::core::ffi::c_void,
                ::core::mem::size_of::<grid_extd_entry>() as size_t,
            );
            let fresh0 = idx;
            idx = idx.wrapping_add(1);
            (*gce).c2rust_unnamed.offset = fresh0;
        }
        px = px.wrapping_add(1);
    }
    free((*gl).extddata as *mut ::core::ffi::c_void);
    (*gl).extddata = new_extddata;
    (*gl).extdsize = new_extdsize as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn grid_get_line(mut gd: *mut grid, mut line: u_int) -> *mut grid_line {
    return (*gd).linedata.offset(line as isize) as *mut grid_line;
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_time(mut gl: *const grid_line) -> time_t {
    if (*gl).time == 0 as u_int {
        return 0 as time_t;
    }
    return start_time.tv_sec as time_t + (*gl).time as time_t - 1 as time_t;
}
unsafe extern "C" fn grid_line_set_time(mut gl: *mut grid_line) {
    if current_time == 0 as time_t {
        (*gl).time = 0 as u_int;
    } else {
        (*gl).time = (current_time as __time_t - start_time.tv_sec + 1 as __time_t) as u_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn grid_adjust_lines(mut gd: *mut grid, mut lines: u_int) {
    (*gd).linedata = xreallocarray(
        (*gd).linedata as *mut ::core::ffi::c_void,
        lines as size_t,
        ::core::mem::size_of::<grid_line>() as size_t,
    ) as *mut grid_line;
}
unsafe extern "C" fn grid_clear_cell(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut bg: u_int,
    mut moved: ::core::ffi::c_int,
) {
    let mut gl: *mut grid_line = (*gd).linedata.offset(py as isize) as *mut grid_line;
    let mut gce: *mut grid_cell_entry = (*gl).celldata.offset(px as isize) as *mut grid_cell_entry;
    let mut gee: *mut grid_extd_entry = ::core::ptr::null_mut::<grid_extd_entry>();
    let mut old_offset: u_int = (*gce).c2rust_unnamed.offset;
    let mut had_extd: ::core::ffi::c_int = (*gce).flags as ::core::ffi::c_int & GRID_FLAG_EXTENDED;
    memcpy(
        gce as *mut ::core::ffi::c_void,
        &raw const grid_cleared_entry as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell_entry>() as size_t,
    );
    if moved == 0 && had_extd != 0 && old_offset < (*gl).extdsize {
        (*gce).flags = ((*gce).flags as ::core::ffi::c_int | GRID_FLAG_EXTENDED) as u_char;
        (*gce).c2rust_unnamed.offset = old_offset;
        gee = grid_extended_cell(gl, gce, &raw const grid_cleared_cell);
        if bg != 8 as u_int {
            (*gee).bg = bg as ::core::ffi::c_int;
        }
    } else if bg != 8 as u_int {
        if bg & (COLOUR_FLAG_RGB | COLOUR_FLAG_THEME) as u_int != 0 {
            grid_get_extended_cell(gl, gce, (*gce).flags as ::core::ffi::c_int);
            gee = grid_extended_cell(gl, gce, &raw const grid_cleared_cell);
            (*gee).bg = bg as ::core::ffi::c_int;
        } else {
            if bg & COLOUR_FLAG_256 as u_int != 0 {
                (*gce).flags = ((*gce).flags as ::core::ffi::c_int | GRID_FLAG_BG256) as u_char;
            }
            (*gce).c2rust_unnamed.data.bg = bg as u_char;
        }
    }
}
unsafe extern "C" fn grid_check_y(
    mut gd: *mut grid,
    mut from: *const ::core::ffi::c_char,
    mut py: u_int,
) -> ::core::ffi::c_int {
    if py >= (*gd).hsize.wrapping_add((*gd).sy) {
        log_debug(
            b"%s: y out of range: %u\0" as *const u8 as *const ::core::ffi::c_char,
            from,
            py,
        );
        return -(1 as ::core::ffi::c_int);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn grid_cells_look_equal(
    mut gc1: *const grid_cell,
    mut gc2: *const grid_cell,
) -> ::core::ffi::c_int {
    let mut flags1: ::core::ffi::c_int = (*gc1).flags as ::core::ffi::c_int;
    let mut flags2: ::core::ffi::c_int = (*gc2).flags as ::core::ffi::c_int;
    if (*gc1).fg != (*gc2).fg || (*gc1).bg != (*gc2).bg {
        return 0 as ::core::ffi::c_int;
    }
    if (*gc1).attr as ::core::ffi::c_int != (*gc2).attr as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if flags1 & !GRID_FLAG_CLEARED != flags2 & !GRID_FLAG_CLEARED {
        return 0 as ::core::ffi::c_int;
    }
    if (*gc1).link != (*gc2).link {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn grid_cells_equal(
    mut gc1: *const grid_cell,
    mut gc2: *const grid_cell,
) -> ::core::ffi::c_int {
    if grid_cells_look_equal(gc1, gc2) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*gc1).data.width as ::core::ffi::c_int != (*gc2).data.width as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if (*gc1).data.size as ::core::ffi::c_int != (*gc2).data.size as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return (memcmp(
        &raw const (*gc1).data.data as *const u_char as *const ::core::ffi::c_void,
        &raw const (*gc2).data.data as *const u_char as *const ::core::ffi::c_void,
        (*gc1).data.size as size_t,
    ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn grid_set_tab(mut gc: *mut grid_cell, mut width: u_int) {
    memset(
        &raw mut (*gc).data.data as *mut u_char as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[u_char; 32]>() as size_t,
    );
    (*gc).flags = ((*gc).flags as ::core::ffi::c_int | GRID_FLAG_TAB) as u_char;
    (*gc).flags = ((*gc).flags as ::core::ffi::c_int & !GRID_FLAG_PADDING) as u_char;
    (*gc).data.have = width as u_char;
    (*gc).data.size = (*gc).data.have;
    (*gc).data.width = (*gc).data.size;
    memset(
        &raw mut (*gc).data.data as *mut u_char as *mut ::core::ffi::c_void,
        ' ' as i32,
        (*gc).data.size as size_t,
    );
}
unsafe extern "C" fn grid_free_line(mut gd: *mut grid, mut py: u_int) {
    let mut gl: *mut grid_line = (*gd).linedata.offset(py as isize) as *mut grid_line;
    free((*gl).celldata as *mut ::core::ffi::c_void);
    free((*gl).extddata as *mut ::core::ffi::c_void);
    memset(
        gl as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<grid_line>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn grid_free_lines(mut gd: *mut grid, mut py: u_int, mut ny: u_int) {
    let mut yy: u_int = 0;
    yy = py;
    while yy < py.wrapping_add(ny) {
        grid_free_line(gd, yy);
        yy = yy.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_create(mut sx: u_int, mut sy: u_int, mut hlimit: u_int) -> *mut grid {
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    gd = xcalloc(1 as size_t, ::core::mem::size_of::<grid>() as size_t) as *mut grid;
    (*gd).sx = sx;
    (*gd).sy = sy;
    if hlimit != 0 as u_int {
        (*gd).flags = GRID_HISTORY;
    }
    (*gd).hlimit = hlimit;
    if (*gd).sy != 0 as u_int {
        (*gd).linedata = xcalloc(
            (*gd).sy as size_t,
            ::core::mem::size_of::<grid_line>() as size_t,
        ) as *mut grid_line;
    }
    grid_check_is_clear(gd);
    return gd;
}
#[no_mangle]
pub unsafe extern "C" fn grid_destroy(mut gd: *mut grid) {
    grid_free_lines(gd, 0 as u_int, (*gd).hsize.wrapping_add((*gd).sy));
    free((*gd).linedata as *mut ::core::ffi::c_void);
    free(gd as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn grid_compare(mut ga: *mut grid, mut gb: *mut grid) -> ::core::ffi::c_int {
    let mut gla: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut glb: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut gca: grid_cell = grid_cell {
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
    let mut gcb: grid_cell = grid_cell {
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
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    if (*ga).sx != (*gb).sx || (*ga).sy != (*gb).sy {
        return 1 as ::core::ffi::c_int;
    }
    yy = 0 as u_int;
    while yy < (*ga).sy {
        gla = (*ga).linedata.offset(yy as isize) as *mut grid_line;
        glb = (*gb).linedata.offset(yy as isize) as *mut grid_line;
        if (*gla).cellsize as ::core::ffi::c_int != (*glb).cellsize as ::core::ffi::c_int {
            return 1 as ::core::ffi::c_int;
        }
        xx = 0 as u_int;
        while xx < (*gla).cellsize as u_int {
            grid_get_cell(ga, xx, yy, &raw mut gca);
            grid_get_cell(gb, xx, yy, &raw mut gcb);
            if grid_cells_equal(&raw mut gca, &raw mut gcb) == 0 {
                return 1 as ::core::ffi::c_int;
            }
            xx = xx.wrapping_add(1);
        }
        yy = yy.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn grid_trim_history(mut gd: *mut grid, mut ny: u_int) {
    let mut remaining: u_int = 0;
    grid_free_lines(gd, 0 as u_int, ny);
    remaining = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(ny);
    memmove(
        (*gd).linedata.offset(0 as ::core::ffi::c_int as isize) as *mut grid_line
            as *mut ::core::ffi::c_void,
        (*gd).linedata.offset(ny as isize) as *mut grid_line as *const ::core::ffi::c_void,
        (remaining as size_t).wrapping_mul(::core::mem::size_of::<grid_line>() as size_t),
    );
    memset(
        (*gd).linedata.offset(remaining as isize) as *mut grid_line as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (ny as size_t).wrapping_mul(::core::mem::size_of::<grid_line>() as size_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn grid_collect_history(mut gd: *mut grid, mut all: ::core::ffi::c_int) {
    let mut ny: u_int = 0;
    if (*gd).hsize == 0 as u_int || (*gd).hsize < (*gd).hlimit {
        return;
    }
    if all != 0 {
        ny = (*gd).hsize.wrapping_sub((*gd).hlimit);
    } else {
        ny = (*gd).hlimit.wrapping_div(10 as u_int);
    }
    if ny < 1 as u_int {
        ny = 1 as u_int;
    }
    if ny > (*gd).hsize {
        ny = (*gd).hsize;
    }
    grid_trim_history(gd, ny);
    (*gd).hsize = (*gd).hsize.wrapping_sub(ny);
    (*gd).scroll_collected = (*gd).scroll_collected.wrapping_add(ny);
    if (*gd).hscrolled > (*gd).hsize {
        (*gd).hscrolled = (*gd).hsize;
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_remove_history(mut gd: *mut grid, mut ny: u_int) {
    let mut yy: u_int = 0;
    let mut start: u_int = 0;
    if ny > (*gd).hsize {
        return;
    }
    start = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(ny);
    yy = 0 as u_int;
    while yy < ny {
        grid_free_line(gd, start.wrapping_add(yy));
        yy = yy.wrapping_add(1);
    }
    memset(
        (*gd).linedata.offset(start as isize) as *mut grid_line as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (ny as size_t).wrapping_mul(::core::mem::size_of::<grid_line>() as size_t),
    );
    (*gd).hsize = (*gd).hsize.wrapping_sub(ny);
}
#[no_mangle]
pub unsafe extern "C" fn grid_scroll_history(mut gd: *mut grid, mut bg: u_int) {
    let mut yy: u_int = 0;
    yy = (*gd).hsize.wrapping_add((*gd).sy);
    (*gd).linedata = xreallocarray(
        (*gd).linedata as *mut ::core::ffi::c_void,
        yy.wrapping_add(1 as u_int) as size_t,
        ::core::mem::size_of::<grid_line>() as size_t,
    ) as *mut grid_line;
    grid_empty_line(gd, yy, bg);
    (*gd).hscrolled = (*gd).hscrolled.wrapping_add(1);
    grid_compact_line((*gd).linedata.offset((*gd).hsize as isize) as *mut grid_line);
    grid_line_set_time((*gd).linedata.offset((*gd).hsize as isize) as *mut grid_line);
    (*gd).hsize = (*gd).hsize.wrapping_add(1);
    (*gd).scroll_added = (*gd).scroll_added.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn grid_clear_history(mut gd: *mut grid) {
    grid_trim_history(gd, (*gd).hsize);
    (*gd).hscrolled = 0 as u_int;
    (*gd).hsize = 0 as u_int;
    (*gd).scroll_generation = (*gd).scroll_generation.wrapping_add(1);
    (*gd).linedata = xreallocarray(
        (*gd).linedata as *mut ::core::ffi::c_void,
        (*gd).sy as size_t,
        ::core::mem::size_of::<grid_line>() as size_t,
    ) as *mut grid_line;
}
#[no_mangle]
pub unsafe extern "C" fn grid_scroll_history_region(
    mut gd: *mut grid,
    mut upper: u_int,
    mut lower: u_int,
    mut bg: u_int,
) {
    let mut gl_history: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut gl_upper: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut yy: u_int = 0;
    yy = (*gd).hsize.wrapping_add((*gd).sy);
    (*gd).linedata = xreallocarray(
        (*gd).linedata as *mut ::core::ffi::c_void,
        yy.wrapping_add(1 as u_int) as size_t,
        ::core::mem::size_of::<grid_line>() as size_t,
    ) as *mut grid_line;
    gl_history = (*gd).linedata.offset((*gd).hsize as isize) as *mut grid_line;
    memmove(
        gl_history.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        gl_history as *const ::core::ffi::c_void,
        ((*gd).sy as size_t).wrapping_mul(::core::mem::size_of::<grid_line>() as size_t),
    );
    upper = upper.wrapping_add(1);
    gl_upper = (*gd).linedata.offset(upper as isize) as *mut grid_line;
    lower = lower.wrapping_add(1);
    memcpy(
        gl_history as *mut ::core::ffi::c_void,
        gl_upper as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_line>() as size_t,
    );
    grid_line_set_time(gl_history);
    memmove(
        gl_upper as *mut ::core::ffi::c_void,
        gl_upper.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        (lower.wrapping_sub(upper) as size_t)
            .wrapping_mul(::core::mem::size_of::<grid_line>() as size_t),
    );
    grid_empty_line(gd, lower, bg);
    (*gd).hscrolled = (*gd).hscrolled.wrapping_add(1);
    (*gd).hsize = (*gd).hsize.wrapping_add(1);
    (*gd).scroll_added = (*gd).scroll_added.wrapping_add(1);
}
unsafe extern "C" fn grid_expand_line(
    mut gd: *mut grid,
    mut py: u_int,
    mut sx: u_int,
    mut bg: u_int,
) {
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut xx: u_int = 0;
    gl = (*gd).linedata.offset(py as isize) as *mut grid_line;
    if sx <= (*gl).cellsize as u_int {
        return;
    }
    if sx < (*gd).sx.wrapping_div(4 as u_int) {
        sx = (*gd).sx.wrapping_div(4 as u_int);
    } else if sx < (*gd).sx.wrapping_div(2 as u_int) {
        sx = (*gd).sx.wrapping_div(2 as u_int);
    } else if (*gd).sx > sx {
        sx = (*gd).sx;
    }
    (*gl).celldata = xreallocarray(
        (*gl).celldata as *mut ::core::ffi::c_void,
        sx as size_t,
        ::core::mem::size_of::<grid_cell_entry>() as size_t,
    ) as *mut grid_cell_entry;
    if ((*gl).cellsize as u_int) < sx {
        memset(
            (*gl)
                .celldata
                .offset((*gl).cellsize as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            (sx.wrapping_sub((*gl).cellsize as u_int) as size_t)
                .wrapping_mul(::core::mem::size_of::<grid_cell_entry>() as size_t),
        );
    }
    xx = (*gl).cellsize as u_int;
    while xx < sx {
        grid_clear_cell(gd, xx, py, bg, 0 as ::core::ffi::c_int);
        xx = xx.wrapping_add(1);
    }
    (*gl).cellsize = sx as u_short;
}
#[no_mangle]
pub unsafe extern "C" fn grid_empty_line(mut gd: *mut grid, mut py: u_int, mut bg: u_int) {
    memset(
        (*gd).linedata.offset(py as isize) as *mut grid_line as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<grid_line>() as size_t,
    );
    if !(bg == 8 as u_int || bg == 9 as u_int) {
        grid_expand_line(gd, py, (*gd).sx, bg);
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_peek_line(mut gd: *mut grid, mut py: u_int) -> *const grid_line {
    if grid_check_y(
        gd,
        b"grid_peek_line\0" as *const u8 as *const ::core::ffi::c_char,
        py,
    ) != 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null::<grid_line>();
    }
    return (*gd).linedata.offset(py as isize) as *mut grid_line;
}
unsafe extern "C" fn grid_get_cell1(mut gl: *mut grid_line, mut px: u_int, mut gc: *mut grid_cell) {
    let mut gce: *mut grid_cell_entry = (*gl).celldata.offset(px as isize) as *mut grid_cell_entry;
    let mut gee: *mut grid_extd_entry = ::core::ptr::null_mut::<grid_extd_entry>();
    if (*gce).flags as ::core::ffi::c_int & GRID_FLAG_EXTENDED != 0 {
        if (*gce).c2rust_unnamed.offset >= (*gl).extdsize {
            memcpy(
                gc as *mut ::core::ffi::c_void,
                &raw const grid_default_cell as *const ::core::ffi::c_void,
                ::core::mem::size_of::<grid_cell>() as size_t,
            );
        } else {
            gee = (*gl).extddata.offset((*gce).c2rust_unnamed.offset as isize)
                as *mut grid_extd_entry;
            (*gc).flags = (*gee).flags;
            (*gc).attr = (*gee).attr;
            (*gc).fg = (*gee).fg;
            (*gc).bg = (*gee).bg;
            (*gc).us = (*gee).us;
            (*gc).link = (*gee).link;
            if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                grid_set_tab(gc, (*gee).data as u_int);
            } else {
                utf8_to_data((*gee).data, &raw mut (*gc).data);
            }
        }
        return;
    }
    (*gc).flags =
        ((*gce).flags as ::core::ffi::c_int & !(GRID_FLAG_FG256 | GRID_FLAG_BG256)) as u_char;
    (*gc).attr = (*gce).c2rust_unnamed.data.attr as u_short;
    (*gc).fg = (*gce).c2rust_unnamed.data.fg as ::core::ffi::c_int;
    if (*gce).flags as ::core::ffi::c_int & GRID_FLAG_FG256 != 0 {
        (*gc).fg |= COLOUR_FLAG_256;
    }
    (*gc).bg = (*gce).c2rust_unnamed.data.bg as ::core::ffi::c_int;
    if (*gce).flags as ::core::ffi::c_int & GRID_FLAG_BG256 != 0 {
        (*gc).bg |= COLOUR_FLAG_256;
    }
    (*gc).us = 8 as ::core::ffi::c_int;
    utf8_set(&raw mut (*gc).data, (*gce).c2rust_unnamed.data.data);
    (*gc).link = 0 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn grid_get_cell(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut gc: *mut grid_cell,
) {
    if grid_check_y(
        gd,
        b"grid_get_cell\0" as *const u8 as *const ::core::ffi::c_char,
        py,
    ) != 0 as ::core::ffi::c_int
        || px >= (*(*gd).linedata.offset(py as isize)).cellsize as u_int
    {
        memcpy(
            gc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    } else {
        grid_get_cell1((*gd).linedata.offset(py as isize) as *mut grid_line, px, gc);
    };
}
#[no_mangle]
pub unsafe extern "C" fn grid_set_cell(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut gc: *const grid_cell,
) {
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut gce: *mut grid_cell_entry = ::core::ptr::null_mut::<grid_cell_entry>();
    if grid_check_y(
        gd,
        b"grid_set_cell\0" as *const u8 as *const ::core::ffi::c_char,
        py,
    ) != 0 as ::core::ffi::c_int
    {
        return;
    }
    grid_expand_line(gd, py, px.wrapping_add(1 as u_int), 8 as u_int);
    gl = (*gd).linedata.offset(py as isize) as *mut grid_line;
    if px.wrapping_add(1 as u_int) > (*gl).cellused as u_int {
        (*gl).cellused = px.wrapping_add(1 as u_int) as u_short;
    }
    gce = (*gl).celldata.offset(px as isize) as *mut grid_cell_entry;
    if grid_need_extended_cell(gce, gc) != 0 {
        grid_extended_cell(gl, gce, gc);
    } else {
        grid_store_cell(gce, gc, (*gc).data.data[0 as ::core::ffi::c_int as usize]);
    };
}
#[no_mangle]
pub unsafe extern "C" fn grid_set_padding(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut bg: ::core::ffi::c_int,
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
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_padding_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    gc.bg = bg;
    grid_set_cell(gd, px, py, &raw mut gc);
}
#[no_mangle]
pub unsafe extern "C" fn grid_set_cells(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut gc: *const grid_cell,
    mut s: *const ::core::ffi::c_char,
    mut slen: size_t,
) {
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut gce: *mut grid_cell_entry = ::core::ptr::null_mut::<grid_cell_entry>();
    let mut gee: *mut grid_extd_entry = ::core::ptr::null_mut::<grid_extd_entry>();
    let mut i: u_int = 0;
    if grid_check_y(
        gd,
        b"grid_set_cells\0" as *const u8 as *const ::core::ffi::c_char,
        py,
    ) != 0 as ::core::ffi::c_int
    {
        return;
    }
    grid_expand_line(
        gd,
        py,
        (px as size_t).wrapping_add(slen) as u_int,
        8 as u_int,
    );
    gl = (*gd).linedata.offset(py as isize) as *mut grid_line;
    if (px as size_t).wrapping_add(slen) > (*gl).cellused as size_t {
        (*gl).cellused = (px as size_t).wrapping_add(slen) as u_short;
    }
    i = 0 as u_int;
    while (i as size_t) < slen {
        gce = (*gl).celldata.offset(px.wrapping_add(i) as isize) as *mut grid_cell_entry;
        if grid_need_extended_cell(gce, gc) != 0 {
            gee = grid_extended_cell(gl, gce, gc);
            (*gee).data = utf8_build_one(*s.offset(i as isize) as u_char);
        } else {
            grid_store_cell(gce, gc, *s.offset(i as isize) as u_char);
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_clear(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut ny: u_int,
    mut bg: u_int,
) {
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    let mut ox: u_int = 0;
    let mut sx: u_int = 0;
    if nx == 0 as u_int || ny == 0 as u_int {
        return;
    }
    if px == 0 as u_int && nx == (*gd).sx {
        grid_clear_lines(gd, py, ny, bg);
        return;
    }
    if grid_check_y(
        gd,
        b"grid_clear\0" as *const u8 as *const ::core::ffi::c_char,
        py,
    ) != 0 as ::core::ffi::c_int
    {
        return;
    }
    if grid_check_y(
        gd,
        b"grid_clear\0" as *const u8 as *const ::core::ffi::c_char,
        py.wrapping_add(ny).wrapping_sub(1 as u_int),
    ) != 0 as ::core::ffi::c_int
    {
        return;
    }
    let mut current_block_21: u64;
    yy = py;
    while yy < py.wrapping_add(ny) {
        gl = (*gd).linedata.offset(yy as isize) as *mut grid_line;
        sx = (*gd).sx;
        if sx > (*gl).cellsize as u_int {
            sx = (*gl).cellsize as u_int;
        }
        ox = nx;
        if bg == 8 as u_int || bg == 9 as u_int {
            if px > sx {
                current_block_21 = 14523784380283086299;
            } else {
                if px.wrapping_add(nx) > sx {
                    ox = sx.wrapping_sub(px);
                }
                current_block_21 = 7149356873433890176;
            }
        } else {
            current_block_21 = 7149356873433890176;
        }
        match current_block_21 {
            7149356873433890176 => {
                grid_expand_line(gd, yy, px.wrapping_add(ox), 8 as u_int);
                xx = px;
                while xx < px.wrapping_add(ox) {
                    grid_clear_cell(gd, xx, yy, bg, 0 as ::core::ffi::c_int);
                    xx = xx.wrapping_add(1);
                }
            }
            _ => {}
        }
        yy = yy.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_clear_lines(
    mut gd: *mut grid,
    mut py: u_int,
    mut ny: u_int,
    mut bg: u_int,
) {
    let mut yy: u_int = 0;
    if ny == 0 as u_int {
        return;
    }
    if grid_check_y(
        gd,
        b"grid_clear_lines\0" as *const u8 as *const ::core::ffi::c_char,
        py,
    ) != 0 as ::core::ffi::c_int
    {
        return;
    }
    if grid_check_y(
        gd,
        b"grid_clear_lines\0" as *const u8 as *const ::core::ffi::c_char,
        py.wrapping_add(ny).wrapping_sub(1 as u_int),
    ) != 0 as ::core::ffi::c_int
    {
        return;
    }
    yy = py;
    while yy < py.wrapping_add(ny) {
        grid_free_line(gd, yy);
        grid_empty_line(gd, yy, bg);
        yy = yy.wrapping_add(1);
    }
    if py != 0 as u_int {
        let ref mut fresh1 = (*(*gd).linedata.offset(py.wrapping_sub(1 as u_int) as isize)).flags;
        *fresh1 = (*fresh1 as ::core::ffi::c_int & !GRID_LINE_WRAPPED) as u_short;
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_move_lines(
    mut gd: *mut grid,
    mut dy: u_int,
    mut py: u_int,
    mut ny: u_int,
    mut bg: u_int,
) {
    let mut yy: u_int = 0;
    if ny == 0 as u_int || py == dy {
        return;
    }
    if grid_check_y(
        gd,
        b"grid_move_lines\0" as *const u8 as *const ::core::ffi::c_char,
        py,
    ) != 0 as ::core::ffi::c_int
    {
        return;
    }
    if grid_check_y(
        gd,
        b"grid_move_lines\0" as *const u8 as *const ::core::ffi::c_char,
        py.wrapping_add(ny).wrapping_sub(1 as u_int),
    ) != 0 as ::core::ffi::c_int
    {
        return;
    }
    if grid_check_y(
        gd,
        b"grid_move_lines\0" as *const u8 as *const ::core::ffi::c_char,
        dy,
    ) != 0 as ::core::ffi::c_int
    {
        return;
    }
    if grid_check_y(
        gd,
        b"grid_move_lines\0" as *const u8 as *const ::core::ffi::c_char,
        dy.wrapping_add(ny).wrapping_sub(1 as u_int),
    ) != 0 as ::core::ffi::c_int
    {
        return;
    }
    yy = dy;
    while yy < dy.wrapping_add(ny) {
        if !(yy >= py && yy < py.wrapping_add(ny)) {
            grid_free_line(gd, yy);
        }
        yy = yy.wrapping_add(1);
    }
    if dy != 0 as u_int {
        let ref mut fresh2 = (*(*gd).linedata.offset(dy.wrapping_sub(1 as u_int) as isize)).flags;
        *fresh2 = (*fresh2 as ::core::ffi::c_int & !GRID_LINE_WRAPPED) as u_short;
    }
    memmove(
        (*gd).linedata.offset(dy as isize) as *mut grid_line as *mut ::core::ffi::c_void,
        (*gd).linedata.offset(py as isize) as *mut grid_line as *const ::core::ffi::c_void,
        (ny as size_t).wrapping_mul(::core::mem::size_of::<grid_line>() as size_t),
    );
    yy = py;
    while yy < py.wrapping_add(ny) {
        if yy < dy || yy >= dy.wrapping_add(ny) {
            grid_empty_line(gd, yy, bg);
        }
        yy = yy.wrapping_add(1);
    }
    if py != 0 as u_int && (py < dy || py >= dy.wrapping_add(ny)) {
        let ref mut fresh3 = (*(*gd).linedata.offset(py.wrapping_sub(1 as u_int) as isize)).flags;
        *fresh3 = (*fresh3 as ::core::ffi::c_int & !GRID_LINE_WRAPPED) as u_short;
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_move_cells(
    mut gd: *mut grid,
    mut dx: u_int,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut xx: u_int = 0;
    if nx == 0 as u_int || px == dx {
        return;
    }
    if grid_check_y(
        gd,
        b"grid_move_cells\0" as *const u8 as *const ::core::ffi::c_char,
        py,
    ) != 0 as ::core::ffi::c_int
    {
        return;
    }
    gl = (*gd).linedata.offset(py as isize) as *mut grid_line;
    grid_expand_line(gd, py, px.wrapping_add(nx), 8 as u_int);
    grid_expand_line(gd, py, dx.wrapping_add(nx), 8 as u_int);
    memmove(
        (*gl).celldata.offset(dx as isize) as *mut grid_cell_entry as *mut ::core::ffi::c_void,
        (*gl).celldata.offset(px as isize) as *mut grid_cell_entry as *const ::core::ffi::c_void,
        (nx as size_t).wrapping_mul(::core::mem::size_of::<grid_cell_entry>() as size_t),
    );
    if dx.wrapping_add(nx) > (*gl).cellused as u_int {
        (*gl).cellused = dx.wrapping_add(nx) as u_short;
    }
    xx = px;
    while xx < px.wrapping_add(nx) {
        if !(xx >= dx && xx < dx.wrapping_add(nx)) {
            grid_clear_cell(gd, xx, py, bg, 1 as ::core::ffi::c_int);
        }
        xx = xx.wrapping_add(1);
    }
}
unsafe extern "C" fn grid_string_cells_fg(
    mut gc: *const grid_cell,
    mut values: *mut ::core::ffi::c_int,
) -> size_t {
    let mut n: size_t = 0;
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut c: ::core::ffi::c_int = 0;
    n = 0 as size_t;
    if (*gc).fg & COLOUR_FLAG_THEME != 0 {
        c = colour_theme_terminal_colour(((*gc).fg & 0xff as ::core::ffi::c_int) as u_int);
        if c == 8 as ::core::ffi::c_int {
            let fresh31 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh31 as isize) = 39 as ::core::ffi::c_int;
        } else {
            let fresh32 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh32 as isize) = c + 30 as ::core::ffi::c_int;
        }
    } else if (*gc).fg & COLOUR_FLAG_256 != 0 {
        let fresh33 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh33 as isize) = 38 as ::core::ffi::c_int;
        let fresh34 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh34 as isize) = 5 as ::core::ffi::c_int;
        let fresh35 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh35 as isize) = (*gc).fg & 0xff as ::core::ffi::c_int;
    } else if (*gc).fg & COLOUR_FLAG_RGB != 0 {
        let fresh36 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh36 as isize) = 38 as ::core::ffi::c_int;
        let fresh37 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh37 as isize) = 2 as ::core::ffi::c_int;
        colour_split_rgb((*gc).fg, &raw mut r, &raw mut g, &raw mut b);
        let fresh38 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh38 as isize) = r as ::core::ffi::c_int;
        let fresh39 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh39 as isize) = g as ::core::ffi::c_int;
        let fresh40 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh40 as isize) = b as ::core::ffi::c_int;
    } else {
        match (*gc).fg {
            0..=7 => {
                let fresh41 = n;
                n = n.wrapping_add(1);
                *values.offset(fresh41 as isize) = (*gc).fg + 30 as ::core::ffi::c_int;
            }
            8 => {
                let fresh42 = n;
                n = n.wrapping_add(1);
                *values.offset(fresh42 as isize) = 39 as ::core::ffi::c_int;
            }
            90..=97 => {
                let fresh43 = n;
                n = n.wrapping_add(1);
                *values.offset(fresh43 as isize) = (*gc).fg;
            }
            _ => {}
        }
    }
    return n;
}
unsafe extern "C" fn grid_string_cells_bg(
    mut gc: *const grid_cell,
    mut values: *mut ::core::ffi::c_int,
) -> size_t {
    let mut n: size_t = 0;
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut c: ::core::ffi::c_int = 0;
    n = 0 as size_t;
    if (*gc).bg & COLOUR_FLAG_THEME != 0 {
        c = colour_theme_terminal_colour(((*gc).bg & 0xff as ::core::ffi::c_int) as u_int);
        if c == 8 as ::core::ffi::c_int {
            let fresh18 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh18 as isize) = 49 as ::core::ffi::c_int;
        } else {
            let fresh19 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh19 as isize) = c + 40 as ::core::ffi::c_int;
        }
    } else if (*gc).bg & COLOUR_FLAG_256 != 0 {
        let fresh20 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh20 as isize) = 48 as ::core::ffi::c_int;
        let fresh21 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh21 as isize) = 5 as ::core::ffi::c_int;
        let fresh22 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh22 as isize) = (*gc).bg & 0xff as ::core::ffi::c_int;
    } else if (*gc).bg & COLOUR_FLAG_RGB != 0 {
        let fresh23 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh23 as isize) = 48 as ::core::ffi::c_int;
        let fresh24 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh24 as isize) = 2 as ::core::ffi::c_int;
        colour_split_rgb((*gc).bg, &raw mut r, &raw mut g, &raw mut b);
        let fresh25 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh25 as isize) = r as ::core::ffi::c_int;
        let fresh26 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh26 as isize) = g as ::core::ffi::c_int;
        let fresh27 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh27 as isize) = b as ::core::ffi::c_int;
    } else {
        match (*gc).bg {
            0..=7 => {
                let fresh28 = n;
                n = n.wrapping_add(1);
                *values.offset(fresh28 as isize) = (*gc).bg + 40 as ::core::ffi::c_int;
            }
            8 => {
                let fresh29 = n;
                n = n.wrapping_add(1);
                *values.offset(fresh29 as isize) = 49 as ::core::ffi::c_int;
            }
            90..=97 => {
                let fresh30 = n;
                n = n.wrapping_add(1);
                *values.offset(fresh30 as isize) = (*gc).bg + 10 as ::core::ffi::c_int;
            }
            _ => {}
        }
    }
    return n;
}
unsafe extern "C" fn grid_string_cells_us(
    mut gc: *const grid_cell,
    mut values: *mut ::core::ffi::c_int,
) -> size_t {
    let mut n: size_t = 0;
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut c: ::core::ffi::c_int = 0;
    n = 0 as size_t;
    if (*gc).us & COLOUR_FLAG_THEME != 0 {
        c = colour_theme_terminal_colour(((*gc).us & 0xff as ::core::ffi::c_int) as u_int);
        if c == 8 as ::core::ffi::c_int {
            let fresh6 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh6 as isize) = 59 as ::core::ffi::c_int;
        } else {
            let fresh7 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh7 as isize) = 58 as ::core::ffi::c_int;
            let fresh8 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh8 as isize) = 5 as ::core::ffi::c_int;
            let fresh9 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh9 as isize) = c;
        }
    } else if (*gc).us & COLOUR_FLAG_256 != 0 {
        let fresh10 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh10 as isize) = 58 as ::core::ffi::c_int;
        let fresh11 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh11 as isize) = 5 as ::core::ffi::c_int;
        let fresh12 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh12 as isize) = (*gc).us & 0xff as ::core::ffi::c_int;
    } else if (*gc).us & COLOUR_FLAG_RGB != 0 {
        let fresh13 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh13 as isize) = 58 as ::core::ffi::c_int;
        let fresh14 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh14 as isize) = 2 as ::core::ffi::c_int;
        colour_split_rgb((*gc).us, &raw mut r, &raw mut g, &raw mut b);
        let fresh15 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh15 as isize) = r as ::core::ffi::c_int;
        let fresh16 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh16 as isize) = g as ::core::ffi::c_int;
        let fresh17 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh17 as isize) = b as ::core::ffi::c_int;
    }
    return n;
}
unsafe extern "C" fn grid_string_cells_add_code(
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut n: u_int,
    mut s: *mut ::core::ffi::c_int,
    mut newc: *mut ::core::ffi::c_int,
    mut oldc: *mut ::core::ffi::c_int,
    mut nnewc: size_t,
    mut noldc: size_t,
    mut flags: ::core::ffi::c_int,
) {
    let mut i: u_int = 0;
    let mut tmp: [::core::ffi::c_char; 64] = [0; 64];
    let mut reset: ::core::ffi::c_int = (n != 0 as u_int
        && *s.offset(0 as ::core::ffi::c_int as isize) == 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    if nnewc == 0 as size_t {
        return;
    }
    if reset == 0
        && nnewc == noldc
        && memcmp(
            newc as *const ::core::ffi::c_void,
            oldc as *const ::core::ffi::c_void,
            nnewc.wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>() as size_t),
        ) == 0 as ::core::ffi::c_int
    {
        return;
    }
    if reset != 0
        && (*newc.offset(0 as ::core::ffi::c_int as isize) == 49 as ::core::ffi::c_int
            || *newc.offset(0 as ::core::ffi::c_int as isize) == 39 as ::core::ffi::c_int)
    {
        return;
    }
    if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
        strlcat(
            buf,
            b"\\033[\0" as *const u8 as *const ::core::ffi::c_char,
            len,
        );
    } else {
        strlcat(
            buf,
            b"\x1B[\0" as *const u8 as *const ::core::ffi::c_char,
            len,
        );
    }
    i = 0 as u_int;
    while (i as size_t) < nnewc {
        if (i.wrapping_add(1 as u_int) as size_t) < nnewc {
            xsnprintf(
                &raw mut tmp as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                b"%d;\0" as *const u8 as *const ::core::ffi::c_char,
                *newc.offset(i as isize),
            );
        } else {
            xsnprintf(
                &raw mut tmp as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                *newc.offset(i as isize),
            );
        }
        strlcat(buf, &raw mut tmp as *mut ::core::ffi::c_char, len);
        i = i.wrapping_add(1);
    }
    strlcat(buf, b"m\0" as *const u8 as *const ::core::ffi::c_char, len);
}
unsafe extern "C" fn grid_string_cells_add_hyperlink(
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut id: *const ::core::ffi::c_char,
    mut uri: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if strlen(uri)
        .wrapping_add(strlen(id))
        .wrapping_add(17 as size_t)
        >= len
    {
        return 0 as ::core::ffi::c_int;
    }
    if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
        strlcat(
            buf,
            b"\\033]8;\0" as *const u8 as *const ::core::ffi::c_char,
            len,
        );
    } else {
        strlcat(
            buf,
            b"\x1B]8;\0" as *const u8 as *const ::core::ffi::c_char,
            len,
        );
    }
    if *id as ::core::ffi::c_int != '\0' as i32 {
        let id_bytes = CStr::from_ptr(id).to_bytes();
        let mut bytes = Vec::with_capacity(id_bytes.len() + 4);
        bytes.extend_from_slice(b"id=");
        bytes.extend_from_slice(id_bytes);
        bytes.push(b';');
        let tmp = CString::new(bytes).expect("C string ID contains no interior NUL");
        strlcat(buf, tmp.as_ptr(), len);
    } else {
        strlcat(buf, b";\0" as *const u8 as *const ::core::ffi::c_char, len);
    }
    strlcat(buf, uri, len);
    if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
        strlcat(
            buf,
            b"\\033\\\\\0" as *const u8 as *const ::core::ffi::c_char,
            len,
        );
    } else {
        strlcat(
            buf,
            b"\x1B\\\0" as *const u8 as *const ::core::ffi::c_char,
            len,
        );
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn grid_string_cells_code(
    mut lastgc: *const grid_cell,
    mut gc: *const grid_cell,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut flags: ::core::ffi::c_int,
    mut sc: *mut screen,
    mut has_link: *mut ::core::ffi::c_int,
) {
    let mut oldc: [::core::ffi::c_int; 64] = [0; 64];
    let mut newc: [::core::ffi::c_int; 64] = [0; 64];
    let mut s: [::core::ffi::c_int; 128] = [0; 128];
    let mut noldc: size_t = 0;
    let mut nnewc: size_t = 0;
    let mut n: size_t = 0;
    let mut i: size_t = 0;
    let mut attr: u_int = (*gc).attr as u_int;
    let mut lastattr: u_int = (*lastgc).attr as u_int;
    let mut tmp: [::core::ffi::c_char; 64] = [0; 64];
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    static mut attrs: [C2RustUnnamed_1; 13] = [
        C2RustUnnamed_1 {
            mask: GRID_ATTR_BRIGHT as u_int,
            code: 1 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_DIM as u_int,
            code: 2 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_ITALICS as u_int,
            code: 3 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_UNDERSCORE as u_int,
            code: 4 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_BLINK as u_int,
            code: 5 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_REVERSE as u_int,
            code: 7 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_HIDDEN as u_int,
            code: 8 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_STRIKETHROUGH as u_int,
            code: 9 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_UNDERSCORE_2 as u_int,
            code: 42 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_UNDERSCORE_3 as u_int,
            code: 43 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_UNDERSCORE_4 as u_int,
            code: 44 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_UNDERSCORE_5 as u_int,
            code: 45 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_OVERLINE as u_int,
            code: 53 as u_int,
        },
    ];
    n = 0 as size_t;
    i = 0 as size_t;
    while i
        < (::core::mem::size_of::<[C2RustUnnamed_1; 13]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_1>() as usize)
    {
        if !attr & attrs[i as usize].mask != 0 && lastattr & attrs[i as usize].mask != 0
            || (*lastgc).us != 8 as ::core::ffi::c_int && (*gc).us == 8 as ::core::ffi::c_int
        {
            let fresh4 = n;
            n = n.wrapping_add(1);
            s[fresh4 as usize] = 0 as ::core::ffi::c_int;
            lastattr &= GRID_ATTR_CHARSET as u_int;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    i = 0 as size_t;
    while i
        < (::core::mem::size_of::<[C2RustUnnamed_1; 13]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_1>() as usize)
    {
        if attr & attrs[i as usize].mask != 0 && lastattr & attrs[i as usize].mask == 0 {
            let fresh5 = n;
            n = n.wrapping_add(1);
            s[fresh5 as usize] = attrs[i as usize].code as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    *buf = '\0' as i32 as ::core::ffi::c_char;
    if n > 0 as size_t {
        if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
            strlcat(
                buf,
                b"\\033[\0" as *const u8 as *const ::core::ffi::c_char,
                len,
            );
        } else {
            strlcat(
                buf,
                b"\x1B[\0" as *const u8 as *const ::core::ffi::c_char,
                len,
            );
        }
        i = 0 as size_t;
        while i < n {
            if s[i as usize] < 10 as ::core::ffi::c_int {
                xsnprintf(
                    &raw mut tmp as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                    b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                    s[i as usize],
                );
            } else {
                xsnprintf(
                    &raw mut tmp as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                    b"%d:%d\0" as *const u8 as *const ::core::ffi::c_char,
                    s[i as usize] / 10 as ::core::ffi::c_int,
                    s[i as usize] % 10 as ::core::ffi::c_int,
                );
            }
            strlcat(buf, &raw mut tmp as *mut ::core::ffi::c_char, len);
            if i.wrapping_add(1 as size_t) < n {
                strlcat(buf, b";\0" as *const u8 as *const ::core::ffi::c_char, len);
            }
            i = i.wrapping_add(1);
        }
        strlcat(buf, b"m\0" as *const u8 as *const ::core::ffi::c_char, len);
    }
    nnewc = grid_string_cells_fg(gc, &raw mut newc as *mut ::core::ffi::c_int);
    noldc = grid_string_cells_fg(lastgc, &raw mut oldc as *mut ::core::ffi::c_int);
    grid_string_cells_add_code(
        buf,
        len,
        n as u_int,
        &raw mut s as *mut ::core::ffi::c_int,
        &raw mut newc as *mut ::core::ffi::c_int,
        &raw mut oldc as *mut ::core::ffi::c_int,
        nnewc,
        noldc,
        flags,
    );
    nnewc = grid_string_cells_bg(gc, &raw mut newc as *mut ::core::ffi::c_int);
    noldc = grid_string_cells_bg(lastgc, &raw mut oldc as *mut ::core::ffi::c_int);
    grid_string_cells_add_code(
        buf,
        len,
        n as u_int,
        &raw mut s as *mut ::core::ffi::c_int,
        &raw mut newc as *mut ::core::ffi::c_int,
        &raw mut oldc as *mut ::core::ffi::c_int,
        nnewc,
        noldc,
        flags,
    );
    nnewc = grid_string_cells_us(gc, &raw mut newc as *mut ::core::ffi::c_int);
    noldc = grid_string_cells_us(lastgc, &raw mut oldc as *mut ::core::ffi::c_int);
    grid_string_cells_add_code(
        buf,
        len,
        n as u_int,
        &raw mut s as *mut ::core::ffi::c_int,
        &raw mut newc as *mut ::core::ffi::c_int,
        &raw mut oldc as *mut ::core::ffi::c_int,
        nnewc,
        noldc,
        flags,
    );
    if attr & GRID_ATTR_CHARSET as u_int != 0 && lastattr & GRID_ATTR_CHARSET as u_int == 0 {
        if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
            strlcat(
                buf,
                b"\\016\0" as *const u8 as *const ::core::ffi::c_char,
                len,
            );
        } else {
            strlcat(
                buf,
                b"\x0E\0" as *const u8 as *const ::core::ffi::c_char,
                len,
            );
        }
    }
    if attr & GRID_ATTR_CHARSET as u_int == 0 && lastattr & GRID_ATTR_CHARSET as u_int != 0 {
        if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
            strlcat(
                buf,
                b"\\017\0" as *const u8 as *const ::core::ffi::c_char,
                len,
            );
        } else {
            strlcat(
                buf,
                b"\x0F\0" as *const u8 as *const ::core::ffi::c_char,
                len,
            );
        }
    }
    if !sc.is_null() && !(*sc).hyperlinks.is_null() && (*lastgc).link != (*gc).link {
        if hyperlinks_get(
            (*sc).hyperlinks,
            (*gc).link,
            &raw mut uri,
            &raw mut id,
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        ) != 0
        {
            *has_link = grid_string_cells_add_hyperlink(buf, len, id, uri, flags);
        } else if *has_link != 0 {
            grid_string_cells_add_hyperlink(
                buf,
                len,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                flags,
            );
            *has_link = 0 as ::core::ffi::c_int;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_string_cells(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut lastgc: *mut *mut grid_cell,
    mut flags: ::core::ffi::c_int,
    mut s: *mut screen,
) -> *mut ::core::ffi::c_char {
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
    static mut lastgc1: grid_cell = grid_cell {
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
    let mut data: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut code: [::core::ffi::c_char; 8192] = [0; 8192];
    let mut len: size_t = 0;
    let mut off: size_t = 0;
    let mut size: size_t = 0;
    let mut codelen: size_t = 0;
    let mut xx: u_int = 0;
    let mut end: u_int = 0;
    let mut has_link: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut gl: *const grid_line = ::core::ptr::null::<grid_line>();
    if !lastgc.is_null() && (*lastgc).is_null() {
        memcpy(
            &raw mut lastgc1 as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
        *lastgc = &raw mut lastgc1;
    }
    len = 128 as size_t;
    buf = xmalloc(len) as *mut ::core::ffi::c_char;
    off = 0 as size_t;
    gl = grid_peek_line(gd, py);
    if gl.is_null() {
        *buf.offset(0 as ::core::ffi::c_int as isize) = '\0' as i32 as ::core::ffi::c_char;
        return buf;
    }
    if flags & GRID_STRING_EMPTY_CELLS != 0 {
        end = (*gl).cellsize as u_int;
    } else {
        end = (*gl).cellused as u_int;
    }
    xx = px;
    while xx < px.wrapping_add(nx) {
        if xx >= end {
            break;
        }
        grid_get_cell(gd, xx, py, &raw mut gc);
        if !(gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
            if !lastgc.is_null() && flags & GRID_STRING_WITH_SEQUENCES != 0 {
                grid_string_cells_code(
                    *lastgc,
                    &raw mut gc,
                    &raw mut code as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
                    flags,
                    s,
                    &raw mut has_link,
                );
                codelen = strlen(&raw mut code as *mut ::core::ffi::c_char);
                memcpy(
                    *lastgc as *mut ::core::ffi::c_void,
                    &raw mut gc as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
            } else {
                codelen = 0 as size_t;
            }
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                data = b"\t\0" as *const u8 as *const ::core::ffi::c_char;
                size = 1 as size_t;
            } else {
                data = &raw mut gc.data.data as *mut u_char as *const ::core::ffi::c_char;
                size = gc.data.size as size_t;
                if flags & GRID_STRING_ESCAPE_SEQUENCES != 0
                    && size == 1 as size_t
                    && *data as ::core::ffi::c_int == '\\' as i32
                {
                    data = b"\\\\\0" as *const u8 as *const ::core::ffi::c_char;
                    size = 2 as size_t;
                }
            }
            while len
                < off
                    .wrapping_add(size)
                    .wrapping_add(codelen)
                    .wrapping_add(1 as size_t)
            {
                buf = xreallocarray(buf as *mut ::core::ffi::c_void, 2 as size_t, len)
                    as *mut ::core::ffi::c_char;
                len = len.wrapping_mul(2 as size_t);
            }
            if codelen != 0 as size_t {
                memcpy(
                    buf.offset(off as isize) as *mut ::core::ffi::c_void,
                    &raw mut code as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                    codelen,
                );
                off = off.wrapping_add(codelen);
            }
            memcpy(
                buf.offset(off as isize) as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                size,
            );
            off = off.wrapping_add(size);
        }
        xx = xx.wrapping_add(1);
    }
    if has_link != 0 {
        grid_string_cells_add_hyperlink(
            &raw mut code as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
            flags,
        );
        codelen = strlen(&raw mut code as *mut ::core::ffi::c_char);
        while len
            < off
                .wrapping_add(size)
                .wrapping_add(codelen)
                .wrapping_add(1 as size_t)
        {
            buf = xreallocarray(buf as *mut ::core::ffi::c_void, 2 as size_t, len)
                as *mut ::core::ffi::c_char;
            len = len.wrapping_mul(2 as size_t);
        }
        memcpy(
            buf.offset(off as isize) as *mut ::core::ffi::c_void,
            &raw mut code as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            codelen,
        );
        off = off.wrapping_add(codelen);
    }
    if flags & GRID_STRING_TRIM_SPACES != 0 {
        while off > 0 as size_t
            && *buf.offset(off.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == ' ' as i32
        {
            off = off.wrapping_sub(1);
        }
    }
    *buf.offset(off as isize) = '\0' as i32 as ::core::ffi::c_char;
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn grid_duplicate_lines(
    mut dst: *mut grid,
    mut dy: u_int,
    mut src: *mut grid,
    mut sy: u_int,
    mut ny: u_int,
) {
    let mut dstl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut srcl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut yy: u_int = 0;
    if dy.wrapping_add(ny) > (*dst).hsize.wrapping_add((*dst).sy) {
        ny = (*dst).hsize.wrapping_add((*dst).sy).wrapping_sub(dy);
    }
    if sy.wrapping_add(ny) > (*src).hsize.wrapping_add((*src).sy) {
        ny = (*src).hsize.wrapping_add((*src).sy).wrapping_sub(sy);
    }
    grid_free_lines(dst, dy, ny);
    yy = 0 as u_int;
    while yy < ny {
        srcl = (*src).linedata.offset(sy as isize) as *mut grid_line;
        dstl = (*dst).linedata.offset(dy as isize) as *mut grid_line;
        memcpy(
            dstl as *mut ::core::ffi::c_void,
            srcl as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_line>() as size_t,
        );
        if (*srcl).cellsize as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            (*dstl).celldata = xreallocarray(
                NULL,
                (*srcl).cellsize as size_t,
                ::core::mem::size_of::<grid_cell_entry>() as size_t,
            ) as *mut grid_cell_entry;
            memcpy(
                (*dstl).celldata as *mut ::core::ffi::c_void,
                (*srcl).celldata as *const ::core::ffi::c_void,
                ((*srcl).cellsize as size_t)
                    .wrapping_mul(::core::mem::size_of::<grid_cell_entry>() as size_t),
            );
        } else {
            (*dstl).celldata = ::core::ptr::null_mut::<grid_cell_entry>();
        }
        if (*srcl).extdsize != 0 as u_int {
            (*dstl).extdsize = (*srcl).extdsize;
            (*dstl).extddata = xreallocarray(
                NULL,
                (*dstl).extdsize as size_t,
                ::core::mem::size_of::<grid_extd_entry>() as size_t,
            ) as *mut grid_extd_entry;
            memcpy(
                (*dstl).extddata as *mut ::core::ffi::c_void,
                (*srcl).extddata as *const ::core::ffi::c_void,
                ((*dstl).extdsize as size_t)
                    .wrapping_mul(::core::mem::size_of::<grid_extd_entry>() as size_t),
            );
        } else {
            (*dstl).extddata = ::core::ptr::null_mut::<grid_extd_entry>();
        }
        sy = sy.wrapping_add(1);
        dy = dy.wrapping_add(1);
        yy = yy.wrapping_add(1);
    }
}
unsafe extern "C" fn grid_reflow_dead(mut gl: *mut grid_line) {
    memset(
        gl as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<grid_line>() as size_t,
    );
    (*gl).flags = GRID_LINE_DEAD as u_short;
}
unsafe extern "C" fn grid_reflow_add(mut gd: *mut grid, mut n: u_int) -> *mut grid_line {
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut sy: u_int = (*gd).sy.wrapping_add(n);
    (*gd).linedata = xreallocarray(
        (*gd).linedata as *mut ::core::ffi::c_void,
        sy as size_t,
        ::core::mem::size_of::<grid_line>() as size_t,
    ) as *mut grid_line;
    gl = (*gd).linedata.offset((*gd).sy as isize) as *mut grid_line;
    memset(
        gl as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (n as size_t).wrapping_mul(::core::mem::size_of::<grid_line>() as size_t),
    );
    (*gd).sy = sy;
    return gl;
}
unsafe extern "C" fn grid_reflow_move(
    mut gd: *mut grid,
    mut from: *mut grid_line,
) -> *mut grid_line {
    let mut to: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    to = grid_reflow_add(gd, 1 as u_int);
    memcpy(
        to as *mut ::core::ffi::c_void,
        from as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_line>() as size_t,
    );
    grid_reflow_dead(from);
    return to;
}
unsafe extern "C" fn grid_reflow_join(
    mut target: *mut grid,
    mut gd: *mut grid,
    mut sx: u_int,
    mut yy: u_int,
    mut width: u_int,
    mut already: ::core::ffi::c_int,
) {
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut from: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
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
    let mut lines: u_int = 0;
    let mut left: u_int = 0;
    let mut i: u_int = 0;
    let mut to: u_int = 0;
    let mut line: u_int = 0;
    let mut want: u_int = 0 as u_int;
    let mut at: u_int = 0;
    let mut wrapped: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if already == 0 {
        to = (*target).sy;
        gl = grid_reflow_move(target, (*gd).linedata.offset(yy as isize) as *mut grid_line);
    } else {
        to = (*target).sy.wrapping_sub(1 as u_int);
        gl = (*target).linedata.offset(to as isize) as *mut grid_line;
    }
    at = (*gl).cellused as u_int;
    lines = 0 as u_int;
    while !(yy.wrapping_add(1 as u_int).wrapping_add(lines) == (*gd).hsize.wrapping_add((*gd).sy)) {
        line = yy.wrapping_add(1 as u_int).wrapping_add(lines);
        if !((*(*gd).linedata.offset(line as isize)).flags as ::core::ffi::c_int)
            & GRID_LINE_WRAPPED
            != 0
        {
            wrapped = 0 as ::core::ffi::c_int;
        }
        if (*(*gd).linedata.offset(line as isize)).cellused as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if wrapped == 0 {
                break;
            }
            lines = lines.wrapping_add(1);
        } else {
            grid_get_cell1(
                (*gd).linedata.offset(line as isize) as *mut grid_line,
                0 as u_int,
                &raw mut gc,
            );
            if width.wrapping_add(gc.data.width as u_int) > sx {
                break;
            }
            width = width.wrapping_add(gc.data.width as u_int);
            grid_set_cell(target, at, to, &raw mut gc);
            at = at.wrapping_add(1);
            from = (*gd).linedata.offset(line as isize) as *mut grid_line;
            want = 1 as u_int;
            while want < (*from).cellused as u_int {
                grid_get_cell1(from, want, &raw mut gc);
                if width.wrapping_add(gc.data.width as u_int) > sx {
                    break;
                }
                width = width.wrapping_add(gc.data.width as u_int);
                grid_set_cell(target, at, to, &raw mut gc);
                at = at.wrapping_add(1);
                want = want.wrapping_add(1);
            }
            lines = lines.wrapping_add(1);
            if wrapped == 0 || want != (*from).cellused as u_int || width == sx {
                break;
            }
        }
    }
    if lines == 0 as u_int || from.is_null() {
        return;
    }
    left = ((*from).cellused as u_int).wrapping_sub(want);
    if left != 0 as u_int {
        grid_move_cells(
            gd,
            0 as u_int,
            want,
            yy.wrapping_add(lines),
            left,
            8 as u_int,
        );
        (*from).cellused = left as u_short;
        (*from).cellsize = (*from).cellused;
        lines = lines.wrapping_sub(1);
    } else if wrapped == 0 {
        (*gl).flags = ((*gl).flags as ::core::ffi::c_int & !GRID_LINE_WRAPPED) as u_short;
    }
    i = yy.wrapping_add(1 as u_int);
    while i < yy.wrapping_add(1 as u_int).wrapping_add(lines) {
        free((*(*gd).linedata.offset(i as isize)).celldata as *mut ::core::ffi::c_void);
        free((*(*gd).linedata.offset(i as isize)).extddata as *mut ::core::ffi::c_void);
        grid_reflow_dead((*gd).linedata.offset(i as isize) as *mut grid_line);
        i = i.wrapping_add(1);
    }
    if (*gd).hscrolled > to.wrapping_add(lines) {
        (*gd).hscrolled = (*gd).hscrolled.wrapping_sub(lines);
    } else if (*gd).hscrolled > to {
        (*gd).hscrolled = to;
    }
}
unsafe extern "C" fn grid_reflow_split(
    mut target: *mut grid,
    mut gd: *mut grid,
    mut sx: u_int,
    mut yy: u_int,
    mut at: u_int,
) {
    let mut gl: *mut grid_line = (*gd).linedata.offset(yy as isize) as *mut grid_line;
    let mut first: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
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
    let mut line: u_int = 0;
    let mut lines: u_int = 0;
    let mut width: u_int = 0;
    let mut i: u_int = 0;
    let mut xx: u_int = 0;
    let mut used: u_int = (*gl).cellused as u_int;
    let mut flags: ::core::ffi::c_int = (*gl).flags as ::core::ffi::c_int;
    if !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_EXTENDED != 0 {
        lines = (1 as u_int).wrapping_add(
            (((*gl).cellused as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u_int)
                .wrapping_div(sx),
        );
    } else {
        lines = 2 as u_int;
        width = 0 as u_int;
        i = at;
        while i < used {
            grid_get_cell1(gl, i, &raw mut gc);
            if width.wrapping_add(gc.data.width as u_int) > sx {
                lines = lines.wrapping_add(1);
                width = 0 as u_int;
            }
            width = width.wrapping_add(gc.data.width as u_int);
            i = i.wrapping_add(1);
        }
    }
    line = (*target).sy.wrapping_add(1 as u_int);
    first = grid_reflow_add(target, lines);
    width = 0 as u_int;
    xx = 0 as u_int;
    i = at;
    while i < used {
        grid_get_cell1(gl, i, &raw mut gc);
        if width.wrapping_add(gc.data.width as u_int) > sx {
            let ref mut fresh44 = (*(*target).linedata.offset(line as isize)).flags;
            *fresh44 = (*fresh44 as ::core::ffi::c_int | GRID_LINE_WRAPPED) as u_short;
            line = line.wrapping_add(1);
            width = 0 as u_int;
            xx = 0 as u_int;
        }
        width = width.wrapping_add(gc.data.width as u_int);
        grid_set_cell(target, xx, line, &raw mut gc);
        xx = xx.wrapping_add(1);
        i = i.wrapping_add(1);
    }
    if flags & GRID_LINE_WRAPPED != 0 {
        let ref mut fresh45 = (*(*target).linedata.offset(line as isize)).flags;
        *fresh45 = (*fresh45 as ::core::ffi::c_int | GRID_LINE_WRAPPED) as u_short;
    }
    (*gl).cellused = at as u_short;
    (*gl).cellsize = (*gl).cellused;
    (*gl).flags = ((*gl).flags as ::core::ffi::c_int | GRID_LINE_WRAPPED) as u_short;
    memcpy(
        first as *mut ::core::ffi::c_void,
        gl as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_line>() as size_t,
    );
    grid_reflow_dead(gl);
    if yy <= (*gd).hscrolled {
        (*gd).hscrolled = (*gd).hscrolled.wrapping_add(lines.wrapping_sub(1 as u_int));
    }
    if width < sx && flags & GRID_LINE_WRAPPED != 0 {
        grid_reflow_join(target, gd, sx, yy, width, 1 as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_reflow(mut gd: *mut grid, mut sx: u_int) {
    let mut target: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
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
    let mut yy: u_int = 0;
    let mut width: u_int = 0;
    let mut i: u_int = 0;
    let mut at: u_int = 0;
    target = grid_create((*gd).sx, 0 as u_int, 0 as u_int);
    yy = 0 as u_int;
    while yy < (*gd).hsize.wrapping_add((*gd).sy) {
        gl = (*gd).linedata.offset(yy as isize) as *mut grid_line;
        if !((*gl).flags as ::core::ffi::c_int & GRID_LINE_DEAD != 0) {
            width = 0 as u_int;
            at = width;
            if !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_EXTENDED != 0 {
                width = (*gl).cellused as u_int;
                if width > sx {
                    at = sx;
                } else {
                    at = width;
                }
            } else {
                i = 0 as u_int;
                while i < (*gl).cellused as u_int {
                    grid_get_cell1(gl, i, &raw mut gc);
                    if at == 0 as u_int && width.wrapping_add(gc.data.width as u_int) > sx {
                        at = i;
                    }
                    width = width.wrapping_add(gc.data.width as u_int);
                    i = i.wrapping_add(1);
                }
            }
            if width == sx {
                grid_reflow_move(target, gl);
            } else if width > sx {
                grid_reflow_split(target, gd, sx, yy, at);
            } else if (*gl).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0 {
                grid_reflow_join(target, gd, sx, yy, width, 0 as ::core::ffi::c_int);
            } else {
                grid_reflow_move(target, gl);
            }
        }
        yy = yy.wrapping_add(1);
    }
    if (*target).sy < (*gd).sy {
        grid_reflow_add(target, (*gd).sy.wrapping_sub((*target).sy));
    }
    (*gd).hsize = (*target).sy.wrapping_sub((*gd).sy);
    if (*gd).hscrolled > (*gd).hsize {
        (*gd).hscrolled = (*gd).hsize;
    }
    free((*gd).linedata as *mut ::core::ffi::c_void);
    (*gd).linedata = (*target).linedata;
    free(target as *mut ::core::ffi::c_void);
    (*gd).scroll_generation = (*gd).scroll_generation.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn grid_wrap_position(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut wx: *mut u_int,
    mut wy: *mut u_int,
) {
    let mut ax: u_int = 0 as u_int;
    let mut ay: u_int = 0 as u_int;
    let mut yy: u_int = 0;
    yy = 0 as u_int;
    while yy < py {
        if (*(*gd).linedata.offset(yy as isize)).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED
            != 0
        {
            ax = ax.wrapping_add((*(*gd).linedata.offset(yy as isize)).cellused as u_int);
        } else {
            ax = 0 as u_int;
            ay = ay.wrapping_add(1);
        }
        yy = yy.wrapping_add(1);
    }
    if px >= (*(*gd).linedata.offset(yy as isize)).cellused as u_int {
        ax = UINT_MAX as u_int;
    } else {
        ax = ax.wrapping_add(px);
    }
    *wx = ax;
    *wy = ay;
}
#[no_mangle]
pub unsafe extern "C" fn grid_unwrap_position(
    mut gd: *mut grid,
    mut px: *mut u_int,
    mut py: *mut u_int,
    mut wx: u_int,
    mut wy: u_int,
) {
    let mut yy: u_int = 0;
    let mut ay: u_int = 0 as u_int;
    let mut ey: u_int = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    yy = 0 as u_int;
    while yy < (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int) {
        if ay == wy {
            break;
        }
        if !((*(*gd).linedata.offset(yy as isize)).flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED
            != 0
        {
            ay = ay.wrapping_add(1);
        }
        yy = yy.wrapping_add(1);
    }
    if wx == UINT_MAX {
        while yy < ey
            && (*(*gd).linedata.offset(yy as isize)).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED
                != 0
        {
            yy = yy.wrapping_add(1);
        }
        wx = (*(*gd).linedata.offset(yy as isize)).cellused as u_int;
    } else {
        while (*(*gd).linedata.offset(yy as isize)).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED
            != 0
        {
            if wx < (*(*gd).linedata.offset(yy as isize)).cellused as u_int {
                break;
            }
            wx = wx.wrapping_sub((*(*gd).linedata.offset(yy as isize)).cellused as u_int);
            yy = yy.wrapping_add(1);
        }
    }
    *px = wx;
    *py = yy;
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_length(mut gd: *mut grid, mut py: u_int) -> u_int {
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
    let mut px: u_int = 0;
    px = (*grid_get_line(gd, py)).cellsize as u_int;
    if px > (*gd).sx {
        px = (*gd).sx;
    }
    while px > 0 as u_int {
        grid_get_cell(gd, px.wrapping_sub(1 as u_int), py, &raw mut gc);
        if gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0
            || gc.data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
            || *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int != ' ' as i32
        {
            break;
        }
        px = px.wrapping_sub(1);
    }
    return px;
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_limit(mut gd: *mut grid, mut py: u_int) -> u_int {
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
    let mut px: u_int = 0;
    px = grid_line_length(gd, py);
    if px == 0 as u_int {
        return 0 as u_int;
    }
    px = px.wrapping_sub(1);
    while px > 0 as u_int {
        grid_get_cell(gd, px, py, &raw mut gc);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            break;
        }
        px = px.wrapping_sub(1);
    }
    return px;
}
#[no_mangle]
pub unsafe extern "C" fn grid_in_set(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut set: *const ::core::ffi::c_char,
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
    let mut tmp_gc: grid_cell = grid_cell {
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
    let mut pxx: u_int = 0;
    let mut has_tab: ::core::ffi::c_int = 0;
    let mut has_space: ::core::ffi::c_int = 0;
    has_tab = (strchr(set, '\t' as i32) != NULL as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
    has_space = (strchr(set, ' ' as i32) != NULL as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
    grid_get_cell(gd, px, py, &raw mut gc);
    if gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        if has_tab == 0 && has_space == 0 {
            return 0 as ::core::ffi::c_int;
        }
        pxx = px;
        loop {
            pxx = pxx.wrapping_sub(1);
            grid_get_cell(gd, pxx, py, &raw mut tmp_gc);
            if !(pxx > 0 as u_int && tmp_gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
                break;
            }
        }
        if (has_tab != 0 || has_space != 0)
            && tmp_gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0
            || has_space != 0 && utf8_has_whitespace(&raw mut tmp_gc.data) != 0
        {
            return (tmp_gc.data.width as u_int).wrapping_sub(px.wrapping_sub(pxx))
                as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
    if (has_tab != 0 || has_space != 0) && gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        return gc.data.width as ::core::ffi::c_int;
    }
    if has_space != 0 && utf8_has_whitespace(&raw mut gc.data) != 0 {
        return if gc.data.width as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            gc.data.width as ::core::ffi::c_int
        };
    }
    return utf8_cstrhas(set, &raw mut gc.data);
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_flags_string(
    mut flags: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 128] = [0; 128];
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if flags & GRID_LINE_WRAPPED != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"WRAPPED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_EXTENDED != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"EXTENDED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_DEAD != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"DEAD,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_START_PROMPT != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"START_PROMPT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_SECOND_PROMPT != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"SECOND_PROMPT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_START_COMMAND != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"START_COMMAND,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_START_OUTPUT != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"START_OUTPUT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_END_OUTPUT != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"END_OUTPUT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_HYPERLINK != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"HYPERLINK,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int == '\0' as i32 {
        return b"NONE\0" as *const u8 as *const ::core::ffi::c_char;
    }
    s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
        '\0' as i32 as ::core::ffi::c_char;
    return &raw mut s as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn grid_cell_flags_string(
    mut flags: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 128] = [0; 128];
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if flags & GRID_FLAG_FG256 != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"FG256,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_BG256 != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"BG256,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_PADDING != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"PADDING,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_EXTENDED != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"EXTENDED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_SELECTED != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"SELECTED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_CLEARED != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"CLEARED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_TAB != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"TAB,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_NOPALETTE != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"NOPALETTE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int == '\0' as i32 {
        return b"NONE\0" as *const u8 as *const ::core::ffi::c_char;
    }
    s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
        '\0' as i32 as ::core::ffi::c_char;
    return &raw mut s as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn grid_cell_attr_string(
    mut attr: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 256] = [0; 256];
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if attr & GRID_ATTR_CHARSET != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"CHARSET,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_BRIGHT != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"BRIGHT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_DIM != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"DIM,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_UNDERSCORE != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"UNDERSCORE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_BLINK != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"BLINK,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_REVERSE != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"REVERSE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_HIDDEN != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"HIDDEN,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_ITALICS != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"ITALICS,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_STRIKETHROUGH != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"STRIKETHROUGH,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_UNDERSCORE_2 != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"UNDERSCORE_2,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_UNDERSCORE_3 != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"UNDERSCORE_3,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_UNDERSCORE_4 != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"UNDERSCORE_4,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_UNDERSCORE_5 != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"UNDERSCORE_5,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_OVERLINE != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"OVERLINE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int == '\0' as i32 {
        return b"NONE\0" as *const u8 as *const ::core::ffi::c_char;
    }
    s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
        '\0' as i32 as ::core::ffi::c_char;
    return &raw mut s as *mut ::core::ffi::c_char;
}
