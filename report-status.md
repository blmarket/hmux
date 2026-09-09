# Command interface milestone: agent work plan

Updated 2026-09-08. Implementation baseline: `7a0d11ea4`. Paths are relative to `hmux/` unless prefixed with `../`. This document is a continuing work plan: each agent verifies the current source and updates the task queue and completion records. File references identify starting points, not a substitute for inspecting current code.

**Execution contract**

When the user says **`execute ./report-status.md`**, carry out one complete iteration of this work plan. The user has designated that invocation as authorization to implement the selected slice, run its required validation, update this report, and commit all task-related changes locally on **`h1`**. Remote publication requires separate explicit user approval. Reading, reviewing, or discussing this file alone does not trigger execution or publication. Explicit instructions in the invoking message take precedence.

- Read applicable `AGENTS.md` and the rest of this report, verify `h1`, and inspect the working tree and local commits. Reconcile the queue with current code before choosing work. Preserve unrelated local changes.
- Complete the default next ready implementation slice, or the slice explicitly selected by the user. Make routine implementation decisions autonomously within its contract. One invocation completes one bounded slice, not every remaining family.
- Run the required gates and inspect their results. Keep implementation and conformance changes separate. Do not publish an implementation with unresolved required validation failures or changes requiring outstanding human signoff; record the dependency and report it accurately instead.
- Update the queue and completion record with the actual result, validation evidence, remaining escapes, and the next ready slice. Include the report update in the task commit. Commit only task-related files; do not sweep unrelated work into the commit.
- Keep validated commits local on `h1`. Report the commit, review range and test results; the user reviews and decides whether to push. Do not push to any remote without explicit approval.
- If the default slice is already complete, select the next ready slice after verifying its prerequisites. If all migration work is complete, perform Z1. If Z1 is already complete, verify and report milestone completion rather than inventing more work. If no task can proceed, report the concrete dependency instead of marking unfinished work done.

This execution contract does not authorize changing public traits or foundational choices without the signoff required below. It also does not authorize remote publication, parallel agents, force-pushing, discarding unrelated changes, or weakening gates.

**Goal and completion contract**

Make commands consumers of well-defined subsystem interfaces. A command may parse arguments, choose command-specific policy, resolve targets through the targeting API, invoke operations, format results, and choose its return status. The subsystem owns storage, resource lifetimes, borrowing, and the invariants that keep related state consistent.

The milestone is complete when:

- Production command handlers, their helpers, and their deferred callbacks do not extract raw client/session/window/pane/TTY payloads or mutate another subsystem's storage.
- After consumer migration, Rust module and item visibility prevents `cmd` from accessing subsystem internals.
- Operations define target validity, inputs, effects, failures, ownership, and callback behavior. Commands do not need undocumented implementation knowledge to use them correctly.
- Operations preserve relationships such as pane ownership, layout membership, inherited options, and registration. Commands do not manually coordinate those invariants through field setters.
- Remaining unsafe calls have explicit, justified contracts at the consumer boundary. A method does not become safe merely because its raw access moved into a wrapper. Eliminate caller preconditions that the command cannot establish through its interfaces.
- Observable results, errors, selection, hooks, notifications, and asynchronous completion remain covered by the applicable conformance gates.
- A final audit covers every production command and callback, and finds no untracked interface escapes. Passing tests alone does not establish this architectural result.

Start with `src/cmd/cmd_*.rs`, including helpers outside those files. `find.rs`, `parse.rs`, and `queue.rs` implement command infrastructure and may access their own storage; assess their interfaces to command consumers separately. Typed snapshots and scoped access through an established trait are acceptable when the operation contract supports them. Counting removed `unsafe` blocks, getters, or renamed functions is not a completion measure.

**Decisions agents must preserve**

- Conformance E2E scenarios are the trusted behavior gates. Unit tests can support internal invariants, but neither their count nor removal establishes the quality of user-visible coverage.
- Implementation and conformance changes must be separate reviewed changes. Do not modify hmux implementation and conformance scenarios, harnesses, expected results, exclusions, or test-selection configuration in the same change. Two commits in one combined review do not satisfy this separation.
- If a material behavior lacks coverage, prepare a conformance-only change first, establish its expected behavior independently, and have it reviewed through the normal process before the dependent implementation. Keep implementation work isolated; do not weaken the oracle to accommodate it. Existing sufficient coverage does not need duplication.
- Use existing gates. `make test-commands` is the primary dedicated command gate; run hooks, queue, and notification suites when affected. No new combined test target is needed. There is no separate milestone task to police unit-test deletion.
- Reuse existing APIs when semantics match. A boolean name-existence query cannot replace a query that distinguishes missing, unique, and ambiguous matches. A similarly named alert operation may perform additional effects; inspect it before reuse.
- Do not copy an entire store merely to remove raw payload access. Prefer a scoped read when the consumer can finish synchronously; require a concrete lifetime or consistency need for an owned snapshot.
- Keep command-specific ordering and option policy in commands. Move reusable state invariants into operations. Do not move an entire handler behind an opaque wrapper merely to satisfy a search check.
- Preserve object identity, allocation lifetime, registration, and membership as distinct concepts. Do not replace references with IDs, or strong with weak references, as a universal cleanup rule.
- Preserve distinct contexts, including control-mode and attached-client paths. Preserve static dispatch and existing representations during ordinary consumer migrations. Encapsulation does not require trait objects, a new crate, or a giant server interface.
- Follow `AGENTS.md`. Existing public trait changes require explicit human signoff, including methods, bounds, associated items, and semantics. Foundational component replacement or a competing implementation also requires signoff. Prefer inherent methods and private request/result types; keep optional capabilities separate from `TmuxServer`.
- Use rustdoc for needed code contracts. Do not add code comments referencing this report or other disposable lowercase document filenames.

**How to run one agent iteration**

1. Read applicable `AGENTS.md`, this report, git status, and recent commits. Verify `h1` and check whether a local change already completed the task. Preserve unrelated work; do not reset or overwrite it. Follow any applicable task-claim protocol if selecting work from `tasks.md`.
2. Select one ready task below, or one explicitly named by the user. A family is not a single implementation assignment: take its stated first slice. Inspect callers and existing APIs before editing. Record the selected slice and scope in the report if the work spans multiple iterations.
3. State the operation contract and invariants before implementation: retained versus resolved targets, missing/ambiguous cases, mutation order, errors, redraw/hook/notification effects, and callback/reentrancy boundaries. Identify the existing conformance scenarios that exercise them.
4. Migrate the bounded consumers. Add only missing subsystem operations, retain their legitimate safety requirements, and avoid representation changes or unrelated formatting/lint cleanup. Keep conformance files and gate configuration unchanged in an implementation iteration.
5. Verify the architectural result by inspecting the diff and searching the affected production handlers and helpers. Classify remaining escapes rather than hiding them in command-local adapters. Run the applicable existing gates below. Do not change expectations or exclusions to get green results.
6. Update this report: task state, migrated consumers, reused/new operations, preserved invariants, remaining adapters, exact validation commands/results, and next ready task. Mark only the completed slice done. If validation fails, report the failure and keep the task open; distinguish a demonstrated baseline failure from an assumption.
7. Commit locally on `h1` under the execution contract or other explicit session authorization. Keep messages for `hmux/` commits free of private context. Provide the local review range and leave publication to the user unless they explicitly approve a push.

One iteration should end with a reviewable improvement or a concrete dependency. If a public-trait or foundational-design decision needs signoff, prepare the proposed contract and impact before asking; do not broaden an ordinary migration into that redesign. Work on independent ready tasks while a dependency remains unresolved. Do not launch parallel agents merely because this report is intended for repeated use; follow the active user's delegation instructions.

**Task queue**

`Done` means the stated slice passed its gates. `Ready` means a bounded next task is identified, not that implementation details have already been approved. `Family` entries must be split before implementation. Conformance filters are starting points; inspect selected tests and add other affected existing suites.

