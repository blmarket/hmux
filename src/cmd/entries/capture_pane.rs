use crate::src::arguments::{args_get, args_has, args_strtonum_and_expand_result};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::control::control_write;
use crate::src::ffi::libc::{memcpy, snprintf, strcmp};
use crate::src::file::{file_can_print, file_print, file_print_buffer};
use crate::src::grid::{
    grid_cell_attr_string, grid_cell_flags_string, grid_clear_history, grid_get_cell,
    grid_get_line, grid_line_flags_string, grid_line_time, grid_peek_line, grid_string_cells_bytes,
};
use crate::src::hyperlinks::hyperlinks_get;
use crate::src::input::input_pending;
use crate::src::paste::paste_set_owned;
use crate::src::reactor::{evbuffer_get_length, evbuffer_pullup};
use crate::src::screen::screen_reset_hyperlinks;
use crate::src::server_fn::server_redraw_window;
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::colour::COLOUR_FLAG_256;
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::event::*;
use crate::src::shared::grid::*;
use crate::src::shared::hyperlinks::hyperlinks;
use crate::src::shared::limits::{INT_MIN, SHRT_MAX};
use crate::src::shared::pane::window_pane;
use crate::src::shared::screen::screen;
use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
use crate::src::shared::window::{window, window_mode_entry};
use crate::src::style::colour::colour_format;
use crate::src::text::utf8::utf8_strvis;
use crate::src::window::window_pane_reset_mode_all;
use std::ffi::{CStr, CString};
use std::io::Write;
pub static mut cmd_capture_pane_entry: cmd_entry =  {
    cmd_entry {
        name: b"capture-pane\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"capturep\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"ab:CeE:FHIJLMNpPqRS:Tt:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage:
            b"[-aCeFHIJLMNpPqRT] [-b buffer-name] [-E end-line] [-S start-line] [-t target-pane]\0"
                as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_capture_pane_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static mut cmd_clear_history_entry: cmd_entry =  {
    cmd_entry {
        name: b"clear-history\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"clearhist\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"Ht:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-H] [-t target-pane]\0" as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_capture_pane_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
fn cmd_capture_pane_append(buf: &mut Vec<u8>, line: &[u8]) {
    buf.extend_from_slice(line);
}
fn cmd_capture_pane_colour(value: ::core::ffi::c_int) -> CString {
    let mut bytes = colour_format(value).into_bytes();
    bytes.extend_from_slice(format!("[{:x}]", value as u32).as_bytes());
    CString::new(bytes).expect("colour text and hexadecimal suffix contain no NUL")
}
unsafe fn cmd_capture_pane_cell(s: &screen, xx: u_int, yy: u_int) -> CString {
    let mut gd: *mut grid = s.grid;
    let mut hl: *mut hyperlinks = s.hyperlinks;
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
    let mut c: [::core::ffi::c_char; 33] = [0; 33];
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut iid: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut flags: u_int = 0;
    grid_get_cell(gd, xx, yy, &raw mut gc);
    memcpy(
        &raw mut c as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        &raw mut gc.data.data as *mut u_char as *const ::core::ffi::c_void,
        gc.data.size as size_t,
    );
    c[gc.data.size as usize] = '\0' as i32 as ::core::ffi::c_char;
    let source_len = CStr::from_ptr(c.as_ptr()).to_bytes().len();
    // Match utf8_stravis's maximum expansion without allocating a C buffer.
    let mut data = vec![0u8; 4 * (source_len + 1)];
    let data_len = utf8_strvis(
        data.as_mut_ptr().cast(),
        c.as_ptr(),
        source_len,
        VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL,
    );
    data.truncate(data_len);
    let (link, linkid) = if gc.link != 0 as u_int
        && hyperlinks_get(
            hl,
            gc.link,
            &raw mut uri,
            &raw mut iid,
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        ) != 0
    {
        let link = CStr::from_ptr(uri).to_owned();
        let linkid = if !iid.is_null() && *iid as ::core::ffi::c_int != '\0' as i32 {
            CStr::from_ptr(iid).to_owned()
        } else {
            c"NONE".to_owned()
        };
        (link, linkid)
    } else {
        (c"NONE".to_owned(), c"NONE".to_owned())
    };
    flags = gc.flags as u_int;
    if gc.fg & COLOUR_FLAG_256 != 0 {
        flags |= GRID_FLAG_FG256 as u_int;
    }
    if gc.bg & COLOUR_FLAG_256 != 0 {
        flags |= GRID_FLAG_BG256 as u_int;
    }
    let f = cmd_capture_pane_colour(gc.fg);
    let b = cmd_capture_pane_colour(gc.bg);
    let u = cmd_capture_pane_colour(gc.us);
    let mut line = Vec::new();
    write!(
        &mut line,
        "\t\tC {},{} data=({},{},",
        yy, xx, gc.data.width, gc.data.size
    )
    .expect("writing to a byte vector succeeds");
    line.extend_from_slice(&data);
    line.extend_from_slice(b") flags=");
    line.extend_from_slice(CStr::from_ptr(grid_cell_flags_string(flags as i32)).to_bytes());
    write!(&mut line, "[{:x}] attr=", flags).expect("writing to a byte vector succeeds");
    line.extend_from_slice(CStr::from_ptr(grid_cell_attr_string(gc.attr as i32)).to_bytes());
    write!(&mut line, "[{:x}] fg=", gc.attr).expect("writing to a byte vector succeeds");
    line.extend_from_slice(f.to_bytes());
    line.extend_from_slice(b" bg=");
    line.extend_from_slice(b.to_bytes());
    line.extend_from_slice(b" us=");
    line.extend_from_slice(u.to_bytes());
    line.extend_from_slice(b" link=");
    line.extend_from_slice(link.to_bytes());
    line.extend_from_slice(b" linkid=");
    line.extend_from_slice(linkid.to_bytes());
    line.push(b'\n');
    CString::new(line).expect("cell fields contain no NUL")
}
unsafe fn cmd_capture_pane_grid(wp: &window_pane) -> Vec<u8> {
    let s = &wp.base;
    let mut gd: *mut grid = s.grid;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut od: *mut osc133_data = ::core::ptr::null_mut::<osc133_data>();
    let mut buf = Vec::new();
    let mut p: [::core::ffi::c_char; 11] = [0; 11];
    let mut yy: u_int = 0;
    let mut xx: u_int = 0;
    let mut total: u_int = (*gd).hsize.wrapping_add((*gd).sy);
    let header = format!(
        "G {}x{} ({}/{})\n",
        (*gd).sx,
        (*gd).sy,
        (*gd).hsize,
        (*gd).hlimit
    );
    cmd_capture_pane_append(&mut buf, header.as_bytes());
    yy = 0 as u_int;
    while yy < total {
        gl = grid_get_line(gd, yy);
        if yy < (*gd).hsize {
            snprintf(
                &raw mut p as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>() as size_t,
                b"-\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            snprintf(
                &raw mut p as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>() as size_t,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                yy.wrapping_sub((*gd).hsize),
            );
        }
        od = &raw mut (*gl).osc133_data;
        let mut row = Vec::new();
        write!(&mut row, "\tL {} (", yy).expect("writing to a byte vector succeeds");
        row.extend_from_slice(CStr::from_ptr(p.as_ptr()).to_bytes());
        row.extend_from_slice(b") flags=");
        row.extend_from_slice(
            CStr::from_ptr(grid_line_flags_string((*gl).flags as ::core::ffi::c_int)).to_bytes(),
        );
        write!(
            &mut row,
            "[{:x}] {}/{}",
            (*gl).flags as u32,
            (*gl).cellused as u32,
            (*gl).cellsize as u32
        )
        .expect("writing to a byte vector succeeds");
        if (*gl).flags as ::core::ffi::c_int & GRID_LINE_OSC133_FLAGS != 0 {
            write!(
                &mut row,
                " osc133={},{},{},{},{}",
                (*od).prompt_col as u32,
                (*od).cmd_col as u32,
                (*od).out_start_col as u32,
                (*od).out_end_col as u32,
                (*od).exit_status as u32
            )
            .expect("writing to a byte vector succeeds");
        }
        row.push(b'\n');
        cmd_capture_pane_append(&mut buf, &row);
        xx = 0 as u_int;
        while xx < (*gd).sx {
            let cell = cmd_capture_pane_cell(s, xx, yy);
            cmd_capture_pane_append(&mut buf, cell.as_bytes());
            xx = xx.wrapping_add(1);
        }
        yy = yy.wrapping_add(1);
    }
    buf
}
unsafe fn cmd_capture_pane_pending(args: *mut args, wp: &window_pane) -> Vec<u8> {
    let mut pending: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut buf = Vec::new();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmp: [::core::ffi::c_char; 5] = [0; 5];
    let mut linelen: size_t = 0;
    let mut i: u_int = 0;
    pending = input_pending(wp.ictx);
    if pending.is_null() {
        return buf;
    }
    line =
        evbuffer_pullup(pending, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_char;
    linelen = evbuffer_get_length(&*(pending));
    if args_has(args, 'C' as i32 as u_char) != 0 {
        i = 0 as u_int;
        while (i as size_t) < linelen {
            if *line.offset(i as isize) as ::core::ffi::c_int >= ' ' as i32
                && *line.offset(i as isize) as ::core::ffi::c_int != '\\' as i32
            {
                tmp[0 as ::core::ffi::c_int as usize] = *line.offset(i as isize);
                tmp[1 as ::core::ffi::c_int as usize] = '\0' as i32 as ::core::ffi::c_char;
            } else {
                snprintf(
                    &raw mut tmp as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>() as size_t,
                    b"\\%03hho\0" as *const u8 as *const ::core::ffi::c_char,
                    *line.offset(i as isize) as ::core::ffi::c_int,
                );
            }
            cmd_capture_pane_append(&mut buf, CStr::from_ptr(tmp.as_ptr()).to_bytes());
            i = i.wrapping_add(1);
        }
    } else {
        if linelen != 0 {
            cmd_capture_pane_append(&mut buf, std::slice::from_raw_parts(line.cast(), linelen));
        }
    }
    buf
}
unsafe fn cmd_capture_pane_hyperlinks(
    mut gd: *mut grid,
    mut s: *mut screen,
    mut py: u_int,
    links: &mut Vec<u_int>,
) -> Vec<u8> {
    let mut gl: *const grid_line = grid_peek_line(gd, py);
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
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut line = Vec::new();
    let mut i: u_int = 0;
    if (*s).hyperlinks.is_null() || !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_HYPERLINK != 0
    {
        return line;
    }
    i = 0 as u_int;
    while i < (*gl).cellused as u_int {
        grid_get_cell(gd, i, py, &raw mut gc);
        if !(gc.link == 0 as u_int) {
            if !links.contains(&gc.link) {
                if !(hyperlinks_get(
                    (*s).hyperlinks,
                    gc.link,
                    &raw mut uri,
                    ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                ) == 0)
                {
                    if links.len() == (*gd).sx as usize {
                        break;
                    }
                    links.push(gc.link);
                    if !line.is_empty() {
                        cmd_capture_pane_append(&mut line, b" ");
                    }
                    cmd_capture_pane_append(&mut line, CStr::from_ptr(uri).to_bytes());
                }
            }
        }
        i = i.wrapping_add(1);
    }
    line
}
unsafe fn cmd_capture_pane_history(
    mut args: *mut args,
    mut item: *mut cmdq_item,
    wp: &mut window_pane,
) -> Option<Vec<u8>> {
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut gl: *const grid_line = ::core::ptr::null::<grid_line>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let mut gc: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut n: ::core::ffi::c_int = 0;
    let mut join_lines: ::core::ffi::c_int = 0;
    let mut number_lines: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut show_flags: ::core::ffi::c_int = 0;
    let mut show_time: ::core::ffi::c_int = 0;
    let mut hyperlinks: ::core::ffi::c_int = 0;
    let mut links: Vec<u_int> = Vec::new();
    let mut i: u_int = 0;
    let mut sx: u_int = 0;
    let mut top: u_int = 0;
    let mut bottom: u_int = 0;
    let mut tmp: u_int = 0;
    let mut buf = Vec::new();
    let mut b: [::core::ffi::c_char; 64] = [0; 64];
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut Sflag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut Eflag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    sx = (*wp.base.grid).sx;
    if args_has(args, 'a' as i32 as u_char) != 0 {
        gd = wp.base.saved_grid;
        if gd.is_null() {
            if args_has(args, 'q' as i32 as u_char) == 0 {
                cmdq_error(
                    item,
                    b"no alternate screen\0" as *const u8 as *const ::core::ffi::c_char,
                );
                return None;
            }
            return Some(buf);
        }
        s = &raw mut wp.base;
    } else if args_has(args, 'M' as i32 as u_char) != 0 {
        wme = wp.modes.active;
        if !wme.is_null() && (*(*wme).mode).get_screen.is_some() {
            s = (*(*wme).mode)
                .get_screen
                .expect("non-null function pointer")(wme);
            gd = (*s).grid;
        } else {
            s = &raw mut wp.base;
            gd = wp.base.grid;
        }
    } else {
        s = &raw mut wp.base;
        gd = wp.base.grid;
    }
    Sflag = args_get(args, 'S' as i32 as u_char);
    if !Sflag.is_null()
        && strcmp(Sflag, b"-\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        top = 0 as u_int;
    } else {
        match args_strtonum_and_expand_result(
            args,
            'S' as i32 as u_char,
            INT_MIN as ::core::ffi::c_longlong,
            SHRT_MAX as ::core::ffi::c_longlong,
            item,
        ) {
            Ok(value) => {
                n = value as ::core::ffi::c_int;
                if n < 0 as ::core::ffi::c_int && n.unsigned_abs() > (*gd).hsize {
                    top = 0 as u_int;
                } else {
                    top = (*gd).hsize.wrapping_add(n as u_int);
                }
            }
            Err(_) => {
                top = (*gd).hsize;
            }
        }
        if top > (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int) {
            top = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
        }
    }
    Eflag = args_get(args, 'E' as i32 as u_char);
    if !Eflag.is_null()
        && strcmp(Eflag, b"-\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        bottom = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    } else {
        match args_strtonum_and_expand_result(
            args,
            'E' as i32 as u_char,
            INT_MIN as ::core::ffi::c_longlong,
            SHRT_MAX as ::core::ffi::c_longlong,
            item,
        ) {
            Ok(value) => {
                n = value as ::core::ffi::c_int;
                if n < 0 as ::core::ffi::c_int && n.unsigned_abs() > (*gd).hsize {
                    bottom = 0 as u_int;
                } else {
                    bottom = (*gd).hsize.wrapping_add(n as u_int);
                }
            }
            Err(_) => {
                bottom = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
            }
        }
        if bottom > (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int) {
            bottom = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
        }
    }
    if bottom < top {
        tmp = bottom;
        bottom = top;
        top = tmp;
    }
    join_lines = args_has(args, 'J' as i32 as u_char);
    if args_has(args, 'e' as i32 as u_char) != 0 {
        flags |= GRID_STRING_WITH_SEQUENCES;
    }
    if args_has(args, 'C' as i32 as u_char) != 0 {
        flags |= GRID_STRING_ESCAPE_SEQUENCES;
    }
    if join_lines == 0 && args_has(args, 'T' as i32 as u_char) == 0 {
        flags |= GRID_STRING_EMPTY_CELLS;
    }
    if join_lines == 0 && args_has(args, 'N' as i32 as u_char) == 0 {
        flags |= GRID_STRING_TRIM_SPACES;
    }
    number_lines = args_has(args, 'L' as i32 as u_char);
    show_flags = args_has(args, 'F' as i32 as u_char);
    show_time = args_has(args, 'I' as i32 as u_char);
    hyperlinks = args_has(args, 'H' as i32 as u_char);
    if hyperlinks != 0 {
        links = Vec::with_capacity((*gd).sx as usize);
    }
    i = top;
    while i <= bottom {
        let line = if hyperlinks != 0 {
            cmd_capture_pane_hyperlinks(gd, s, i, &mut links)
        } else {
            let mut line = grid_string_cells_bytes(gd, 0 as u_int, i, sx, &raw mut gc, flags, s);
            if let Some(nul) = line.iter().position(|&byte| byte == 0) {
                line.truncate(nul);
            }
            line
        };
        if hyperlinks == 0 || !line.is_empty() {
            gl = grid_peek_line(gd, i);
            if number_lines != 0 {
                if i >= (*gd).hsize {
                    n = i.wrapping_sub((*gd).hsize) as ::core::ffi::c_int;
                } else {
                    n = i as ::core::ffi::c_int - (*gd).hsize as ::core::ffi::c_int;
                }
                n = snprintf(
                    &raw mut b as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                    b"%d \0" as *const u8 as *const ::core::ffi::c_char,
                    n,
                );
                if n >= 0 as ::core::ffi::c_int {
                    cmd_capture_pane_append(
                        &mut buf,
                        &std::slice::from_raw_parts(b.as_ptr().cast(), n as usize),
                    );
                }
            }
            if show_time != 0 {
                n = snprintf(
                    &raw mut b as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                    b"%llu \0" as *const u8 as *const ::core::ffi::c_char,
                    grid_line_time(gl) as ::core::ffi::c_ulonglong,
                );
                if n >= 0 as ::core::ffi::c_int {
                    cmd_capture_pane_append(
                        &mut buf,
                        &std::slice::from_raw_parts(b.as_ptr().cast(), n as usize),
                    );
                }
            }
            if show_flags != 0 {
                cp = &raw mut b as *mut ::core::ffi::c_char;
                *cp = '\0' as i32 as ::core::ffi::c_char;
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_DEAD != 0 {
                    let fresh0 = cp;
                    cp = cp.offset(1);
                    *fresh0 = 'D' as i32 as ::core::ffi::c_char;
                }
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_HYPERLINK != 0 {
                    let fresh1 = cp;
                    cp = cp.offset(1);
                    *fresh1 = 'H' as i32 as ::core::ffi::c_char;
                }
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_START_OUTPUT != 0 {
                    let fresh2 = cp;
                    cp = cp.offset(1);
                    *fresh2 = 'O' as i32 as ::core::ffi::c_char;
                }
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_START_PROMPT != 0 {
                    let fresh3 = cp;
                    cp = cp.offset(1);
                    *fresh3 = 'P' as i32 as ::core::ffi::c_char;
                }
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0 {
                    let fresh4 = cp;
                    cp = cp.offset(1);
                    *fresh4 = 'W' as i32 as ::core::ffi::c_char;
                }
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_EXTENDED != 0 {
                    let fresh5 = cp;
                    cp = cp.offset(1);
                    *fresh5 = 'X' as i32 as ::core::ffi::c_char;
                }
                if &raw mut b as *mut ::core::ffi::c_char == cp {
                    let fresh6 = cp;
                    cp = cp.offset(1);
                    *fresh6 = '-' as i32 as ::core::ffi::c_char;
                }
                let fresh7 = cp;
                cp = cp.offset(1);
                *fresh7 = ' ' as i32 as ::core::ffi::c_char;
                *cp = '\0' as i32 as ::core::ffi::c_char;
                cmd_capture_pane_append(&mut buf, CStr::from_ptr(b.as_ptr()).to_bytes());
            }
            cmd_capture_pane_append(&mut buf, &line);
            if join_lines == 0 || (*gl).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED == 0 {
                buf.push(b'\n');
            }
        }
        i = i.wrapping_add(1);
    }
    Some(buf)
}
unsafe fn cmd_capture_pane_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut c: *mut client = cmdq_get_client(item);
    let mut wp: *mut window_pane = (*cmdq_get_target(item)).wp;
    let mut buf: Vec<u8>;
    let mut cause: Option<CString> = None;
    let mut bufname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if cmd_get_entry(self_0) == &raw const cmd_clear_history_entry {
        window_pane_reset_mode_all(wp);
        grid_clear_history((*wp).base.grid);
        if args_has(args, 'H' as i32 as u_char) != 0 {
            screen_reset_hyperlinks((*wp).screen);
        }
        server_redraw_window((*wp).window as *mut window);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'R' as i32 as u_char) != 0 {
        buf = cmd_capture_pane_grid(&*wp);
    } else if args_has(args, 'P' as i32 as u_char) != 0 && args_has(args, 'H' as i32 as u_char) == 0
    {
        buf = cmd_capture_pane_pending(args, &*wp);
    } else {
        match cmd_capture_pane_history(args, item, &mut *wp) {
            Some(history) => buf = history,
            None => return CMD_RETURN_ERROR,
        }
    }
    if args_has(args, 'p' as i32 as u_char) != 0 {
        let len = if buf.last() == Some(&b'\n') {
            buf.len() - 1
        } else {
            buf.len()
        };
        // Give both C printing paths a valid pointer for empty output. The
        // terminator also preserves control_write's first-NUL behavior.
        buf.push(0);
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_write(
                c,
                b"%.*s\0" as *const u8 as *const ::core::ffi::c_char,
                len as ::core::ffi::c_int,
                buf.as_ptr().cast::<::core::ffi::c_char>(),
            );
        } else {
            if file_can_print(c) == 0 {
                cmdq_error(
                    item,
                    b"can't write to client\0" as *const u8 as *const ::core::ffi::c_char,
                );
                return CMD_RETURN_ERROR;
            }
            file_print_buffer(c, buf.as_mut_ptr().cast(), len);
            file_print(c, b"\n\0" as *const u8 as *const ::core::ffi::c_char);
        }
    } else {
        bufname = ::core::ptr::null::<::core::ffi::c_char>();
        if args_has(args, 'b' as i32 as u_char) != 0 {
            bufname = args_get(args, 'b' as i32 as u_char);
        }
        if paste_set_owned(buf.into_boxed_slice(), bufname, Some(&mut cause))
            != 0 as ::core::ffi::c_int
        {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause.as_ref().unwrap().as_ptr(),
            );
            return CMD_RETURN_ERROR;
        }
    }
    return CMD_RETURN_NORMAL;
}
