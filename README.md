# hmux

c2rust translation of tmux

`agentmon` creates and monitors coding-agent runs in git worktrees through hmux:

```sh
nix run .#agentmon
```

Run it inside an hmux pane to discover the current server, or pass
`-- --socket /path/to/socket` from another terminal. `nix develop` also puts
`agentmon` and its companion `looper` on `PATH`.

See [agentmon-tui/README.md](agentmon-tui/README.md) for usage and development.
