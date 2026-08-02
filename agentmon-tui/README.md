# hmux-agentmon

`agentmon` is a reviewable TUI for creating coding-agent runs in sibling git
worktrees and monitoring their real hmux agent status.

## HOWTO

```sh
uv run agentmon
```

## Launch commands

Creating a run is split into three composable dashboard commands:

- `n` — prepare a worktree: create a sibling worktree for a new or existing
  branch, optionally resetting an existing branch to the base branch first.
- `i` — set up the instruction: open `$EDITOR` on the worktree's
  `instruction.md` and commit the result. Legacy `prompt.md` files seed the
  editor and are retired on the next commit.
- `l` — launch an agent in the selected worktree: pick Codex or Claude Code
  plus an initial model/effort, with the instruction pre-filled into an
  editable text area. Leaving it empty starts the agent without an initial
  prompt. Within the launch dialog, `a`/`m`/`e` cycle the agent, model, and
  effort pickers and `i` jumps into the instruction box (the letter shortcuts
  are active while the instruction box is not focused).

The original single-flow commands remain available as "Simple run" (`s`, also
in the Ctrl+P command palette) and "Populate from run" (palette only). Registered worktrees without an
hmux window are always listed under their repository, so a freshly prepared
worktree can be targeted by `i` and `l` right away.

Press `u` on the dashboard (or run "Subscription quotas" from the Ctrl+P
command palette) to toggle a
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
