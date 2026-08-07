# hmux-agentmon

`agentmon` is a reviewable TUI for creating coding-agent runs in sibling git
worktrees and monitoring their real hmux agent status. `looper` is a headless
companion that re-runs one prompt for as long as quota pacing allows.

## HOWTO

```sh
uv run agentmon
```

See command palette(Ctrl + P) for list of available commands.

## looper

Run it from a pane inside hmux, in the worktree you want worked on:

```sh
cat prompt.md | looper
```

`looper` splits its own window and gives the bottom three quarters to the
agent, keeping the top for its log. Each cycle waits for pacing, starts the
agent on the prompt, ends the run once the agent is done, and commits whatever
the run changed. `nix develop` puts `looper` on `$PATH`; from this directory
`uv run looper` works too.

The preset is hardcoded for now: codex, `gpt-5.6-luna`, effort `max`.

### Pacing

Pacing compares each of the agent provider's quota windows against the clock: a
window is on pace while the share of it spent is no larger than the share of it
elapsed. Over pace, `looper` sleeps until an even burn would have caught up —
for a fully spent window, that is its reset, so an exhausted quota simply means
a long wait rather than a new run. Every window must be on pace before a run
starts, and only the agent's own provider gates it: a spent Claude
subscription has no bearing on a codex loop.

Two deliberate non-blockers: a window the provider left undated cannot be
paced, and a provider that reported nothing at all (expired credentials, say)
leaves nothing to pace against. Both are passed over with a note in the log
rather than stalling the loop forever.

### Ending a run

An interactive agent never exits on its own — "finished" means it is sitting
idle at its prompt — so `looper` is what ends the run, with `/exit` and a
`kill-pane` fallback. An agent that goes `blocked` wants a human instead: the
loop stops there and leaves the pane up for you, without committing.

`--no-commit` leaves the worktree dirty, `-n` caps the number of runs, and
`--run-timeout` ends a run that overstays it. Ctrl-C stops the loop and leaves
any running agent alone.

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
