use crate::args::RustArguments;
use crate::args::{
    args_get_str, args_has, args_percentage, args_string_str, args_strtonum, args_to_vector,
    args_value_list,
};
use crate::cmd::cmd_get_args;
use crate::cmd::cmdq_item_ref_of;
use crate::environ::EnvironmentStore;
use crate::environ::{RustEnvironment, new_environment_box};
use crate::ffi::strtol;
use crate::fmt_args;
use crate::format::format_single_from_target;
use crate::format::{format_add, format_create_from_target, format_expand};
use crate::log::log_debug;
use crate::options::{OptionsEngine, RustOptionsEngine};
use crate::overlay::menu_create;
use crate::overlay::{menu_add_item_for_client, menu_display_for_client};
use crate::overlay::{popup_display_for_client, popup_modify_for_client, popup_present_for_client};
use crate::server::{client_clear_overlay, client_working_directory};

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    _PATH_BSHELL, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_STRING, BOX_LINES_DEFAULT,
    BOX_LINES_NONE, CMD_AFTERHOOK, CMD_CLIENT_CFLAG, CMD_FIND_PANE, CMD_RETURN_ERROR,
    CMD_RETURN_NORMAL, CMD_RETURN_WAIT, MENU_NOMOUSE, MENU_STAYOPEN, POPUP_CLOSEANYKEY,
    POPUP_CLOSEEXIT, POPUP_CLOSEEXITZERO, UINT_MAX,
};
use crate::text::{KeyStringCodec, RustKeyStringCodec};
use crate::tmux::checkshell;
use crate::types::{
    ClientRef, OptionsRef, args, args_parse_t, args_parse_type, box_lines, menu_item, u_char, u_int,
};
use ::std::ffi::{CStr, CString};

