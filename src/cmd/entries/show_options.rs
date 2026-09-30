use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::cmd::queue::{cmdq_error, cmdq_print};
use crate::src::cmd::{cmd_get_args_mut, cmd_get_entry};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_add_time, format_create_from_target, format_expand_cstring, format_free,
    format_single_from_target_cstring,
};
use crate::src::hooks::{
    hooks_is_event, hooks_monitor_get_fire_count, hooks_monitor_get_fire_time,
    hooks_monitor_to_cstring,
};
use crate::src::options::options_table_entry;
use crate::src::options::{
    options_array_item, options_array_iter, options_get_fire_count, options_get_fire_time,
    options_get_monitor_data, options_is_array, options_is_string, options_iter,
    options_match_owned, options_scope_from_flags, options_scope_from_name, options_to_cstring,
    OptionMatchFailure, OptionsScope,
};
use crate::src::options_table::options_table;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args_parse;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_FIND_CANFAIL};
use crate::src::shared::format::format_tree;
use crate::src::shared::monitor::{
    MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_PANE, MONITOR_SESSION, MONITOR_WINDOW,
};
use crate::src::shared::options::options_entry;
use crate::src::shared::options::*;
use crate::src::shared::options::{OPTIONS_TABLE_IS_HOOK, OPTIONS_TABLE_NONE};
use std::ffi::{CStr, CString};
use std::time::{Duration, UNIX_EPOCH};

pub const SHOW_OPTIONS_TEMPLATE: [::core::ffi::c_char; 202] = unsafe {
    ::core::mem::transmute::<
        [u8; 202],
        [::core::ffi::c_char; 202],
    >(
        *b"#{?option_value_only,#{option_value},#{option_name}#{?option_has_array_key,[#{option_array_key}],}#{?option_is_parent,*,}#{?option_has_value, #{?option_is_string,#{q/a:option_value},#{option_value}},}}\0",
    )
};
pub const SHOW_HOOKS_MONITOR_TEMPLATE: [::core::ffi::c_char; 61] = unsafe {
    ::core::mem::transmute::<[u8; 61], [::core::ffi::c_char; 61]>(
        *b"#{option_name}:#{hook_monitor_target}:#{hook_monitor_format}\0",
    )
};
pub static cmd_show_options_entry: cmd_entry = {
    cmd_entry {
        name: c"show-options",
        alias: Some(c"show"),
        args: args_parse {
            template: c"AgF:Hpqst:vw",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-AgHpqsvw] [-F format] [-t target-pane] [option]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_show_options_exec),
    }
};
pub static cmd_show_window_options_entry: cmd_entry = {
    cmd_entry {
        name: c"show-window-options",
        alias: Some(c"showw"),
        args: args_parse {
            template: c"F:gvt:",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-gv] [-F format] [-t target-window] [option]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_show_options_exec),
    }
};
pub static cmd_show_hooks_entry: cmd_entry = {
    cmd_entry {
        name: c"show-hooks",
        alias: None,
        args: args_parse {
            template: c"BF:gpt:w",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-Bgpw] [-F format] [-t target-pane] [hook]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_show_options_exec),
    }
};
unsafe fn cmd_show_options_exec(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let item = item_handle.get();
    let args =
        cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let target = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let window = std::ptr::eq(
        cmd_get_entry(self_0.get_unchecked()),
        &cmd_show_window_options_entry,
    ) as i32;
    let show_hooks = std::ptr::eq(cmd_get_entry(self_0.get_unchecked()), &cmd_show_hooks_entry);
    let monitor = show_hooks && args_has(args, b'B') != 0;
    let quiet = args_has(args, b'q') != 0;
    let mut selected = None;
    let mut cause = None;
    if args_count(args) == 0 {
        let scope = options_scope_from_flags(args, window, target, &mut selected, &mut cause);
        if scope == OPTIONS_TABLE_NONE {
            return show_options_scope_error(item_handle, quiet, cause.as_deref());
        }
        let selected = selected.expect("selected option scope");
        if monitor {
            let names = selected.with_local(|options| {
                options_iter(options)
                    .map(|entry| entry.name.clone())
                    .collect::<Vec<_>>()
            });
            for name in names {
                // Preserve the existing snapshot-and-live-lookup traversal: a
                // removed entry ends the scan, even if later names still exist.
                let Some(record) = selected.with_entry(&name, |entry| show_monitor_record(entry))
                else {
                    break;
                };
                if let Some(record) = record {
                    cmd_show_hooks_print_monitor(self_0.clone(), item_handle, record);
                }
            }
            return CMD_RETURN_NORMAL;
        }
        return cmd_show_options_all(self_0, item_handle, scope, &selected);
    }

    let argument = format_single_from_target_cstring(
        item_handle,
        args_string(&mut *args, 0).map_or(std::ptr::null(), |value| value.as_ptr()),
    );
    let parsed = match options_match_owned(&argument) {
        Ok(parsed) => parsed,
        Err(failure) => {
            if quiet {
                return CMD_RETURN_NORMAL;
            }
            // The tmux oracle reports malformed option syntax as ambiguous too.
            let ambiguous = matches!(
                failure,
                OptionMatchFailure::Parse | OptionMatchFailure::Ambiguous
            );
            cmdq_error(item_handle, |out| {
                out.write_all(if ambiguous {
                    b"ambiguous option: "
                } else {
                    b"invalid option: "
                })?;
                write_cstr(out, argument.as_ptr())
            });
            return CMD_RETURN_ERROR;
        }
    };
    let scope = options_scope_from_name(
        args,
        window,
        parsed.name.as_ptr(),
        target,
        &mut selected,
        &mut cause,
    );
    if scope == OPTIONS_TABLE_NONE {
        return show_options_scope_error(item_handle, quiet, cause.as_deref());
    }
    let selected = selected.expect("selected option scope");
    let resolved = show_options_resolve(&selected, &parsed.name, args_has(args, b'A') != 0);
    let Some((resolved, parent)) = resolved else {
        if parsed.name.as_bytes().first() == Some(&b'@') && !quiet {
            cmdq_error(item_handle, |out| {
                out.write_all(b"invalid option: ")?;
                write_cstr(out, argument.as_ptr())
            });
            return CMD_RETURN_ERROR;
        }
        return CMD_RETURN_NORMAL;
    };
    if monitor {
        if let Some(record) = resolved
            .with_entry(&parsed.name, |entry| show_monitor_record(entry))
            .flatten()
        {
            cmd_show_hooks_print_monitor(self_0, item_handle, record);
        }
    } else {
        cmd_show_options_print(
            self_0,
            item_handle,
            &resolved,
            &parsed.name,
            parsed.array_key.as_deref(),
            parent,
            true,
        );
    }
    CMD_RETURN_NORMAL
}

