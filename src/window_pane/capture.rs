#[cfg(test)]
use crate::src::server_client::ClientFixture as _;
use super::WindowPane;
use crate::src::arguments::{args_get, args_has, args_strtonum_and_expand_result};
use crate::src::ffi::libc::snprintf;
use crate::src::grid::{
    grid_cell_attr_display, grid_cell_flags_display, grid_clear_history, grid_default_cell,
    grid_get_cell, grid_get_line, grid_line_flags_display, grid_line_time, grid_peek_line,
    grid_string_cells_bytes,
};
use crate::src::hyperlinks::hyperlinks_get;
use crate::src::input::input_pending;
use crate::src::reactor::{evbuffer_get_length, evbuffer_pullup};
use crate::src::screen::screen_reset_hyperlinks;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::colour::COLOUR_FLAG_256;
use crate::src::shared::command::cmdq_item;
use crate::src::shared::command::*;
use crate::src::shared::grid::*;
use crate::src::shared::limits::{INT_MIN, SHRT_MAX};
use crate::src::shared::pane::window_pane;
use crate::src::shared::screen::screen;
use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
use crate::src::style::colour::colour_format;
use crate::src::text::utf8::utf8_strvis;
use std::ffi::{CStr, CString};
use std::io::Write;
use std::{cell::UnsafeCell, rc::Rc};

#[derive(Clone, Copy)]
enum CaptureBound {
    Edge,
    Value(i32),
    Default,
}
unsafe fn parse_bound(
    arguments: &mut args,
    item: &Rc<UnsafeCell<cmdq_item>>,
    flag: u8,
) -> CaptureBound {
    if args_get(arguments, flag).is_some_and(|value| value == c"-") {
        CaptureBound::Edge
    } else {
        match args_strtonum_and_expand_result(
            arguments,
            flag,
            INT_MIN as i64,
            SHRT_MAX as i64,
            Some(item),
        ) {
            Ok(value) => CaptureBound::Value(value as i32),
            Err(_) => CaptureBound::Default,
        }
    }
}
fn resolve_bound(bound: CaptureBound, history: u32, height: u32, end: bool) -> u32 {
    let last = history.wrapping_add(height).wrapping_sub(1);
    let line = match bound {
        CaptureBound::Edge if end => last,
        CaptureBound::Edge => 0,
        CaptureBound::Default if end => last,
        CaptureBound::Default => history,
        CaptureBound::Value(value) if value < 0 && value.unsigned_abs() > history => 0,
        CaptureBound::Value(value) => history.wrapping_add(value as u32),
    };
    line.min(last)
}

pub(super) unsafe fn capture(
    owner: &Rc<UnsafeCell<window_pane>>,
    arguments: &mut args,
    item: &Rc<UnsafeCell<cmdq_item>>,
) -> Result<Vec<u8>, CString> {
    if args_has(arguments, b'R') != 0 {
        return Ok(cmd_capture_pane_grid(&*owner.get()));
    }
    if args_has(arguments, b'P') != 0 && args_has(arguments, b'H') == 0 {
        return Ok(cmd_capture_pane_pending(arguments, &mut *owner.get()));
    }
    if args_has(arguments, b'a') != 0 && (*owner.get()).base.saved_grid.is_none() {
        return if args_has(arguments, b'q') == 0 {
            Err(c"no alternate screen".to_owned())
        } else {
            Ok(Vec::new())
        };
    }
    // Format expansion may replace modes, screens, or history. Select the current
    // screen only after both expansions, before borrowing its grid or link table.
    let bounds = (
        parse_bound(arguments, item, b'S'),
        parse_bound(arguments, item, b'E'),
    );
    let active = (*owner.get()).active_mode_entry();
    let get_screen =
        if args_has(arguments, b'a') == 0 && args_has(arguments, b'M') != 0 && active.is_alive() {
            active.get_unchecked().mode.get_screen
        } else {
            None
        };
    let mode_screen = get_screen.map_or(std::ptr::null_mut(), |get_screen| get_screen(active));
    cmd_capture_pane_history(arguments, &*owner.get(), mode_screen, bounds)
}