pub(crate) static cmd_display_menu_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"display-menu",
        alias: Some(c"menu"),
        args: args_parse_t {
            template: c"b:c:C:H:s:S:MOt:T:x:y:",
            lower: 1 as core::ffi::c_int,
            upper: -(1 as core::ffi::c_int),
            cb: Some(
                cmd_display_menu_args_parse,
            ),
        },
        usage: c"[-MO] [-b border-lines] [-c target-client] [-C starting-choice] [-H selected-style] [-s style] [-S border-style] [-t target-pane] [-T title] [-x position] [-y position] name [key] [command] ...",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK | CMD_CLIENT_CFLAG,
        exec: cmd_display_menu_exec,
    }
};
pub(crate) static cmd_display_popup_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"display-popup",
        alias: Some(c"popup"),
        args: args_parse_t {
            template: c"Bb:Cc:d:e:Eh:kNs:S:t:T:w:x:y:",
            lower: 0 as core::ffi::c_int,
            upper: -(1 as core::ffi::c_int),
            cb: None,
        },
        usage: c"[-BCEkN] [-b border-lines] [-c target-client] [-d start-directory] [-e environment] [-h height] [-s style] [-S border-style] [-t target-pane] [-T title] [-w width] [-x position] [-y position] [shell-command [argument ...]]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK | CMD_CLIENT_CFLAG,
        exec: cmd_display_popup_exec,
    }
};
unsafe fn cmd_display_menu_args_parse(
    args: &args,
    idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    let args = RustArguments::from_ref(args);
    unsafe {
        let mut i: u_int = 0 as u_int;
        let mut type_0: args_parse_type;
        loop {
            type_0 = ARGS_PARSE_STRING;
            if i == idx {
                break;
            }
            let fresh0 = i;
            i = i.wrapping_add(1);
            if args_string_str(args, fresh0)
                .expect("argument index checked")
                .is_empty()
            {
                continue;
            }
            type_0 = ARGS_PARSE_STRING;
            let fresh1 = i;
            i = i.wrapping_add(1);
            if fresh1 == idx {
                break;
            }
            type_0 = ARGS_PARSE_COMMANDS_OR_STRING;
            let fresh2 = i;
            i = i.wrapping_add(1);
            if fresh2 == idx {
                break;
            }
        }
        type_0
    }
}
unsafe fn cmd_display_menu_get_pos(
    tc: &ClientRef,
    item: &cmdq_item,
    args: &RustArguments,
    w: u_int,
    h: u_int,
) -> Option<(u_int, u_int)> {
    unsafe {
        let target = &item.target;
        let event_state_ref = item.state_ref();
        let event = event_state_ref.event_snapshot();
        let session = tc.attached_session();
        let target_index = target.winlink_ref().map(|link| link.index());
        let pane = target.pane_ref()?;
        let size = tc.terminal_size();
        let (sx, sy) = (size.width, size.height);
        let mut top: core::ffi::c_int;
        let mut n: core::ffi::c_long;
        if w > sx || h > sy {
            return None;
        }
        let mut ft = format_create_from_target(item);
        if event.m.valid != 0 {
            format_add(&mut ft, c"popup_mouse_x", c"%u", fmt_args![event.m.x]);
            format_add(&mut ft, c"popup_mouse_y", c"%u", fmt_args![event.m.y]);
        }
        if let Some((at, lines)) = tc.status_placement() {
            top = at;
            if top == 0 as core::ffi::c_int {
                top = lines as core::ffi::c_int;
            } else {
                top = 0 as core::ffi::c_int;
            }
            let position = (session.as_ref()?.options()).number(c"status-position") as u_int;
            if let Some((line, start)) = tc.first_window_status_range(lines, target_index) {
                format_add(
                    &mut ft,
                    c"popup_window_status_line_x",
                    c"%u",
                    fmt_args![start],
                );
                if position == 0 as u_int {
                    format_add(
                        &mut ft,
                        c"popup_window_status_line_y",
                        c"%u",
                        fmt_args![line.wrapping_add(1 as u_int).wrapping_add(h)],
                    );
                } else {
                    format_add(
                        &mut ft,
                        c"popup_window_status_line_y",
                        c"%u",
                        fmt_args![sy.wrapping_sub(lines).wrapping_add(line)],
                    );
                }
            }
            if position == 0 as u_int {
                format_add(
                    &mut ft,
                    c"popup_status_line_y",
                    c"%u",
                    fmt_args![lines.wrapping_add(h)],
                );
            } else {
                format_add(
                    &mut ft,
                    c"popup_status_line_y",
                    c"%u",
                    fmt_args![sy.wrapping_sub(lines)],
                );
            }
        } else {
            top = 0 as core::ffi::c_int;
        }
        format_add(&mut ft, c"popup_width", c"%u", fmt_args![w]);
        format_add(&mut ft, c"popup_height", c"%u", fmt_args![h]);
        n = sx.wrapping_sub(1 as u_int) as core::ffi::c_long / 2 as core::ffi::c_long
            - w.wrapping_div(2 as u_int) as core::ffi::c_long;
        if n < 0 as core::ffi::c_long {
            format_add(
                &mut ft,
                c"popup_centre_x",
                c"%u",
                fmt_args![0 as core::ffi::c_int],
            );
        } else {
            format_add(&mut ft, c"popup_centre_x", c"%ld", fmt_args![n]);
        }
        n = sy
            .wrapping_sub(1 as u_int)
            .wrapping_div(2 as u_int)
            .wrapping_add(h.wrapping_div(2 as u_int)) as core::ffi::c_long;
        if n >= sy as core::ffi::c_long {
            format_add(
                &mut ft,
                c"popup_centre_y",
                c"%u",
                fmt_args![sy.wrapping_sub(h)],
            );
        } else {
            format_add(&mut ft, c"popup_centre_y", c"%ld", fmt_args![n]);
        }
        if event.m.valid != 0 {
            n = event.m.x as core::ffi::c_long - w.wrapping_div(2 as u_int) as core::ffi::c_long;
            if n < 0 as core::ffi::c_long {
                format_add(
                    &mut ft,
                    c"popup_mouse_centre_x",
                    c"%u",
                    fmt_args![0 as core::ffi::c_int],
                );
            } else {
                format_add(&mut ft, c"popup_mouse_centre_x", c"%ld", fmt_args![n]);
            }
            n = event.m.y.wrapping_sub(h.wrapping_div(2 as u_int)) as core::ffi::c_long;
            if n + h as core::ffi::c_long >= sy as core::ffi::c_long {
                format_add(
                    &mut ft,
                    c"popup_mouse_centre_y",
                    c"%u",
                    fmt_args![sy.wrapping_sub(h)],
                );
            } else {
                format_add(&mut ft, c"popup_mouse_centre_y", c"%ld", fmt_args![n]);
            }
            n = event.m.y as core::ffi::c_long + h as core::ffi::c_long;
            if n >= sy as core::ffi::c_long {
                format_add(
                    &mut ft,
                    c"popup_mouse_top",
                    c"%u",
                    fmt_args![sy.wrapping_sub(1 as u_int)],
                );
            } else {
                format_add(&mut ft, c"popup_mouse_top", c"%ld", fmt_args![n]);
            }
            n = event.m.y.wrapping_sub(h) as core::ffi::c_long;
            if n < 0 as core::ffi::c_long {
                format_add(
                    &mut ft,
                    c"popup_mouse_bottom",
                    c"%u",
                    fmt_args![0 as core::ffi::c_int],
                );
            } else {
                format_add(&mut ft, c"popup_mouse_bottom", c"%ld", fmt_args![n]);
            }
        }
        let (_bigger, ox, oy, _off_sx, _off_sy) = tc.cached_window_offset();
        let geometry = pane.geometry()?;
        n = ((top + geometry.y) as u_int)
            .wrapping_sub(oy)
            .wrapping_add(h) as core::ffi::c_long;
        if n >= sy as core::ffi::c_long {
            format_add(
                &mut ft,
                c"popup_pane_top",
                c"%u",
                fmt_args![sy.wrapping_sub(h)],
            );
        } else {
            format_add(&mut ft, c"popup_pane_top", c"%ld", fmt_args![n]);
        }
        format_add(
            &mut ft,
            c"popup_pane_bottom",
            c"%u",
            fmt_args![
                ((top + geometry.y) as u_int)
                    .wrapping_add(geometry.height)
                    .wrapping_sub(oy)
            ],
        );
        format_add(
            &mut ft,
            c"popup_pane_left",
            c"%u",
            fmt_args![(geometry.x as u_int).wrapping_sub(ox)],
        );
        n = geometry.x as core::ffi::c_long + geometry.width as core::ffi::c_long
            - ox as core::ffi::c_long
            - w as core::ffi::c_long;
        if n < 0 as core::ffi::c_long {
            format_add(
                &mut ft,
                c"popup_pane_right",
                c"%u",
                fmt_args![0 as core::ffi::c_int],
            );
        } else {
            format_add(&mut ft, c"popup_pane_right", c"%ld", fmt_args![n]);
        }
        let xp = match args_get_str(args, 'x' as i32 as u_char) {
            None => c"#{popup_centre_x}",
            Some(given) => match given.to_bytes() {
                b"C" => c"#{popup_centre_x}",
                b"R" => c"#{popup_pane_right}",
                b"P" => c"#{popup_pane_left}",
                b"M" => c"#{popup_mouse_centre_x}",
                b"W" => c"#{popup_window_status_line_x}",
                _ => given,
            },
        };
        let p = format_expand(&mut ft, xp);
        n = strtol(
            p.as_ptr(),
            core::ptr::null_mut::<*mut core::ffi::c_char>(),
            10 as core::ffi::c_int,
        );
        if n + w as core::ffi::c_long >= sx as core::ffi::c_long {
            n = sx.wrapping_sub(w) as core::ffi::c_long;
        } else if n < 0 as core::ffi::c_long {
            n = 0 as core::ffi::c_long;
        }
        let px: u_int = n as u_int;
        log_debug(
            c"%s: -x: %s = %s = %u (-w %u)",
            fmt_args![c"cmd_display_menu_get_pos", xp, p.as_c_str(), px, w],
        );
        let yp = match args_get_str(args, 'y' as i32 as u_char) {
            None => c"#{popup_centre_y}",
            Some(given) => match given.to_bytes() {
                b"C" => c"#{popup_centre_y}",
                b"P" => c"#{popup_pane_bottom}",
                b"M" => c"#{popup_mouse_top}",
                b"S" => c"#{popup_status_line_y}",
                b"W" => c"#{popup_window_status_line_y}",
                _ => given,
            },
        };
        let p = format_expand(&mut ft, yp);
        n = strtol(
            p.as_ptr(),
            core::ptr::null_mut::<*mut core::ffi::c_char>(),
            10 as core::ffi::c_int,
        );
        if n < h as core::ffi::c_long {
            n = 0 as core::ffi::c_long;
        } else {
            n -= h as core::ffi::c_long;
        }
        if n + h as core::ffi::c_long >= sy as core::ffi::c_long {
            n = sy.wrapping_sub(h) as core::ffi::c_long;
        } else if n < 0 as core::ffi::c_long {
            n = 0 as core::ffi::c_long;
        }
        let py: u_int = n as u_int;
        log_debug(
            c"%s: -y: %s = %s = %u (-h %u)",
            fmt_args![c"cmd_display_menu_get_pos", yp, p.as_c_str(), py, h],
        );
        Some((px, py))
    }
}
unsafe fn cmd_display_menu_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let mut current_block: u64;
    let args: &RustArguments = cmd_get_args(self_0);
    let event_state_ref = item.state_ref();
    let event = event_state_ref.event_snapshot();
    let mut tc = item.target_client().expect("the command has a client");
    let target = item.target.clone();
    let mut menu_item = menu_item::default();
    let style = args_get_str(args, 's' as i32 as u_char);
    let border_style = args_get_str(args, 'S' as i32 as u_char);
    let selected_style = args_get_str(args, 'H' as i32 as u_char);
    let mut lines: box_lines = BOX_LINES_DEFAULT;
    let mut cause = None;
    let mut number_cause = None;
    let mut flags: core::ffi::c_int = 0 as core::ffi::c_int;
    let mut starting_choice: core::ffi::c_int = 0 as core::ffi::c_int;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut i: u_int;
    let count: u_int = args.argument_count();
    let session = target.session().expect("the command target has a session");
    let o = {
        session
            .curw()
            .and_then(|link| link.window())
            .expect("the target session has a current window")
            .options()
    };
    if tc.overlay().is_some() {
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'C' as i32 as u_char) != 0 {
        if args_get_str(args, 'C' as i32 as u_char) == Some(c"-") {
            starting_choice = -(1 as core::ffi::c_int);
            current_block = 4166486009154926805;
        } else {
            starting_choice = args_strtonum(
                args,
                'C' as i32 as u_char,
                0 as core::ffi::c_longlong,
                UINT_MAX as core::ffi::c_longlong,
                &mut number_cause,
            ) as core::ffi::c_int;
            if let Some(cause) = number_cause.as_ref() {
                unsafe { item.error(c"starting choice %s", fmt_args![cause.as_c_str()]) };
                current_block = 17781493482496987363;
            } else {
                current_block = 4166486009154926805;
            }
        }
    } else {
        current_block = 4166486009154926805;
    }
    if current_block == 4166486009154926805 {
        let title = if args_has(args, 'T' as i32 as u_char) != 0 {
            match args_get_str(args, 'T' as i32 as u_char) {
                Some(given) => unsafe { format_single_from_target(item, given) },
                None => c"".to_owned(),
            }
        } else {
            c"".to_owned()
        };
        let mut menu = menu_create(&title);
        i = 0 as u_int;
        loop {
            if !(i != count) {
                current_block = 15925075030174552612;
                break;
            }
            let fresh3 = i;
            i = i.wrapping_add(1);
            let name = unsafe { args_string_str(args, fresh3).expect("argument index checked") };
            if name.is_empty() {
                unsafe {
                    menu_add_item_for_client(&mut menu, None, Some(item), &tc, Some(&target))
                };
            } else if count.wrapping_sub(i) < 2 as u_int {
                unsafe { item.error(c"not enough arguments", fmt_args![]) };
                current_block = 17781493482496987363;
                break;
            } else {
                let fresh4 = i;
                i = i.wrapping_add(1);
                let key = unsafe { args_string_str(args, fresh4).expect("argument index checked") };
                menu_item.name = Some(name);
                menu_item.key = RustKeyStringCodec.parse_key(key);
                let fresh5 = i;
                i = i.wrapping_add(1);
                let cmd = unsafe { args_string_str(args, fresh5).expect("argument index checked") };
                menu_item.command = Some(cmd);
                unsafe {
                    menu_add_item_for_client(
                        &mut menu,
                        Some(&menu_item),
                        Some(item),
                        &tc,
                        Some(&target),
                    )
                };
            }
        }
        match current_block {
            17781493482496987363 => {}
            _ => {
                if menu.items.is_empty()
                    || match unsafe {
                        cmd_display_menu_get_pos(
                            &tc,
                            item,
                            args,
                            menu.width.wrapping_add(4 as u_int),
                            (menu.items.len() as u_int).wrapping_add(2 as u_int),
                        )
                    } {
                        Some((at_px, at_py)) => {
                            (px, py) = (at_px, at_py);
                            false
                        }
                        None => true,
                    }
                {
                    current_block = 11305506228944373502;
                } else {
                    if let Some(value) = args_get_str(args, 'b' as i32 as u_char) {
                        let definition = o.with_entry(c"menu-border-lines", false, |entry| {
                            RustOptionsEngine
                                .definition(entry)
                                .expect("menu border option exists")
                        });
                        unsafe {
                            lines = RustOptionsEngine.find_choice(definition, value, &mut cause)
                                as box_lines
                        };
                        if let Some(cause) = cause.as_ref() {
                            unsafe {
                                item.error(c"menu-border-lines %s", fmt_args![cause.as_c_str()])
                            };
                            current_block = 17781493482496987363;
                        } else {
                            current_block = 11048769245176032998;
                        }
                    } else {
                        current_block = 11048769245176032998;
                    }
                    match current_block {
                        17781493482496987363 => {}
                        _ => {
                            if args_has(args, 'O' as i32 as u_char) != 0 {
                                flags |= MENU_STAYOPEN;
                            }
                            if event.m.valid == 0 && args_has(args, 'M' as i32 as u_char) == 0 {
                                flags |= MENU_NOMOUSE;
                            }
                            if unsafe {
                                menu_display_for_client(
                                    menu,
                                    flags,
                                    starting_choice,
                                    Some(item),
                                    px,
                                    py,
                                    &mut tc,
                                    lines,
                                    style,
                                    selected_style,
                                    border_style,
                                    Some(&target),
                                    None,
                                ) != 0 as core::ffi::c_int
                            } {
                                current_block = 11305506228944373502;
                            } else {
                                return CMD_RETURN_WAIT;
                            }
                        }
                    }
                }
                match current_block {
                    17781493482496987363 => {}
                    _ => {
                        return CMD_RETURN_NORMAL;
                    }
                }
            }
        }
    }
    CMD_RETURN_ERROR
}
unsafe fn cmd_display_popup_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let mut current_block: u64;
    let args: &RustArguments = cmd_get_args(self_0);
    let target = &item.target;
    let s = target.session().expect("popup target has a session");
    let mut tc = item.target_client().expect("the command has a client");
    let size = tc.terminal_size();
    let (sx, sy) = (size.width, size.height);
    let default_command;
    let mut shellcmd: Option<&CStr> = None;
    let style = args_get_str(args, 's' as i32 as u_char);
    let border_style = args_get_str(args, 'S' as i32 as u_char);
    let mut cwd: Option<CString> = None;
    let mut cause = None;
    let mut percentage_cause = None;
    let mut argv: Vec<CString> = Vec::new();
    let title: Option<CString>;
    let modify: core::ffi::c_int = unsafe { popup_present_for_client(&tc) };
    let mut flags: core::ffi::c_int = -(1 as core::ffi::c_int);
    let mut lines: box_lines = BOX_LINES_DEFAULT;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut w: u_int = 0;
    let mut h: u_int = 0;
    let count: u_int = args.argument_count();
    let mut env: Option<Box<RustEnvironment>> = None;
    let o = {
        s.curw()
            .and_then(|link| link.window())
            .expect("the popup session has a current window")
            .options()
    };
    if args_has(args, 'C' as i32 as u_char) != 0 {
        unsafe { client_clear_overlay(&mut tc) };
        return CMD_RETURN_NORMAL;
    }
    if modify == 0 && tc.overlay().is_some() {
        return CMD_RETURN_NORMAL;
    }
    if modify == 0 {
        h = sy.wrapping_div(2 as u_int);
        if args_has(args, 'h' as i32 as u_char) != 0 {
            unsafe {
                h = args_percentage(
                    args,
                    'h' as i32 as u_char,
                    1 as core::ffi::c_longlong,
                    sy as core::ffi::c_longlong,
                    sy as core::ffi::c_longlong,
                    &mut percentage_cause,
                ) as u_int
            };
            if let Some(cause) = percentage_cause.as_ref() {
                unsafe { item.error(c"height %s", fmt_args![cause.as_c_str()]) };
                current_block = 5914453722638945560;
            } else {
                current_block = 8831408221741692167;
            }
        } else {
            current_block = 8831408221741692167;
        }
        match current_block {
            5914453722638945560 => {}
            _ => {
                w = sx.wrapping_div(2 as u_int);
                if args_has(args, 'w' as i32 as u_char) != 0 {
                    unsafe {
                        w = args_percentage(
                            args,
                            'w' as i32 as u_char,
                            1 as core::ffi::c_longlong,
                            sx as core::ffi::c_longlong,
                            sx as core::ffi::c_longlong,
                            &mut percentage_cause,
                        ) as u_int
                    };
                    if let Some(cause) = percentage_cause.as_ref() {
                        unsafe { item.error(c"width %s", fmt_args![cause.as_c_str()]) };
                        current_block = 5914453722638945560;
                    } else {
                        current_block = 12147880666119273379;
                    }
                } else {
                    current_block = 12147880666119273379;
                }
                match current_block {
                    5914453722638945560 => {}
                    _ => {
                        if w > sx {
                            w = sx;
                        }
                        if h > sy {
                            h = sy;
                        }
                        if match unsafe { cmd_display_menu_get_pos(&tc, item, args, w, h) } {
                            Some((at_px, at_py)) => {
                                (px, py) = (at_px, at_py);
                                false
                            }
                            None => true,
                        } {
                            current_block = 4318653505921602179;
                        } else {
                            if let Some(value) = args_get_str(args, 'd' as i32 as u_char) {
                                unsafe { cwd = Some(format_single_from_target(item, value)) };
                            } else {
                                unsafe {
                                    cwd = Some(client_working_directory(Some(&tc), Some(&s)))
                                };
                            }
                            if count == 0 as u_int {
                                default_command = Some(s.options().string_ref(c"default-command"));
                                shellcmd = default_command.as_deref();
                            } else if count == 1 as u_int {
                                unsafe {
                                    shellcmd = Some(
                                        args_string_str(args, 0).expect("argument count checked"),
                                    )
                                };
                            }
                            if count <= 1 as u_int
                                && shellcmd.is_none_or(|shellcmd| shellcmd.is_empty())
                            {
                                let configured = { s.options().string_ref(c"default-shell") };
                                let shell = if unsafe { checkshell(Some(&configured)) == 0 } {
                                    _PATH_BSHELL
                                } else {
                                    &configured
                                };
                                argv.push(shell.to_owned());
                            } else {
                                unsafe { argv = args_to_vector(args) };
                            }
                            if args_has(args, 'e' as i32 as u_char) >= 1 as core::ffi::c_int {
                                let mut e = new_environment_box();
                                for av in args_value_list(args, 'e' as i32 as u_char) {
                                    e.put(av.value.string(), 0);
                                }
                                env = Some(e);
                            }
                            current_block = 14447253356787937536;
                        }
                    }
                }
            }
        }
    } else {
        current_block = 14447253356787937536;
    }
    if current_block == 14447253356787937536 {
        let value = args_get_str(args, 'b' as i32 as u_char);
        if args_has(args, 'B' as i32 as u_char) != 0 {
            lines = BOX_LINES_NONE;
            current_block = 12556861819962772176;
        } else if let Some(value) = value {
            let definition = o.with_entry(c"popup-border-lines", false, |entry| {
                RustOptionsEngine
                    .definition(entry)
                    .expect("popup border option exists")
            });
            unsafe {
                lines = RustOptionsEngine.find_choice(definition, value, &mut cause) as box_lines
            };
            if let Some(cause) = cause.as_ref() {
                unsafe { item.error(c"popup-border-lines %s", fmt_args![cause.as_c_str()]) };
                current_block = 5914453722638945560;
            } else {
                current_block = 12556861819962772176;
            }
        } else {
            current_block = 12556861819962772176;
        }
        match current_block {
            5914453722638945560 => {}
            _ => {
                if args_has(args, 'T' as i32 as u_char) != 0 {
                    unsafe {
                        title = Some(format_single_from_target(
                            item,
                            args_get_str(args, 'T' as i32 as u_char).unwrap_or(c""),
                        ))
                    };
                } else {
                    title = Some(c"".to_owned());
                }
                if args_has(args, 'N' as i32 as u_char) != 0 || modify == 0 {
                    flags = 0 as core::ffi::c_int;
                }
                if args_has(args, 'E' as i32 as u_char) > 1 as core::ffi::c_int {
                    if flags == -(1 as core::ffi::c_int) {
                        flags = 0 as core::ffi::c_int;
                    }
                    flags |= POPUP_CLOSEEXITZERO;
                } else if args_has(args, 'E' as i32 as u_char) != 0 {
                    if flags == -(1 as core::ffi::c_int) {
                        flags = 0 as core::ffi::c_int;
                    }
                    flags |= POPUP_CLOSEEXIT;
                }
                if args_has(args, 'k' as i32 as u_char) != 0 {
                    if flags == -(1 as core::ffi::c_int) {
                        flags = 0 as core::ffi::c_int;
                    }
                    flags |= POPUP_CLOSEANYKEY;
                }
                if modify != 0 {
                    unsafe {
                        popup_modify_for_client(
                            &mut tc,
                            title.as_deref(),
                            style,
                            border_style,
                            lines,
                            flags,
                        )
                    };
                } else if unsafe {
                    !(popup_display_for_client(
                        flags,
                        lines,
                        Some(
                            &cmdq_item_ref_of(item)
                                .expect("the running item is on its command queue"),
                        ),
                        px,
                        py,
                        w,
                        h,
                        env.as_deref(),
                        shellcmd,
                        &argv,
                        cwd.as_deref(),
                        title.as_deref(),
                        &mut tc,
                        Some(&s),
                        style,
                        border_style,
                        None,
                    ) != 0 as core::ffi::c_int)
                } {
                    return CMD_RETURN_WAIT;
                }
                current_block = 4318653505921602179;
            }
        }
    }
    match current_block {
        5914453722638945560 => CMD_RETURN_ERROR,
        _ => CMD_RETURN_NORMAL,
    }
}
