use crate::args::args_parse_t;
use crate::args::RustArguments;
use crate::cmd::cmdq_item;
use crate::cmd::cmdq_item_weak_of;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::consts::{
    CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_WAIT, SPAWN_BEFORE,
    SPAWN_DETACHED, SPAWN_EMPTY, SPAWN_FLOATING, SPAWN_FULLSIZE, SPAWN_ZOOM,
};
use crate::environ::EnvironmentStore;
use crate::environ::new_environment_box;
use crate::fmt_args;
use crate::format::{format_create_for_client, format_defaults_for_handles, format_expand};
use crate::spawn::spawn_pane;
use crate::types::{OptionsRef, cmd_find_state, spawn_context, u_char, u_int};
use crate::window::WinlinkRef;
use ::std::ffi::CString;

pub const SPLIT_WINDOW_TEMPLATE: &core::ffi::CStr =
    c"#{session_name}:#{window_index}.#{pane_index}";
pub(crate) static cmd_new_pane_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"new-pane",
        alias: Some(c"newp"),
        args: args_parse_t {
            template: c"bc:de:EfF:hIkl:Lm:p:PR:s:S:t:vx:X:y:Y:Z",
            lower: 0 as core::ffi::c_int,
            upper: -(1 as core::ffi::c_int),
            cb: None,
        },
        usage: c"[-bdefhIklPvZ] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-x width] [-y height] [-X x-position] [-Y y-position] [-t target-pane] [shell-command [argument ...]]",
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
        flags: 0 as core::ffi::c_int,
        exec: cmd_split_window_exec,
    }
};
pub(crate) static cmd_split_window_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"split-window",
        alias: Some(c"splitw"),
        args: args_parse_t {
            template: c"bc:de:EfF:hIkl:m:p:PR:s:S:t:vZ",
            lower: 0 as core::ffi::c_int,
            upper: -(1 as core::ffi::c_int),
            cb: None,
        },
        usage: c"[-bdefhIklPvZ] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-t target-pane] [shell-command [argument ...]]",
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
        flags: 0 as core::ffi::c_int,
        exec: cmd_split_window_exec,
    }
};
unsafe fn cmd_split_window_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);
    let current_state_ref = item.state_ref();
    let mut sc = spawn_context::default();
    let tc = item.target_client();
    let session = item.target.session().expect("a split target has a session");
    let link = item
        .target
        .wl_idx
        .and_then(|index| WinlinkRef::new(session.clone(), index))
        .expect("a split target has a window link");
    let owner = link.window().expect("a split target has a window owner");
    let pane = item.target.pane_ref().expect("a split target has a pane");
    let lc;
    let mut fs = cmd_find_state::default();
    let mut input: core::ffi::c_int;

    let mut flags: core::ffi::c_int;
    let mut cause: Option<CString> = None;
    let mut floating_cause = None;
    let count: u_int = args.argument_count();
    let is_floating: core::ffi::c_int = if cmd_get_entry(self_0).name == cmd_new_pane_entry.name {
        (({
            let flag = 'L' as i32 as u_char;
            args.argument_flag_count(flag)
        }) == 0) as core::ffi::c_int
    } else {
        0 as core::ffi::c_int
    };
    flags = if is_floating != 0 {
        SPAWN_FLOATING
    } else {
        0 as core::ffi::c_int
    };
    if ({
        let flag = 'b' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        flags |= SPAWN_BEFORE;
    }
    if ({
        let flag = 'f' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        flags |= SPAWN_FULLSIZE;
    }
    input = {
        let flag = 'I' as i32 as u_char;
        args.argument_flag_count(flag)
    };
    let empty: core::ffi::c_int = if input != 0 {
        1 as core::ffi::c_int
    } else {
        {
            let flag = 'E' as i32 as u_char;
            args.argument_flag_count(flag)
        }
    };
    if {
        empty != 0
            && count != 0 as u_int
            && (count != 1 as u_int
                || !args
                    .argument_string(0)
                    .expect("argument count checked")
                    .is_empty())
    } {
        unsafe { item.error(c"command cannot be given for empty pane", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    if empty != 0 {
        flags |= SPAWN_EMPTY;
    }
    if is_floating != 0 {
        unsafe { lc = owner.floating_layout_cell(item, args, &mut floating_cause) };
        if let Some(cause) = floating_cause.as_ref() {
            unsafe { item.error(c"size or position %s", fmt_args![cause.as_c_str()]) };
            return CMD_RETURN_ERROR;
        }
    } else {
        let mut tiled_cause = CString::default();
        unsafe { lc = owner.tiled_layout_cell(item, args, &pane, flags, &mut tiled_cause) };
        if !tiled_cause.as_bytes().is_empty() {
            unsafe { item.error(c"size or position %s", fmt_args![tiled_cause.as_c_str()]) };
            return CMD_RETURN_ERROR;
        }
    }
    sc.item = cmdq_item_weak_of(item);
    sc.s = Some(session.clone());
    sc.wl_idx = Some(link.index());
    sc.wp0 = Some(pane.clone());
    unsafe { sc.argv = args.to_vector() };
    sc.environ = Some(new_environment_box());
    for av in {
        let flag = 'e' as i32 as u_char;
        args.argument_flag_values(flag)
    } {
        sc.environ
            .as_deref_mut()
            .expect("the spawn environment is initialized")
            .put(av.string(), 0);
    }
    sc.idx = -(1 as core::ffi::c_int);
    sc.cwd = {
        let flag = 'c' as i32 as u_char;
        args.argument_flag_string(flag)
    };
    sc.flags = flags;
    if ({
        let flag = 'd' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        sc.flags |= SPAWN_DETACHED;
    }
    if ({
        let flag = 'Z' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        sc.flags |= SPAWN_ZOOM;
    }
    let Some(new_pane) = (unsafe { spawn_pane(&mut sc, lc.as_ref(), &mut cause) }) else {
        let cause = cause.unwrap();
        unsafe { item.error(c"create pane failed: %s", fmt_args![cause.as_c_str()]) };
        drop(sc.environ.take());
        return CMD_RETURN_ERROR;
    };
    {
        let options = unsafe { new_pane.options().expect("the spawned pane is present") };
        if let Some(style) = args.argument_flag_string(b's') {
            unsafe { new_pane.set_window_style(style) };
        }
        if let Some(style) = args.argument_flag_string(b'S') {
            unsafe { options.set_string(c"pane-active-border-style", 0, c"%s", fmt_args![style]) };
        }
        if let Some(style) = args.argument_flag_string(b'R') {
            unsafe { options.set_string(c"pane-border-style", 0, c"%s", fmt_args![style]) };
        }
        if ({
            let flag = 'k' as i32 as u_char;
            args.argument_flag_count(flag)
        }) != 0
            || ({
                let flag = 'm' as i32 as u_char;
                args.argument_flag_count(flag)
            }) != 0
        {
            unsafe { (options).set_number(c"remain-on-exit", 3 as core::ffi::c_longlong) };
            if ({
                let flag = 'm' as i32 as u_char;
                args.argument_flag_count(flag)
            }) != 0
            {
                unsafe {
                    (options).set_string(
                        c"remain-on-exit-format",
                        0 as core::ffi::c_int,
                        c"%s",
                        fmt_args![args.argument_flag_string(b'm')],
                    )
                };
            }
        }
    }
    if input != 0 {
        match unsafe {
            new_pane
                .start_input(&crate::cmd::cmdq_item_ref_of(item).expect("the command has an owner"))
        } {
            Err(cause) => {
                unsafe { owner.discard_pane(&new_pane, is_floating == 0) };
                unsafe { item.error(c"%s", fmt_args![cause.as_c_str()]) };
                drop(sc.environ.take());
                return CMD_RETURN_ERROR;
            }
            Ok(1) => {
                input = 0 as core::ffi::c_int;
            }
            _ => {}
        }
    }
    if ({
        let flag = 'd' as i32 as u_char;
        args.argument_flag_count(flag)
    }) == 0
    {
        unsafe {
            current_state_ref.update_current_link(&link, Some(&new_pane), 0 as core::ffi::c_int)
        };
    }
    if is_floating == 0 {
        unsafe { owner.pop_zoom() };
        unsafe { owner.redraw() };
    }
    unsafe { session.request_redraw() };
    if ({
        let flag = 'P' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        let template = {
            let flag = 'F' as i32 as u_char;
            args.argument_flag_string(flag)
        }
        .unwrap_or(SPLIT_WINDOW_TEMPLATE);
        let cp = unsafe {
            let mut ft = format_create_for_client(item.client().as_ref(), Some(item), 0, 0);
            format_defaults_for_handles(
                &mut ft,
                tc.as_ref(),
                Some(&session),
                Some(&link),
                Some(&new_pane),
            );
            format_expand(&mut ft, template)
        };
        unsafe { item.print(c"%s", fmt_args![cp.as_c_str()]) };
    }
    unsafe {
        crate::cmd::cmd_find_from_link_ref(&mut fs, &link, Some(&new_pane), 0 as core::ffi::c_int)
    };
    unsafe {
        (crate::cmd::cmdq_item_ref_of(item).expect("the command has an owner")).insert_session_hook(
            Some(&session),
            Some(&fs),
            c"after-split-window",
            fmt_args![],
        )
    };
    drop(sc.environ.take());
    if input != 0 {
        return CMD_RETURN_WAIT;
    }
    CMD_RETURN_NORMAL
}
