//! Option edits whose parsing and cleanup may reenter model operations.
//! Only bounded publication/removal steps borrow the owning model.
use super::*;

fn command_parser(value: &CStr) -> cmd_parse_result {
    unsafe { cmd_parse_from_string(value, std::ptr::null_mut()) }
}

fn parse_commands(
    value: &CStr,
    parse: &mut impl FnMut(&CStr) -> cmd_parse_result,
) -> Result<Option<std::rc::Rc<std::cell::RefCell<cmd_list>>>, CString> {
    let mut parsed = parse(value);
    if parsed.status != crate::src::shared::command::CMD_PARSE_SUCCESS {
        return Err(parsed
            .error
            .take()
            .unwrap_or_else(|| c"invalid command".to_owned()));
    }
    Ok(parsed.take_cmdlist())
}

impl OptionsScope {
    /// Set a scalar; command parsing finishes before any option component borrow.
    pub unsafe fn set_from_string(
        &self,
        definition: Option<&'static options_table_entry>,
        name: &CStr,
        value: Option<&CStr>,
        append: bool,
    ) -> Result<(), CString> {
        self.set_from_string_with_parser(definition, name, value, append, &mut command_parser)
    }

    unsafe fn set_from_string_with_parser(
        &self,
        definition: Option<&'static options_table_entry>,
        name: &CStr,
        value: Option<&CStr>,
        append: bool,
        parse: &mut impl FnMut(&CStr) -> cmd_parse_result,
    ) -> Result<(), CString> {
        if definition.is_none() {
            if !name.to_bytes().starts_with(b"@") {
                return Err(c"bad option name".to_owned());
            }
            let value = value.ok_or_else(|| c"empty value".to_owned())?;
            self.with_local(|table| {
                // Formatting this already-owned value cannot reenter the model;
                // user string replacement retains its existing monitor.
                options_set_string(table, name.as_ptr(), append as i32, |out| {
                    write_cstr(out, value.as_ptr())
                });
            });
            return Ok(());
        }
        let definition = definition.unwrap();
        if definition.type_0 != OPTIONS_TABLE_COMMAND {
            // Scalar strings/numbers/flags do not parse commands. Missing local
            // defaults have the same non-command type, and validation is pure.
            let mut cause = None;
            let result = self.with_local(|table| {
                options_from_string(
                    table,
                    definition,
                    name.as_ptr(),
                    value.map_or(std::ptr::null(), CStr::as_ptr),
                    append as i32,
                    &mut cause,
                )
            });
            return if result == 0 {
                Ok(())
            } else {
                Err(cause.expect("scalar option error"))
            };
        }
        let value = value.ok_or_else(|| c"empty value".to_owned())?;
        let commands = parse_commands(value, parse)?;
        // Preserve set-command's sequence: parse the requested value first;
        // if absent, publish and parse its default before installing the value.
        if self.with_entry(name, |_| ()).is_none() {
            self.reset_default_with_parser(definition, parse);
        }
        self.with_entry(name, |entry| {
            entry.value = options_value::Command(OptionCommand(commands));
        })
        .expect("command option remains installed while parsing");
        Ok(())
    }

    /// Publish an empty local array before parsing its first supplied item.
    pub unsafe fn ensure_array(&self, definition: &'static options_table_entry) {
        assert!(definition.flags & OPTIONS_TABLE_IS_ARRAY != 0);
        let name = definition.name.expect("named array definition");
        if self.with_entry(name, |_| ()).is_none() {
            self.with_local(|table| {
                options_empty(table, definition);
            });
        }
    }

    pub unsafe fn clear_array(&self, name: &CStr) {
        self.with_entry(name, |entry| options_array_clear(entry));
    }