unsafe fn show_options_scope_error(
    item: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    quiet: bool,
    cause: Option<&CStr>,
) -> cmd_retval {
    if quiet {
        return CMD_RETURN_NORMAL;
    }
    cmdq_error(item, |out| {
        write_cstr(out, cause.expect("scope failure").as_ptr())
    });
    CMD_RETURN_ERROR
}

/// Capture the defining scope once. A later local override must not redirect an
/// inherited array scan to a different owner after a formatting callback.
unsafe fn show_options_resolve(
    scope: &OptionsScope,
    name: &CStr,
    include_parent: bool,
) -> Option<(OptionsScope, i32)> {
    if let Some(local) = scope.resolve(name, true) {
        return Some((local, 0));
    }
    include_parent
        .then(|| scope.resolve(name, false))
        .flatten()
        .map(|parent| (parent, 1))
}

/// Everything needed by formatting is owned before the option borrow ends.
struct ShowOptionRecord {
    name: CString,
    value: CString,
    array_key: Option<CString>,
    parent: i32,
    is_array: i32,
    is_string: i32,
    is_hook: i32,
    is_user: i32,
    has_value: i32,
    fire_count: u32,
    fire_time: time_t,
}

enum ShowOptionRows {
    Record(ShowOptionRecord),
    Array { identity: u64, keys: Vec<CString> },
}

