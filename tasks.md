## Rule

The items here are independent small tasks. First pull the master branch for the
latest task states, find unchecked item, then first mark it + commit + update
master so that you can acquire the item. If conflict happens you failed to
acquire lock. Start over.

Once you acquired the task, go implement it + make it formatted with
`cargo
fmt`. Once succeeded, update this doc by adding commit hash on the entry
the entry you worked + make `git commit` + try to rebase on top of
origin/master + push it. In case you can't resolve the merge conflict, consider
reset everything and start over from fresh origin/master.

## Tasks

- [x] (d0321528f) window_mode::NONE is never used other than providing default
      value for other `window_mode` values. It's better use `Default` derive.
      After removing NONE, it seems possible to mark some fields non-optional.
      Do it. Init, free, and resize are required; `WindowModeOptional` derives
      `Default` for the remaining optional behavior.
- [x] (e871bdfc3) Some functions are only used by test code. If there's no
      production value let's move those functions to tests module. e.g.
      window_panes_position Moved 77 helpers into test modules; shared helpers
      have test-only re-exports.
- [x] (a1e7aabca, af033e64f) many code in ./src/cmd/ access window_pane by raw
      pointer, which bypass encapsulatino of WindowPane. Make sure no direct
      access to window_pane to happen. Audited all command modules: pane state
      uses WindowPane trait borrows, with no raw pane pointers or concrete pane
      access. Updated stale move documentation.
- [x] (a55660f1c) OwnedWindowPane does not look right - RustWindowPaneRef /
      RustWindowPaneWeak should be the right abstraction like other ...Ref /
      ...Weak pairs. RustWindowPaneRef owns the pane; targets and callbacks use
      RustWindowPaneWeak. Destruction consumes the sole strong owner at the tmux
      3.7b free point.
- [x] (e99e649db) `rg -L '\(.*: &mut client'` in ./src/cmd for context - cmd
      should not have raw client access. Go migrate all of them to use ClientRef
      instead. Command callbacks, lookup, parsing, and queues now accept
      ClientRef. Direct client state access uses scoped handle accessors;
      payload views are limited to calls into existing subsystem APIs.
- [x] (af67cedc9) `cmd_display_menu_exec` - we will panic if tc is none, but the
      check repeats multiple times. Better unwrap at declaration. Check this
      happen in other code (similar expect on an same instance - account small
      mappers which does not affect existability, such as as_ref, as_mut) Menu
      and popup handlers bind required client/session handles at declaration.
      Similar repeated checks in capture, copy mode, buffer display,
      configuration reporting, source-file depth tracking, and queue
      messages/guards/errors now use one binding per scope. Optional-client
      branches retain their behavior.
- [ ] args_parse_type is enum from original tmux, but converted to integer type
      by transpiler. Read tmux enum definition, recover the enum type, and
      convert existing enum usage to use the enum type instead of integer type.
