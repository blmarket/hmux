# Remaining C allocations and ownership migration

Audit of commit `3fe87cd`, 2026-09-23. The working tree was clean before this report. Policy: [plan-ownership.md](plan-ownership.md). This is the requested inventory and proposal; no production migration was performed.

## Scope and result

There are **255 direct production allocation/reallocation call sites in 39 source files**, plus **14 calls inside allocator infrastructure**, **7 in-source test calls**, and **7 integration-test calls**. Counts are syntactic sites, not distinct allocations or runtime frequencies. Calls through an allocating helper are represented by that helper's implementation, not counted again at each consumer. Exported adapters are included even if they currently have no internal callers. Library-managed allocations are listed separately below.

Searched all Rust source in `src`, `hmux-buffer`, `hmux-cmdparse`, `hmux-rt`, and `tests` for malloc/calloc/realloc families, string duplication, formatting, and allocating foreign APIs; inspected producer implementations and representative ownership/destruction paths. No direct C allocator calls were found in the three workspace subcrates. FFI declarations, imports, comments, Rust `Box`/`Vec`/`CString` allocations, and descriptor-only operations are excluded from the count. This is a source audit, not a heap profile or proof of reachability. It does not enumerate allocations hidden entirely inside dependency/library implementations.

| Allocator | Production call sites |
| --- | ---: |
| `calloc` | 7 |
| `malloc` | 3 |
| `realloc` | 2 |
| `strndup` | 1 |
| `xasprintf` | 119 |
| `xcalloc` | 4 |
| `xmalloc` | 3 |
| `xmemdup` | 3 |
| `xrealloc` | 2 |
| `xreallocarray` | 11 |
| `xrecallocarray` | 1 |
| `xstrdup` | 99 |

## Proposed batches and completion conditions

| Batch | Ownership boundary and proposed migration | Callers, teardown, and completion condition |
| --- | --- | --- |
| A — string-return adapters | Use existing `*_cstring`/owned helpers and return `CString`, `Option<CString>`, or byte buffers internally. Borrow as `&CStr` for synchronous consumers. | Format expansion, command printing, names, grid/copy extraction, hooks, monitor, events, JSON, options, paste, regex substitution, and terminal helpers. Replace each caller's `free` at the same time; remove duplicate Rust → libc copies. Keep an adapter only for a demonstrated foreign contract. `extern "C"`/`no_mangle` alone does not demonstrate one. |
| B — error outputs | Replace owned `char **cause`/error return conventions with `Result<T, CString>` (or `Option<CString>` for stored optional diagnostics). | Arguments/parser, options, JSON, layout, spawn, session, server/client, tty, paste, and command entries. Change their command/reporting consumers together and remove all success/error-path frees. Preserve exact byte messages, null/empty distinctions, and error timing. |
| C — command argument values and parser tokens | `Vec<CString>` for argv; an owned value enum for strings versus command lists; `Vec` for value arrays. Retain existing command-list retain/release behavior pending a separate reference-count audit. | `cmd_prepend/append/unpack/copy/free_argv`, `args_value` copy/free paths, parser construction/free paths, client dispatch, spawn/job consumers. Temporary pointer arrays may borrow the owners for exec/foreign calls. Remove union bitwise ownership copies and all calloc/free paths before introducing drop-bearing fields. Lexer scratch already uses `Vec`; migrate its completed-token handoff too. |
| D — prompt and UTF-8 buffers | Use existing `utf8_fromcstr_vec` and `utf8_tocstr_cstring`; own prompt cells as `Vec<utf8_data>`, with slices for access. Use `Vec<u8>`/`CString` for vis output. | Prompt initialization, history replacement, completion, paste, key editing, callbacks, destruction, and other raw UTF-8 consumers. Keep required sentinel cells until all scans are converted; end borrows before reentrant callbacks. Remove the three prompt realloc sites and every old buffer free/replacement. |
| E — screen and visible ranges | Screen title: `CString`; path: `Option<CString>`; title stack: `VecDeque<CString>`; tabs: `Vec<bitstr_t>`; visible ranges: `Vec<visible_range>`. | Screen init/reinit/free, title push/pop/set, path set, resize/tab operations, embedded screen owners, client overlay/range reset and destruction. Audit initialization and moves of containing records. Preserve title stack transfer/order and explicitly zero newly grown ranges; remove redundant capacity/count bookkeeping where possible. |
| F — imsg | `Box` for owned ibuf records, `Vec<u8>` for owned payloads, boxed slice or `Vec` for reader scratch; explicit borrowed slice views for stack/sub-buffer ibufs. | `ibuf_open/dynamic/reserve/free`, queue `OwnedIbuf`, imsg message ownership, reader callbacks, drain/clear, descriptor transfer and cancellation. `ibuf` currently derives `Copy`; remove that for owning records and distinguish borrowed views before adding owners. Preserve `freezero` wiping semantics, errno, fallible allocation, FD close/transfer order, and address stability. Existing queue ownership does not eliminate C payload/record allocations. |
| G — leaf owners and local work | Process name → `CString`; cached home → optional owned string; fuzzy masks → `Vec<bitstr_t>`; editor result → `Option<Vec<u8>>`; find-window temporaries → owned C strings. | Audit process/global lifetime, fuzzy output callers, and every editor callback that currently receives/frees/transfers its buffer. Editor data is length-delimited binary, not a C string. Distinguish failure from empty results as current behavior requires. |
| H — foreign boundaries | Keep matching foreign allocators/deallocators behind scoped owners; replace APIs only in a coherent behavior-preserving batch. | Variadic formatting, libc environment, stdio/history input, glob, POSIX regex, ncurses, systemd. Details below. Do not use `CString::from_raw`, `Box::from_raw`, or `Vec::from_raw_parts` on libc-owned allocations. |