unsafe fn show_option_record(
    entry: &mut options_entry,
    array_key: Option<&CStr>,
    parent: i32,
) -> ShowOptionRecord {
    let table = options_table_entry(entry);
    ShowOptionRecord {
        name: entry.name.clone(),
        value: options_to_cstring(entry, array_key.map_or(std::ptr::null(), CStr::as_ptr), 0),
        array_key: array_key.map(CStr::to_owned),
        parent,
        is_array: options_is_array(entry),
        is_string: options_is_string(entry),
        is_hook: table.is_some_and(|table| table.flags & OPTIONS_TABLE_IS_HOOK != 0) as i32,
        is_user: table.is_none() as i32,
        has_value: 1,
        fire_count: options_get_fire_count(entry),
        fire_time: options_get_fire_time(entry),
    }
}

unsafe fn show_option_rows(
    entry: &mut options_entry,
    array_key: Option<&CStr>,
    parent: i32,
    suppress_empty_parent: bool,
) -> ShowOptionRows {
    if array_key.is_none() && options_is_array(entry) != 0 {
        let keys = options_array_iter(entry)
            .map(|item| item.key.clone())
            .collect::<Vec<_>>();
        if !keys.is_empty() {
            return ShowOptionRows::Array {
                identity: entry.id(),
                keys,
            };
        }
        let mut record = show_option_record(entry, None, parent);
        record.value = CString::default();
        record.has_value = 0;
        if suppress_empty_parent {
            record.parent = 0;
        }
        return ShowOptionRows::Record(record);
    }
    ShowOptionRows::Record(show_option_record(entry, array_key, parent))
}

unsafe fn cmd_show_options_print(
    self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    scope: &OptionsScope,
    name: &CStr,
    array_key: Option<&CStr>,
    parent: i32,
    suppress_empty_parent: bool,
) {
    show_option_each(
        scope,
        name,
        array_key,
        parent,
        suppress_empty_parent,
        |record| {
            cmd_show_options_print_record(self_0.clone(), item_handle, record);
        },
    );
}

/// Delivery may reenter or replace options. Only owned rows reach the callback.
unsafe fn show_option_each(
    scope: &OptionsScope,
    name: &CStr,
    array_key: Option<&CStr>,
    parent: i32,
    suppress_empty_parent: bool,
    mut print: impl FnMut(ShowOptionRecord),
) {
    let Some(rows) = scope.with_entry(name, |entry| {
        show_option_rows(entry, array_key, parent, suppress_empty_parent)
    }) else {
        return;
    };
    match rows {
        ShowOptionRows::Record(record) => print(record),
        ShowOptionRows::Array { identity, keys } => {
            for key in keys {
                // A callback may remove or replace the option or one of its
                // items. Resolve each snapshotted key under a fresh borrow.
                let record = scope
                    .with_entry(name, |entry| {
                        if entry.id() != identity
                            || options_is_array(entry) == 0
                            || options_array_item(entry, key.as_ptr()).is_null()
                        {
                            return None;
                        }
                        Some(show_option_record(entry, Some(&key), parent))
                    })
                    .flatten();
                let Some(record) = record else { break };
                print(record);
            }
        }
    }
}

unsafe fn cmd_show_options_print_record(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    record: ShowOptionRecord,
) {
    let args = cmd_get_args_mut(self_0.get_mut_unchecked()).expect("show options arguments");
    let template = args_get(args, b'F').map(CStr::to_owned);
    let value_only = args_has(args, b'v');
    if record.has_value == 0 && template.is_none() && value_only != 0 {
        return;
    }
    let show_hooks = std::ptr::eq(cmd_get_entry(self_0.get_unchecked()), &cmd_show_hooks_entry);
    let mut ft_owner = format_create_from_target(item_handle);
    let ft = &raw mut *ft_owner;
    show_option_add_formats(ft, &record, value_only as i32);
    if show_hooks {
        show_hook_add_fire_formats(ft, record.fire_count, record.fire_time);
    }
    let template = template
        .as_deref()
        .unwrap_or(CStr::from_ptr(SHOW_OPTIONS_TEMPLATE.as_ptr()));
    let line = format_expand_cstring(ft, template.as_ptr());
    format_free(ft_owner);
    cmdq_print(item_handle, |out| write_cstr(out, line.as_ptr()));
}

