# hmux-agentmon

`agentmon` is a reviewable TUI for creating coding-agent runs in sibling git
worktrees and monitoring their real hmux agent status.

## HOWTO

```sh
uv run agentmon
```

See command palette(Ctrl + P) for list of available commands.

## Multi-pane windows

The dashboard shows one row per hmux window, not per pane. When a window mixes
agent and non-agent panes, the agent pane represents the window (preferring the
active pane on ties), and the row's badge, state, and transcript all come from
that single pane; sibling shells or apps are not listed. Panes whose working
directory is inside a git repository are preferred so the row keeps branch and
worktree context, but that preference never lets an agentless pane outrank an
agent pane — an agent running outside any repository still represents its
window, just without repository details. One caveat remains: a window holding
two agent panes surfaces only one of them.