Suggested order: A/B in subsystem-sized batches, then C, D, E, G, F; H can proceed independently where foreign contracts are clear. A/B have many callers, so complete one producer/consumer family at a time rather than globally changing signatures without their consumers.

Ordinary strings, buffers, and scoped records need ordinary ownership. Use `RefBox<T>` only if the object audit establishes escaping non-owning references, with checked `Weak` access and explicit observer removal. Do not add IDs, pointer-key registries, or blanket `Rc`/`Arc`. A temporary pointer passed to C is not a reason to use `RefBox`. Preserve one-to-one field roles and document any local ABI exception.

## Complete direct production inventory

Every direct production call site is listed below, grouped by containing function. Line numbers refer to the audited revision. Multiple sites in one row represent separate branches or allocation/growth steps. The migration column assigns each to the batches above; it does not claim the whole caller graph is already converted.

| File / function | Allocation sites (`allocator:line`) | Proposed migration |
| --- | --- | --- |
| [src/arguments.rs](src/arguments.rs) · `args_copy_value` | `xstrdup:349` | C: owned argument value/array; migrate clone, command-list references, union and free paths. |
| [src/arguments.rs](src/arguments.rs) · `args_parse_flag_argument` | `xstrdup:408`, `xasprintf:417`, `xasprintf:444` | C/B: own parsed argument string and diagnostic together; migrate parser consumers. |
| [src/arguments.rs](src/arguments.rs) · `args_parse_flags` | `xasprintf:547`, `xasprintf:556` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/arguments.rs](src/arguments.rs) · `args_parse` | `xasprintf:662`, `xasprintf:680`, `xasprintf:697`, `xasprintf:706` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/arguments.rs](src/arguments.rs) · `args_copy_copy_value` | `xstrdup:727`, `xstrdup:737` | C: owned argument value/array; migrate clone, command-list references, union and free paths. |
| [src/arguments.rs](src/arguments.rs) · `args_from_vector` | `xcalloc:902`, `xstrdup:910` | C: owned argument value/array; migrate clone, command-list references, union and free paths. |
| [src/arguments.rs](src/arguments.rs) · `from_argv` | `xstrdup:930` | C: owned argument value/array; migrate clone, command-list references, union and free paths. |
| [src/arguments.rs](src/arguments.rs) · `args_print` | `xstrdup:989` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/arguments.rs](src/arguments.rs) · `args_escape` | `xstrdup:1133` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/arguments.rs](src/arguments.rs) · `args_make_commands_get_command` | `xstrdup:1457` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/arguments.rs](src/arguments.rs) · `args_result_to_c` | `xstrdup:1740` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/cmd/core.rs](src/cmd/core.rs) · `cmd_prepend_argv` | `xreallocarray:306`, `xstrdup:312` | C: `Vec<CString>` argv owner plus scoped pointer views; replace cmd_free_argv contract. |
| [src/cmd/core.rs](src/cmd/core.rs) · `cmd_append_argv` | `xreallocarray:329`, `xstrdup:337` | C: `Vec<CString>` argv owner plus scoped pointer views; replace cmd_free_argv contract. |
| [src/cmd/core.rs](src/cmd/core.rs) · `cmd_unpack_argv` | `xcalloc:385`, `xstrdup:398` | C: `Vec<CString>` argv owner plus scoped pointer views; replace cmd_free_argv contract. |
| [src/cmd/core.rs](src/cmd/core.rs) · `cmd_copy_argv` | `xcalloc:422`, `xstrdup:430` | C: `Vec<CString>` argv owner plus scoped pointer views; replace cmd_free_argv contract. |
| [src/cmd/core.rs](src/cmd/core.rs) · `cmd_stringify_argv` | `xstrdup:459` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/cmd/core.rs](src/cmd/core.rs) · `cmd_get_alias` | `xstrdup:542` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/cmd/core.rs](src/cmd/core.rs) · `cmd_find` | `xasprintf:609`, `xasprintf:618` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/cmd/core.rs](src/cmd/core.rs) · `cmd_parse` | `xasprintf:659`, `xasprintf:676`, `xasprintf:685` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/cmd/core.rs](src/cmd/core.rs) · `cmd_print` | `xstrdup:726` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/cmd/core.rs](src/cmd/core.rs) · `cmd_list_print` | `xstrdup:887` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/cmd/core.rs](src/cmd/core.rs) · `cmd_template_replace` | `xstrdup:1055`, `xstrdup:1059` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/cmd/entries/find_window.rs](src/cmd/entries/find_window.rs) · `cmd_find_window_exec` | `xasprintf:135`, `xasprintf:151`, `xasprintf:163`, `xasprintf:175`, `xasprintf:189`, `xasprintf:196`, `xasprintf:205` | G: compose local filter strings in owned byte-preserving storage; remove branch-specific frees. |
| [src/cmd/parse.rs](src/cmd/parse.rs) · `cmd_parse_get_error` | `xstrdup:174`, `xasprintf:176` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/cmd/parse.rs](src/cmd/parse.rs) · `build_parser_commands` | `xstrdup:392` | C: own parser value/token strings; migrate union transfers and parser error cleanup. |
| [src/cmd/parse.rs](src/cmd/parse.rs) · `cmd_parse_build_command` | `xstrdup:604` | C: own parser value/token strings; migrate union transfers and parser error cleanup. |
| [src/cmd/parse.rs](src/cmd/parse.rs) · `cmd_parse_from_arguments` | `xstrdup:983` | C: own parser value/token strings; migrate union transfers and parser error cleanup. |
| [src/cmd/parse.rs](src/cmd/parse.rs) · `into_raw` | `xmalloc:1104` | C: own parser value/token strings; migrate union transfers and parser error cleanup. |
| [src/compat/imsg/imsg_buffer.rs](src/compat/imsg/imsg_buffer.rs) · `ibuf_open` | `calloc:115`, `calloc:120` | F: Own record/payload/reader storage; separate borrowed ibuf views and preserve wiping/FD teardown. |
| [src/compat/imsg/imsg_buffer.rs](src/compat/imsg/imsg_buffer.rs) · `ibuf_dynamic` | `calloc:138`, `calloc:143` | F: Own record/payload/reader storage; separate borrowed ibuf views and preserve wiping/FD teardown. |
| [src/compat/imsg/imsg_buffer.rs](src/compat/imsg/imsg_buffer.rs) · `ibuf_reserve` | `realloc:174` | F: Own record/payload/reader storage; separate borrowed ibuf views and preserve wiping/FD teardown. |
| [src/compat/imsg/imsg_buffer.rs](src/compat/imsg/imsg_buffer.rs) · `ibuf_get_string` | `strndup:713` | F: Return owned `CString`; preserve bounded first-NUL scan and rpos/error behavior. |
| [src/compat/imsg/imsg_buffer.rs](src/compat/imsg/imsg_buffer.rs) · `msgbuf_new_reader` | `malloc:827` | F: Own record/payload/reader storage; separate borrowed ibuf views and preserve wiping/FD teardown. |
| [src/compat/systemd.rs](src/compat/systemd.rs) · `systemd_create_socket` | `xstrdup:85`, `xasprintf:92` | H/B: own local path/unit/slice strings and diagnostics as `CString`; scope bus handles with matching cleanup. |
| [src/compat/systemd.rs](src/compat/systemd.rs) · `systemd_move_to_new_cgroup` | `xasprintf:158`, `xasprintf:182`, `xasprintf:197`, `xasprintf:206`, `xasprintf:230`, `xasprintf:243`, `xasprintf:256`, `xasprintf:278`, `xasprintf:293`, `xstrdup:302`, `xasprintf:316`, `xasprintf:334`, `xasprintf:354`, `xasprintf:397`, `xasprintf:415`, `xasprintf:428`, `xasprintf:445`, `xasprintf:452`, `xasprintf:468`, `xasprintf:481`, `xasprintf:497`, `xasprintf:511` | H/B: own local path/unit/slice strings and diagnostics as `CString`; scope bus handles with matching cleanup. |
| [src/compat/vis.rs](src/compat/vis.rs) · `stravis` | `calloc:1073`, `realloc:1079` | D/H: `Vec<u8>` builder and owned result; preserve fallible C adapter if required. |
| [src/environ.rs](src/environ.rs) · `environ_push` | `xcalloc:541` | H: libc environment seed; retain C allocation until environment mutation/exec boundary is redesigned. |
| [src/events_payload.rs](src/events_payload.rs) · `event_payload_item_print` | `xmemdup:664` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/events_payload.rs](src/events_payload.rs) · `event_payload_print` | `xmemdup:702` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/format/expression.rs](src/format/expression.rs) · `format_pretty_time` | `xstrdup:59` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/format/expression.rs](src/format/expression.rs) · `format_expand_time` | `xstrdup:3519` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/format/expression.rs](src/format/expression.rs) · `format_expand` | `xstrdup:3526` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/format/expression.rs](src/format/expression.rs) · `format_single` | `xstrdup:3573` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/format/expression.rs](src/format/expression.rs) · `format_single_from_state` | `xstrdup:3596` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/format/expression.rs](src/format/expression.rs) · `format_single_from_target` | `xstrdup:3612` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/format.rs](src/format.rs) · `format_grid_word` | `xstrdup:596` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/format.rs](src/format.rs) · `format_grid_line` | `xstrdup:692` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/format.rs](src/format.rs) · `format_grid_hyperlink` | `xstrdup:746` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/format_draw.rs](src/format_draw.rs) · `format_trim_output` | `xmalloc:1865` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/fuzzy.rs](src/fuzzy.rs) · `fuzzy_match` | `calloc:927` | G: return owned bit mask; migrate mask consumers/free with this compatibility adapter. |
| [src/grid/core.rs](src/grid/core.rs) · `grid_string_cells` | `xmalloc:1687` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/hooks.rs](src/hooks.rs) · `hooks_monitor_to_string` | `xstrdup:754` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/json.rs](src/json.rs) · `json_find_string` | `xasprintf:320`, `xasprintf:332` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/json.rs](src/json.rs) · `json_find_number` | `xasprintf:354`, `xasprintf:366` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/json.rs](src/json.rs) · `json_find_boolean` | `xasprintf:388`, `xasprintf:400` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/json.rs](src/json.rs) · `json_find_object` | `xasprintf:422`, `xasprintf:434` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/json.rs](src/json.rs) · `json_find_array` | `xasprintf:456`, `xasprintf:468` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/json.rs](src/json.rs) · `json_error` | `xasprintf:491`, `xasprintf:507` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/json.rs](src/json.rs) · `json_to_string` | `xmemdup:1314` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/layout/core.rs](src/layout/core.rs) · `layout_resize_floating_pane_to` | `xstrdup:1044`, `xstrdup:1055` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/layout/core.rs](src/layout/core.rs) · `layout_resize_floating_pane` | `xstrdup:1085`, `xstrdup:1096`, `xstrdup:1108` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/layout/core.rs](src/layout/core.rs) · `layout_get_tiled_cell` | `xstrdup:1941`, `xasprintf:1975`, `xasprintf:1995`, `xstrdup:2011` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/layout/core.rs](src/layout/core.rs) · `layout_floating_args_parse` | `xasprintf:2103`, `xasprintf:2128`, `xasprintf:2153`, `xasprintf:2173`, `xstrdup:2217`, `xstrdup:2221` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/layout/core.rs](src/layout/core.rs) · `layout_split_floating_cell` | `xstrdup:2395` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/layout/custom.rs](src/layout/custom.rs) · `layout_dump` | `xstrdup:277` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/layout/custom.rs](src/layout/custom.rs) · `layout_parse` | `xasprintf:653`, `xasprintf:662`, `xstrdup:679`, `xstrdup:741` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/layout/custom.rs](src/layout/custom.rs) · `layout_parse_json` | `xstrdup:1043` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/layout/custom.rs](src/layout/custom.rs) · `layout_parse_json_layout` | `xasprintf:1114`, `xasprintf:1132`, `xasprintf:1147`, `xasprintf:1165`, `xasprintf:1183`, `xstrdup:1202`, `xasprintf:1220`, `xasprintf:1273`, `xasprintf:1308`, `xstrdup:1352` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/layout/custom.rs](src/layout/custom.rs) · `layout_construct` | `xstrdup:1420`, `xstrdup:1426`, `xstrdup:1436`, `xstrdup:1440`, `xstrdup:1454`, `xstrdup:1459`, `xstrdup:1463` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/layout/custom.rs](src/layout/custom.rs) · `layout_parse_ctx_check_indexes` | `xstrdup:1563`, `xstrdup:1588`, `xstrdup:1613` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/mode_tree.rs](src/mode_tree.rs) · `mode_tree_start` | `xstrdup:711` | G: optional `CString` search/filter fields; update replace, callback, mode refresh and teardown together. |
| [src/mode_tree.rs](src/mode_tree.rs) · `mode_tree_search_callback` | `xstrdup:1999` | G: optional `CString` search/filter fields; update replace, callback, mode refresh and teardown together. |
| [src/mode_tree.rs](src/mode_tree.rs) · `mode_tree_filter_callback` | `xstrdup:2026` | G: optional `CString` search/filter fields; update replace, callback, mode refresh and teardown together. |
| [src/monitor.rs](src/monitor.rs) · `monitor_parse` | `xstrdup:851`, `xstrdup:852` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/names.rs](src/names.rs) · `default_window_name` | `xstrdup:221` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/names.rs](src/names.rs) · `parse_window_name` | `xstrdup:260` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/options.rs](src/options.rs) · `options_default_to_string` | `xstrdup:464` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/options.rs](src/options.rs) · `options_array_set` | `xstrdup:648`, `xasprintf:654`, `xasprintf:721`, `xstrdup:738` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/options.rs](src/options.rs) · `options_to_string` | `xstrdup:880` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/options.rs](src/options.rs) · `options_parse` | `xstrdup:929`, `xstrdup:931` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/options.rs](src/options.rs) · `options_match_command` | `xstrdup:1018`, `xstrdup:1019` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/options.rs](src/options.rs) · `options_scope_from_name` | `xasprintf:1358`, `xasprintf:1377`, `xasprintf:1383`, `xasprintf:1396`, `xasprintf:1402`, `xasprintf:1428`, `xasprintf:1434` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/options.rs](src/options.rs) · `options_scope_from_flags` | `xasprintf:1466`, `xasprintf:1472`, `xasprintf:1488`, `xasprintf:1494`, `xasprintf:1510`, `xasprintf:1516` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/options.rs](src/options.rs) · `options_from_string_check` | `xasprintf:1627`, `xasprintf:1637`, `xasprintf:1648`, `xasprintf:1660` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/options.rs](src/options.rs) · `options_from_string_flag` | `xasprintf:1695` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/options.rs](src/options.rs) · `options_find_choice` | `xasprintf:1724` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/options.rs](src/options.rs) · `options_from_string` | `xasprintf:1777`, `xasprintf:1786`, `xasprintf:1826`, `xasprintf:1840`, `xasprintf:1854` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/osdep_linux.rs](src/osdep_linux.rs) · `osdep_get_name` | `xstrdup:24` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/paste.rs](src/paste.rs) · `paste_get_top` | `xstrdup:250` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/paste.rs](src/paste.rs) · `paste_rename` | `xstrdup:355`, `xstrdup:361`, `xasprintf:367`, `xasprintf:378` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/paste.rs](src/paste.rs) · `paste_set_inner` | `xstrdup:456`, `xasprintf:462` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/paste.rs](src/paste.rs) · `paste_make_sample` | `xstrdup:504` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/proc.rs](src/proc.rs) · `proc_start` | `xstrdup:273` | G: own `tmuxproc.name` as `CString`; update log borrowers and process lifetime. |
| [src/prompt.rs](src/prompt.rs) · `prompt_paste` | `xreallocarray:1422` | D: prompt `Vec<utf8_data>`; convert all init/edit/replace/free paths together. |
| [src/prompt.rs](src/prompt.rs) · `prompt_replace_complete` | `xreallocarray:1539` | D: prompt `Vec<utf8_data>`; convert all init/edit/replace/free paths together. |
| [src/prompt.rs](src/prompt.rs) · `prompt_key` | `xreallocarray:10778` | D: prompt `Vec<utf8_data>`; convert all init/edit/replace/free paths together. |
| [src/reactor/buffer.rs](src/reactor/buffer.rs) · `read_line` | `malloc:93` | A: return owned line bytes with length; update evbuffer readln/readline consumers and frees. |
| [src/regsub.rs](src/regsub.rs) · `regsub` | `xstrdup:73` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/screen.rs](src/screen.rs) · `screen_init` | `xstrdup:101` | E: owned title and title-history strings; migrate push/pop/set/free together. |
| [src/screen.rs](src/screen.rs) · `screen_reset_tabs` | `calloc:186` | E: owned tabs vector. |
| [src/screen.rs](src/screen.rs) · `screen_push_title` | `xstrdup:355` | E: owned title and title-history strings; migrate push/pop/set/free together. |
| [src/server.rs](src/server.rs) · `server_create_socket` | `xasprintf:375` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/server_client.rs](src/server_client.rs) · `server_client_ensure_ranges` | `xrecallocarray:993` | E: `Vec<visible_range>` with explicit zeroed growth; migrate reset/free and raw consumers. |
| [src/server_client.rs](src/server_client.rs) · `server_client_open` | `xasprintf:1259`, `xstrdup:1267` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/server_client.rs](src/server_client.rs) · `server_client_dispatch_command` | `xstrdup:4735` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/server_fn.rs](src/server_fn.rs) · `server_link_window` | `xasprintf:441`, `xasprintf:453` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/session.rs](src/session.rs) · `session_attach` | `xasprintf:805` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/spawn.rs](src/spawn.rs) · `spawn_window` | `xasprintf:333`, `xasprintf:367`, `xasprintf:398`, `xasprintf:418` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/spawn.rs](src/spawn.rs) · `spawn_pane` | `xasprintf:535`, `xasprintf:542`, `xasprintf:591`, `xasprintf:816`, `xasprintf:913`, `xasprintf:919`, `xasprintf:937`, `xasprintf:943` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/spawn.rs](src/spawn.rs) · `spawn_editor_finish` | `malloc:1059` | G: transfer `Option<Vec<u8>>` to editor callbacks; preserve binary length and failure semantics. |
| [src/text/utf8.rs](src/text/utf8.rs) · `utf8_stravis` | `xreallocarray:1725`, `xrealloc:1728` | D/A: owned UTF-8 cells or byte/C-string result; migrate raw callers and preserve sentinel/NUL semantics. |
| [src/text/utf8.rs](src/text/utf8.rs) · `utf8_stravisx` | `xreallocarray:1778`, `xrealloc:1781` | D/A: owned UTF-8 cells or byte/C-string result; migrate raw callers and preserve sentinel/NUL semantics. |
| [src/text/utf8.rs](src/text/utf8.rs) · `utf8_sanitize` | `xstrdup:1832` | D/A: owned UTF-8 cells or byte/C-string result; migrate raw callers and preserve sentinel/NUL semantics. |
| [src/text/utf8.rs](src/text/utf8.rs) · `utf8_fromcstr` | `xreallocarray:1916`, `xreallocarray:1945` | D/A: owned UTF-8 cells or byte/C-string result; migrate raw callers and preserve sentinel/NUL semantics. |
| [src/text/utf8.rs](src/text/utf8.rs) · `utf8_tocstr` | `xreallocarray:1998`, `xreallocarray:2011` | D/A: owned UTF-8 cells or byte/C-string result; migrate raw callers and preserve sentinel/NUL semantics. |
| [src/text/utf8.rs](src/text/utf8.rs) · `utf8_padcstr` | `xstrdup:2081` | D/A: owned UTF-8 cells or byte/C-string result; migrate raw callers and preserve sentinel/NUL semantics. |
| [src/text/utf8.rs](src/text/utf8.rs) · `utf8_rpadcstr` | `xstrdup:2088` | D/A: owned UTF-8 cells or byte/C-string result; migrate raw callers and preserve sentinel/NUL semantics. |
| [src/tmux.rs](src/tmux.rs) · `shell_argv0` | `xstrdup:736` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/tmux.rs](src/tmux.rs) · `clean_name` | `xstrdup:790` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/tmux.rs](src/tmux.rs) · `find_home` | `xstrdup:878` | G: own cached home bytes; expose borrowed `&CStr` while preserving missing-home behavior. |
| [src/tty_term.rs](src/tty_term.rs) · `tty_term_create` | `xasprintf:1536`, `xasprintf:1541` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/window.rs](src/window.rs) · `window_pane_start_input` | `xstrdup:4760` | B: return/store owned diagnostic (`Result<_, CString>`); migrate all cause consumers and frees. |
| [src/window_copy.rs](src/window_copy.rs) · `window_copy_get_word` | `xstrdup:1161` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/window_copy.rs](src/window_copy.rs) · `window_copy_get_line` | `xstrdup:1183` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |
| [src/window_copy.rs](src/window_copy.rs) · `window_copy_get_hyperlink` | `xstrdup:1204` | A: return/transfer owned byte-preserving string(s); use existing owned helper where available and remove consumer frees. |

