## Rule

The items here are independent small tasks. First pull the master branch for
the latest task states, find unchecked item, then first mark it + commit +
update master so that you can acquire the item. If conflict happens you failed
to acquire lock. Start over.

Once you acquired the task, go implement it. Once succeeded. Also update this
doc by adding commit hash on the entry the entry you worked + make `git
commit` + try to rebase on top of origin/master + push it. On merge conflicts,
try to resolve it to make it clean ff commit. otherwise consider it fail,
revert the marker and finish the job.

## Tasks

- [x] `args_escape` may return CString
- [x] `xasprintf` may take first argument to be &mut CString or return CString
- [x] `utf8_stravis` may take first argument to be &mut CString or return CString
- [x] `layout_get_tiled_cell` last argument cause should be &mut CString
- [x] see `sessions` access and see they can be migrated to `session_owners`.
- [x] see `alerts_callback`, I'm wondering w_ref.as_ptr() is unnecessary as
  callee can be promoted to get w_ref directly. Update tasks.md listing other
  similar patterns which can be fixed. After then, try to fix the code do the
  same without as_ptr().
- [x] see `binding_of`, where the first argument does not need to be raw
  pointers. Find such cases and add them to tasks.md for fix.
- [x] I see bunch of nonsense casting such as `&(*w)`...
  can we do better than this? it happened at least on format_cb_window_layout
  but it's all over the place. Can we find such cases using AST matchers so
  that we don't miss them systematically?
- [x] See `format_add_window_neighbor`, I can see `&mut *nft` which
  is effectively just &mut nft? find all such nonsense code and report at
  task.md.
- [x] `make audit-deref` reports 203 `cstr_ptr(&(*p).field)` round
  trips left.
  They spread over more fields than the option ones did — `name` 74,
  `prompt_last` 52, `prompt_string` and `message_string` 12 each, `ttyname` 9,
  then `title`, `search`, `exit_session`, `cwd`, `shell`, `searchstr`, `path`
  and a dozen singles — so it wants a name per accessor rather than one sweep.
  `session_name` and `session_cwd` are the shape already there.
- [x] `peer_ptr(&(*c).peer)` 28 times and
  `environ_ptr(&(*x).environ)` 11: same treatment as the option sets, one
  accessor per owner.
- [x] `cmd_get_args_ptr(&*item.cmd())` and the two `commands_ptr` /
  `args_ptr` sites are the tail of the round trips; they deref a handle
  rather than a field, so they belong with the `as_ptr()` promotions above.
- [x] `make audit-deref AUDIT_ARGS=` reports 3602 borrows through a deref
  that are not round trips — `&(*c).prompt_buffer` and friends, where the
  callee really does want the borrow. Those are only worth touching where
  the owner can be promoted to a reference for the whole function; the tool
  is what says which functions carry enough of them to be worth it.
- [x] `screen_write_offset_timer` takes `*mut window` and looks the handle up
  again, though its only caller is the offset timer callback that has already
  upgraded one. Promote it to `&WindowRef`, the way `alerts_timer` now is.
- [x] `resize.rs`'s `recalculate_size` is reached as
  `recalculate_size(w_ref.as_ptr(), now)` from the `each_window()` walk in
  `recalculate_sizes_now`; promote it to take the handle.
- [x] `server_client_loop` walks `window_refs()` and hands
  `server_client_check_window_resize` a pointer straight out of the handle;
  promote that callee too.
- [x] `options/store.rs` opens four `each_window()` walks with
  `let w = w_ref.as_ptr();`. The bodies are field pokes, but the per-window
  work could move into helpers taking `&WindowRef`.
- [x] `name_time_callback` takes `&window` and is reached as
  `name_time_callback(&*w_ref.as_ptr())` from the name timer, which already
  holds the handle; let it take `&WindowRef`.
- [x] The `each_session()` walks now hand over `SessionRef`, but every body
  still opens with `s_ref.as_ptr()` because the callees take `*mut session`.
  `session_clear_attached`, `session_update_history`, `status_update_cache`,
  `server_status_session`, `server_renumber_session` and `session_destroy` are
  the ones worth promoting to `&SessionRef` first.