    /// Set/remove one item in an existing local array. A parser may consult any
    /// model; only the resulting owned command list reaches the publication step.
    pub unsafe fn set_array_item(
        &self,
        name: &CStr,
        key: &CStr,
        value: Option<&CStr>,
        append: bool,
    ) -> Result<(), CString> {
        self.set_array_item_with_parser(name, key, value, append, &mut command_parser)
    }

    unsafe fn set_array_item_with_parser(
        &self,
        name: &CStr,
        key: &CStr,
        value: Option<&CStr>,
        append: bool,
        parse: &mut impl FnMut(&CStr) -> cmd_parse_result,
    ) -> Result<(), CString> {
        let (identity, definition) = self
            .with_entry(name, |entry| (entry.id(), entry.tableentry))
            .expect("local array option");
        let Some(definition) =
            definition.filter(|definition| definition.flags & OPTIONS_TABLE_IS_ARRAY != 0)
        else {
            return Err(c"not an array".to_owned());
        };
        let Some(key) = options_array_correct_key(key) else {
            return Err(options_string_cause(c"bad array key: %s", &[key.as_ptr()]));
        };
        if definition.type_0 == OPTIONS_TABLE_COMMAND && value.is_some() {
            let commands = parse_commands(value.unwrap(), parse)?;
            self.with_entry(name, |entry| {
                assert_eq!(entry.id(), identity, "array option replaced while parsing");
                let item = options_array_item(entry, key.as_ptr());
                let item = if item.is_null() {
                    options_array_new(entry, key.as_ptr())
                } else {
                    options_value_free(&raw mut (*item).value);
                    item
                };
                (*item).value = options_value::Command(OptionCommand(commands));
            })
            .expect("array option remains installed while parsing");
            return Ok(());
        }
        let mut cause = None;
        let result = self
            .with_entry(name, |entry| {
                assert_eq!(entry.id(), identity, "array option identity");
                options_array_set(
                    entry,
                    key.as_ptr(),
                    value.map_or(std::ptr::null(), CStr::as_ptr),
                    append as i32,
                    &mut cause,
                )
            })
            .expect("local array option");
        if result == 0 {
            Ok(())
        } else {
            Err(cause.expect("array option error"))
        }
    }

    /// Append split values incrementally. A later parse error preserves every
    /// preceding insertion and stops before parsing any following token.
    pub unsafe fn assign_array(&self, name: &CStr, value: &CStr) -> Result<(), CString> {
        self.assign_array_with_parser(name, value, &mut command_parser)
    }

    unsafe fn assign_array_with_parser(
        &self,
        name: &CStr,
        value: &CStr,
        parse: &mut impl FnMut(&CStr) -> cmd_parse_result,
    ) -> Result<(), CString> {
        let (identity, separator) = self
            .with_entry(name, |entry| {
                (
                    entry.id(),
                    entry
                        .tableentry
                        .expect("array definition")
                        .separator
                        .unwrap_or(c" ,"),
                )
            })
            .expect("local array option");
        if value.to_bytes().is_empty() {
            return Ok(());
        }
        let values = if separator.to_bytes().is_empty() {
            vec![value.to_owned()]
        } else {
            value
                .to_bytes()
                .split(|byte| separator.to_bytes().contains(byte))
                .filter(|part| !part.is_empty())
                .map(|part| CString::new(part).expect("option token contains no NUL"))
                .collect()
        };
        for value in values {
            let index = self
                .with_entry(name, |entry| {
                    assert_eq!(
                        entry.id(),
                        identity,
                        "array assignment keeps its original entry"
                    );
                    let mut index = 0;
                    while index < UINT_MAX && options_array_get_index(entry, index).is_some() {
                        index += 1;
                    }
                    index
                })
                .expect("array option remains installed during assignment");
            // The no-separator branch historically permits UINT_MAX itself;
            // separated assignment stops after the last ordinary index is full.
            if index == UINT_MAX && !separator.to_bytes().is_empty() {
                break;
            }
            let key = CString::new(index.to_string()).expect("numeric array key");
            self.set_array_item_with_parser(name, &key, Some(&value), false, parse)?;
        }
        Ok(())
    }

