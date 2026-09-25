use crate::src::ffi::libc::memcmp;
use crate::src::grid::{
    grid_get_cell, grid_get_line, grid_in_set, grid_line_length, grid_line_limit,
};
use crate::src::shared::abi::*;
use crate::src::shared::grid::grid_reader;
use crate::src::shared::grid::WHITESPACE;
use crate::src::shared::grid::*;

#[no_mangle]
pub unsafe extern "C" fn grid_reader_start(
    mut gr: *mut grid_reader,
    mut gd: *mut grid,
    mut cx: u_int,
    mut cy: u_int,
) {
    (*gr).gd = gd;
    (*gr).cx = cx;
    (*gr).cy = cy;
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_get_cursor(
    mut gr: *mut grid_reader,
    mut cx: *mut u_int,
    mut cy: *mut u_int,
) {
    *cx = (*gr).cx;
    *cy = (*gr).cy;
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_line_length(mut gr: *mut grid_reader) -> u_int {
    return grid_line_length((*gr).gd, (*gr).cy);
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_cursor_right(
    mut gr: *mut grid_reader,
    mut wrap: ::core::ffi::c_int,
    mut all: ::core::ffi::c_int,
    mut onemore: ::core::ffi::c_int,
) {
    let mut px: u_int = 0;
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
    if all != 0 {
        px = (*(*gr).gd).sx;
    } else if onemore != 0 {
        px = grid_reader_line_length(gr);
    } else {
        px = grid_line_limit((*gr).gd, (*gr).cy);
    }
    if wrap != 0
        && (*gr).cx >= px
        && (*gr).cy
            < (*(*gr).gd)
                .hsize
                .wrapping_add((*(*gr).gd).sy)
                .wrapping_sub(1 as u_int)
    {
        grid_reader_cursor_start_of_line(gr, 0 as ::core::ffi::c_int);
        grid_reader_cursor_down(gr);
    } else if (*gr).cx < px {
        (*gr).cx = (*gr).cx.wrapping_add(1);
        while (*gr).cx < px {
            grid_get_cell((*gr).gd, (*gr).cx, (*gr).cy, &raw mut gc);
            if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            (*gr).cx = (*gr).cx.wrapping_add(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_cursor_left(
    mut gr: *mut grid_reader,
    mut wrap: ::core::ffi::c_int,
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
    while (*gr).cx > 0 as u_int {
        grid_get_cell((*gr).gd, (*gr).cx, (*gr).cy, &raw mut gc);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            break;
        }
        (*gr).cx = (*gr).cx.wrapping_sub(1);
    }
    if (*gr).cx == 0 as u_int
        && (*gr).cy > 0 as u_int
        && (wrap != 0
            || (*grid_get_line((*gr).gd, (*gr).cy.wrapping_sub(1 as u_int))).flags
                as ::core::ffi::c_int
                & GRID_LINE_WRAPPED
                != 0)
    {
        grid_reader_cursor_up(gr);
        grid_reader_cursor_end_of_line(gr, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    } else if (*gr).cx > 0 as u_int {
        (*gr).cx = (*gr).cx.wrapping_sub(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_cursor_down(mut gr: *mut grid_reader) {
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
    if (*gr).cy
        < (*(*gr).gd)
            .hsize
            .wrapping_add((*(*gr).gd).sy)
            .wrapping_sub(1 as u_int)
    {
        (*gr).cy = (*gr).cy.wrapping_add(1);
    }
    while (*gr).cx > 0 as u_int {
        grid_get_cell((*gr).gd, (*gr).cx, (*gr).cy, &raw mut gc);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            break;
        }
        (*gr).cx = (*gr).cx.wrapping_sub(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_cursor_up(mut gr: *mut grid_reader) {
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
    if (*gr).cy > 0 as u_int {
        (*gr).cy = (*gr).cy.wrapping_sub(1);
    }
    while (*gr).cx > 0 as u_int {
        grid_get_cell((*gr).gd, (*gr).cx, (*gr).cy, &raw mut gc);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            break;
        }
        (*gr).cx = (*gr).cx.wrapping_sub(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_cursor_start_of_line(
    mut gr: *mut grid_reader,
    mut wrap: ::core::ffi::c_int,
) {
    if wrap != 0 {
        while (*gr).cy > 0 as u_int
            && (*grid_get_line((*gr).gd, (*gr).cy.wrapping_sub(1 as u_int))).flags
                as ::core::ffi::c_int
                & GRID_LINE_WRAPPED
                != 0
        {
            (*gr).cy = (*gr).cy.wrapping_sub(1);
        }
    }
    (*gr).cx = 0 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_cursor_end_of_line(
    mut gr: *mut grid_reader,
    mut wrap: ::core::ffi::c_int,
    mut all: ::core::ffi::c_int,
) {
    let mut yy: u_int = 0;
    if wrap != 0 {
        yy = (*(*gr).gd)
            .hsize
            .wrapping_add((*(*gr).gd).sy)
            .wrapping_sub(1 as u_int);
        while (*gr).cy < yy
            && (*grid_get_line((*gr).gd, (*gr).cy)).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED
                != 0
        {
            (*gr).cy = (*gr).cy.wrapping_add(1);
        }
    }
    if all != 0 {
        (*gr).cx = (*(*gr).gd).sx;
    } else {
        (*gr).cx = grid_reader_line_length(gr);
    };
}
unsafe extern "C" fn grid_reader_handle_wrap(
    mut gr: *mut grid_reader,
    mut xx: *mut u_int,
    mut yy: *mut u_int,
) -> ::core::ffi::c_int {
    while (*gr).cx > *xx {
        if (*gr).cy == *yy {
            return 0 as ::core::ffi::c_int;
        }
        grid_reader_cursor_start_of_line(gr, 0 as ::core::ffi::c_int);
        grid_reader_cursor_down(gr);
        if (*grid_get_line((*gr).gd, (*gr).cy)).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0
        {
            *xx = (*(*gr).gd).sx.wrapping_sub(1 as u_int);
        } else {
            *xx = grid_reader_line_length(gr);
        }
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_in_set(
    mut gr: *mut grid_reader,
    mut set: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return grid_in_set((*gr).gd, (*gr).cx, (*gr).cy, set);
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_cursor_next_word(
    mut gr: *mut grid_reader,
    mut separators: *const ::core::ffi::c_char,
) {
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    let mut width: u_int = 0;
    if (*grid_get_line((*gr).gd, (*gr).cy)).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0 {
        xx = (*(*gr).gd).sx.wrapping_sub(1 as u_int);
    } else {
        xx = grid_reader_line_length(gr);
    }
    yy = (*(*gr).gd)
        .hsize
        .wrapping_add((*(*gr).gd).sy)
        .wrapping_sub(1 as u_int);
    if grid_reader_handle_wrap(gr, &raw mut xx, &raw mut yy) == 0 {
        return;
    }
    if grid_reader_in_set(gr, WHITESPACE.as_ptr()) == 0 {
        if grid_reader_in_set(gr, separators) != 0 {
            loop {
                (*gr).cx = (*gr).cx.wrapping_add(1);
                if !(grid_reader_handle_wrap(gr, &raw mut xx, &raw mut yy) != 0
                    && grid_reader_in_set(gr, separators) != 0
                    && grid_reader_in_set(gr, WHITESPACE.as_ptr()) == 0)
                {
                    break;
                }
            }
        } else {
            loop {
                (*gr).cx = (*gr).cx.wrapping_add(1);
                if !(grid_reader_handle_wrap(gr, &raw mut xx, &raw mut yy) != 0
                    && !(grid_reader_in_set(gr, separators) != 0
                        || grid_reader_in_set(gr, WHITESPACE.as_ptr()) != 0))
                {
                    break;
                }
            }
        }
    }
    while grid_reader_handle_wrap(gr, &raw mut xx, &raw mut yy) != 0 && {
        width = grid_reader_in_set(gr, WHITESPACE.as_ptr()) as u_int;
        width != 0
    } {
        (*gr).cx = (*gr).cx.wrapping_add(width);
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_cursor_next_word_end(
    mut gr: *mut grid_reader,
    mut separators: *const ::core::ffi::c_char,
) {
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    if (*grid_get_line((*gr).gd, (*gr).cy)).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0 {
        xx = (*(*gr).gd).sx.wrapping_sub(1 as u_int);
    } else {
        xx = grid_reader_line_length(gr);
    }
    yy = (*(*gr).gd)
        .hsize
        .wrapping_add((*(*gr).gd).sy)
        .wrapping_sub(1 as u_int);
    while grid_reader_handle_wrap(gr, &raw mut xx, &raw mut yy) != 0 {
        if grid_reader_in_set(gr, WHITESPACE.as_ptr()) != 0 {
            (*gr).cx = (*gr).cx.wrapping_add(1);
        } else if grid_reader_in_set(gr, separators) != 0 {
            loop {
                (*gr).cx = (*gr).cx.wrapping_add(1);
                if !(grid_reader_handle_wrap(gr, &raw mut xx, &raw mut yy) != 0
                    && grid_reader_in_set(gr, separators) != 0
                    && grid_reader_in_set(gr, WHITESPACE.as_ptr()) == 0)
                {
                    break;
                }
            }
            return;
        } else {
            loop {
                (*gr).cx = (*gr).cx.wrapping_add(1);
                if !(grid_reader_handle_wrap(gr, &raw mut xx, &raw mut yy) != 0
                    && !(grid_reader_in_set(gr, WHITESPACE.as_ptr()) != 0
                        || grid_reader_in_set(gr, separators) != 0))
                {
                    break;
                }
            }
            return;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_cursor_previous_word(
    mut gr: *mut grid_reader,
    mut separators: *const ::core::ffi::c_char,
    mut already: ::core::ffi::c_int,
    mut stop_at_eol: ::core::ffi::c_int,
) {
    let mut oldx: ::core::ffi::c_int = 0;
    let mut oldy: ::core::ffi::c_int = 0;
    let mut at_eol: ::core::ffi::c_int = 0;
    let mut word_is_letters: ::core::ffi::c_int = 0;
    if already != 0 || grid_reader_in_set(gr, WHITESPACE.as_ptr()) != 0 {
        loop {
            if (*gr).cx > 0 as u_int {
                (*gr).cx = (*gr).cx.wrapping_sub(1);
                if !(grid_reader_in_set(gr, WHITESPACE.as_ptr()) == 0) {
                    continue;
                }
                word_is_letters = (grid_reader_in_set(gr, separators) == 0) as ::core::ffi::c_int;
                break;
            } else {
                if (*gr).cy == 0 as u_int {
                    return;
                }
                grid_reader_cursor_up(gr);
                grid_reader_cursor_end_of_line(
                    gr,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                if !(stop_at_eol != 0 && (*gr).cx > 0 as u_int) {
                    continue;
                }
                oldx = (*gr).cx as ::core::ffi::c_int;
                (*gr).cx = (*gr).cx.wrapping_sub(1);
                at_eol = grid_reader_in_set(gr, WHITESPACE.as_ptr());
                (*gr).cx = oldx as u_int;
                if !(at_eol != 0) {
                    continue;
                }
                word_is_letters = 0 as ::core::ffi::c_int;
                break;
            }
        }
    } else {
        word_is_letters = (grid_reader_in_set(gr, separators) == 0) as ::core::ffi::c_int;
    }
    loop {
        oldx = (*gr).cx as ::core::ffi::c_int;
        oldy = (*gr).cy as ::core::ffi::c_int;
        if (*gr).cx == 0 as u_int {
            if (*gr).cy == 0 as u_int
                || !((*grid_get_line((*gr).gd, (*gr).cy.wrapping_sub(1 as u_int))).flags
                    as ::core::ffi::c_int)
                    & GRID_LINE_WRAPPED
                    != 0
            {
                break;
            }
            grid_reader_cursor_up(gr);
            grid_reader_cursor_end_of_line(gr, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        }
        if (*gr).cx > 0 as u_int {
            (*gr).cx = (*gr).cx.wrapping_sub(1);
        }
        if !(grid_reader_in_set(gr, WHITESPACE.as_ptr()) == 0
            && word_is_letters != grid_reader_in_set(gr, separators))
        {
            break;
        }
    }
    (*gr).cx = oldx as u_int;
    (*gr).cy = oldy as u_int;
}
unsafe extern "C" fn grid_reader_cell_equals_data(
    mut gc: *const grid_cell,
    mut ud: *const utf8_data,
) -> ::core::ffi::c_int {
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0
        && (*ud).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && *(&raw const (*ud).data as *const u_char) as ::core::ffi::c_int == '\t' as i32
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*gc).data.size as ::core::ffi::c_int != (*ud).size as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return (memcmp(
        &raw const (*gc).data.data as *const u_char as *const ::core::ffi::c_void,
        &raw const (*ud).data as *const u_char as *const ::core::ffi::c_void,
        (*gc).data.size as size_t,
    ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_cursor_jump(
    mut gr: *mut grid_reader,
    mut jc: *const utf8_data,
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
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    px = (*gr).cx;
    yy = (*(*gr).gd)
        .hsize
        .wrapping_add((*(*gr).gd).sy)
        .wrapping_sub(1 as u_int);
    py = (*gr).cy;
    while py <= yy {
        xx = grid_line_length((*gr).gd, py);
        while px < xx {
            grid_get_cell((*gr).gd, px, py, &raw mut gc);
            if grid_reader_cell_equals_data(&raw mut gc, jc) != 0 {
                (*gr).cx = px;
                (*gr).cy = py;
                return 1 as ::core::ffi::c_int;
            }
            px = px.wrapping_add(1);
        }
        if py == yy
            || (*grid_get_line((*gr).gd, py)).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED == 0
        {
            return 0 as ::core::ffi::c_int;
        }
        px = 0 as u_int;
        py = py.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_cursor_jump_back(
    mut gr: *mut grid_reader,
    mut jc: *const utf8_data,
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
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut xx: u_int = 0;
    xx = (*gr).cx.wrapping_add(1 as u_int);
    py = (*gr).cy.wrapping_add(1 as u_int);
    while py > 0 as u_int {
        px = xx;
        while px > 0 as u_int {
            grid_get_cell(
                (*gr).gd,
                px.wrapping_sub(1 as u_int),
                py.wrapping_sub(1 as u_int),
                &raw mut gc,
            );
            if grid_reader_cell_equals_data(&raw mut gc, jc) != 0 {
                (*gr).cx = px.wrapping_sub(1 as u_int);
                (*gr).cy = py.wrapping_sub(1 as u_int);
                return 1 as ::core::ffi::c_int;
            }
            px = px.wrapping_sub(1);
        }
        if py == 1 as u_int
            || (*grid_get_line((*gr).gd, py.wrapping_sub(2 as u_int))).flags as ::core::ffi::c_int
                & GRID_LINE_WRAPPED
                == 0
        {
            return 0 as ::core::ffi::c_int;
        }
        xx = grid_line_length((*gr).gd, py.wrapping_sub(2 as u_int));
        py = py.wrapping_sub(1);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn grid_reader_cursor_back_to_indentation(mut gr: *mut grid_reader) {
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
    let mut py: u_int = 0;
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    let mut oldx: u_int = 0;
    let mut oldy: u_int = 0;
    yy = (*(*gr).gd)
        .hsize
        .wrapping_add((*(*gr).gd).sy)
        .wrapping_sub(1 as u_int);
    oldx = (*gr).cx;
    oldy = (*gr).cy;
    grid_reader_cursor_start_of_line(gr, 1 as ::core::ffi::c_int);
    py = (*gr).cy;
    while py <= yy {
        xx = grid_line_length((*gr).gd, py);
        px = 0 as u_int;
        while px < xx {
            grid_get_cell((*gr).gd, px, py, &raw mut gc);
            if (gc.data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
                || *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int != ' ' as i32)
                && !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_TAB != 0
                && !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0
            {
                (*gr).cx = px;
                (*gr).cy = py;
                return;
            }
            px = px.wrapping_add(1);
        }
        if !((*grid_get_line((*gr).gd, py)).flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
            break;
        }
        py = py.wrapping_add(1);
    }
    (*gr).cx = oldx;
    (*gr).cy = oldy;
}