- [x] `key_bindings_add`, `key_bindings_remove`, `key_bindings_reset`,
  `key_bindings_remove_table` and `key_bindings_reset_table` each look a
  `KeyTableRef` up and drop it to `*mut key_table` on the next line, and
  `key_bindings_init_done` and `cmd_send_keys.rs` do the same. The callees
  they hand the pointer to — `key_bindings_get`, `key_bindings_next`,
  `key_bindings_take_defaults` — could take `&KeyTableRef`.
- [x] `file.rs` degrades `ClientFileRef` to `*mut client_file` in around
  eighteen places, most of them one `let cf = cf_ref.as_ptr();` per function.
  The file callbacks are the natural cut: promote them to `&ClientFileRef`.
- [x] `modes/widget.rs`, `modes/client.rs`, `modes/buffer.rs` and
  `cmd_source_file.rs` each open a callback with
  `let data = data_ref.as_ptr();`; same treatment.
- [x] see xmalloc and xcmalloc functions, they are only used by test code. Find
  such unused functions and try to remove them or move them to tests only
  utility.
- [x] 34 raw-pointer parameters across 29 functions are never used as pointers
  at all — every mention in the body is `&*p` or `&mut *p`, so the pointer is
  there only to be undone. `sort.rs`'s comparators (`sort_buffer_cmp`,
  `sort_client_cmp`, `sort_session_cmp`, `sort_winlink_cmp`), `environ_copy`,
  `environ_set`, `environ_clear`, `environ_update`, `tty_check_fg`,
  `tty_check_bg`, `tty_check_us`, `tty_fake_bce`, `grid_cells_look_equal`,
  `grid_set_tab`, `format_grid_line`, `format_grid_hyperlink`,
  `window_copy_init`, `window_copy_stringify`, `window_tree_draw_label`,
  `window_tree_get_target`, `options_value_free`, `options_is_string`,
  `cmdq_find_flag`, `cmdq_print_data`, `server_client_command_done`,
  `status_prompt_accept`, `key_from_data`, `buffer_bytes` and `list_concat`
  are the whole list. One signature each; a caller that holds a pointer moves
  the deref to the call. `sort_pane_cmp` is the same shape one step out — it
  opens `let a = &*wpa;` but keeps `wpa` for `window_pane_index`, so it comes
  with that callee.
- [x] `binding_of` takes `*mut args` only because `args_value` and
  `args_values` do, so `cmd_bind_key_exec` reaches for
  `cmd_get_args_ptr(self_0)` two lines after `cmd_get_args(self_0)` handed it
  `&args`; `cmdq_insert_hook` binds both forms on adjacent lines.
  `args_value`, `args_values`, `args_print` and `args_copy` are the whole
  `*mut args` set outside the mode entry points — promote them to `&args` and
  three of `cmd_get_args_ptr`'s six callers go away.
- [x] (4cff4268) The three left are `WindowMode::init` and `WindowMode::command` in
  `modes/dispatch.rs`, which take `wme: *mut window_mode_entry`,
  `args: *mut args` and `m: *mut mouse_event` and immediately hand the table
  `&mut *wme`. The mode `init`/`command` function-pointer table is the cut:
  it is ours, not tmux's, so the whole row can move to references.
- [x] (46613fe2) `winlinks` is a `BTreeMap<c_int, Box<winlink>>`, yet every
  lookup goes through `&raw mut (*s).windows` — 81 sites outside the tests
  and 150 in them. `winlink_count` is `(*wwl).len()`, and `winlinks_first`,
  `winlinks_last`, `winlinks_next`, `winlinks_prev`, `winlink_find_by_index`,
  `winlink_find_by_window` and `winlink_find_by_window_id` only read. Those
  take `&winlinks`; `winlink_add`, `winlink_remove` and the
  `winlink_stack_push`/`_remove` pair take `&mut`. The finders still hand back
  `*mut winlink`, so keep the provenance honest — a shared borrow that a
  caller then writes through is worse than what is there now.
- [x] `grid_cell` is passed by pointer everywhere: 182 non-test call sites
  across 38 callees build a `&raw mut gc` or `&raw const grid_default_cell`
  just to satisfy a `*const grid_cell` parameter. `screen_write_cell`,
  `screen_write_putc`/`_puts`/`_nputs`/`_text`, `screen_write_collect_add`,
  `grid_cells_equal`, `screen_select_cell`, `screen_set_selection`,
  `style_apply`, `style_add`, `tty_attributes`, `tty_cell`,
  `tty_default_attributes`, `tty_default_colours` and
  `screen_redraw_border_set` are the ones with the most callers. `*const
  grid_cell` becomes `&grid_cell` and `*mut grid_cell` `&mut grid_cell`; it is
  big but it is the single widest instance of the pattern.
