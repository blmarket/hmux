# Rc model reference audit

This tracks the second item in `../issues.md`. A stored `Rc` is justified by a
matching model-reference lifecycle; a local upgrade used only to guard access
does not make its `Weak` field an owning holder. The full inventory remains in
`rc-entity-inventory.md`.

## Completed field reviews

| Holder | Result | Evidence |
| --- | --- | --- |
| `spawn_context.wp0` | `Weak`; the context did not have a matching pane release. | `spawn_window` retains the source pane locally through layout rebuilding; `spawn_pane` and logging retain local upgrades. Regression verifies context drop does not keep the pane alive. |
| `spawn_context.tc` | `Weak`; the context did not have a matching client unref. | `spawn_window` and `spawn_pane` retain an upgrade while using the client. Regression verifies expiry after the independent owner drops. |
| `spawn_context.s` | `Weak`; the context did not have a matching session remove-ref. | `spawn_window`, `spawn_pane`, and pane-created payload setup retain local upgrades. Regression verifies expiry after the independent owner drops. |
| `monitor_set.session` | Retained `Rc` with explicit release. | `monitor_create_session` upgrades once; `monitor_clear` takes that value and calls `session_remove_ref`. |
| `cmd_run_shell_data.s` | Retained `Rc` with explicit release. | Creation upgrades the session observer; `Drop` takes it and calls `session_remove_ref`. |
| `format_tree.client` | Retained `Rc` with explicit release. | Constructors retain or clone the client; `format_clear` takes it and calls `server_client_unref_owned`. |
| Event payload model values and targets | Retained `Rc` with explicit release. | `events_payload.rs` sets model values/targets and releases each through its model-specific function in item drop or `event_payload_free_target`. |

The release-pair entries establish how the current translation balances those
stored references. They do not by themselves finish the upstream-equivalence
review of every creation site.

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