pub(super) unsafe fn clear_history(owner: &Rc<UnsafeCell<window_pane>>, reset_links: bool) {
    owner.reset_all_modes();
    let pane = &mut *owner.get();
    grid_clear_history(pane.base.grid_mut());
    if reset_links {
        screen_reset_hyperlinks(&mut *pane.screen_ptr());
    }
}

fn cmd_capture_pane_append(buf: &mut Vec<u8>, line: &[u8]) {
    buf.extend_from_slice(line);
}
fn cmd_capture_pane_colour(value: ::core::ffi::c_int) -> CString {
    let mut bytes = colour_format(value).into_bytes();
    bytes.extend_from_slice(format!("[{:x}]", value as u32).as_bytes());
    CString::new(bytes).expect("colour text and hexadecimal suffix contain no NUL")
}
unsafe fn cmd_capture_pane_cell(s: &screen, xx: u_int, yy: u_int) -> CString {
    let gd = s.grid();
    let hl = s.hyperlinks.as_ref();
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
    let mut flags: u_int = 0;
    grid_get_cell(&*gd, xx, yy, &mut gc);
    let bytes = &gc.data.data[..gc.data.size as usize];
    let source = &bytes[..bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len())];
    let mut data = vec![0u8; 4 * (source.len() + 1)];
    let data_len = utf8_strvis(&mut data, source, VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL);
    data.truncate(data_len);
    let entry = if gc.link == 0 {
        None
    } else {
        hl.and_then(|table| hyperlinks_get(table, gc.link))
    };
    let (link, linkid) = if let Some(link) = entry {
        let id = if link.internal_id.as_bytes().is_empty() {
            c"NONE"
        } else {
            link.internal_id.as_c_str()
        };
        (link.uri.clone(), id.to_owned())
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
    write!(
        &mut line,
        ") flags={}[{:x}] attr={}[{:x}] fg=",
        grid_cell_flags_display(flags as i32),
        flags,
        grid_cell_attr_display(gc.attr as i32),
        gc.attr,
    )
    .expect("writing to a byte vector succeeds");
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
    let gd = s.grid();
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
        let gl = grid_get_line(&*gd, yy);
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
        let od = &gl.osc133_data;
        let mut row = Vec::new();
        write!(&mut row, "\tL {} (", yy).expect("writing to a byte vector succeeds");
        row.extend_from_slice(CStr::from_ptr(p.as_ptr()).to_bytes());
        write!(
            &mut row,
            ") flags={}",
            grid_line_flags_display(gl.flags as i32)
        )
        .expect("writing to a byte vector succeeds");
        write!(
            &mut row,
            "[{:x}] {}/{}",
            gl.flags as u32, gl.cellused as u32, gl.cellsize as u32
        )
        .expect("writing to a byte vector succeeds");
        if gl.flags as ::core::ffi::c_int & GRID_LINE_OSC133_FLAGS != 0 {
            write!(
                &mut row,
                " osc133={},{},{},{},{}",
                od.prompt_col as u32,
                od.cmd_col as u32,
                od.out_start_col as u32,
                od.out_end_col as u32,
                od.exit_status as u32
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
unsafe fn cmd_capture_pane_pending(args: *mut args, wp: &mut window_pane) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmp: [::core::ffi::c_char; 5] = [0; 5];
    let mut linelen: size_t = 0;
    let mut i: u_int = 0;
    let pending = input_pending(wp.ictx.as_deref_mut().expect("pane input context"));
    line = evbuffer_pullup(pending, -1).map_or(std::ptr::null_mut(), |bytes| bytes.as_mut_ptr())
        as *mut ::core::ffi::c_char;
    linelen = evbuffer_get_length(&*pending);
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
    gd: &grid,
    s: &screen,
    mut py: u_int,
    links: &mut Vec<u_int>,
) -> Vec<u8> {
    let gl = grid_peek_line(gd, py).expect("capture line is in the grid");
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
    let mut line = Vec::new();
    let mut i: u_int = 0;
    if s.hyperlinks.is_none() || !(gl.flags as ::core::ffi::c_int) & GRID_LINE_HYPERLINK != 0 {
        return line;
    }
    i = 0 as u_int;
    while i < gl.cellused as u_int {
        grid_get_cell(gd, i, py, &mut gc);
        if !(gc.link == 0 as u_int) {
            if !links.contains(&gc.link) {
                if let Some(link) = hyperlinks_get(
                    s.hyperlinks.as_ref().expect("screen hyperlink table"),
                    gc.link,
                ) {
                    if links.len() == gd.sx as usize {
                        break;
                    }
                    links.push(gc.link);
                    if !line.is_empty() {
                        cmd_capture_pane_append(&mut line, b" ");
                    }
                    cmd_capture_pane_append(&mut line, link.uri.as_bytes());
                }
            }
        }
        i = i.wrapping_add(1);
    }
    line
}
unsafe fn cmd_capture_pane_history(
    mut args: *mut args,
    wp: &window_pane,
    mode_screen: *mut screen,
    bounds: (CaptureBound, CaptureBound),
) -> Result<Vec<u8>, CString> {
    let mut gc = grid_default_cell;
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
    sx = wp.base.grid().sx;
    let (gd, s) = if args_has(args, b'a') != 0 {
        let Some(saved) = wp.base.saved_grid.as_deref() else {
            if args_has(args, b'q') == 0 {
                return Err(c"no alternate screen".to_owned());
            }
            return Ok(buf);
        };
        (saved, &wp.base)
    } else {
        let s = mode_screen.as_ref().unwrap_or(&wp.base);
        (s.grid(), s)
    };
    top = resolve_bound(bounds.0, gd.hsize, gd.sy, false);
    bottom = resolve_bound(bounds.1, gd.hsize, gd.sy, true);
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
        links = Vec::with_capacity(gd.sx as usize);
    }
    i = top;
    while i <= bottom {
        let line = if hyperlinks != 0 {
            cmd_capture_pane_hyperlinks(gd, s, i, &mut links)
        } else {
            let mut line =
                grid_string_cells_bytes(gd, 0 as u_int, i, sx, Some(&mut gc), flags, Some(s));
            if let Some(nul) = line.iter().position(|&byte| byte == 0) {
                line.truncate(nul);
            }
            line
        };
        if hyperlinks == 0 || !line.is_empty() {
            let gl = grid_peek_line(gd, i).expect("capture range is in the grid");
            if number_lines != 0 {
                if i >= gd.hsize {
                    n = i.wrapping_sub(gd.hsize) as ::core::ffi::c_int;
                } else {
                    n = i as ::core::ffi::c_int - gd.hsize as ::core::ffi::c_int;
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
                if gl.flags as ::core::ffi::c_int & GRID_LINE_DEAD != 0 {
                    let fresh0 = cp;
                    cp = cp.offset(1);
                    *fresh0 = 'D' as i32 as ::core::ffi::c_char;
                }
                if gl.flags as ::core::ffi::c_int & GRID_LINE_HYPERLINK != 0 {
                    let fresh1 = cp;
                    cp = cp.offset(1);
                    *fresh1 = 'H' as i32 as ::core::ffi::c_char;
                }
                if gl.flags as ::core::ffi::c_int & GRID_LINE_START_OUTPUT != 0 {
                    let fresh2 = cp;
                    cp = cp.offset(1);
                    *fresh2 = 'O' as i32 as ::core::ffi::c_char;
                }
                if gl.flags as ::core::ffi::c_int & GRID_LINE_START_PROMPT != 0 {
                    let fresh3 = cp;
                    cp = cp.offset(1);
                    *fresh3 = 'P' as i32 as ::core::ffi::c_char;
                }
                if gl.flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0 {
                    let fresh4 = cp;
                    cp = cp.offset(1);
                    *fresh4 = 'W' as i32 as ::core::ffi::c_char;
                }
                if gl.flags as ::core::ffi::c_int & GRID_LINE_EXTENDED != 0 {
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
            if join_lines == 0 || gl.flags as ::core::ffi::c_int & GRID_LINE_WRAPPED == 0 {
                buf.push(b'\n');
            }
        }
        i = i.wrapping_add(1);
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::cmd::queue::{cmdq_append, cmdq_get_callback_owned, cmdq_next};
    use crate::src::format::format_add_owned_cb;
    use crate::src::grid::{grid_create, grid_set_cell};
    use crate::src::options::{options_create, options_free};
    use crate::src::screen::screen_free;
    use crate::src::shared::arguments::{args_entry, args_value};
    use crate::src::shared::client::client;
    use crate::src::shared::format::format_tree;
    use crate::src::shared::window::{window_mode, window_mode_entry};
    use std::cell::Cell;

    static REPLACING_MODE: std::sync::LazyLock<window_mode> =
        std::sync::LazyLock::new(|| window_mode {
            name: c"replace-capture-screen-test",
            formats: Some(add_replacement_format),
            ..window_mode::default()
        });

    unsafe fn add_replacement_format(
        entry: refbox::Weak<window_mode_entry>,
        tree: *mut format_tree,
    ) {
        let pane = entry.get_unchecked().wp.clone();
        let calls = entry.get_unchecked().retained_data::<Cell<u32>>().unwrap();
        format_add_owned_cb(tree, c"replace_screen", move |_| {
            assert_eq!(calls.replace(calls.get() + 1), 0);
            let pane = pane.upgrade().unwrap();
            let base = &mut (*pane.get()).base;
            screen_free(base);
            base.grid = Some(grid_create(1, 2, 0));
            let mut cell = grid_default_cell;
            cell.data.size = 1;
            cell.data.width = 1;
            cell.data.data[0] = b'X';
            grid_set_cell(base.grid_mut(), 0, 0, &cell);
            cell.data.data[0] = b'Y';
            grid_set_cell(base.grid_mut(), 0, 1, &cell);
            Some(c"0".to_owned())
        });
    }

    fn set_argument(arguments: &mut args, flag: u8, value: Option<&CStr>) {
        arguments.tree.insert(
            flag,
            Box::new(args_entry {
                flag,
                values: value
                    .map(|value| Box::new(args_value::string(value.to_owned())))
                    .into_iter()
                    .collect(),
                count: 1,
                flags: 0,
            }),
        );
    }

    #[test]
    fn format_replacement_selects_current_screen_and_alternate_errors_skip_expansion() {
        unsafe {
            let pane = window_pane::new();
            (*pane.get()).options = Some(options_create(None));
            (*pane.get()).base.grid = Some(grid_create(3, 1, 0));
            let calls = Rc::new(Cell::new(0u32));
            (*pane.get())
                .modes
                .push(refbox::RefBox::new(window_mode_entry {
                    wp: Rc::downgrade(&pane),
                    swp: Default::default(),
                    mode: &REPLACING_MODE,
                    boxed_data: None,
                    data_owner: Some(calls.clone()),
                    prefix: 0,
                    kill: 0,
                }));
            let item = cmdq_get_callback_owned(c"capture replacement test", None);
            (*item.get()).target.wp = Rc::downgrade(&pane);
            let mut arguments = args::empty();
            set_argument(&mut arguments, b'S', Some(c"#{replace_screen}"));
            set_argument(&mut arguments, b'a', None);

            assert_eq!(
                pane.capture(&mut arguments, &item),
                Err(c"no alternate screen".to_owned())
            );
            set_argument(&mut arguments, b'q', None);
            assert_eq!(pane.capture(&mut arguments, &item), Ok(Vec::new()));
            assert_eq!(calls.get(), 0);
            arguments.tree.remove(&b'a');
            assert_eq!(pane.capture(&mut arguments, &item), Ok(b"X\nY\n".to_vec()));
            assert_eq!(calls.get(), 1);

            let client = crate::src::shared::client::ClientRef::fixture_with_queue();
            (*item.get()).flags |= CMDQ_FIRED;
            cmdq_append(Some(&client), item);
            assert_eq!(cmdq_next(Some(&client)), 0);
            drop(client);
            (*pane.get()).modes.clear();
            screen_free(&mut (*pane.get()).base);
            options_free((*pane.get()).options.take().unwrap());
            pane.release(c"capture replacement test");
        }
    }
}