- [x] `*mut screen` the same way: 140 non-test sites, nearly all of them
  `&raw mut (*wp).base`, `&raw mut (*wp).status_screen` or
  `&raw mut (*sl).screen`. `screen_init`, `screen_free`, `screen_reinit`,
  `screen_resize`, `screen_set_title`, `screen_clear_selection`,
  `screen_write_start`, `screen_write_start_pane`, `screen_write_fast_copy`
  and `screen_write_preview` are the set. `screen_grid_ptr` and
  `screen_saved_grid_ptr` are the accessors already carved out for this, so
  they go with it.
- [x] (e9e89cbb) `*mut tty` is reached as `&raw mut (*c).tty` at 46 non-test
  sites. `tty_init`, `tty_open`, `tty_close`, `tty_free`, `tty_start_tty`,
  `tty_stop_tty`, `tty_raw`, `tty_reset`, `tty_resize`, `tty_set_size`,
  `tty_set_title`, `tty_set_path`, `tty_set_selection`,
  `tty_set_progress_bar`, `tty_update_mode`, `tty_sync_start`,
  `tty_repeat_requests`, `tty_send_requests`, `tty_clipboard_query`,
  `tty_window_offset`, `tty_window_offset1` and `tty_window_bigger` all take
  it. They mutate, so it is `&mut tty` for most and `&tty` for
  `tty_window_bigger` and the query side.
- [x] (bd177cfa) `colour_palette_init`, `_clear`, `_free`, `_get`, `_set`,
  `_from_defaults` and `options_load_pane_colours` take `*mut colour_palette`,
  reached as `&raw mut (*wp).palette` or `&raw mut (*pd).palette`. All but
  `_init` open with a null check, and `tty_check_fg`/`_bg`/`_us` do pass null,
  so this one wants `Option<&colour_palette>` / `Option<&mut colour_palette>`
  rather than a plain reference.
- [x] (f04ad283) `mouse_event` is passed as `*mut` but only read: `cmd_mouse_at` and
  `cmd_mouse_window` open with `let m = &*m;`, and `cmd_mouse_pane`,
  `input_key_pane`, `window_copy_start_drag` and the two
  `cmd_resize_pane_mouse_update_*` helpers take the same parameter. 17
  non-test call sites, mostly `&raw mut m` on a local. `tty_keys_mouse` is the
  one that fills the event rather than reading it, so it wants `&mut`.
- [x] (8f3a3743) Out-parameters that could be return values: `tty_default_features` and
  `tty_add_features` (`feat: *mut c_int`), `mode_tree_key` and
  `tty_keys_next1` (`key: *mut key_code`), `screen_redraw_two_panes`
  (`type_0: *mut layout_type`), `utf8_from_data` (`uc: *mut utf8_char`),
  `job_transfer` (`pid: *mut pid_t`) and `window_pane_get_new_data` /
  `window_pane_update_used_data` (`wpo: *mut window_pane_offset`). Each
  caller declares a local only to take `&raw mut` of it. `compat/`'s
  `ibuf_get_n8`/`_n16`/`_n32`/`_n64` and friends have the same shape but
  mirror upstream imsg — leave them.
- [x] (9b55661f) The `.as_ptr()` promotions have a tail on the command side:
  `cmd_list_first`, `_at`, `_all`, `_all_have`, `_any_have`, `_print`,
  `_append`, `_append_all`, `_copy` and `_move` take `*mut cmd_list` from
  callers holding a `CmdListRef`, and `cmdq_continue`, `cmdq_get_state_ref`,
  `cmdq_insert_after`, `cmdq_add_format` and `cmdq_add_formats` do the same
  from a held item or state handle. 24 and 36 non-test sites.