| ID | State | Slice and starting points | Acceptance criteria and behavior to preserve |
|---|---|---|---|
| S1 | Done — `7a0d11ea4` | Session queries in `cmd_new_window.rs` and `cmd_kill_window.rs`. | No direct session storage queries. Preserve unique-name ambiguity, duplicate links, and mutation-aware bulk killing. Formatting consumers are complete under F1; hook adapters remain Q1. |
| S2 | Done — commit containing the S2 record | Session index/link queries in `cmd_move_window.rs` and `cmd_break_pane.rs`. Inspect `room_for`, destination membership checks, and the moved-window lookup. | Use existing `SessionRef` link/current-index/shuffle/first-link operations where equivalent. Preserve fallback to the destination's current window, same-session index remapping, occupied-index errors, and link selection order. Do not change pane-transfer ownership or layout operations. Start with `CMD=move_window|link_window|break_pane` and affected hooks/notifications. |
| S3 | Done — commit containing the S3 record | Remaining session-state queries and alert requests in `cmd_kill_session.rs`, then inventory other session consumers. | Separate group selection, alert clearing, and destruction. Verify exact alert side effects before using `clear_alerts`. Preserve `-a`, `-g`, `-C`, ungrouped-session behavior, and teardown order. Do not redesign destruction in a query migration. Use kill-session conformance and affected session hooks/notifications. |
| S4 | Done — commit containing the S4 record | Session effects in rename/move/break/swap/select/split/join, new-session selection/notification, show-environment scoped reads and display-panes delay. | Existing session operations and scoped environment access preserve ordering and value states without copying the store. Final consumer audit completed; lifecycle, pane, output and queue adapters remain in their named families. |
| A1 | Skipped — user decision | Temporary syntax checker and exception baseline are not needed. | After consumer migration, enforce the boundary through Rust visibility under Z1. Do not implement A1. |
| F1 | Done — `664db2042` | Production command formatting consumers and reached helpers use handle/context/snapshot interfaces; context and expansion entry contracts are documented. | Absent clients, explicit/inherited targets, linked-window identity, expansion timing and callback/job effects were audited and gated. Internal safety remains H1; visibility enforcement and broader milestone verification remain Z1. |
| C1 | Done — commit containing the C1 family record | Attachment, switch, detach/suspend, lock and new-session lifecycle consumers; session teardown, access-denial exit and latest-client adapters. | Existing lifecycle stages preserve target/issuer identity, weak ownership, temporary retention, flags, environment semantics, failure paths and ordered output/hooks/notifications. Internal borrow safety remains H1; visibility enforcement remains Z1. |
| C2 | Done — commit containing the C2 family record | Refresh-client panning/redraw, flags, control sizes, subscriptions, pane flow, clipboard/colour reports, and select-pane client redraw. | Consumer operations preserve target identity, attached/control distinctions, parsing/errors, clamps, TTY/flag coordination and deferred delivery. Internal safety remains H1; visibility enforcement remains Z1. |
| P1 | Family — join/move-pane and break-pane slices done | Multi-pane break-pane transfer and layout/colour completion use window operations in the commit containing its record. Swap-pane ownership/geometry is also complete below. Pane creation, rotation, selection/marking and mode consumers are migrated and validated below. Next: O1 output and deferred delivery. | Preserve release-before-layout-close ordering, pane identity, inherited options/colours, new-window layout and membership. Commands retain naming, index, selection and output policy; single-pane relinking remains its distinct path. |
| O1 | Family | Output/clipboard, prompt/overlay, and deferred shell/file consumers. First slice: capture-pane output routing. | Use operation APIs without client/TTY payload extraction. Later slices cover load/save/set-buffer, prompt/menu/display-panes, and shell/source. Preserve client loss, cancellation, absent targets, queue continuation, overlay replacement, and output/error routing. |
| Q1 | Family | Command context and queue APIs used by handlers. First slice: current-state updates and hook insertion for `new-window`. | Commands use target/state/hook operations rather than payload adapters. Preserve target-resolution timing, insertion order, waits/resume, errors, and nested-hook suppression. Infrastructure can retain its own storage implementation. |
| H1 | Ongoing per migrated subsystem | Internal safety of operations, including `SessionRef` unchecked views and `ClientRef` mixed checked/unchecked access. | Define and enforce borrow/reentrancy contracts. Do not blanket-convert to safe or extend borrows across callbacks. Track unresolved boundaries. Deep storage redesign is a separate scoped task, not a prerequisite for every consumer migration. |
| Z1 | Final, after migration families | Restrict internal visibility, inventory all production command handlers/helpers/callbacks and run milestone validation. | Rust privacy prevents `cmd` from accessing subsystem internals. Resolve remaining escapes against the completion contract and confirm independent conformance coverage. Do not call the milestone done merely because the rows above were checked off. |

Default next implementation task: **Z1 visibility enforcement as a separate slice**. The residual command adapters and callback effects are separated from module restructuring and access restrictions. Keep their implementation and validation in separate local commits; milestone completion still requires the visibility slice and its full gate.

**Visibility enforcement and interface review**

Finish consumer migrations, then restrict internal modules, payload types, fields and adapters to their owning implementation. Arrange the module hierarchy and re-exports so `cmd` cannot reach them. Verify the compiler-enforced restrictions from the actual command-module context.

During migration, review handlers and reached helpers/callbacks and record remaining work under the owning family. Do not add a temporary syntax checker or maintain an exception baseline. A1 is skipped by user decision; boundary enforcement belongs to Z1. The existing dereference audit remains a separate borrow-investigation utility.

An operation returning a raw payload or mutable implementation flags may still be an escape even if its name ends in `Ref`. Conversely, a typed snapshot or established trait operation can be a valid interface. Review effects and lifetimes, not naming alone.

For high-risk changes, seek independent review of the contract and diff with specific counterexamples: a moved target, retained-but-unregistered object, callback reentry, failure midway through a transition, or optimized-build lifetime behavior. Review supplements compiler boundaries and E2E coverage; it does not replace them.

**Validation for implementation iterations**

From this directory, the project gates are `make -C .. ...`; the local Makefile does not define `test` or `unit`. Verify `tmux -V` is exactly `tmux 3.7b` before conformance work. Use short scratch socket paths and the existing harness environment handling.

- During development, run `make -C .. test-commands 'CMD=<affected module regex>' SUT=hmux`. Confirm a nonzero relevant selection and inspect the scenario names.
- Run `test-hooks HOOK=...` for affected hooks, `test-queue Q=...` for command ordering/deferred execution, and `test-notifications NOTIF=...` for control notifications. Use actual existing module names. A successful hooks command with zero selected tests is not coverage.
- Before completing code changes, run `make -C .. test SUT=hmux`, `make -C .. lint SUT=hmux`, affected dedicated conformance suites, scoped formatting checks, and diff checks, as required by contributor guidance. The full suite is slow; run it after development is ready.
- Run existing leak/lifecycle/sanitizer gates when ownership or unsafe lifetime changes justify them. Use VT coverage for rendering/parser changes. Do not add unrelated gates or repeat successful suites without a new change or unresolved concern.
- Preserve the hmux nextest profile, ignored-case handling, and existing exclusions. Record pass/fail and skipped/excluded selections. A test being ignored for hmux0 says nothing about hmux.
- Lint currently suppresses warn-level diagnostics and does not enforce main-crate formatting. Run scoped formatting checks separately; verify compiler-enforced visibility when completing Z1.
- Final milestone validation uses the existing full hmux suite plus command, hook, queue, and notification suites, with lifecycle/leak/VT checks where the completed work warrants them. No new aggregate target is required.

For independently added E2E coverage, derive expectations from tmux 3.7b or the approved component contract. Preserve meaningful event ordering and target identity when normalizing output. Favor transition scenarios over isolated happy paths: mark → move → destroy old owner → use saved target; start deferred work → detach → complete; link twice → unlink/kill → inspect other sessions. Document narrow intentional component divergences in README.md instead of creating a competing implementation.

**Lessons to carry into every iteration**

These corrective commits identify failure mechanisms; they do not establish which agent caused an issue or that a reviewed conformance gate was weakened.

| Commit | Lesson |
|---|---|
| `f939e7118` — restore direct pane references | An ID plus an old owner lookup can lose a moved target. Preserve required identity independently of membership. |
| `74efb6f12` — restore registered-pane checks for queued keys | Accessible/retained and registered are different. Key and paste contexts can intentionally use different validity rules. |
| `14bf1bd3d` — preserve menu widths | Borrow/string migrations can drop derived-state updates. Account for rendering effects as well as returned text. |
| `32e3ab0d9` — release client shutdown fix | Debug execution does not establish optimized lifetime correctness. Preserve relevant release lifecycle coverage. |
| `96a028cf3` — restore static dispatch | Architectural mistakes can pass behavior tests. Preserve dispatch/representation constraints during ordinary migrations. |

**Completion record: S1**

Commit `7a0d11ea4`, pushed to master. Migrated `new-window` and `kill-window` session queries; no conformance files, expectations, gate configuration, or public traits changed.

Added `SessionRef::unique_window_index_named` and `first_other_window`. Reused membership, current-index, link-count, shuffle, unlink, and redraw APIs. Preserved ambiguity for distinct windows and duplicate links to one window, and re-querying after each bulk kill. Remaining `new-window` formatting and hook payload adapters are F1/Q1 scope.

Validation on that implementation:

