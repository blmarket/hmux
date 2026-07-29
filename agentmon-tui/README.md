# hmux-agentmon

`agentmon` is a reviewable TUI for creating coding-agent runs in sibling git
worktrees and monitoring their real hmux agent status.

## HOWTO

```sh
uv run agentmon
```

The dashboard uses a fixed, non-scrolling transcript preview. Space is assigned
to the latest interaction first, then the first interaction, then earlier
interactions working backward from the end; overflowing excerpts end in `…`,
and timestamps use the system's local timezone. Press `t` to open the complete
transcript in `less`; use `/` to search, `n`/`N` to move between matches, and
`q` to return to the dashboard.