unsafe fn show_option_add_formats(
    ft: *mut format_tree,
    record: &ShowOptionRecord,
    value_only: i32,
) {
    format_add(ft, c"option_name".as_ptr(), |out| {
        write_cstr(out, record.name.as_ptr())
    });
    format_add(ft, c"option_value".as_ptr(), |out| {
        write_cstr(out, record.value.as_ptr())
    });
    for (name, value) in [
        (c"option_value_only", value_only),
        (c"option_is_parent", record.parent),
        (c"option_is_array", record.is_array),
        (c"option_is_string", record.is_string),
        (c"option_is_hook", record.is_hook),
        (c"option_is_user", record.is_user),
        (c"option_has_value", record.has_value),
        (c"option_has_array_key", record.array_key.is_some() as i32),
    ] {
        format_add(ft, name.as_ptr(), |out| write!(out, "{value}"));
    }
    format_add(ft, c"option_array_key".as_ptr(), |out| {
        write_cstr(out, record.array_key.as_deref().unwrap_or(c"").as_ptr())
    });
}

unsafe fn show_hook_add_fire_formats(ft: *mut format_tree, count: u32, time: time_t) {
    format_add(ft, c"hook_fire_count".as_ptr(), |out| {
        write!(out, "{count}")
    });
    if time != 0 {
        let offset = Duration::from_secs(time.unsigned_abs());
        let timestamp = if time > 0 {
            UNIX_EPOCH + offset
        } else {
            UNIX_EPOCH - offset
        };
        format_add_time(ft, c"hook_fire_time".as_ptr(), timestamp);
    }
}

struct ShowMonitorRecord {
    option: ShowOptionRecord,
    target: CString,
    format: CString,
}

unsafe fn show_monitor_record(entry: &mut options_entry) -> Option<ShowMonitorRecord> {
    let value = hooks_monitor_to_cstring(entry)?;
    let monitor = options_get_monitor_data(entry)?;
    let target = match monitor.type_0 {
        MONITOR_SESSION => CString::default(),
        MONITOR_PANE => CString::new(format!("%{}", monitor.id)).unwrap(),
        MONITOR_ALL_PANES => c"%*".to_owned(),
        MONITOR_WINDOW => CString::new(format!("@{}", monitor.id)).unwrap(),
        MONITOR_ALL_WINDOWS => c"@*".to_owned(),
        _ => return None,
    };
    let format = monitor.format.clone();
    let option = ShowOptionRecord {
        name: entry.name.clone(),
        value,
        array_key: None,
        parent: 0,
        is_array: 0,
        is_string: 1,
        is_hook: 1,
        is_user: 1,
        has_value: 1,
        fire_count: hooks_monitor_get_fire_count(entry),
        fire_time: hooks_monitor_get_fire_time(entry),
    };
    Some(ShowMonitorRecord {
        option,
        target,
        format,
    })
}

unsafe fn cmd_show_hooks_print_monitor(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    record: ShowMonitorRecord,
) {
    let args = cmd_get_args_mut(self_0.get_mut_unchecked()).expect("show hook arguments");
    let template = args_get(args, b'F')
        .map(CStr::to_owned)
        .unwrap_or_else(|| CStr::from_ptr(SHOW_HOOKS_MONITOR_TEMPLATE.as_ptr()).to_owned());
    let mut ft_owner = format_create_from_target(item_handle);
    let ft = &raw mut *ft_owner;
    show_option_add_formats(ft, &record.option, 0);
    format_add(ft, c"hook_monitor_target".as_ptr(), |out| {
        write_cstr(out, record.target.as_ptr())
    });
    format_add(ft, c"hook_monitor_format".as_ptr(), |out| {
        write_cstr(out, record.format.as_ptr())
    });
    show_hook_add_fire_formats(ft, record.option.fire_count, record.option.fire_time);
    let line = format_expand_cstring(ft, template.as_ptr());
    format_free(ft_owner);
    cmdq_print(item_handle, |out| write_cstr(out, line.as_ptr()));
}

