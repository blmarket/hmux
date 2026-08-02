# hmux-agentmon

`agentmon` is a reviewable TUI for creating coding-agent runs in sibling git
worktrees and monitoring their real hmux agent status.

## HOWTO

```sh
uv run agentmon
```

Open the command palette (Ctrl+P) and run "Subscription quotas" to toggle a
floating dialog with Codex and Claude quota usage (weekly, 5h, and per-model
windows as reported by each provider). Results are cached for five minutes in
`~/.cache/agentmon/quota.json`; press `r` inside the dialog to force a
refresh. Other features can read the same data through
`agentmon.quota.QuotaService.report()`.

The dashboard uses a fixed, non-scrolling transcript preview. Space is assigned
to the latest interaction first, then the first interaction, then earlier
interactions working backward from the end; overflowing excerpts end in `…`,
and timestamps use the system's local timezone. Press `t` to open the complete
transcript in `less`; use `/` to search, `n`/`N` to move between matches, and
`q` to return to the dashboard.
