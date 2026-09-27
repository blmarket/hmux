# Rc ownership audit

The former `shared::rc::take` call sites now obtain ownership from an existing
`Rc` or by upgrading a `Weak` created with the allocation. Ownership is never
transferred through a raw pointer. `shared::rc` only provides borrowed pointer
views and deferred dropping of an ordinary `Rc`.

| Former source | Authoritative owner and transfer |
| --- | --- |
| Command-list construction, copying, and parse results | Constructors return `Rc<UnsafeCell<cmd_list>>`. Arguments, options, bindings, and parse results move or clone it. Queue items clone the supplied `Rc`. |
| Command-queue state construction and linking | Constructors return `Rc<UnsafeCell<cmdq_state>>`. Queue items clone it; optional borrowed owners replace nullable owning pointers. |
| Key-table lookup and insertion | The table index stores `Rc<UnsafeCell<key_table>>`. `key_bindings_get_table` returns an optional clone. Client assignments and dispatch retain that typed handle. Removal returns the indexed `Rc`. |
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

`ClientOwner` and `WindowOwner` remain drop-policy guards, not aliases: their
release paths preserve deferred client cleanup and live window-close callbacks,
respectively. Both contain ordinary `Rc`s and release typed handles.

## Verification

- Source inspection covers every former `take` source listed above, including
  its indirect users through `release`, `downgrade`, and `strong_count`.
- The raw allocation/retention/reconstruction helpers have been deleted. The
  source contains no `Rc::from_raw`, `Rc::into_raw`, or manual strong-count updates.
- `cargo test --workspace` covers ownership release, reentrant prompt teardown,
  command-list survival, session and pane index removal, live window-close
  notification, and file completion/cancellation. The added cancellation test
  checks that cancelling completion releases both the file and its client.
- `key_bindings_get_table` returns `Option<Rc<UnsafeCell<key_table>>>`, and
  `key_bindings_add` takes `&CStr` as its first parameter.