## Indirect allocations and genuine foreign ownership

These are additional ownership boundaries, not included in the 255 direct-site count. Borrowed outputs must not be freed merely because they came from C.

| Boundary / sites | Existing allocation and destruction | Proposed path |
| --- | --- | --- |
| History line input: `src/prompt_history.rs:102`, wrapper at `:25` | `getline` → `__getdelim` allocates/reallocates `line`; released with `free` at `:115`. Stored history already owns `CString`s. | Read bytes into a reusable `Vec<u8>` with byte-oriented buffered input, preserving newline stripping, first-NUL parsing, and EOF behavior. Alternatively keep a scoped libc buffer owner while retaining stdio. |
| Stdio: `src/cfg.rs:218`; `src/file.rs:656,796`; `src/log.rs:49`; `src/osdep_linux.rs:36`; `src/prompt_history.rs:89,131`; `src/spawn.rs:1050,1091` | `fopen`/`fdopen` return library-managed `FILE`s; release via `fclose`. Log storage persists across calls. | Scoped FILE owner using `fclose`, with explicit global log ownership/replacement; or `File`/byte I/O when all consumers permit it. Preserve descriptor transfer on `fdopen` success and failure; spawn already documents that distinction. Never libc-free a FILE directly. |
| Globs: `src/cmd/entries/source_file.rs:403`; `src/compat/getdtablecount.rs:31` | `glob` allocates result storage; `globfree` releases it. | Scoped glob-result owner retaining libc matching, sorting, errors, and partial-result cleanup. Borrow path strings only while the result lives. |
| Regex: `src/format/expression.rs:856`; `src/regsub.rs:104`; `src/window.rs:4322`; `src/window_copy.rs:6335,6940` | `regcomp` initializes library-owned internals in `regex_t`; successful compilations need `regfree`. | Scoped successfully compiled POSIX regex owner. Preserve POSIX flags, locale, match offsets and invalid-pattern behavior; replacing the regex engine is a separate semantic change. |
| Terminfo: `src/tty_term.rs:1620` | `setupterm` creates library terminal state; `del_curterm` at `:1496,1699`. `tigetstr` results are borrowed from it. | Scope terminal ownership and global `cur_term` interaction. Copy required capability bytes before deletion; preserve cleanup on error paths and do not free individual borrowed capability pointers. |
| Systemd strings: `src/compat/systemd.rs:300,362,366` | `sd_pid_get_user_slice`, `sd_pid_get_user_unit`, `sd_pid_get_unit` produce libc-owned strings; slice/unit are freed at `:314,395`. | Local foreign-string owner using libc `free`, or copy to `CString` then immediately free. Preserve fallback branches and ownership on each return path. |
| Systemd objects: `src/compat/systemd.rs:156,164,188,435` | Default bus, signal slot, method message, reply/error storage; cleanup at `:539–543` uses `sd_bus_error_free` and matching message/slot/bus unref functions. Message append/container operations may grow library storage. | Scoped owners around each returned reference and initialized error. Audit callback lifetime and unregistration ordering; keep strings borrowed from messages within message lifetime. Existing systemd reference counting is library-managed; do not substitute an unaudited Rust `Rc`. |
| Process environment: `src/environ.rs:541,552`; `src/client.rs:777`; `src/osdep_linux.rs:93`; `src/job.rs:348` | Explicit empty `environ` allocation plus libc-managed storage from `setenv`. `environ_push` frees its seed only when libc replaced the pointer. | Keep allocator-compatible environment interaction isolated. A larger migration can build owned environment strings and a scoped envp for exec, but only after auditing all intervening `getenv`/`setenv` and spawn behavior. Do not insert a Rust Vec buffer into storage libc may resize/free. |
| Variadic formatting: `src/xmalloc.rs:143,164,177` | `vasprintf` allocates libc strings. `xvasprintf_cstring` and `try_vasprintf_cstring` already copy to `CString` and free the foreign allocation locally. | These are contained allocations, still real C allocations. Retain the adapter for C varargs; replace selected internal format producers with byte-preserving Rust construction where equivalent. Changing callers to the owned helper removes escaped C ownership but does not eliminate its temporary C allocation. |

