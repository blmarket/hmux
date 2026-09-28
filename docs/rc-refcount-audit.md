# Rc model reference audit

This tracks the second item in `../issues.md`. Commit `5606919f` contains the
initial C-to-Rust transcription with explicit tmux reference operations and is
the local baseline for comparing ownership transitions. A stored `Rc` is justified by a
matching model-reference lifecycle; a local upgrade used only to guard access
does not make its `Weak` field an owning holder. The full inventory remains in
`rc-entity-inventory.md`.

## Completed field reviews

| Holder | Result | Evidence |
| --- | --- | --- |
| `spawn_context.wp0` | `Weak`; the context did not have a matching pane release. | Baseline `src/spawn.rs` stores a raw `wp0` and has no matching pane add-ref. Current `spawn_window` retains the source pane locally through layout rebuilding; `spawn_pane` and logging retain local upgrades. Regression verifies context drop does not keep the pane alive. |
| `spawn_context.tc` | `Weak`; the context did not have a matching client unref. | Baseline `src/spawn.rs` stores a raw `tc` and has no matching client ref. Current `spawn_window` and `spawn_pane` retain an upgrade while using the client. Regression verifies expiry after the independent owner drops. |
| `spawn_context.s` | `Weak`; the context did not have a matching session remove-ref. | Baseline `src/spawn.rs` stores a raw `s` and has no matching session add-ref. Current `spawn_window`, `spawn_pane`, and pane-created payload setup retain local upgrades. Regression verifies expiry after the independent owner drops. |
| `monitor_set.session` | Retained `Rc` with explicit release. | Baseline `src/monitor.rs::monitor_create_session` calls `session_add_ref`; current `monitor_create_session` upgrades once and `monitor_clear` calls `session_remove_ref`. |
| `cmd_run_shell_data.s` | Retained `Rc` with explicit release. | Baseline `src/cmd_run_shell.rs` calls `session_add_ref`; current creation upgrades the session observer and `Drop` calls `session_remove_ref`. |
| `format_tree.client` | Retained `Rc` with explicit release. | Baseline `src/format.rs::format_create` increments `client.references` and `format_free` calls `server_client_unref`; current constructors retain or clone the client and `format_clear` calls `server_client_unref_owned`. |
| `cmdq_item.client_owner` | Retained `Rc` with explicit release. | Baseline `src/cmd_queue.rs::cmdq_append` increments `client.references` for each queued item and `cmdq_remove` calls `server_client_unref`; current append clones the owner and removal calls `server_client_unref_owned`. |
| `args_command_state.client` | Retained `Rc` with explicit release. | Baseline `src/arguments.rs::args_make_commands_prepare` increments the target client's references and `args_make_commands_free` calls `server_client_unref`; current preparation stores the upgraded client and `Drop` calls `server_client_unref_owned`. |
| `client_file.c` | Retained `Rc` with explicit release. | Baseline `src/file.rs::file_create_with_client` increments `client.references` and `file_free` calls `server_client_unref`; current file creation clones the client owner and `file_destroy` calls `server_client_unref_owned`. |
| `mode_tree_prompt.mtd` | Retained `Rc` with explicit release. | Baseline `src/mode_tree.rs::mode_tree_set_prompt` increments the tree's references and its prompt free callback calls `mode_tree_remove_ref`; current prompt creation clones the tree and its free callback drops that clone after user cleanup. |
| Event payload model values and targets | Retained `Rc` with explicit release. | `events_payload.rs` sets model values/targets and releases each through its model-specific function in item drop or `event_payload_free_target`. |

These baseline comparisons verify the listed ownership transitions. They do
not finish the review of every creation site.

## Remaining audit

- Review stored strong model holders in command queues, prepared commands,
  clients, file records, prompts, mode data, and callback captures. Trace each
  clone or upgrade to its final release and compare it with the intended tmux
  ownership transition. Convert unmatched lasting holders to `Weak` or a bounded
  borrow while preserving scoped guards for access.
- Review ordinary `Rc` clones in code paths that transfer ownership into those
  holders; distinguish a new reference from a temporary guard or moved owner.
- Keep `src/` free of `Option<Weak<T>>` fields. Empty `Weak` is the absent case;
  use allocation identity where an expired explicit target must stay distinct.
