## Rule

The items here are independent small tasks. First pull the master branch for
the latest task states, find unchecked item, then first mark it + commit +
update master so that you can acquire the item. If conflict happens you failed
to acquire lock. Start over.

Once you acquired the task, go implement it. Once succeeded. Also update this
doc by removing the entry you worked + make `git commit`. But don't push it to
master, so human can review and merge. Agents can only update master only for
modifying `tasks.md` file.

## Tasks

- [x] `args_escape` may return CString
- [x] `xasprintf` may take first argument to be &mut CString or return CString
- [x] `utf8_stravis` may take first argument to be &mut CString or return CString
- [x] `layout_get_tiled_cell` last argument cause should be &mut CString
- [ ] see `sessions` access and see they can be migrated to `session_owners`.