    /// Remove a local entry/item or reset a global built-in to its default.
    /// False means no local entry existed and callers must not push changes.
    pub unsafe fn remove_or_default(
        &self,
        name: &CStr,
        key: Option<&CStr>,
    ) -> Result<bool, CString> {
        let Some(definition) = self.with_entry(name, |entry| entry.tableentry) else {
            return Ok(false);
        };
        if let Some(key) = key {
            self.set_array_item(name, key, None, false)?;
        } else if self.is_global() && definition.is_some() {
            self.reset_default_with_parser(definition.unwrap(), &mut command_parser);
        } else {
            self.remove_entry(name);
        }
        Ok(true)
    }

    /// Leave the emptied entry published while its monitor is destroyed, then
    /// unlink that same identity. A reentrant replacement belongs to its caller.
    unsafe fn remove_entry(&self, name: &CStr) -> bool {
        self.remove_entry_with_cleanup(name, drop)
    }

    unsafe fn remove_entry_with_cleanup(
        &self,
        name: &CStr,
        cleanup: impl FnOnce(Option<Box<crate::src::hooks::hooks_monitor>>),
    ) -> bool {
        let Some((identity, canonical_name, monitor)) = self.with_entry(name, |entry| {
            options_array_clear(entry);
            options_value_free(&mut entry.value);
            (entry.id(), entry.name.clone(), entry.monitor_data.take())
        }) else {
            return false;
        };
        cleanup(monitor);
        let removed = self.with_local(|table| {
            if options_get_only(table, &canonical_name).is_some_and(|entry| entry.id() == identity)
            {
                table.tree.remove(canonical_name.to_bytes())
            } else {
                None
            }
        });
        drop(removed);
        true
    }

    unsafe fn reset_default_with_parser(
        &self,
        definition: &'static options_table_entry,
        parse: &mut impl FnMut(&CStr) -> cmd_parse_result,
    ) {
        let name = definition.name.expect("named option definition");
        // Removal can reenter and replace the name. Finish each replacement's
        // teardown before publishing the fresh default record.
        while self.remove_entry(name) {}
        let identity = self.with_local(|table| (*options_empty(table, definition)).id());
        if definition.flags & OPTIONS_TABLE_IS_ARRAY != 0 {
            if let Some(values) = definition.default_arr {
                for (index, value) in values.iter().enumerate() {
                    let key = CString::new(index.to_string()).expect("numeric array key");
                    // Default loading preserves the legacy ignored-error behavior.
                    let _ = self.set_array_item_with_parser(name, &key, Some(value), false, parse);
                }
            } else {
                let value = CStr::from_ptr(definition.default_str_ptr());
                let _ = self.assign_array_with_parser(name, value, parse);
            }
            return;
        }
        let value = match definition.type_0 {
            OPTIONS_TABLE_STRING => {
                options_value::String(CStr::from_ptr(definition.default_str_ptr()).to_owned())
            }
            OPTIONS_TABLE_COMMAND => {
                let mut parsed = parse(CStr::from_ptr(definition.default_str_ptr()));
                if parsed.status != crate::src::shared::command::CMD_PARSE_SUCCESS {
                    return;
                }
                options_value::Command(OptionCommand(parsed.take_cmdlist()))
            }
            _ => options_value::Number(definition.default_num),
        };
        self.with_entry(name, |entry| {
            assert_eq!(
                entry.id(),
                identity,
                "default option replaced while parsing"
            );
            entry.value = value;
        })
        .expect("default option remains installed while parsing");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::cmd::cmd_list_new;
    use crate::src::shared::command::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS};
    use std::rc::Rc;