`getcwd` and `realpath` calls in this tree use caller-provided buffers, so they are not allocating-return sites. `getenv`, passwd lookup, locale, and other library static/borrowed results are not caller-owned heap allocations. File descriptors from `dup`/socket/open are resources, not C heap allocations. Library-internal caches remain outside the direct-site inventory.

## Allocator infrastructure (14 sites)

These implement the allocator contracts above; counting them as additional independent production objects would double-count calls through the wrappers.

| File / function | Allocation/delegation sites | Migration path |
| --- | --- | --- |
| `src/xmalloc.rs` / `xmalloc` | `malloc:17` | Remove once the last required C-allocation caller/ABI is gone. |
| `src/xmalloc.rs` / `xcalloc` | `calloc:35` | Same; migrate containing record initialization and destruction together. |
| `src/xmalloc.rs` / `xrealloc` | `xreallocarray:49` | Remove after raw resizable buffers migrate. |
| `src/xmalloc.rs` / `xreallocarray` | `reallocarray:61` | Same; preserve overflow and zero-size contracts at any retained adapter. |
| `src/xmalloc.rs` / `xrecallocarray` | `recallocarray:81` | Replace range growth with initialized elements; retain adapter only if required. |
| `src/xmalloc.rs` / `xstrdup` | `strdup:93` | Internal callers should use owned strings; retained foreign results stay libc-owned. |
| `src/xmalloc.rs` / `xstrndup` | `strndup:105` | Preserve bounded first-NUL semantics; remove when no contract needs it. |
| `src/xmalloc.rs` / `xmemdup` | `xmalloc:117` | Preserve all requested bytes and trailing NUL; choose bytes rather than CString where interior NUL is meaningful. |
| `src/xmalloc.rs` / `xasprintf` | `xvasprintf:133` | Narrow to required variadic adapters. |
| `src/xmalloc.rs` / `xvasprintf` | `vasprintf:143` | Keep matching `free` and current fatal allocation-error behavior. |
| `src/xmalloc.rs` / `xvasprintf_cstring` | `xvasprintf:164` | Already contains C ownership locally; optional later elimination of temporary allocation. |
| `src/xmalloc.rs` / `try_vasprintf_cstring` | `vasprintf:177` | Same, retaining fallible behavior and cleanup. |
| `src/compat/recallocarray.rs` / `recallocarray` | `calloc:21`, `malloc:51` | Retain C semantics, including zeroing/wiping, until callers are migrated or exported compatibility requirements are removed. |

