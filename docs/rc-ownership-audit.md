# Rc ownership audit

The current [entity and holder inventory](rc-entity-inventory.md) tracks the 12
application Rc entities, infrastructure Rc values, ownership classifications,
and remaining raw relationships. It also records the weak window-index migration.

The former `shared::rc::take` call sites now obtain ownership from an existing
`Rc` or by upgrading a `Weak` created with the allocation. Ownership is never
transferred through a raw pointer. `shared::rc` only provides borrowed pointer
views and deferred dropping of an ordinary `Rc`.

| Former source | Authoritative owner and transfer |
| --- | --- |
| Command-list construction, copying, and parse results | Constructors return `Rc<RefCell<cmd_list>>`. Arguments, options, bindings, and parse results move or clone it. Queue items clone the supplied `Rc`. |
| Command-queue state construction and linking | Constructors return `Rc<cmdq_state>`. Queue items clone it; optional borrowed owners replace nullable owning pointers. |
| Key-table lookup and insertion | The table index stores `Rc<RefCell<key_table>>`. `key_bindings_get_table` returns an optional clone. Client assignments and dispatch retain that typed handle. Removal returns the indexed `Rc`. |
| Mode-tree allocation and prompt retention | Mode records store `Option<Rc<UnsafeCell<mode_tree_data>>>`. Prompt and key handlers receive typed owners. Logical teardown consumes the mode's handle while callbacks can keep their own handles alive. |
| Tree and customization mode data | The mode entry stores `Rc<dyn Any>` pointing to the typed allocation. Prompt and queued callback owners retain typed `Rc`s. The raw mode-data field is a borrowed callback view. |
| Client registration and release | `ClientRegistry` stores the initial `Rc`. Client-loss cleanup transfers it to deferred release. Command items, queued key events, and event payloads retain explicit owners. |
| Session construction, destruction, and event targets | The session index stores `Rc<UnsafeCell<session>>`. Removal returns that owner, including during rename. Destruction defers its drop. Event payloads retain a typed session owner alongside the borrowed target state. |
| Window construction, link retention, alerts, and event targets | Construction returns `Rc<UnsafeCell<window>>`. Winlinks, alerts, and event payloads retain typed owners. `window_remove_ref` consumes an `Rc` and preserves notification before the final release. |
| Pane allocation and event references | The pane index owns `Rc<UnsafeCell<window_pane>>`. Logical destruction removes and consumes its owner. Event payloads retain their own `Rc`s. Window lists and asynchronous observers use `Weak`. |
| File-transfer construction, completion, and retries | The stream index owns active `Rc<UnsafeCell<client_file>>` records. Completion and retry events retain typed handles. Completion's drop guard unlinks the record on dispatch or cancellation, breaking the file/client ownership cycle. |
| Test fixtures and deferred-release tests | Fixtures construct ordinary `Rc`s, downgrade them normally, and drop them explicitly. No fixture reconstructs an owner from an address. |

## Borrowed callbacks and observers

Legacy callback signatures still pass raw model addresses. A callback may need
to keep its model alive across reentrant teardown, or schedule work that should
observe its later disappearance. Such models record a `Weak<UnsafeCell<Self>>`
using `Rc::new_cyclic`. Callers clone or upgrade that observer; they never infer
ownership from the address or increment a raw reference count. The self-observer
does not keep the allocation's value alive. Owning fields and constructor return
types remain explicit `Rc`s; `KeyTableOwner` has been removed.

Client references use ordinary `Rc<UnsafeCell<client>>` values. Cleanup explicitly
passes retained references to `server_client_unref_owned` for deferred release.
Optional record fields support absent clients and taking references at teardown.
Window holders now store `Rc<UnsafeCell<window>>` directly and perform explicit releases.
Lookup and traversal callers call `window_remove_ref` on completion and early exits.
Winlink release shares the notification helper with `window_remove_ref`, then
takes and drops its field so callbacks can still inspect the link during
notification. Ordinary callers pass their Rc directly to `window_remove_ref`.
Normal unlinking releases that reference before dropping the winlink.

## Verification

- Source inspection covers every former `take` source listed above, including
  its indirect users through `release`, `downgrade`, and `strong_count`.
- The raw allocation/retention/reconstruction helpers have been deleted. The
  source contains no `Rc::from_raw`, `Rc::into_raw`, or manual strong-count updates.
- `cargo test --workspace` covers ownership release, reentrant prompt teardown,
  command-list survival, session and pane index removal, live window-close
  notification, and file completion/cancellation. The added cancellation test
  checks that cancelling completion releases both the file and its client.
- `key_bindings_get_table` returns `Option<Rc<RefCell<key_table>>>`, and
  `key_bindings_add` takes `&CStr` as its first parameter.

## First 20 borrowed-pointer migrations

These construction paths already had an authoritative client `Rc`, making them
low-risk candidates: they no longer project a raw client pointer only for the
format constructor to upgrade the client's self-observer back into an `Rc`.
`format_create_with_client` accepts `Option<&Rc<UnsafeCell<client>>>` and clones
it into the tree. A `Weak` would change the existing retention contract.

| # | Migrated call site | Handle source |
| --- | --- | --- |
| 1 | `list_buffers.rs`: list-buffers formatting | Queue client |
| 2 | `list_commands.rs`: list-commands formatting | Queue client |
| 3 | `list_clients.rs`: list-clients formatting | Queue client |
| 4 | `list_sessions.rs`: list-sessions formatting | Queue client |
| 5 | `list_windows.rs`: list-windows formatting | Queue client |
| 6 | `list_panes.rs`: list-panes formatting | Queue client |
| 7 | `list_keys.rs`: list-keys formatting | Queue client |
| 8 | `kill_window.rs`: window filter | Queue client |
| 9 | `kill_session.rs`: session filter | Queue client |
| 10 | `kill_pane.rs`: pane filter | Queue client |
| 11 | `pipe_pane.rs`: command expansion | Queue client |
| 12 | `display_message.rs`: message formatting | Queue client |
| 13 | `wait_for.rs`: event filter | Queue client |
| 14 | `format_create_defaults` | Queue client |
| 15 | `format_loop_sessions` | Parent format tree client |
| 16 | `format_loop_windows` | Parent format tree client |
| 17 | `format_loop_panes` | Parent format tree client |
| 18 | `format_loop_add_option` | Parent format tree client |
| 19 | `format_loop_add_array_item` | Parent format tree client |
| 20 | `format_loop_environ` | Parent format tree client |

The first 14 remove direct `shared::rc::as_ptr` projections; the last six remove
projections through `client_rc_ptr`. Loop functions that previously saved a raw
client address now retain a cloned handle across iterations. Legacy constructors
still accept raw pointers for callers awaiting migration, and both paths share
the same construction and deferred-release implementation.

The typed-constructor lifetime test checks allocation identity, survival after
the source handle is dropped, deferred release on dispatch and cancellation,
and construction without a client.