- `make -C .. test SUT=hmux`: 2,679 unit tests and 1,299 conformance tests passed; 6 existing conformance exclusions. The target also completed its additional tests/doctests.
- `make -C .. test-commands 'CMD=new_window|kill_window|unlink_window'`: 32 passed.
- `make -C .. test-hooks 'HOOK=after_new_window|window_linked|window_unlinked|session_closed|session_window_changed'`: 5 passed.
- `make -C .. test-notifications 'NOTIF=window_add_close|session_window_changed|sessions_changed'`: 18 passed.
- Lint, scoped rustfmt checks, and diff checks passed.
- Direct comparisons against tmux 3.7b matched duplicate names, duplicate links to one named window, and `new-window -S -a` with no match. These ad-hoc comparisons supplement the committed gates; they are not new committed conformance coverage.

**Completion record: S2**

Slice: session index/link queries in `cmd_move_window.rs` (including
`room_for`, shared with link-window) and `cmd_break_pane.rs`. Implementation
revision: the commit containing this record, based on `8475e65f7`.

Reused `SessionRef::link`, `current_index`, `shuffle_windows`, and
`first_link_to`; no new subsystem API or public-trait changes. Commands retain
option policy and source-index remapping. Destination membership is checked at
the point of use; a missing target link falls back to the currently linked
window, and no anchor/no available index still fails without moving a window.
The retained source session/window handles preserve identity independently of
index changes. After single-pane relinking and source unlinking, lookup still
selects the first destination link in ascending index order, including when
multiple links refer to the same window. Occupied-index errors and the order
of unzoom, relink/unlink, selection, redraw, and notification remain unchanged.

The migrated queries run synchronously without callbacks and retain no payload
borrows across shuffle/link/unlink operations. Existing unsafe operation
preconditions remain in force; the command helper documents its exclusive
access requirement. No ownership representation or layout transition changed.
The broader unchecked handle-access audit remains H1 work.

