# hmux

c2rust translation of tmux

## Format plugins

The `agent` and `git` plugins run by default. They publish pane format variables
in a server-owned registry, separate from user options and the environment.
`#{git_branch}` reads plugin output; `#{@git_branch}` remains a user-controlled
option. A plugin cannot register a built-in name or another plugin's name, or
publish a variable it did not declare.

The agent plugin uses the imported `hmux-agent` detectors for Codex, Claude,
Pi, Agy, and OpenCode. It publishes `pane_agent`, `pane_agent_state`,
`pane_agent_pid`, `pane_agent_session_id`, `pane_agent_model`, and
`pane_state_emoji`. The emoji also classifies ordinary shells and commands.
It refreshes every 200 ms, using the pane's existing output-generation counter
to avoid rereading unchanged screens.

The Git plugin publishes `git_worktree`, `git_worktree_path`, `git_subdir`,
`git_repo`, `git_branch`, `git_head`, `git_action`, `git_action_step`, and
`git_action_total`. It refreshes every 500 ms using Git metadata files, including
linked worktrees and rebase progress. It does not launch Git or report dirty
state. Reftable repositories do not provide a branch or commit through this
file-based reader.

Format expansion reads the last published snapshot. New panes receive the
declared defaults until their first refresh (empty metadata, agent state `none`,
and the no-process emoji). Refresh errors retain the previous snapshot. Missing
panes and omitted values disappear on the next successful refresh. A registered
variable without pane context expands to empty and does not fall back to a
same-named environment variable. Options, built-in formats and explicit
format-context entries retain their existing precedence.

These variables work with normal tmux formatting, including status lines,
`list-panes -F`, `display-message -a`, pane loops, hook monitors, and control-mode
`refresh-client -B` subscriptions. Changed snapshots request status redraws;
subscriptions retain tmux's existing polling cadence.

### Configuration

Set `TMUX_C2RS_PLUGINS` in the environment that starts the server:

```sh
TMUX_C2RS_PLUGINS=agent,git hmux
TMUX_C2RS_PLUGINS=git hmux
TMUX_C2RS_PLUGINS=none hmux
```

An unset variable enables both built-ins; `all` enables every built-in provider.
`none` or an empty value disables all providers. Names are case-insensitive and
comma-separated. Unknown names are ignored, matching hmux. The setting is read
once when a new server starts; changing a client's environment does not change
an existing server.

With any plugin enabled, `window-status-format` and
`window-status-current-format` default to hmux's agent-state/worktree labels,
including model colours and operation progress. These defaults are applied
before `.tmux.conf`, which can override them. Disabling every plugin keeps the
upstream status defaults. Prefix keys and server lifetime defaults are unchanged.

### Adding a provider

Implement `src::plugin::Plugin`: declare `Variable` names and initial values,
choose a positive refresh interval, and fill `PaneValues` from the read-only
`Host` in `refresh`. Add the provider to the built-in list in `plugin::init`, then
enable it by name or with `all`. Providers are compiled-in Rust modules.

Each provider has one owner and one cache. A shared cancellable timer schedules
refreshes; callbacks and provider destructors run outside registry borrows.
Only a successful complete snapshot replaces the previous values. The host
adapts pane storage to weak observations without giving providers raw model
pointers. Screen tails, process data, cwd and cursor state are copied out of
the pane owner. Server shutdown cancels the timer before releasing providers.

Run the focused checks with:

```sh
cargo nextest run -p hmux -E 'test(src::plugin::) or test(src::window_pane::observability::) or binary(plugins)'
cargo nextest run --workspace
```

The parent repository's `make test SUT=hmux` compares the engine against pinned
tmux with plugins disabled by `scripts/hmux-sut.sh`.