unsafe fn cmd_show_options_all(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    scope: i32,
    selected: &OptionsScope,
) -> cmd_retval {
    let show_hooks = std::ptr::eq(cmd_get_entry(self_0.get_unchecked()), &cmd_show_hooks_entry);
    let args = cmd_get_args_mut(self_0.get_mut_unchecked()).expect("show options arguments");
    let include_hooks = args_has(args, b'H') != 0;
    let include_parent = args_has(args, b'A') != 0;
    let names = selected.with_local(|options| {
        options_iter(options)
            .map(|entry| entry.name.clone())
            .collect::<Vec<_>>()
    });
    for name in names {
        let Some((user, monitor)) = selected.with_entry(&name, |entry| {
            (
                options_table_entry(entry).is_none(),
                options_get_monitor_data(entry).is_some(),
            )
        }) else {
            break;
        };
        if !user {
            continue;
        }
        let user_hook = name.as_bytes().first() == Some(&b'@')
            && (hooks_is_event(name.as_ptr()) != 0 || monitor);
        if (show_hooks && user_hook) || (!show_hooks && (!user_hook || include_hooks)) {
            cmd_show_options_print(self_0.clone(), item_handle, selected, &name, None, 0, false);
        }
    }
    for definition in options_table
        .iter()
        .take_while(|entry| entry.name.is_some())
    {
        if !definition.scope & scope != 0 {
            continue;
        }
        let hook = definition.flags & OPTIONS_TABLE_IS_HOOK != 0;
        if (!show_hooks && !include_hooks && hook) || (show_hooks && !hook) {
            continue;
        }
        let name = definition.name.unwrap();
        if let Some((resolved, parent)) = show_options_resolve(selected, name, include_parent) {
            cmd_show_options_print(
                self_0.clone(),
                item_handle,
                &resolved,
                name,
                None,
                parent,
                false,
            );
        }
    }
    CMD_RETURN_NORMAL
}

#[cfg(test)]
mod owned_row_tests {
    use super::*;
    use crate::src::options::{
        options_array_set, options_create, options_empty, options_get_only_mut,
        options_remove_or_default, options_set_string,
    };
    use crate::src::shared::options::options;
    use crate::src::tmux::{global_options, global_s_options};

