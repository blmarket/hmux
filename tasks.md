## Rule

The items here are independent small tasks. First pull the master branch for
the latest task states, find unchecked item, then first mark it + commit +
update master so that you can acquire the item. If conflict happens you failed
to acquire lock. Start over.

Once you acquired the task, go implement it. Once succeeded. Also update this
doc by adding commit hash on the entry the entry you worked + make `git
commit`. But don't push it to
master, so human can review and merge. Agents can only update master only for
modifying `tasks.md` file.

## Tasks

- [x] (deadbeef) `args_escape` may return CString
- [x] (beefdead) `xasprintf` may take first argument to be &mut CString or return CString
- [x] (l33t1234) `utf8_stravis` may take first argument to be &mut CString or return CString
- [x] `layout_get_tiled_cell` last argument cause should be &mut CString
- [x] (9fcfd0ca) see `sessions` access and see they can be migrated to `session_owners`.
- [x] see `alerts_callback`, I'm wondering w_ref.as_ptr() is unnecessary as
  callee can be promoted to get w_ref directly. Update tasks.md listing other
  similar patterns which can be fixed. After then, try to fix the code do the
  same without as_ptr().
- [x] (fb898888, def8aee3) I see bunch of nonsense casting such as `&(*w)`...
  can we do better than this? it happened at least on format_cb_window_layout
  but it's all over the place. Can we find such cases using AST matchers so
  that we don't miss them systematically?
- [x] See `format_add_window_neighbor`, I can see `&mut *nft` which is
  effectively just &mut nft? find all such nonsense code and report at task.md.
- [x] `make audit-deref` reports 203 `cstr_ptr(&(*p).field)` round trips left.
  They spread over more fields than the option ones did — `name` 74,
  `prompt_last` 52, `prompt_string` and `message_string` 12 each, `ttyname` 9,
  then `title`, `search`, `exit_session`, `cwd`, `shell`, `searchstr`, `path`
  and a dozen singles — so it wants a name per accessor rather than one sweep.
  `session_name` and `session_cwd` are the shape already there.
- [x] `peer_ptr(&(*c).peer)` 28 times and `environ_ptr(&(*x).environ)` 11:
  same treatment as the option sets, one accessor per owner.
- [x] `cmd_get_args_ptr(&*item.cmd())` and the two `commands_ptr` /
  `args_ptr` sites are the tail of the round trips; they deref a handle
  rather than a field, so they belong with the `as_ptr()` promotions above.
- [x] `make audit-deref AUDIT_ARGS=` reports 3602 borrows through a deref
  that are not round trips — `&(*c).prompt_buffer` and friends, where the
  callee really does want the borrow. Those are only worth touching where
  the owner can be promoted to a reference for the whole function; the tool
  is what says which functions carry enough of them to be worth it.
- [ ] `screen_write_offset_timer` takes `*mut window` and looks the handle up
  again, though its only caller is the offset timer callback that has already
  upgraded one. Promote it to `&WindowRef`, the way `alerts_timer` now is.
- [ ] `resize.rs`'s `recalculate_size` is reached as
  `recalculate_size(w_ref.as_ptr(), now)` from the `each_window()` walk in
  `recalculate_sizes_now`; promote it to take the handle.
- [ ] `server_client_loop` walks `window_refs()` and hands
  `server_client_check_window_resize` a pointer straight out of the handle;
  promote that callee too.
- [ ] `options/store.rs` opens four `each_window()` walks with
  `let w = w_ref.as_ptr();`. The bodies are field pokes, but the per-window
  work could move into helpers taking `&WindowRef`.
- [ ] `name_time_callback` takes `&window` and is reached as
  `name_time_callback(&*w_ref.as_ptr())` from the name timer, which already
  holds the handle; let it take `&WindowRef`.
- [ ] The `each_session()` walks now hand over `SessionRef`, but every body
  still opens with `s_ref.as_ptr()` because the callees take `*mut session`.
  `session_clear_attached`, `session_update_history`, `status_update_cache`,
  `server_status_session`, `server_renumber_session` and `session_destroy` are
  the ones worth promoting to `&SessionRef` first.
- [ ] `key_bindings_add`, `key_bindings_remove`, `key_bindings_reset`,
  `key_bindings_remove_table` and `key_bindings_reset_table` each look a
  `KeyTableRef` up and drop it to `*mut key_table` on the next line, and
  `key_bindings_init_done` and `cmd_send_keys.rs` do the same. The callees
  they hand the pointer to — `key_bindings_get`, `key_bindings_next`,
  `key_bindings_take_defaults` — could take `&KeyTableRef`.
- [ ] `file.rs` degrades `ClientFileRef` to `*mut client_file` in around
  eighteen places, most of them one `let cf = cf_ref.as_ptr();` per function.
  The file callbacks are the natural cut: promote them to `&ClientFileRef`.
- [ ] `modes/widget.rs`, `modes/client.rs`, `modes/buffer.rs` and
  `cmd_source_file.rs` each open a callback with
  `let data = data_ref.as_ptr();`; same treatment.