## Test-only allocation sites

These are excluded from production counts. Update fixtures when their target contract migrates; keep libc allocations in tests specifically exercising a retained foreign-input contract.

| File | Sites | Purpose / migration |
| --- | --- | --- |
| `src/format/jobs.rs` | `xstrdup:402` | Foreign output handoff fixture; retain if that boundary remains. |
| `src/format/tree.rs` | `xstrdup:402` | Foreign callback value fixture; migrate with callback contract. |
| `src/monitor.rs` | `xstrdup:1375,1389,1402` | Reentrant value replacement/removal fixtures; preserve lifetime regression coverage. |
| `src/paste.rs` | `xmalloc:537`, `xstrdup:577` | Incoming payload/name ownership fixtures; migrate with paste input contract. |
| `tests/arguments_conversion.rs` | `xstrdup:61,354,401,435`; `xcalloc:352,433` | Argument string/value-array fixtures; change alongside batch C. |
| `tests/format_entry_owner.rs` | `xstrdup:15` | Foreign format callback fixture; retain or change with callback boundary. |

## Validation for the eventual migrations

Completion means removing the manual ownership responsibility across the boundary, including the deallocation sites, rather than merely returning a raw pointer from a new owner. Audit every transfer, early return, containing-record initialization, bitwise copy, and callback before implementation. Keep byte identity, first-NUL versus explicit-length behavior, null versus empty, object addresses, existing IDs, notification order, and reference-count lifetime extensions.

Use existing ownership/behavior tests as starting points: `arguments_conversion`, `cmd_list_print_owner`, `cmd_file_owner`, `options_storage`, `prompt_history_owner`, `subjects_utf8`, `ibufq_owner`, `model_message`, `model_prompt`, `model_client`, `model_terminal`, `source_file_pattern`, and the format/paste/monitor inline tests. Add focused regressions for newly changed failure, transfer, and reentrancy paths. Tests cannot establish allocator compatibility; verify it from source.

This report was checked against the direct-call search inventory, including allocator infrastructure and test fixtures. No build or tests were run because only this document changed. Caller-by-caller lifecycle audits and platform-specific execution remain implementation work; the proposed types here do not imply those migrations are already safe to apply mechanically.
