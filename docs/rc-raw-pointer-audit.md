# Raw pointers to application Rc entities

Reviewed 2026-09-28 at `fcc02084`.

Scope: the 12 application entities in `rc-entity-inventory.md`, scanning Rust
struct fields and function/method parameter types under `src/`, excluding
`src/compat`, test functions and test modules. Counts come from a `syn` syntax
walk, not textual matches of locals, casts or return types. This is a source
inventory, not macro-expanded or configuration-specific compiler output.
Generic pointer APIs, erased `c_void` relationships, raw return values, closure
parameters and infrastructure Rc entities are outside this first pass. Types
spelled through aliases are not recursively resolved; the relevant `NonNull`
callback alias is recorded separately below.

## Inventory

The complete enumeration is in [rc-raw-pointer-parameters.tsv](rc-raw-pointer-parameters.tsv).
Each row identifies the entity, source location, function, parameter and type.

| Entity | Explicit raw struct fields | Raw parameters | Functions/methods containing them |
| --- | ---: | ---: | ---: |
| client | 0 | 121 | 121 |
| window_pane | 0 | 117 | 116 |
| window | 0 | 113 | 112 |
| session | 0 | 71 | 69 |
| client_file | 0 | 0 | 0 |
| cmd_list | 0 | 0 | 0 |
| cmdq_state | 0 | 0 | 0 |
| key_table | 0 | 0 | 0 |
| mode_tree_data | 0 | 0 | 0 |
| window_tree_modedata | 0 | 0 | 0 |
| window_customize_modedata | 0 | 0 | 0 |
| hyperlinks | 0 | 0 | 0 |

There are 422 parameter occurrences across 373 distinct functions/methods;
functions taking multiple entity types appear in multiple table rows.
Four occurrences are output parameters: `cmd_mouse_window::sp`,
`cmd_mouse_pane::sp`, and `window_panes_get_source::{sp, wp}` use double pointers.
The other 418 are direct entity pointer parameters.
No additional explicit raw entity pointers were found nested in struct callback
signatures or function-parameter callback types.

`winlink`, `window_mode_entry`, and `cmdq_item` are not application Rc entities;
their raw pointers belong in a later pass.

## NonNull cases

These still represent raw access even though they do not use `*mut` syntax:

- `src/window.rs:1097`: `WindowTeardownScope.0` holds
  `Option<NonNull<window>>`. The associated thread-local `TEARING_DOWN_WINDOW`
  also holds this pointer. This runs while the final Rc owner is being dropped;
  changing it to an upgradeable Weak would break access during teardown.
- `src/shared/status.rs:37`: `status_prompt_input_cb` accepts
  `Option<NonNull<client>>`. The alias is used by
  `window_pane_prompt.inputcb`, `status_prompt_set`, and `window_pane_prompt`.
  This is a callback argument contract, not a stored owning client reference.
  Migration needs review of reentrant prompt teardown and callback callers.

## Small migration candidates

These candidates were checked against their bodies and source call sites.
They do not require introducing an Rc clone, Weak upgrade, or new owning wrapper.

| Function | Proposed model parameter | Why it is small |
| --- | --- | --- |
| `server_client_key_table_activity_diff` (`src/server_client.rs:1106`) | `&client` | Reads client timestamps and a checked key-table borrow; one call site. |
| `window_replace_name` (`src/window.rs:1031`) | `&mut window` | Only replaces a CString and returns the old value; no callback inside the helper. End the borrow before rename notification. |
| `window_replace_old_layout` (`src/window.rs:1022`) | `&mut window` | Only replaces an optional CString; also used during destruction, so requiring an Rc would be inappropriate. |
| `session_is_linked` (`src/session.rs:572`) | `Option<&session>`, `&window` | Reads group membership and existing strong counts; one call site. Preserve the current nullable-session behavior of `s.as_ref()`. |
| `server_client_check_nested` (`src/server_client.rs:1058`) | `&client` | Reads environment and tty name and compares against indexed panes; two callers, no client mutation or application callbacks. |
| `window_pane_destroy_ready` (`src/window.rs:1153`) | `&window_pane` | Reads pipe-buffer state, fd readiness, flags, wait state and editor presence; does not destroy or retain the pane. |

Start with the first four functions: five raw parameters, small bodies, and no
reference-count changes. Verify callers supply live objects and create borrows
only for the duration of each call. A signature change does not by itself remove
the unsafe pointer projection at an unmigrated caller.

`window_active_pane_is_over_zoom` is another small body, but it repeatedly obtains
an active-pane raw pointer from a Weak-backed accessor. Its migration should also
resolve the active pane once and keep that access valid through the query.
This requires more lifetime review than the simple field helpers above.

## Defer from the first batch

- `session_add_ref`, `window_add_ref`, `window_pane_add_ref`, `client_retain`:
  these are ownership/refcount boundaries; preserve matching explicit releases.
- `window_destroy`, `window_pane_free`, `server_client_free`, `session_free`:
  final teardown has different lifetime guarantees from ordinary operations.
- Layout mutation, event dispatch, pane mode dispatch, and prompt callbacks:
  reentrant paths need aliasing review before introducing Rust references.
- Double-pointer outputs: consider returning a typed result/tuple, but first
  establish whether each result is an owner, observer, or bounded borrow.

For synchronous reads prefer `&T`; for short exclusive mutations use `&mut T`
only where aliasing permits it. Use `&Rc<UnsafeCell<T>>` when an existing owner
must be passed through, and Weak for stored observation. Preserve dedicated
manual free/unref paths; ordinary Drop is not a replacement for their protocol.