- [x] (ef6dd88c) Small Rust containers still reached by pointer: `style_ranges_free` and
  `style_ranges_get_range` (`*mut style_ranges`, a `Vec`), `insert_tail`,
  `insert_head`, `insert_before`, `insert_new_tail` and `replace` in
  `layout/cells.rs` (`*mut layout_cells`), `entries` in `arguments.rs`,
  `item_of_id` and `mode_tree_last` in `modes/widget.rs`, and
  `citem_free_all` in `screen/write.rs`. `style_ranges_init` is not one of
  these — it `ptr::write`s over memory that was never initialised — and
  neither are `list.rs`'s `foreach` walkers, whose whole contract is that the
  list may be mutated while the walk is in progress.
- [ ] see xmalloc and xcmalloc functions, they are only used by test code. Find
  such unused functions and try to remove them or move them to tests only
  utility.
- [ ] 34 raw-pointer parameters across 29 functions are never used as pointers
  at all — every mention in the body is `&*p` or `&mut *p`, so the pointer is
  there only to be undone. `sort.rs`'s comparators (`sort_buffer_cmp`,
  `sort_client_cmp`, `sort_session_cmp`, `sort_winlink_cmp`), `environ_copy`,
  `environ_set`, `environ_clear`, `environ_update`, `tty_check_fg`,
  `tty_check_bg`, `tty_check_us`, `tty_fake_bce`, `grid_cells_look_equal`,
  `grid_set_tab`, `format_grid_line`, `format_grid_hyperlink`,
  `window_copy_init`, `window_copy_stringify`, `window_tree_draw_label`,
  `window_tree_get_target`, `options_value_free`, `options_is_string`,
  `cmdq_find_flag`, `cmdq_print_data`, `server_client_command_done`,
  `status_prompt_accept`, `key_from_data`, `buffer_bytes` and `list_concat`
  are the whole list. One signature each; a caller that holds a pointer moves
  the deref to the call. `sort_pane_cmp` is the same shape one step out — it
  opens `let a = &*wpa;` but keeps `wpa` for `window_pane_index`, so it comes
  with that callee.
- [ ] `binding_of` takes `*mut args` only because `args_value` and
  `args_values` do, so `cmd_bind_key_exec` reaches for
  `cmd_get_args_ptr(self_0)` two lines after `cmd_get_args(self_0)` handed it
  `&args`; `cmdq_insert_hook` binds both forms on adjacent lines.
  `args_value`, `args_values`, `args_print` and `args_copy` are the whole
  `*mut args` set outside the mode entry points — promote them to `&args` and
  three of `cmd_get_args_ptr`'s six callers go away.
- [ ] The three left are `WindowMode::init` and `WindowMode::command` in
  `modes/dispatch.rs`, which take `wme: *mut window_mode_entry`,
  `args: *mut args` and `m: *mut mouse_event` and immediately hand the table
  `&mut *wme`. The mode `init`/`command` function-pointer table is the cut:
  it is ours, not tmux's, so the whole row can move to references.
- [ ] `winlinks` is a `BTreeMap<c_int, Box<winlink>>`, yet every lookup goes
  through `&raw mut (*s).windows` — 81 sites outside the tests and 150 in
  them. `winlink_count` is `(*wwl).len()`, and `winlinks_first`,
  `winlinks_last`, `winlinks_next`, `winlinks_prev`, `winlink_find_by_index`,
  `winlink_find_by_window` and `winlink_find_by_window_id` only read. Those
  take `&winlinks`; `winlink_add`, `winlink_remove` and the
  `winlink_stack_push`/`_remove` pair take `&mut`. The finders still hand back
  `*mut winlink`, so keep the provenance honest — a shared borrow that a
  caller then writes through is worse than what is there now.
- [ ] `grid_cell` is passed by pointer everywhere: 182 non-test call sites
  across 38 callees build a `&raw mut gc` or `&raw const grid_default_cell`
  just to satisfy a `*const grid_cell` parameter. `screen_write_cell`,
  `screen_write_putc`/`_puts`/`_nputs`/`_text`, `screen_write_collect_add`,
  `grid_cells_equal`, `screen_select_cell`, `screen_set_selection`,
  `style_apply`, `style_add`, `tty_attributes`, `tty_cell`,
  `tty_default_attributes`, `tty_default_colours` and
  `screen_redraw_border_set` are the ones with the most callers. `*const
  grid_cell` becomes `&grid_cell` and `*mut grid_cell` `&mut grid_cell`; it is
  big but it is the single widest instance of the pattern.