Remaining escapes found in the two handlers: break-pane's formatting context
and formatted winlink lookup (F1), current queue state update (Q1), pane
transfer/options/flags/palette and latest-client adapters (P1/C1), and raw
session adapters for unlink/status/redraw (S3's remaining-consumer inventory).
This slice removes session storage queries outside the formatting
context reserved for F1; it does not complete either handler's full boundary migration.

Existing command scenarios cover relative insertion, same-session moves,
occupied-index rejection/replacement, single-pane breaking, selection, and
printed targets. Existing unit cases additionally characterize index-only
fallback. Hook and notification gates cover link/unlink, source-session
closure, selection, layout, active panes, and ordering. Conformance scenarios,
harnesses, expectations, exclusions, and gate configuration were unchanged.

Validation on the implementation in this commit, with `tmux -V` reporting
exactly `tmux 3.7b`:

- `make -C .. test-commands 'CMD=move_window|link_window|break_pane' SUT=hmux`: 31 passed, 770 unselected.
- `make -C .. test-hooks 'HOOK=window_linked|window_unlinked|session_closed|session_window_changed|window_layout_changed|window_pane_changed' SUT=hmux`: 6 passed, 91 unselected.
- `make -C .. test-notifications 'NOTIF=window_add_close|session_window_changed|sessions_changed|layout_change|window_pane_changed|ordering' SUT=hmux`: 35 passed, 28 unselected.
- `make -C .. test SUT=hmux`: 2,679 unit tests and 1,299 conformance tests passed, with 6 existing conformance exclusions. Additional tests/doctests passed (one existing ignored test).
- `make -C .. lint SUT=hmux`: passed.
- `rustfmt --check --edition 2024 --config skip_children=true src/cmd/cmd_move_window.rs src/cmd/cmd_break_pane.rs` and `git diff --check`: passed.
- Diff and production-handler/helper escape searches reviewed. Dedicated queue, leak, sanitizer, and VT gates were not required: this slice changes synchronous session queries without changing queue behavior, ownership/lifetimes, or rendering.

Next ready slice: **S3**, kill-session queries/alert requests followed by the
remaining-session-consumer inventory. This record is included in the task
commit for publication to `origin/master` under the execution contract;
remote verification is reported after the push.

**Completion record: S3**

Slice: kill-session session-state queries and alert requests, plus the
remaining-session-consumer inventory. Implementation revision: the commit
containing this record, based on `5f5c4c31a` (equal to fetched origin/master).

`asked_group` now accepts the retained `SessionRef` and calls `group_members`
without extracting a payload or resolving it back through the session registry.
The command still chooses `-C`, then `-a`, then `-g`, then plain destruction.
Group snapshots retain join order; all-session snapshots retain name order and
exclude the target by identity. Ungrouped `-g` still falls through to plain
killing. Target-resolution errors remain in command infrastructure.

Added `SessionRef::clear_alert_flags`: clear every target-session link's alert
bits and the linked windows' alert bits, retaining other sessions' link flags.
It does not reset timers, issue status/redraw requests, or invoke callbacks.
Reused `request_redraw` after clearing. The existing `clear_alerts` operation
was deliberately not reused: its window operation also clears other sessions'
links and requests their status updates. These are distinct effects, not
interchangeable implementations of the same contract. The new unsafe operation
retains an explicit exclusive session-payload-access requirement; window access
uses the existing checked handle operation. The command releases these borrows
before redraw/destruction. No ownership or lifetime representation changed.

At this revision the `destroy` helper still extracted a raw session for
`server_destroy_session` before `SessionRef::destroy`. The C1 record below
completes that consumer migration; internal teardown safety remains H1.

Remaining-session-consumer inventory (production handlers and their in-file
helpers/callbacks; this is not Z1's exhaustive audit):

- **S4**: rename-session status/notification requests; move/break-window
  unlink/status/redraw adapters; swap-window group redraw; new-session's
  first-link storage query; show-environment's environment borrow.
- **F1**: list-sessions/windows/panes, display-message, new-window and
  break-pane formatting contexts and formatted winlink lookup.
- **C1/H1**: attach/switch/new-session environment and attachment adapters,
  lock-server session locking, and kill-session's destruction adapter. These
  consumers are completed under C1 below; internal safety remains H1.
- **Q1**: select-window, switch-client, join/break-pane and new-session
  current-state/hook targeting adapters; display-message best-client lookup.
- **P1**: select-pane marking and pane/window transition contexts,
  split/join-pane redraw/status and resize-window default-size session context.
- **O1/F1**: if-shell/run-shell, pipe-pane, display-panes and display-menu
  formatting contexts, including deferred display/menu helpers.
- `cmd_select_pane.rs`'s direct temporary `curw_idx` mutation occurs in its
  inline tests, not production. `find.rs`, `parse.rs`, `queue.rs` and the
  session lookup in `cmd/mod.rs` retain infrastructure-owned access; their
  consumer interfaces still require Q1/Z1 review. `session_owners` is already
  an owned-handle snapshot interface, not a raw registry escape.

Existing command scenarios cover target errors, `-a`, groups, shared-window
survival, mode teardown and control announcements. The full suite includes
`alert_actions::kill_session_dash_c_clears_alerts_for_the_target_session_only`;
its second session has a separate window, so it does not independently prove
shared-window link isolation. Source review verifies that the moved flag loop
preserves that distinction exactly. Existing unit group-selection tests were
adapted to handles; no conformance scenarios, harnesses, expectations,
exclusions or gate configuration changed. No public traits changed.

Validation (tmux reports exactly `tmux 3.7b`):

- `make -C .. test-commands CMD=kill_session SUT=hmux`: 12 passed, 789 unselected.
- `make -C .. test-hooks 'HOOK=session_closed|client_session_changed|client_detached' SUT=hmux`: 4 passed, 93 unselected.
- `make -C .. test-notifications 'NOTIF=sessions_changed|session_changed|window_add_close|ordering' SUT=hmux`: 19 passed, 44 unselected.
- `make -C .. test SUT=hmux`: 2,679 unit tests and 1,299 conformance
  tests passed; 6 existing conformance exclusions. Additional tests/doctests
  passed (one existing ignored test).
- `make -C .. lint SUT=hmux`: passed.
- `rustfmt --check --edition 2024 --config skip_children=true src/cmd/cmd_kill_session.rs src/session_handle.rs src/tests/test_cmd_kill_session.rs` and `git diff --check`: passed.
- Production-handler/helper escape searches and diff review completed.
  Dedicated queue/leak/sanitizer/VT runs were not required: queue behavior,
  destruction, ownership/lifetimes and rendering were not changed.

Next ready slice: **F1**, one read-only listing command plus the context API.
S4 tracks the remaining narrow session operations separately. This record is
included in the task commit for publication to `origin/master`; final remote
verification is reported after pushing.

**Completion record: F1 — complete**

Completed and published through `664db2042`. Commands and their argument,
listing, spawn and menu helpers now pass handles, target contexts or owned
snapshots to formatting operations. The final audit covered 29 command files
and their reached helpers/callbacks; no remaining F1 formatting payload escape
was found. Detailed slice records remain in git history at that revision.

Preserved independent job/drawn clients, explicit and inherited targets,
registration and link/pane identity rules, expansion/drop order, menu widths
and placement formulas. Entry contracts document weak observations, synchronous
mode/plugin/custom callbacks, and format jobs that update caches/status later.
Public traits, foundational choices and conformance gates were unchanged.

F1 completion leaves these boundaries open:

| Task | Remaining work |
|---|---|
| H1 | Internal checked/unchecked access and callback reentrancy, including unsafe accesses inside the safe `format_each` entry. |
| O1/C2/H1 | Display/menu/file/shell/cwd delivery, client/TTY control and internal lifecycle safety. C1 completes access-denial exit and command lifecycle adapters below. |
| P1 | Pane transfer/options/layout and spawn ownership. Narrow session effects and queries were completed under S4; latest-client updates are completed under C1 below. |
| Q1 | Current-state, hook and deferred queue adapters. |
| Z1 | Visibility enforcement and broader audit, including list-keys key-table storage and global-prefix access. |

Final validation used tmux 3.7b and the existing hmux profile with
`--run-ignored all`, on `547eff945` plus the final rustdoc changes. The later
integration of `0fc309180` changed no hmux code or build/test inputs.

| Gate | Result |
|---|---|
| `make -C .. test SUT=hmux` | 2,679 unit and 1,299 conformance tests passed; 6 existing exclusions. |
| `make -C .. test-commands SUT=hmux` | 799 passed; 2 existing exclusions. |
| `make -C .. test-hooks SUT=hmux` | 97 passed. |
| `make -C .. test-queue SUT=hmux` | 45 passed. |
| `make -C .. test-notifications SUT=hmux` | 63 passed. |
| `make -C .. lint SUT=hmux` | Passed. |

Additional tests/doctests and scoped rustfmt/diff checks passed, with one
existing ignored doctest. Exclusion names and reasons remain in
`../.config/nextest.toml`. Slice-specific coverage is recorded in git history.

The S4 follow-up is complete below. The broader command-interface milestone
remains open.

**Completion record: S4 — complete**

Completed across `b2f2775e0` through `69afe763e`, with the environment read
corrected in the commit containing this record. Earlier slice records remain in
git history at `5938c1054`. S4 consumers in ten command files were migrated:

- Rename-session, move/break/swap-window, select-window, split-window and
  join-pane reuse session unlink/status/redraw/notification operations.
- New-session uses the first linked index after group synchronization and queues
  its creation notification at the original point.
- Show-environment uses a scoped environment read and owns only its output lines;
  display-panes reads its delay through the existing session options handle.

Preserved selection/error policy, group teardown, status versus full redraw,
notification/hook timing and overlay delay behavior. Rustdoc records session and
client borrow exclusions and deferred control/hook delivery. Environment reads
preserve bytes, ordering, empty/valueless states and flags directly from the store.
The environment borrow ends before output or hook dispatch; a named lookup does
not copy unrelated entries. The unnecessary whole-store snapshot and unused
session environment-copy helper were removed. Existing overlay-copy semantics
and global access remain unchanged. The hidden/valueless regression in `8a5913fcf`
was independently checked against tmux 3.7b and baseline hmux before implementation.
Public traits, foundational choices, harnesses, profiles and exclusions were unchanged.

The final audit covered all 62 `cmd_*.rs` files and relevant reached helpers and
callbacks, with no remaining narrow S4 escape. Scoped group-name reads and
EnvironmentStore mutations remain valid interfaces. Remaining boundaries:

| Task | Remaining work |
|---|---|
| Q1 | Current-state and hook adapters in select/new/split/join/break/switch handlers. |
| C2/H1 | Client/TTY control and internal lifecycle/handle borrow safety. C1 completes the command attachment/environment/locking/destruction adapters below. |
| P1 | Marked-pane and resize contexts, pane/window transfer, spawn and layout ownership. |
| O1 | Shell/job/cwd, popup and causes-output contexts. |
| Z1 | Visibility enforcement and the broader milestone audit. |

Family validation ran on `69afe763e`, with tmux 3.7b, `SUT=hmux` and
the existing profile's `--run-ignored all` behavior:

| Gate | Result |
|---|---|
| `make -C .. test SUT=hmux` | 2,679 unit and 1,299 conformance tests passed; 6 existing exclusions. |
| `make -C .. test-commands SUT=hmux` | 800 passed; 2 existing exclusions. |
| `make -C .. test-hooks SUT=hmux` | 97 passed. |
| `make -C .. test-queue SUT=hmux` | 45 passed. |
| `make -C .. test-notifications SUT=hmux` | 63 passed. |
| `make -C .. lint SUT=hmux` | Passed. |

Additional tests/doctests and scoped rustfmt/diff checks passed, with one existing
ignored doctest. Focused environment/display-panes command and hook gates also
passed; the final suite confirms default-delay expiry and queue resumption.
Exclusion names and reasons remain in `../.config/nextest.toml`.

The scoped-read correction was validated on the code in this commit with tmux
3.7b and `SUT=hmux`: `make -C .. test` passed 2,679 unit and 1,299 conformance
tests (6 existing exclusions); `test-commands CMD='show_environment|set_environment'`
passed 14 and `test-hooks HOOK='after_show_environment|after_set_environment'`
passed 2. Lint, scoped rustfmt and diff checks passed. Additional tests/doctests
passed with one existing ignored doctest. The correction changes no test inputs.

The C1 attachment slice is complete below. A1 is skipped by user decision.
The broader milestone remains open.

**Completion record: C1 — attachment slice**

Implementation: the commit containing this record, based on `7e1103ba2`.
Migrated `cmd_attach_session` and its detach-other helper, including the path
shared by `new-session -A`. Client operations reuse the existing nested check,
flag handling, previous-session tracking, terminal opening, pending detach,
session transition and key-table selection. A separate completion operation
sends readiness, queues attachment notification and marks the client attached.
`SessionRef::update_environment_from` uses the existing environment engine
directly, without copying the store into a snapshot.

Preserved target identity, client-list snapshots and retention of the previous
session until command return. Command policy and sequencing remain explicit:
cwd/flag changes precede read-only refusal; previous-session tracking precedes
terminal opening; detach/environment stages precede session assignment; repeat
state controls key-table reset. Readiness, notifications, flags and failure
messages retain their order. Rustdoc records borrow and callback requirements;
the lifecycle operations retain their unsafe contracts.

Remaining adapters: current-target link/pane views are Q1; configuration-causes
output is O1. Switch, detach, lock and other lifecycle consumers remain C1;
internal borrow/reentrancy work remains H1. Visibility enforcement remains Z1.
Public traits, foundational choices, conformance tests, profiles and exclusions
were unchanged.

Validation used tmux 3.7b and `SUT=hmux` on this implementation:

| Gate | Result |
|---|---|
| `make -C .. test SUT=hmux` | 2,679 unit and 1,299 conformance tests passed; 6 existing conformance exclusions. |
| `make -C .. test-commands SUT=hmux 'CMD=attach_session\|new_session'` | 33 passed; 769 outside the selection. |
| `make -C .. test-hooks SUT=hmux 'HOOK=client_attached\|client_session_changed\|client_detached\|after_new_session'` | 5 passed; 92 outside the selection. |
| `make -C .. test-notifications SUT=hmux 'NOTIF=client_lifecycle\|session_changed'` | 6 passed; 57 outside the selection. |
| `make -C .. test-queue SUT=hmux 'Q=insertion_order\|nested_suppression\|global_queue'` | 14 passed; 31 outside the selection. |
| `make -C .. lint SUT=hmux` | Passed. |

The six focused attach unit tests also passed. Additional tests/doctests passed
with one existing ignored doctest. Scoped rustfmt comparison introduced no new
formatting differences; existing unrelated formatting was preserved. Diff checks
passed. The report update was pushed as `7e1103ba2`; this C1 iteration is also
explicitly authorized for publication to `origin/master`, with the resulting
commit and remote verification reported after publication.

The remaining C1 consumers are completed in the family record below.

**Completion record: C1 — family complete**

The commit containing this record completes C1 from `2bc8fecdd`, which contains
the attachment slice. The user authorized parallel work on the remaining family
and requested local commits on `h1` for review, with no push.

Migrated switch-client, detach/suspend-client and lock-client/session/server;
new-session now uses lifecycle operations, terminal-size snapshots and the
existing environment engine. Session teardown prepares clients before destroying
the session; access denial records exit state before changing the ACL. Latest
client updates in new/select-window and break-pane use handles and retain the
existing weak relationship. All operations reuse the existing implementations;
public traits, foundational choices and ownership representations are unchanged.

Preserved issuing versus target clients, read-only toggling, `-T` early return,
registered last-session lookup, selection order, detach return statuses and lock
guards. New-session keeps READY before session assignment, its previous-session
temporary within the original guarded branch, and ATTACHED after `-P` output.
It does not raise `client-attached`. Attach-session retains its distinct READY,
notification and ATTACHED sequence. Environment updates use the original global
options before creation, with `-E`, `-e`, matching and clearing semantics intact.

Three subagents implemented and checked switch, detach/suspend and lock slices.
Independent reviews also checked the final new-session ordering and lifetimes,
teardown, exit state and latest-client identity. Their command/helper audit found
no remaining C1 consumer. Remaining adapters belong to C2 (TTY/control/redraw),
P1 (pane/layout), O1 (output/jobs/files), and Q1 (target state/hooks). Existing
internal borrow/reentrancy obligations remain H1; visibility enforcement is Z1.

Validation used tmux 3.7b and the unchanged hmux profile on this implementation:

| Gate | Result |
|---|---|
| `make -C .. test SUT=hmux` | 2,679 unit and 1,299 conformance tests passed; 6 existing conformance exclusions. |
| `make -C .. test-commands SUT=hmux 'CMD=attach_session\|switch_client\|detach_client\|suspend_client\|lock_client\|lock_session\|lock_server\|new_session\|kill_session\|server_access\|new_window\|select_window\|break_pane'` | 126 passed; 676 outside the selection. |
| `make -C .. test-hooks SUT=hmux` | 97 passed. |
| `make -C .. test-queue SUT=hmux` | 45 passed. |
| `make -C .. test-notifications SUT=hmux` | 63 passed. |
| `make -C .. lint SUT=hmux` | Passed. |

Additional tests/doctests passed with one existing ignored doctest. Scoped
rustfmt comparison found no new formatting deviations in the 13 changed Rust
files; diff checks passed. Conformance tests, harnesses, profiles and exclusions
were unchanged. This family completion remains local on `h1`; nothing was pushed.
Next: **C2 — refresh-client panning and redraw**. A1 remains skipped.

**Completion record: C2 — panning and redraw**

Implementation: the commit containing this record, based on `52cc288d8`.
A subagent migrated panning/reset, forced status/full redraw and flag application;
independent review checked anchor changes, unsigned wrapping, clamps and option
precedence. `ClientRef::pan_window` and `reset_window_pan` own weak anchoring,
cached offsets, TTY coordination and redraw ordering. `refresh_display` owns
forced status regeneration; `apply_flags` is reused for `-F` then `-f`.
Commands retain adjustment validation, direction precedence, current-window
resolution and errors. No callback or payload borrow escapes the new operations;
server-thread and borrow exclusions remain explicit unsafe contracts under H1.

Validation with tmux 3.7b and `SUT=hmux` on this implementation:

- `make -C .. test SUT=hmux`: 2,679 unit and 1,299 conformance tests passed;
  6 existing conformance exclusions. Additional tests/doctests passed, with one
  existing ignored doctest.
- `make -C .. test-commands CMD=refresh_client SUT=hmux`: 11 passed, 791 unselected.
- `make -C .. test-hooks HOOK=after_refresh_client SUT=hmux`: 1 passed, 96 unselected.
- `make -C .. lint SUT=hmux`, `cargo check -q` and `git diff --check`: passed.
- Scoped rustfmt comparison against the parent found no new deviations in the
  three changed Rust files. Handler/helper audit found no panning/redraw escape.

Conformance inputs, public traits and ownership representations are unchanged.
No additional queue/leak/VT gate was needed: queue semantics, resource lifetimes
and rendering implementations are unchanged. Size, subscription, pane-flow and
clipboard/colour-report adapters remain C2; internal safety remains H1 and final
visibility remains Z1. Next: **C2 — control sizes**. This commit stays local on
`h1`; no push is authorized.

**Completion record: C2 — control sizes**

Implementation: the commit containing this record, based on `ffb40e25c`.
A subagent migrated global and per-window control size requests to inherent
`ClientRef` operations; independent review found no behavior or lifetime defect.
Commands keep sscanf acceptance, size validation and errors. The operations own
TTY dimensions/pixel reset, client-local entries, change flags and immediate
recalculation. Setting an unregistered window ID still creates an entry; clearing
only touches an existing entry, sets no change flag and only then recalculates.
Client payload borrows end before recalculation and its layout effects. The clear
operation retains debug-log ordering with its own function label. `is_control`
replaces raw flag inspection for all three control branches without reordering
`-A`, `-B` or `-C`. Unsafe initialization/borrow requirements are documented.

Validation with tmux 3.7b on this implementation:

- `make -C .. test SUT=hmux`: 2,679 unit and 1,299 conformance tests passed,
  including control-mode global/per-window sizes; 6 existing exclusions.
- `make -C .. test-commands CMD=refresh_client SUT=hmux`: 11 passed, 791 unselected.
- `make -C .. test-hooks 'HOOK=after_refresh_client|window_layout_changed' SUT=hmux`:
  2 passed, 95 unselected.
- `make -C .. test-notifications 'NOTIF=layout_change|ordering' SUT=hmux`:
  13 passed, 50 unselected.
- `make -C .. lint SUT=hmux`, `cargo check -q`, diff checks and scoped rustfmt
  baseline comparison passed. Additional tests/doctests passed with one existing
  ignored doctest.

No conformance inputs, public traits or ownership representations changed.
Subscription, pane-flow and clipboard/colour adapters remain C2; internal safety
and final visibility remain H1/Z1. No new queue, resource-lifetime or parser
behavior requires separate queue/leak/VT runs. Next: **C2 — subscriptions**.
This commit remains local on `h1`; nothing was pushed.

**Completion record: C2 — subscriptions**

Implementation: the commit containing this record, based on `73fab3116`.
A subagent prepared and the primary agent reviewed the subscription migration.
`ClientRef::set_control_subscription` and `remove_control_subscription` reuse
existing control storage and timer operations. Commands retain field splitting,
permissive sscanf selectors, incomplete-input no-ops and removal by name.
Replacement resets prior observations, copies name/format and arms the existing
one-second timer as needed; removing the last subscription disarms it. Timer
ownership remains weak and no format expansion or callback runs inline. Explicit
unsafe contracts require initialized control state and exclude payload borrows.

Validation with tmux 3.7b on this implementation:

- `make -C .. test SUT=hmux`: 2,679 unit and 1,299 conformance tests passed,
  including format subscription behavior; 6 existing exclusions.
- `make -C .. test-commands CMD=refresh_client SUT=hmux`: 11 passed, 791 unselected.
- `make -C .. test-hooks HOOK=after_refresh_client SUT=hmux`: 1 passed, 96 unselected.
- `make -C .. lint SUT=hmux`, scoped `rustfmt --check --edition 2024 --config
  skip_children=true` for both changed Rust files, and `git diff --check`: passed.
- Additional tests/doctests passed with one existing ignored doctest.

No conformance inputs, public traits, timer ownership or callback implementation
changed. Existing full-suite and command subscription scenarios cover delivery;
no separate queue/leak/VT run was needed for these forwarding operations.

The wider C2 inventory also found `cmd_select_pane_redraw` coordinating client
TTY/redraw flags. That bounded consumer is now queued after the remaining refresh
pane-flow and clipboard/colour slices. Copy-mode scrollbar context remains pane
mode work under P1; buffer/overlay/terminal-diagnostic delivery remains O1.
Internal safety and visibility remain H1/Z1. Next: **C2 — pane flow control**.
This commit remains local on `h1`; nothing was pushed.

**Completion record: C2 — pane flow control**

Implementation: the commit containing this record, based on `095273e78`.
A subagent migrated on/off/pause/continue dispatch to four inherent `ClientRef`
operations; the primary agent and an independent subagent reviewed the change.
Commands retain parsing, global pane lookup and action policy. Operations accept
the existing weak pane identity, temporarily upgrade that exact allocation for
synchronous access and pass its payload to the existing control implementation.
A dead observation does nothing; command lookup establishes a live observation
with no intervening callback. No registration, stored ownership or asynchronous
retention changes. Explicit unsafe contracts exclude conflicting payload access.

Preserved existing-entry-only on/continue transitions, offset resets, creation
for off/pause, queued-output discarding and one-time buffered pause/continue
lines. These calls dispatch no inline callbacks. Subscription methods remain
unchanged, and the command no longer extracts flow-control client/pane payloads.

Validation with tmux 3.7b on this implementation:

- `make -C .. test SUT=hmux`: 2,679 unit and 1,299 conformance tests passed,
  including explicit flow control and pane on/off behavior; 6 existing exclusions.
- `make -C .. test-commands CMD=refresh_client SUT=hmux`: 11 passed, 791 unselected.
- `make -C .. test-hooks HOOK=after_refresh_client SUT=hmux`: 1 passed, 96 unselected.
- `make -C .. test-notifications NOTIF=ordering SUT=hmux`: 4 passed, 59 unselected.
- `make -C .. lint SUT=hmux`, scoped `rustfmt --check --edition 2024 --config
  skip_children=true` on both changed Rust files, and `git diff --check`: passed.
- Additional tests/doctests passed with one existing ignored doctest.

Conformance inputs and public traits are unchanged. Temporary synchronous pane
retention adds no callback or lifetime transition requiring a separate leak gate;
queue and rendering implementations are unchanged. Remaining C2: clipboard/colour
reports and select-pane client redraw. H1/Z1 remain open. Next: **C2 — clipboard
and colour reports**. This commit remains local on `h1`; nothing was pushed.

**Completion record: C2 — clipboard and colour reports**

Implementation: the commit containing this record, based on `5fb77c961`.
A subagent prepared the migration and independent reviews checked weak pane
identity, parser failure paths and asynchronous delivery. `ClientRef::query_clipboard`
reuses the existing started-TTY/outstanding-query gating, terminal capability,
timer and later reply/import handling. `report_pane_colours` owns the TTY parser
and pane colour pair update. Commands retain global pane lookup, permissive sscanf,
malformed/missing no-ops and the original early-return/flag/report/control order.

The colour operation takes the existing weak pane type. Its unsafe contract
requires immediate live resolution with no intervening callback/removal and
excludes conflicting TTY/pane access. Invalid/incomplete reports preserve parser
outputs; existing -1-to-unknown normalization remains unchanged. Valid reports
update only the recognized colour and corresponding terminal wait flag. No new
redraw or inline callback is introduced. An initial compile failure from an
unavailable `PaneRef` name was corrected to `RustWindowPaneWeak` before validation.

Validation with tmux 3.7b on the corrected implementation:

- `make -C .. test SUT=hmux`: 2,679 unit and 1,299 conformance tests passed,
  including clipboard query/rearming and foreground/background reports;
  6 existing exclusions.
- `make -C .. test-commands CMD=refresh_client SUT=hmux`: 11 passed, 791 unselected.
- `make -C .. test-hooks 'HOOK=after_refresh_client|pane_set_clipboard' SUT=hmux`:
  2 passed, 95 unselected.
- `make -C .. lint SUT=hmux`, `git diff --check` and scoped rustfmt comparison
  against the parent passed; no new deviations in the three changed Rust files.
- Additional tests/doctests passed with one existing ignored doctest.

All four refresh-client helpers and the handler were audited against `52cc288d8`;
no client/session/window/pane/TTY payload extraction or storage mutation remains.
Remaining unsafe parsing touches command input and local outputs. No conformance
inputs, public traits, ownership representations, parser or timer implementations
changed. No separate queue/leak/VT run was needed. The select-pane redraw adapter
remains C2; P1/O1/H1/Z1 retain the wider work. Next: **C2 — select-pane client
redraw**. This commit remains local on `h1`; nothing was pushed.

**Completion record: C2 — select-pane redraw and family complete**

Implementation: the commit containing this record, based on `2292f1373`.
This final slice completes C2 after the five refresh-client slices above.
Three subagents implemented or independently reviewed the bounded changes;
all six implementation commits stay local on `h1` for user review.

`ClientRef::redraw_pane_selection` now owns control/unattached filtering,
current-window identity, linked-session membership and TTY-dependent redraw
coordination. Commands retain client iteration and both selection-policy trigger
sites. A larger current viewport still requests full redraw; otherwise current
windows redraw borders and linked windows redraw status. A linked session with
no current window still receives status redraw. No pane selection, style/z-order,
registration, offset recalculation or callback behavior changes. Unsafe contracts
retain client/TTY/session/window borrow exclusions and initialized TTY ownership.

Validation with tmux 3.7b on this final implementation:

- `make -C .. test SUT=hmux`: 2,679 unit and 1,299 conformance tests passed;
  6 existing exclusions. The existing missing-current-window/select-pane redraw
  unit regression passed, along with full attached/layout behavior coverage.
- `make -C .. test-commands 'CMD=select_pane|last_pane' SUT=hmux`:
  26 passed, 776 unselected.
- `make -C .. test-hooks 'HOOK=after_select_pane|window_pane_changed' SUT=hmux`:
  2 passed, 95 unselected.
- `make -C .. test-notifications 'NOTIF=window_pane_changed|ordering' SUT=hmux`:
  8 passed, 55 unselected.
- `make -C .. lint SUT=hmux`, `git diff --check` and scoped rustfmt baseline
  comparison passed; no new deviations in either changed Rust file.
- Additional tests/doctests passed with one existing ignored doctest.

Each earlier C2 slice has its own passing full suite, affected dedicated gates,
lint and formatting evidence above. No conformance scenarios, harnesses,
expectations, exclusions, profiles, public traits or foundational choices changed.
These synchronous consumer migrations preserve stored ownership and existing
parser/rendering/timer implementations; no additional leak/VT gate was required.

The final actual-tree audit against `52cc288d8` covered refresh-client's handler
and all four helpers, select-pane's redraw helper and their reached operations.
No C2 payload extraction or subsystem-storage coordination remains in those
consumers. Counterexample review covered changed pan anchors and wrapping,
missing size entries, timer replacement/cancellation, weak pane identity, malformed
colour reports and linked sessions without a current window. This closes C2's
consumer migration, not the entire architectural milestone.

Remaining boundaries:

| Task | Remaining work |
|---|---|
| P1 | Pane transfer/options/layout ownership, select-pane active/marked-pane state and pane-mode contexts such as copy-mode scrollbar positioning. |
| O1 | Capture output routing, buffer clipboard delivery, terminal diagnostics, prompts/overlays and deferred shell/file delivery. |
| Q1 | Current-state, target/hook and deferred queue adapters. |
| H1 | Internal unchecked handle access, TTY owner/borrow obligations and control timer format reentrancy. |
| Z1 | Compiler-enforced visibility and the exhaustive production command/helper/callback audit. |

Next ready slice: **P1 — join/move-pane ownership and layout transition**.
C2 is complete. All six task commits are local on `h1`; nothing was pushed.

**Completion record: P1 — join/move-pane transfer**

Implementation: the commit containing this record, based on `de535c5a3`.
Verified a clean tree on `h1`, current source and local history before selecting
this slice. `WindowRef::join_pane` now owns destination split preparation and the
complete transfer: tracked same-window layout collapse, client pane cleanup,
source active/marked/history adjustment, ownership and z-order relinking,
option inheritance, appearance flags, layout assignment and colour loading.
It reuses the existing tiled geometry parser and transfer/layout operations;
no new argument policy or competing implementation was introduced.

Commands retain unzoom order, identical-target rejection, error formatting,
size recalculation, redraw, selection/status policy and last-source-window
removal. The existing weak pane observations identify the same allocations;
window pane lists transfer the strong owner without unregistering the pane.
Before-placement still inserts after the destination in both lists. Geometry
errors precede source removal; after split success, transfer has no recoverable
failure under the documented live-membership contract. Existing failed-split
zoom effects remain intact. Notifications and client cleanup retain their order.

The operation remains unsafe: server-thread execution, conflicting payload access
and target stability during geometry-format callbacks are documented. No payload
borrow crosses format expansion, and transfer does not dispatch queue hooks.
Those inherited internal reentrancy obligations remain H1 work. The production
handler audit finds only the current-state/session payload adapter under Q1;
no pane/window payload extraction or transfer storage coordination remains.
The existing unit fixture now imports its own `WindowPane` trait explicitly.
No public trait, representation, foundational choice or conformance input changed.

Validation with exactly tmux 3.7b on this implementation (final subsequent Rust
edits only clarified rustdoc):

- `cargo check -q`: passed.
- `make -C .. test-commands 'CMD=join_pane|move_pane' SUT=hmux`:
  21 passed, 781 unselected. Reviewed same/cross-window moves, marked sources,
  before/full-size placement, sizes, detached selection and target/geometry errors.
- `make -C .. test-hooks 'HOOK=after_join_pane|after_move_pane|window_layout_changed|window_pane_changed|window_unlinked|session_closed' SUT=hmux`:
  4 passed, 93 unselected; selected modules were the two window-change hooks,
  window-unlinked and session-closed. No dedicated after-join/move modules exist.
- `make -C .. test-notifications 'NOTIF=layout_change|window_pane_changed|window_add_close|sessions_changed|ordering' SUT=hmux`:
  29 passed, 34 unselected.
- `make -C .. test-queue 'Q=insertion_order|chain_error|nested_suppression' SUT=hmux`:
  10 passed, 35 unselected.
- `make -C .. test SUT=hmux`: 2,679 unit and 1,299 conformance tests passed;
  6 existing exclusions. Additional tests/doctests passed, one existing ignored
  doctest. Includes repeated joins after old-owner destruction and option-parent
  updates, plus unit regressions for same-window cell tracking and list order.
- `make -C .. leak SUT=hmux`: 2,817 passed, 10 skipped under the existing leak
  profile. Stored ownership and layout/rendering implementations are unchanged;
  no additional VT or sanitizer gate was needed.
- `make -C .. lint SUT=hmux`: passed. Scoped rustfmt comparison against the
  baseline found no new deviations in all three changed Rust files;
  `git diff --check` passed.

Implementation/conformance separation is preserved: no scenarios, harnesses,
expectations, exclusions or gate configuration changed. P1 remains a family;
break/swap/split, select-pane and pane-mode contexts remain open. Q1, H1 and Z1
retain their queue, safety and visibility/audit boundaries; O1 is unchanged.
Next ready slice: **P1 — multi-pane break-pane transfer into a new window**.
This task commit remains local on `h1`; nothing was pushed.

**Completion record: P1 — multi-pane break-pane transfer**

Implementation: the commit containing this record, based on `6382f9634`.
Verified a clean tree on `h1`, applicable guidance, the report and local history.
`WindowRef::break_pane_into_window` now owns client pane cleanup, source
active/marked/history and pane/z-order release, source layout closure, new-window
creation with the source dimensions, strong pane ownership transfer, option
inheritance and style/theme flags. `finish_broken_pane_layout` completes the
existing layout initialization, changed flag and inherited colour loading.

The two synchronous stages preserve naming between transfer and layout creation:
the command uses the transferred active pane for default naming, or applies `-n`
and disables automatic rename, after recording the latest client. It then finishes
the layout before attachment. Rustdoc specifies the unlinked intermediate window,
unchanged target identity, forbidden intervening callbacks/layout use, live source
membership and server-thread/payload borrow exclusions. The operation has no
recoverable failure after transfer begins. Existing internal unchecked access and
layout/TTY obligations remain H1 work; the operations remain unsafe.

Preserved release-before-layout-close order, allocation identity and registration,
source active/marked/history changes, destination sole-pane/z-order membership,
option-parent lifetime, colours and pixel dimensions. No callback or payload
borrow escapes the operations. The command retains name/index validation,
shuffle/remapping, unzoom and occupied-index error order, the distinct single-pane
relink path, attachment, selection/current-state, redraw/status and formatted
output. Notification construction remains at its original points; queue hooks
are deferred. No new layout or ownership representation was introduced.

The production-handler and reached-operation audit finds no remaining break-pane
pane/window payload extraction or transfer storage coordination. The remaining
current-state/session payload adapter belongs to Q1. Existing unit fixtures now
import their own `WindowPane` and pane-count helper. No test scenarios or assertions
were changed. Counterexample review covered marked sources, source-window
survival/destruction, occupied destinations, same-session index remapping,
single-pane relinking and naming before layout completion.

Validation with exactly tmux 3.7b on this implementation:

- `cargo check -q`: passed.
- `make -C .. test-commands CMD=break_pane SUT=hmux`: 10 passed, 792 unselected.
- `make -C .. test-hooks 'HOOK=window_linked|window_unlinked|session_closed|session_window_changed|window_layout_changed|window_pane_changed|window_renamed' SUT=hmux`:
  8 passed, 89 unselected.
- `make -C .. test-notifications 'NOTIF=window_add_close|session_window_changed|sessions_changed|layout_change|window_pane_changed|window_renamed|ordering' SUT=hmux`:
  39 passed, 24 unselected.
- `make -C .. test-queue 'Q=insertion_order|chain_error|nested_suppression' SUT=hmux`:
  10 passed, 35 unselected.
- `make -C .. test SUT=hmux`: 2,679 unit and 1,299 conformance tests passed;
  6 existing exclusions. Additional tests/doctests passed, one existing ignored
  doctest. Includes break-pane option inheritance after old-window destruction,
  grouped/shared-window transfers and marked-pane transitions.
- `make -C .. leak SUT=hmux`: 2,817 passed, 10 skipped under the existing profile.
- `make -C .. lint SUT=hmux`: passed. Scoped rustfmt comparison with the baseline
  found no new deviations in all three changed Rust files after fixing fixture
  import order; `git diff --check` passed. The existing layout/rendering engine
  and stored ownership representations are unchanged; no additional VT or
  sanitizer gate was needed.

Implementation/conformance separation is preserved: no conformance scenarios,
harnesses, expectations, exclusions or gate configuration changed. No public
traits or foundational choices changed. P1 remains open for swap/split,
select-pane and pane-mode contexts; O1, Q1, H1 and Z1 remain open.
Next ready slice: **P1 — swap-pane ownership and geometry transition**.
This task commit remains local on `h1`; nothing was pushed.

**Completion record: P1 — swap-pane ownership and geometry**

Implementation: the commit containing this record, based on `3c49f045e`.
`WindowRef::exchange_pane_geometry` owns layout-path validation, floating-pane
rejection, client cleanup, pane/z-order exchange, layout binding, option parents,
appearance flags and source-then-destination geometry/resizing. It returns the
prior active states. Commands retain directional source choice, zoom, detached
selection policy, layout repair/redraw and notifications. `finish_pane_exchange`
removes departed history entries and reloads colours after selection, preserving
same-window versus cross-window behavior and marked-pane identity semantics.

The operation remains unsafe: live membership and layout paths are required;
pane-mode resize callbacks must not move/remove targets. No window borrow spans
those callbacks; inherited internal pane borrow obligations remain H1. Existing
weak identities and strong pane-list ownership are unchanged. No production
swap-pane payload extraction remains. No public traits, foundational components,
conformance inputs or gate configuration changed.

Validation with tmux 3.7b on this implementation (final edit only formatted one
operation call):

- `cargo check -q`: passed.
- `make -C .. test-commands 'CMD=swap_pane|last_pane' SUT=hmux`: 18 passed,
  784 unselected.
- `make -C .. test-hooks 'HOOK=window_layout_changed|window_pane_changed' SUT=hmux`:
  2 passed, 95 unselected.
- `make -C .. test-notifications 'NOTIF=layout_change|window_pane_changed|ordering' SUT=hmux`:
  17 passed, 46 unselected.
- `make -C .. test-queue 'Q=insertion_order|chain_error|nested_suppression' SUT=hmux`:
  10 passed, 35 unselected.
- `make -C .. test SUT=hmux`: 2,679 unit and 1,299 conformance passed;
  6 existing exclusions. Additional tests/doctests passed, one ignored doctest.
  Includes repeated option-parent swaps, active/history regressions and marked
  pane return transitions.
- `make -C .. leak SUT=hmux`: 2,817 passed, 10 skipped.
- `make -C .. lint SUT=hmux`: passed. Scoped rustfmt baseline comparison and
  `git diff --check` passed after formatting the new calls. Existing geometry,
  rendering and mode implementations are unchanged; no extra VT gate was needed.

Next: remaining P1 pane creation, rotation, selection/marking and mode consumers.
O1/Q1/H1/Z1 remain open. This commit stays local on `h1`; nothing was pushed.
The user has authorized continued iterations through milestone completion for
one final review, with validated changes retained as separate local commits.

**Completion record: P1 — remaining pane command consumers**

Implementation: the commit containing this record, based on `312e8e23f`.
Migrated split/new-pane styles and options, failed input cleanup, kill-pane,
respawn redraw, rotate-window geometry, resize-window sizing context, find-window
and choose modes, set-option pane option lookup, select/last-pane flags, style,
marking, title, neighbour search and client-local selection, plus copy scrollbar
context. Reused existing pane option, mode, input, flag, title, notification and
neighbour APIs. New operations own paired styles/appearance invalidation,
client/layout/owner discard, rotation geometry, marked-target refresh and client
selection/scrollbar context. Window sizing now accepts a session handle.

Commands preserve argument precedence, zoom, source choice, option policy,
selection, output and notification order. Rotation snapshots geometry before
list mutation and returns the rotated identities for selection; z-order remains
unchanged. Kill/split failure paths retain client cleanup before layout closure
and destruction. Marking retains previous-mark validity and refreshes both old
and resulting marks before floating-pane selection. Titles retain missing-pane
no-ops and notify only on change. Client-local selection stays distinct from
window selection. Mode/resize/teardown operations retain explicit unsafe borrow,
identity and callback contracts; H1 tracks the inherited internals.

No pane/client/TTY payload extraction remains in these production consumers
except the Q1 current-target adapters in split/select/rotate. Inline test fixtures
are not production escapes. Swap-window link/mark coordination is tracked for
Q1/Z1. No public traits, foundational implementations, conformance scenarios,
harnesses, expectations, exclusions or gate configuration changed.

Validation with tmux 3.7b on this implementation:

- `cargo check -q`: passed after adding the qualified queue-item lookup.
- `make -C .. test-commands 'CMD=split_window|new_pane|rotate_window|kill_pane|respawn_pane|resize_window|choose_tree|choose_client|choose_buffer|find_window|set_option|select_pane|last_pane|copy_mode|clock_mode' SUT=hmux`:
  157 passed, 645 unselected. Earlier narrower command run passed 112.
- `make -C .. test-hooks SUT=hmux`: 97 passed.
- `make -C .. test-notifications SUT=hmux`: 63 passed.
- `make -C .. test-queue SUT=hmux`: 45 passed.
- `make -C .. test SUT=hmux`: 2,679 unit and 1,299 conformance passed;
  6 existing exclusions. Additional tests/doctests passed, one ignored doctest.
- `make -C .. leak SUT=hmux`: 2,817 passed, 10 skipped.
- `make -C .. lint SUT=hmux`: passed. Scoped rustfmt baseline comparison on all
  15 changed Rust files and `git diff --check` passed. A formatting pass initially
  removed three existing server re-exports; these were restored before rerunning
  all final gates. No failed validation is outstanding.

Existing layout, rendering, mode, registration and ownership representations are
unchanged; no additional VT or sanitizer gate was required. P1 consumer migration
is complete subject to Z1's exhaustive audit. Next: O1 output/deferred delivery,
then Q1 and final H1/Z1. This validated commit remains local on `h1`.

**Completion record: O1 — output and deferred delivery**

Implementation: the commit containing this record, based on `1b4be5939`.
Capture-pane now delegates control/file output to `ClientRef::print_capture`,
preserving precision/NUL handling, trailing-newline policy and unavailable-client
errors. Clipboard delivery uses the existing TTY selection encoder; command
attachment/death checks remain unchanged. Status message/prompt, file read/write,
job startup, cwd, config loading/causes and popup/menu operations accept client
and session handles. Shell/file/prompt callbacks retain their existing command
policy and ownership, including absent clients, immediate failures, asynchronous
completion and queue resumption. View output and terminal diagnostics use pane
operations and owned terminal-description snapshots.

Display-panes drawing moved without algorithm changes into the overlay subsystem
in `src/overlay/pane_numbers.rs`; command-specific delay/key/wait policy remains
in the command. Static dispatch, clipping, glyph bytes, visible ranges, styles,
terminal cursor effects and redraw order are unchanged. The existing drawing
unit fixture resolves the relocated renderer through a test-only import.
Operations retain unsafe initialization, borrow and callback contracts. No new
callback storage, trait objects, ownership representations or public-trait changes
were introduced; existing internal reentrancy work remains H1.

The audit covered handlers and deferred callbacks in 16 command files plus the
relocated renderer and reached operations. Remaining: display-message's pane
input adapter and new-session's config-causes adapter will be finished with Q1's
target interfaces; list-keys table storage is Z1. Command-owned callback state
is not a client/pane payload escape. No conformance scenarios, harnesses,
expectations, exclusions or gate configuration changed.

Validation with tmux 3.7b on this implementation:

- `cargo check -q`: passed.
- `make -C .. test-commands SUT=hmux`: 800 passed, 2 existing exclusions.
- `make -C .. test-hooks SUT=hmux`: 97 passed.
- `make -C .. test-notifications SUT=hmux`: 63 passed.
- `make -C .. test-queue SUT=hmux`: 45 passed.
- `make -C .. test SUT=hmux`: 2,679 unit and 1,299 conformance passed;
  6 existing exclusions. Additional tests/doctests passed, one ignored doctest.
- `make -C .. leak SUT=hmux`: 2,817 passed, 10 skipped.
- `make -C .. lint SUT=hmux`: passed. Scoped rustfmt baseline comparison,
  the new renderer's rustfmt check and `git diff --check` passed. Rendering and
  parser algorithms are unchanged; no additional VT gate was needed.

Next: Q1 and the recorded residual adapters, then H1/Z1 privacy and exhaustive
audit. This commit remains local on `h1`; no remote publication occurred.

**Completion record: Q1 — targets, hooks and residual consumers**

Implementation: the commit containing this record, based on `7685df075`.
Commands use handle-based current-target updates and hook insertion, preserving
resolution timing, active versus explicit pane index semantics, session fallback,
queue insertion and nested suppression. Read-only event/current uses are owned
snapshots; send-keys receives scoped mutable mouse input through a queue operation
with the existing reentrancy exclusion. Attach still selects the pane/session
before resolving the current link, and select-window retains its conditional
current-target replacement. Existing unsafe target validity contracts remain.

Winlink queries use the existing window operation. Swap-window link exchange now
owns both reverse memberships and source-mark link remapping, deliberately leaving
other mark fields untouched. Commands retain same-window/group refusal, detached
selection, group synchronization/redraw and size recalculation. Sorted key bindings
and global session options use handle APIs. Display-message input and new-session
configuration causes now use the O1 APIs. No production pane/client/TTY/session
payload extraction or key-table borrow remains in the migrated handlers.

Final broad audit still found mutable client retval/source-file-depth field
accessors in confirm/run/source callbacks; these are explicitly queued for H1/Z1,
along with making format enumeration's unchecked access contract unsafe. Command-
owned callback state, scoped paste/environment/ACL traits and ephemeral resolved
target snapshots remain valid interfaces. Public traits and representations are
unchanged; no conformance inputs or gate configuration changed.

Validation with tmux 3.7b:

- `cargo check -q`: passed.
- `make -C .. test-commands SUT=hmux`: 800 passed, 2 existing exclusions.
- `make -C .. test-hooks SUT=hmux`: 97 passed.
- `make -C .. test-notifications SUT=hmux`: 63 passed.
- `make -C .. test-queue SUT=hmux`: 45 passed.
- `make -C .. test SUT=hmux`: 2,679 unit and 1,299 conformance passed;
  6 existing exclusions. Additional tests/doctests passed, one ignored doctest.
  The initial unit build exposed an implicit global-option import in a fixture;
  adding its explicit import fixed the build before the final full run.
- `make -C .. leak SUT=hmux`: 2,817 passed, 10 skipped.
- `make -C .. lint SUT=hmux`: passed. Scoped rustfmt baseline comparison and
  `git diff --check` passed. No rendering/parser implementation changed.

Next: H1/Z1 privacy enforcement and exhaustive production command/helper/callback
audit. All task commits remain local on `h1`; nothing was pushed.

**Completion record: residual command operations and callback contracts**

Implementation: the commit containing this record, based on `99a5d45d4`.
This slice contains no module restructuring or visibility restrictions.

Confirm/run/source callbacks use checked client return-code and source-depth
operations. Prompt queries and wait-channel logging no longer expose client
fields or pointers. Select/last-pane input updates own the corresponding
border/status redraw; respawn-pane uses a screen-redraw request. Configuration
completion and absent-client working-directory queries use subsystem interfaces.
Format enumeration explicitly requires its existing resolver and non-reentrant
borrow contract. The native option-engine adapter obtains retained global option
handles through an accessor, preserving context identity without copying stores.

The attach-session unit fixture explicitly imports its configuration flag after
the command stops importing that flag. Existing public traits, conformance
scenarios, harnesses, assertions, exclusions and gate configuration are unchanged.
Module movement, payload/field visibility restrictions and subprocess fixture
path updates belong to the separate Z1 slice.

Validation on this isolated slice, with tmux 3.7b:

- `cargo check -q`: passed.
- `make -C .. test SUT=hmux`: 2,679 unit and 1,299 conformance tests passed,
  with 6 existing conformance exclusions. The tmux-sys tests and additional
  tests/doctests passed; one existing doctest remains ignored.
- `make -C .. test-commands SUT=hmux`: 800 passed, 2 existing exclusions.
- `make -C .. test-hooks SUT=hmux`: 97 passed.
- `make -C .. test-queue SUT=hmux`: 45 passed.
- `make -C .. test-notifications SUT=hmux`: 63 passed.
- `make -C .. leak SUT=hmux`: 2,817 passed, 10 existing exclusions.
- `make -C .. lint SUT=hmux`, scoped rustfmt baseline comparison and
  `git diff --check`: passed.

This slice is committed locally on `h1`. The separate visibility slice remains
pending; no publication occurred.

**Template for the next completion record**

- Task ID and exact slice; implementation commit (or an unambiguous reference to the commit containing the record).
- Consumers migrated; APIs reused/added; important invariants preserved.
- Remaining escapes and their task IDs. Update the queue and next recommended slice.
- Exact validation commands, tested revision, pass/fail results, selected counts, and relevant exclusions. Do not claim checks not run.
- Confirm implementation/conformance separation and public-trait status.
- Commit/push state when authorized; unresolved dependencies or failures.

**Invocation**

> execute ./report-status.md

This invokes the execution contract above, including validation, report maintenance and a local commit on `h1`. Pushing requires separate explicit user approval. To select a different slice, append its task ID and any scope constraints.
