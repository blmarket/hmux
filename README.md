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

## Intentional behavior differences from tmux

- **Some options carry different defaults, or extra values.** Where an agent
  workflow wants a different starting point than tmux's, hmux ships its own
  default; the option itself stays settable the usual way. For example
  `exit-empty` takes a third value, `after-session`, and defaults to it — hmux
  starts with no session, so we keep it alive until the first one is created.
- `hmux` does not have client, so launching it will create daemon and
  immediately exit. You may want to run `tmux attach` to start using it.
- **A pane's environment starts from the requesting client's.** tmux's server is
  forked from its first client, so its global environment is that client's;
  hmux's daemon starts on its own, so the base for a spawned pane is the
  environment of the client that asked for the spawn. The `set-environment`
  layers and everything tmux sets on top of them — `TERM`, `TERM_PROGRAM`,
  `TERM_PROGRAM_VERSION`, `COLORTERM`, `TMUX`, `TMUX_PANE`, `PATH`, `SHELL`,
  `PWD` — then apply as tmux's do. hmux also always drops the systemd
  socket-activation variables (`LISTEN_PID`, `LISTEN_FDS`, `LISTEN_FDNAMES`),
  which tmux drops only when it was built with systemd support.

## Usage

`nix run` to start hmux server, then `tmux attach`.

Optionally, also start agentmon (either by going into ./agentmon-tui/ then `uv
run agentmon`, or `nix run .#agentmon`) for TUI experience.

## Example

<img width="1273" height="698" alt="current-application-agent" src="https://github.com/user-attachments/assets/bd3d211e-5099-4bc4-b96d-fe97bedaab4d" />
Basic hmux setup - one window running agentmon for orchestration, can have terminal
apps + agents either by agentmon or directly.