- [ ] `*mut screen` the same way: 140 non-test sites, nearly all of them
  `&raw mut (*wp).base`, `&raw mut (*wp).status_screen` or
  `&raw mut (*sl).screen`. `screen_init`, `screen_free`, `screen_reinit`,
  `screen_resize`, `screen_set_title`, `screen_clear_selection`,
  `screen_write_start`, `screen_write_start_pane`, `screen_write_fast_copy`
  and `screen_write_preview` are the set. `screen_grid_ptr` and
  `screen_saved_grid_ptr` are the accessors already carved out for this, so
  they go with it.
- [x] (e9e89cbb) `*mut tty` is reached as `&raw mut (*c).tty` at 46 non-test
  sites. `tty_init`, `tty_open`, `tty_close`, `tty_free`, `tty_start_tty`,
  `tty_stop_tty`, `tty_raw`, `tty_reset`, `tty_resize`, `tty_set_size`,
  `tty_set_title`, `tty_set_path`, `tty_set_selection`,
  `tty_set_progress_bar`, `tty_update_mode`, `tty_sync_start`,
  `tty_repeat_requests`, `tty_send_requests`, `tty_clipboard_query`,
  `tty_window_offset`, `tty_window_offset1` and `tty_window_bigger` all take
  it. They mutate, so it is `&mut tty` for most and `&tty` for
  `tty_window_bigger` and the query side.
- [ ] `colour_palette_init`, `_clear`, `_free`, `_get`, `_set`,
  `_from_defaults` and `options_load_pane_colours` take `*mut colour_palette`,
  reached as `&raw mut (*wp).palette` or `&raw mut (*pd).palette`. All but
  `_init` open with a null check, and `tty_check_fg`/`_bg`/`_us` do pass null,
  so this one wants `Option<&colour_palette>` / `Option<&mut colour_palette>`
  rather than a plain reference.
- [x] (f04ad283) `mouse_event` is passed as `*mut` but only read: `cmd_mouse_at` and
  `cmd_mouse_window` open with `let m = &*m;`, and `cmd_mouse_pane`,
  `input_key_pane`, `window_copy_start_drag` and the two
  `cmd_resize_pane_mouse_update_*` helpers take the same parameter. 17
  non-test call sites, mostly `&raw mut m` on a local. `tty_keys_mouse` is the
  one that fills the event rather than reading it, so it wants `&mut`.
- [x] (8f3a3743) Out-parameters that could be return values: `tty_default_features` and
  `tty_add_features` (`feat: *mut c_int`), `mode_tree_key` and
  `tty_keys_next1` (`key: *mut key_code`), `screen_redraw_two_panes`
  (`type_0: *mut layout_type`), `utf8_from_data` (`uc: *mut utf8_char`),
  `job_transfer` (`pid: *mut pid_t`) and `window_pane_get_new_data` /
  `window_pane_update_used_data` (`wpo: *mut window_pane_offset`). Each
  caller declares a local only to take `&raw mut` of it. `compat/`'s
  `ibuf_get_n8`/`_n16`/`_n32`/`_n64` and friends have the same shape but
  mirror upstream imsg — leave them.
- [ ] The `.as_ptr()` promotions have a tail on the command side:
  `cmd_list_first`, `_at`, `_all`, `_all_have`, `_any_have`, `_print`,
  `_append`, `_append_all`, `_copy` and `_move` take `*mut cmd_list` from
  callers holding a `CmdListRef`, and `cmdq_continue`, `cmdq_get_state_ref`,
  `cmdq_insert_after`, `cmdq_add_format` and `cmdq_add_formats` do the same
  from a held item or state handle. 24 and 36 non-test sites.
- [x] (ef6dd88c) Small Rust containers still reached by pointer: `style_ranges_free` and
  `style_ranges_get_range` (`*mut style_ranges`, a `Vec`), `insert_tail`,
  `insert_head`, `insert_before`, `insert_new_tail` and `replace` in
  `layout/cells.rs` (`*mut layout_cells`), `entries` in `arguments.rs`,
  `item_of_id` and `mode_tree_last` in `modes/widget.rs`, and
  `citem_free_all` in `screen/write.rs`. `style_ranges_init` is not one of
  these — it `ptr::write`s over memory that was never initialised — and
  neither are `list.rs`'s `foreach` walkers, whose whole contract is that the
  list may be mutated while the walk is in progress.