    unsafe fn with_server_scope(run: impl FnOnce(&OptionsScope)) {
        struct Restore(*mut options);
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe {
                    global_options = self.0;
                }
            }
        }
        let mut table = options_create(None);
        let restore = Restore(std::mem::replace(&mut global_options, &mut *table));
        run(&OptionsScope::GlobalServer);
        drop(restore);
    }

    fn success(commands: &Rc<std::cell::RefCell<cmd_list>>) -> cmd_parse_result {
        cmd_parse_result {
            status: CMD_PARSE_SUCCESS,
            cmdlist: Some(commands.clone()),
            error: None,
        }
    }

    #[test]
    fn command_set_parses_request_before_publishing_and_parsing_its_default() {
        unsafe {
            with_server_scope(|scope| {
                let definition = options_table
                    .iter()
                    .find(|entry| entry.name == Some(c"default-client-command"))
                    .unwrap();
                let requested = cmd_list_new();
                let default = cmd_list_new();
                let mut calls = Vec::new();
                scope
                    .set_from_string_with_parser(
                        Some(definition),
                        c"default-client-command",
                        Some(c"requested"),
                        false,
                        &mut |value| {
                            calls.push(value.to_owned());
                            if value == c"requested" {
                                assert!(scope
                                    .with_entry(c"default-client-command", |_| ())
                                    .is_none());
                            } else {
                                assert_eq!(value, c"new-session");
                                assert!(scope
                                    .with_entry(c"default-client-command", |entry| matches!(
                                        entry.value,
                                        options_value::Empty
                                    ))
                                    .unwrap());
                            }
                            scope
                                .set_from_string(None, c"@parser-entered", Some(value), false)
                                .unwrap();
                            if value == c"requested" {
                                success(&requested)
                            } else {
                                success(&default)
                            }
                        },
                    )
                    .unwrap();
                assert_eq!(calls, [c"requested".to_owned(), c"new-session".to_owned()]);
                assert!(scope
                    .with_entry(c"default-client-command", |entry| Rc::ptr_eq(
                        entry.value.commands().unwrap(),
                        &requested
                    ))
                    .unwrap());
                assert_eq!(Rc::strong_count(&default), 1);
            });
        }
    }

    #[test]
    fn failed_array_command_parse_preserves_the_existing_item_and_identity() {
        unsafe {
            with_server_scope(|scope| {
                let definition = options_table
                    .iter()
                    .find(|entry| entry.name == Some(c"after-bind-key"))
                    .unwrap();
                scope.ensure_array(definition);
                let original = cmd_list_new();
                scope
                    .set_array_item_with_parser(
                        c"after-bind-key",
                        c"0",
                        Some(c"old"),
                        false,
                        &mut |_| success(&original),
                    )
                    .unwrap();
                let item_id = scope
                    .with_entry(c"after-bind-key", |entry| {
                        (*options_array_item(entry, c"0".as_ptr())).id()
                    })
                    .unwrap();
                let cause = scope
                    .set_array_item_with_parser(
                        c"after-bind-key",
                        c"0",
                        Some(c"broken"),
                        false,
                        &mut |_| {
                            assert!(scope
                                .with_entry(c"after-bind-key", |entry| Rc::ptr_eq(
                                    options_array_get(entry, c"0").unwrap().commands().unwrap(),
                                    &original
                                ))
                                .unwrap());
                            scope
                                .set_from_string(None, c"@parser-entered", Some(c"yes"), false)
                                .unwrap();
                            cmd_parse_result {
                                status: CMD_PARSE_ERROR,
                                cmdlist: None,
                                error: Some(c"bad command".to_owned()),
                            }
                        },
                    )
                    .unwrap_err();
                assert_eq!(cause.as_c_str(), c"bad command");
                assert_eq!(
                    scope.with_entry(c"after-bind-key", |entry| (*options_array_item(
                        entry,
                        c"0".as_ptr()
                    ))
                    .id()),
                    Some(item_id)
                );
                assert!(scope
                    .with_entry(c"after-bind-key", |entry| Rc::ptr_eq(
                        options_array_get(entry, c"0").unwrap().commands().unwrap(),
                        &original
                    ))
                    .unwrap());
            });
        }
    }

    #[test]
    fn array_assignment_keeps_preceding_commands_and_stops_at_first_parse_error() {
        static DEFINITION: options_table_entry = options_table_entry {
            name: Some(c"test-command-array"),
            alternative_name: None,
            type_0: OPTIONS_TABLE_COMMAND,
            scope: OPTIONS_TABLE_SERVER,
            flags: OPTIONS_TABLE_IS_ARRAY,
            minimum: 0,
            maximum: 0,
            choices: &[],
            default_str: Some(c""),
            default_num: 0,
            default_arr: None,
            separator: Some(c","),
            pattern: None,
            text: None,
            unit: None,
        };
        unsafe {
            with_server_scope(|scope| {
                scope.ensure_array(&DEFINITION);
                let first = cmd_list_new();
                let mut calls = Vec::new();
                let cause = scope
                    .assign_array_with_parser(
                        c"test-command-array",
                        c"first,bad,unreached",
                        &mut |value| {
                            calls.push(value.to_owned());
                            scope
                                .set_from_string(None, c"@parser-entered", Some(value), false)
                                .unwrap();
                            if value == c"first" {
                                assert!(scope
                                    .with_entry(c"test-command-array", |entry| options_array_iter(
                                        entry
                                    )
                                    .next()
                                    .is_none())
                                    .unwrap());
                                success(&first)
                            } else {
                                assert!(scope
                                    .with_entry(c"test-command-array", |entry| Rc::ptr_eq(
                                        options_array_get(entry, c"0").unwrap().commands().unwrap(),
                                        &first
                                    ))
                                    .unwrap());
                                cmd_parse_result {
                                    status: CMD_PARSE_ERROR,
                                    cmdlist: None,
                                    error: Some(c"bad second command".to_owned()),
                                }
                            }
                        },
                    )
                    .unwrap_err();
                assert_eq!(cause.as_c_str(), c"bad second command");
                assert_eq!(calls, [c"first".to_owned(), c"bad".to_owned()]);
                assert_eq!(
                    scope.with_entry(c"test-command-array", |entry| options_array_iter(entry)
                        .count()),
                    Some(1)
                );
            });
        }
    }

    #[test]
    fn removal_frees_value_before_reentrant_cleanup_and_keeps_a_replacement() {
        unsafe {
            with_server_scope(|scope| {
                let definition = options_table
                    .iter()
                    .find(|entry| entry.name == Some(c"after-bind-key"))
                    .unwrap();
                scope.ensure_array(definition);
                let original = cmd_list_new();
                scope
                    .set_array_item_with_parser(
                        c"after-bind-key",
                        c"0",
                        Some(c"old"),
                        false,
                        &mut |_| success(&original),
                    )
                    .unwrap();
                let old_id = scope
                    .with_entry(c"after-bind-key", |entry| entry.id())
                    .unwrap();
                let mut replacement = 0;
                assert!(
                    scope.remove_entry_with_cleanup(c"after-bind-key", |monitor| {
                        assert_eq!(Rc::strong_count(&original), 1);
                        assert!(scope
                            .with_entry(c"after-bind-key", |entry| entry.id() == old_id
                                && matches!(entry.value, options_value::Empty)
                                && entry.monitor_data.is_none())
                            .unwrap());
                        // Simulate synchronous monitor cleanup reentering the owner.
                        scope.remove_entry(c"after-bind-key");
                        scope.ensure_array(definition);
                        replacement = scope
                            .with_entry(c"after-bind-key", |entry| entry.id())
                            .unwrap();
                        assert_ne!(old_id, replacement);
                        drop(monitor);
                    })
                );
                assert_eq!(
                    scope.with_entry(c"after-bind-key", |entry| entry.id()),
                    Some(replacement)
                );
            });
        }
    }
}
