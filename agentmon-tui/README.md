# hmux-agentmon

`agentmon` is a reviewable TUI for creating coding-agent runs in sibling git
worktrees and monitoring their real hmux agent status.

## HOWTO

```sh
uv run agentmon
```

See command palette(Ctrl + P) for list of available commands.

## Opinion

### Multi-pane windows

When a window have multiple panes, then I'm assuming user is more interested in
agent state, other than other panes. Thus "Window state" is collapsed to an
agent state it's running.

We can later revisit this decision if we want more support on long running jobs.

No special rules around "multiple agents in a single window" - I hope you're
okay with having a window per agent for monitoring. Don't want to add subtree
in order to show multiple agents under a window.

Also we can revisit if there's a strong case we need to run multiple agents in
a single window.
