# Execute ownership migration in subsystem batches

Migrate related ownership boundaries together, preserving behavior, identity,
and destruction semantics. Prioritize shared contracts that unblock many callers
and remove repeated manual lifetime management. Executing this prompt means making and
validating production code changes, not merely producing another inventory or a
proposal. Creating this document alone does not execute it.

## Agreed type policy

Choose the simplest type justified by actual ownership and aliasing:

- Keep translated structs and fields mapped one-to-one to their tmux counterparts
  where possible. Preserve corresponding names and roles; do not use an existing
  field as a generic owner for unrelated fields. When Rust ownership or ABI
  constraints require an exception, document the exact constraint and keep the
  exception local until it can be removed.

| Existing role | Preferred representation |
| --- | --- |
| Owned NUL-terminated string | `CString` |
| Borrowed NUL-terminated string | `&CStr` |
| Owned object without escaping weak references | `Box<T>`, or inline `T` when indirection is unnecessary |
| Owned object with escaping non-owning references | `RefBox<T>` at the owner and `refbox::Weak<T>` at observers |
| Temporary access | `&T` / `&mut T` with scoped lifetimes |
| Object with an existing explicit tmux reference count | Consider `Rc<T>` only after auditing its existing retain/release semantics |
| Owned bytes or elements | `Vec<T>`, boxed slices, or an appropriate standard collection |
| Borrowed bytes or elements | Slices |
| Legitimately absent value | `Option` of the appropriate type |

- Do not introduce generational handles, replacement object-ID registries, or
  new identity semantics. Preserve existing numeric IDs and indexes.
- Do not use raw pointer addresses as identifiers or collection keys. Use
  direct ownership/observers or existing semantic IDs and names.
- Do not put everything in `RefBox`. Prefer ordinary ownership when references
  do not escape. A temporary raw pointer for a synchronous C call does not, by
  itself, require `RefBox`.
- Do not introduce `Rc` or `Arc` for objects without an existing reference
  count. An existing counter is a reason to investigate `Rc`, not proof that
  its semantics match. Document which operations retain and release ownership.
- Required weak access must assert that the target is alive, with a useful
  invariant message. Do not silently filter, skip, or default expired links.
  Inspect the installed refbox API: use its checked access/borrow equivalent
  when it has no `upgrade()` method. Treat borrow conflicts as bugs to resolve,
  not reasons to fall back to unchecked pointers. A failure may reveal a tmux
  bug or a migration bug; investigate both.
- Represent intentional absence explicitly. Remove observers at the proper
  teardown point rather than making expired weak references normal control flow.
- Keep strings byte-preserving. Do not assume UTF-8, add lossy conversion, or
  change embedded-NUL handling. Preserve null versus empty where observable.

## Select a coherent batch

Read the current priorities and working diff, then choose one subsystem or
shared producer/consumer contract. State its completion condition and affected
callers before editing. Reuse existing audits unless source changes invalidate
them; do not rediscover the whole repository for each allocation.

Group related local strings and leaf records in one pass. Prefer changing an
internal return contract and its production callers over adding owners that
immediately copy back into C storage. Retain adapters for required foreign
contracts. Do not defer an enabling lifecycle change merely because isolated
string conversions remain; bound graph work to one owner and its observers.

## Complete each ownership boundary within the batch

1. Trace every creation, mutation, ownership transfer, alias, and destruction
   path for the selected allocation, including errors and early returns.
2. Introduce the chosen Rust owner and migrate its actual production callers.
   Remove obsolete manual frees, counts/capacities, and pointer manipulation
   where the new type supplies those responsibilities. Prefer safe operations
   with private owner fields over public raw-pointer accessors.
3. Migrate allocation and every corresponding deallocation together. Tests
   cannot establish allocator compatibility; audit it explicitly. Never
   libc-free Rust-owned storage or reconstruct a Rust owner from an allocation
   that does not satisfy its allocator contract.
4. Audit the containing record before adding a drop-bearing field. `repr(C)`
   alone is not the obstacle: `Copy`, bitwise copies, zero initialization,
   `xcalloc`, `realloc`, and `free` can bypass Rust invariants and destruction.
   Change that whole lifecycle together, or select a smaller target. Remove
   `Copy` when ownership moves into a record. Do not add a custom `Drop` while
   leaving its only destruction path as libc `free`.
5. Preserve required ABI signatures and layouts during a bounded increment.
   Distinguish genuine foreign boundaries from translated internal conventions;
   internal Rust APIs may change with all callers and relevant contracts.
   Keep any necessary compatibility raw pointer private or narrowly exposed,
   document its owner and invalidation rule, and keep unsafe conversion local.
   Merely renaming a raw pointer to `NonNull` or wrapping an existing collection
   without eliminating a manual lifetime responsibility is insufficient.
6. Preserve pointee address stability when aliases depend on it. Never turn
   shared/reentrant raw access into an unrestricted `&mut T`. End Rust borrows
   before callbacks, hooks, or destruction that can reenter the same object.
   Do not extend a borrow's lifetime or add unchecked interior mutability to
   get the conversion compiling.
7. Preserve destruction and notification order, cancellation, traversal order,
   duplicate-key behavior, and existing reference-count lifetime extensions.
   Changes exposed by a failing invariant require investigation and regression
   coverage, not weakening the assertion.

