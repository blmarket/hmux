# hmux

hmux is a **tmux-compatible** server built for the people who work with coding
agents. All you get is almost tmux, with rich agent integrations as a plus.

It speaks tmux's own wire protocols, with the standard `tmux attach`
intended as the main client communicating with this server.

## What agent control? Why?

It recognize and provide basic agent stuff via tmux placeholders, only when the
existing tmux control plane does not support the feature necessary for agent
control. See ./agentmon/ to see an example agent integration.

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

## Known behavior differences from tmux

The server runs everything on one loop, which shows through in a few file
operations. These are deliberate and characterized here rather than described as
exact conformance.

- `source-file`, `load-buffer` and `save-buffer` read and write **regular files
  inline**, the way tmux 3.7b reads its own configuration on its loop.
- Those same commands treat a **FIFO** as something to wait for instead of
  blocking the server on `open`. A writer that opens the FIFO and closes it
  without writing anything is indistinguishable from no writer at all, so
  `source-file`/`load-buffer` keep waiting where tmux's blocking open would have
  returned empty.
- `pipe-pane` output is buffered for a pipe command that stops reading, up to a
  few megabytes; past that the **oldest** pane output is dropped rather than
  stalling the pane. tmux queues without bound.

The compositor repaints from the terminal engine's own VT dump, which shows
through in how OSC 8 hyperlinks reach a client:

- **Hyperlinks are sent to every client.** tmux emits OSC 8 only when the
  client's terminal advertises the `hyperlinks` feature; hmux's pane repaint is
  capability-agnostic (like its colours), so terminals without OSC 8 support
  receive and ignore the sequences.
- **Anonymous hyperlinks may split per row.** Every repainted row is
  self-contained, so a link is closed and reopened at row boundaries. Links
  written with an explicit `id=` stay grouped across rows; anonymous links are
  reopened without an id, where tmux synthesizes a `tmux<n>` id to keep a
  multi-row link hoverable as one.

One divergence is deliberate rather than structural:

- **customize-mode lists each user option once.** tmux 3.7b repeats the first
  user option of the global session table in the session section and leaves
  the session-scoped one out; hmux lists each once, in the scope that owns it.

The daemon is started on its own rather than forked from the first client, which
shows through in one place:

- The global environment is seeded from the **daemon's** own, as tmux seeds
  `global_environ` from the environment its server was forked with. Because that
  daemon was not forked from a client, a spawned **pane** then takes the
  environment of the client that asked for it on top of that seed — tmux's panes
  never see the client's environment at all, only what `update-environment`
  copies into the session. Everything else matches: the `set-environment`
  layers, global then session, minus the hidden and removed names, and the
  `TERM`/`TMUX` variables tmux always writes. A **job** — `#()`, `run-shell`,
  `copy-pipe` — gets tmux's environment exactly, with no client layer.

## Usage

`nix run` to start hmux server, then `tmux attach`.

Optionally, also start agentmon (either by going into ./agentmon-tui/ then `uv
run agentmon`, or `nix run .#agentmon`) for TUI experience.

## Example

<img width="1273" height="698" alt="current-application-agent" src="https://github.com/user-attachments/assets/bd3d211e-5099-4bc4-b96d-fe97bedaab4d" />
Basic hmux setup - one window running agentmon for orchestration, can have terminal
apps + agents either by agentmon or directly.