    /// Existing global-option tests run serially through RUST_TEST_THREADS=1.
    /// Restore globals before dropping tables, including when an assertion fails.
    unsafe fn with_scopes(run: impl FnOnce(&OptionsScope, &OptionsScope)) {
        struct Restore {
            server: *mut options,
            session: *mut options,
        }
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe {
                    global_options = self.server;
                    global_s_options = self.session;
                }
            }
        }
        let mut parent = options_create(None);
        let mut local = options_create(Some(OptionsScope::GlobalServer));
        let restore = Restore {
            server: std::mem::replace(&mut global_options, &mut *parent),
            session: std::mem::replace(&mut global_s_options, &mut *local),
        };
        run(&OptionsScope::GlobalSession, &OptionsScope::GlobalServer);
        drop(restore);
    }

    unsafe fn set_array(scope: &OptionsScope, values: &[(&CStr, &CStr)]) {
        scope.with_local(|options| {
            let definition = options_table
                .iter()
                .find(|entry| entry.name == Some(c"status-format"))
                .unwrap();
            let entry = options_empty(options, definition);
            for (key, value) in values {
                assert_eq!(
                    options_array_set(entry, key.as_ptr(), value.as_ptr(), 0, std::ptr::null_mut()),
                    0
                );
            }
        });
    }

    #[test]
    fn row_delivery_can_remove_the_option_without_invalidating_its_text() {
        unsafe {
            with_scopes(|scope, _| {
                scope.with_local(|options| {
                    options_set_string(options, c"@owned".as_ptr(), 0, |out| {
                        out.write_all(b"owned text")
                    });
                });
                let mut calls = 0;
                show_option_each(scope, c"@owned", None, 0, false, |row| {
                    scope.with_local(|options| {
                        assert_eq!(
                            options_remove_or_default(
                                options_get_only_mut(options, c"@owned").unwrap(),
                                std::ptr::null(),
                                std::ptr::null_mut(),
                            ),
                            0
                        );
                    });
                    assert_eq!(row.name.as_c_str(), c"@owned");
                    assert_eq!(row.value.as_c_str(), c"owned text");
                    assert_eq!((row.is_user, row.is_string, row.has_value), (1, 1, 1));
                    calls += 1;
                });
                assert_eq!(calls, 1);
                assert!(scope.resolve(c"@owned", true).is_none());
            });
        }
    }

    #[test]
    fn array_delivery_relooks_up_values_and_stops_at_first_removed_key() {
        unsafe {
            with_scopes(|scope, _| {
                set_array(
                    scope,
                    &[
                        (c"0", c"zero"),
                        (c"1", c"one"),
                        (c"2", c"two"),
                        (c"3", c"three"),
                    ],
                );
                let mut rows = Vec::new();
                show_option_each(scope, c"status-format", None, 0, false, |row| {
                    if rows.is_empty() {
                        scope
                            .with_entry(c"status-format", |entry| {
                                assert_eq!(
                                    options_array_set(
                                        entry,
                                        c"1".as_ptr(),
                                        c"changed".as_ptr(),
                                        0,
                                        std::ptr::null_mut()
                                    ),
                                    0
                                );
                                assert_eq!(
                                    options_array_set(
                                        entry,
                                        c"2".as_ptr(),
                                        std::ptr::null(),
                                        0,
                                        std::ptr::null_mut()
                                    ),
                                    0
                                );
                                assert_eq!(
                                    options_array_set(
                                        entry,
                                        c"9".as_ptr(),
                                        c"new".as_ptr(),
                                        0,
                                        std::ptr::null_mut()
                                    ),
                                    0
                                );
                            })
                            .unwrap();
                    }
                    rows.push((row.array_key.unwrap(), row.value));
                });
                assert_eq!(
                    rows,
                    [
                        (c"0".to_owned(), c"zero".to_owned()),
                        (c"1".to_owned(), c"changed".to_owned())
                    ]
                );
                assert!(scope
                    .with_entry(c"status-format", |entry| !options_array_item(
                        entry,
                        c"3".as_ptr()
                    )
                    .is_null())
                    .unwrap());
            });
        }
    }

    #[test]
    fn replacing_the_array_entry_during_delivery_ends_the_original_scan() {
        unsafe {
            with_scopes(|scope, _| {
                set_array(scope, &[(c"0", c"old zero"), (c"1", c"old one")]);
                let mut rows = Vec::new();
                show_option_each(scope, c"status-format", None, 0, false, |row| {
                    if rows.is_empty() {
                        set_array(scope, &[(c"0", c"new zero"), (c"1", c"new one")]);
                    }
                    rows.push(row.value);
                });
                assert_eq!(rows, [c"old zero".to_owned()]);
            });
        }
    }

    #[test]
    fn inherited_array_keeps_its_defining_scope_after_a_local_override() {
        unsafe {
            with_scopes(|local, parent| {
                set_array(parent, &[(c"0", c"parent zero"), (c"1", c"parent one")]);
                let (resolved, inherited) =
                    show_options_resolve(local, c"status-format", true).unwrap();
                assert_eq!(inherited, 1);
                let mut rows = Vec::new();
                show_option_each(&resolved, c"status-format", None, inherited, false, |row| {
                    if rows.is_empty() {
                        set_array(local, &[(c"0", c"local zero"), (c"1", c"local one")]);
                        parent
                            .with_entry(c"status-format", |entry| {
                                assert_eq!(
                                    options_array_set(
                                        entry,
                                        c"1".as_ptr(),
                                        c"changed parent".as_ptr(),
                                        0,
                                        std::ptr::null_mut()
                                    ),
                                    0
                                );
                            })
                            .unwrap();
                    }
                    assert_eq!(row.parent, 1);
                    rows.push(row.value);
                });
                assert_eq!(
                    rows,
                    [c"parent zero".to_owned(), c"changed parent".to_owned()]
                );
            });
        }
    }

    #[test]
    fn empty_arrays_keep_single_and_list_parent_markers_distinct() {
        unsafe {
            with_scopes(|scope, _| {
                set_array(scope, &[]);
                let mut rows = Vec::new();
                show_option_each(scope, c"status-format", None, 1, true, |row| rows.push(row));
                show_option_each(scope, c"status-format", None, 1, false, |row| {
                    rows.push(row)
                });
                show_option_each(scope, c"status-format", Some(c"9"), 1, true, |row| {
                    rows.push(row)
                });
                assert_eq!(rows.len(), 3);
                assert_eq!((rows[0].parent, rows[0].has_value), (0, 0));
                assert_eq!((rows[1].parent, rows[1].has_value), (1, 0));
                assert_eq!((rows[2].parent, rows[2].has_value), (1, 1));
                assert!(rows.iter().all(|row| row.value.as_bytes().is_empty()));
                assert_eq!(rows[2].array_key.as_deref(), Some(c"9"));
            });
        }
    }
}
