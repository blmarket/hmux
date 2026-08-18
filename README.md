# hmux

hmux is a **tmux-compatible** server built for the people who work with coding
agents. All you get is almost tmux, with rich agent integrations as a plus.

It speaks tmux's own wire protocols, with the standard `tmux attach`
intended as the main client communicating with this server.

## What agent control? Why?

It recognize and provide basic agent stuff via tmux placeholders, only when the
existing tmux control plane does not support the feature necessary for agent
control. See ./agentmon/ to see an example agent integration.

Recognized agents, and what each pane reports:

| Agent | Program | Lifecycle state | Session id | Model |
|-------|---------|-----------------|------------|-------|
| Codex | `codex` | title + screen | open rollout file | yes |
| Claude Code | `claude` | title + screen | cwd transcript | yes |
| Pi | `pi` | screen | cwd transcript | yes |
| Antigravity CLI | `agy` | screen only | open conversation database | no |

`agy` sets no terminal title at all, so its state comes purely from the screen,
and its conversation lives in a sqlite database instead of a transcript file:
the session id is reported, the model is not. agentmon cannot show a transcript
for an `agy` run for the same reason.

Agents listed as using a "cwd transcript" keep it in a directory named after
their working directory, not after the pane or the process, so several runs
started in one directory share that directory. hmux takes the session id from the agent's own environment
stamp whenever the agent has a live tool subprocess to read it from; between
those, it falls back to the most recently written transcript in that directory
and only accepts a different one after the attributed transcript has stayed
silent for a while and the replacement has grown as the pane worked. A pane that
never runs a tool can therefore report a neighbouring run's session id and
model for a while after switching sessions in place.

## Why not tmux, cmux, or herdr?

- tmux is de facto standard of terminal multiplexer, broadly available.
  - But agent integration can greatly improved with native code, which hmux
    tries to achieve.
- cmux tries to replace your terminal (e.g. Alacritty, Ghostty)
  - Need to replace whole your terminal application stacks with cmux. hmux can
    be a good alternative if you prefer smaller change for agents.
- herdr tries to replace tmux with better agent integrations
  - Hard to co-exist with tmux and limited flexibility on agent control UX. hmux
    can be a good alternative if you prefer to define your own agent control.

## Intentional behavior differences from tmux

- **Some options carry different defaults, or extra values.** Where an agent
  workflow wants a different starting point than tmux's, hmux ships its own
  default; the option itself stays settable the usual way. For example
  `exit-empty` takes a third value, `after-session`, and defaults to it — hmux
  starts with no session, so we keep it alive until the first one is created.
- `hmux` does not have client, so launching it will create daemon and
  immediately exit. You may want to run `tmux attach` to start using it.
- **The first untargeted attach on a fresh daemon creates its session.** Because
  the daemon is started on its own rather than by a client command, an
  `attach-session` with no target on a server that has never held a session
  creates session `0` and attaches to it, where tmux reports `no sessions`. Once
  the server has held a session, both report `no sessions` the same way.
- **Access control is single-user.** `server-access -l` reports the user who owns
  the server, with write access; the `-a`/`-d`/`-r`/`-w` flags validate their
  user argument but keep no per-user ACL, because a server is reachable only
  through a socket its owner owns.
- **A rewrap keeps a scrollback line's timestamp.** `#{top_line_time}` reports
  when the row at the top of a copy-mode view scrolled into the history. hmux's
  emulator rewraps a resized pane one *logical* line at a time, so a logical
  line keeps its stamp on whichever row the new width leaves it starting on.
  tmux rewraps row by row and drops the stamp of any line it has to split, which
  reports such a line as unstamped after a narrowing resize.

## Usage

`nix run` to start hmux server, then `tmux attach`.

Optionally, also start agentmon (either by going into ./agentmon-tui/ then `uv
run agentmon`, or `nix run .#agentmon`) for TUI experience.

## Example

<img width="1273" height="698" alt="current-application-agent" src="https://github.com/user-attachments/assets/bd3d211e-5099-4bc4-b96d-fe97bedaab4d" />
Basic hmux setup - one window running agentmon for orchestration, can have terminal
apps + agents either by agentmon or directly.
