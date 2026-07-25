# hmux

hmux is a **tmux-compatible** server built for the people who work with coding
agents. All you get is almost tmux, with rich agent integrations as a plus.

It speaks tmux's own wire protocols, with the standard `tmux attach`
intended as the main client communicating with this server.

## What agent control? Why?

It recognize and provide basic agent stuff via tmux placeholders - see
`PROTOCOL.md` for details. Everything is interfaced with tmux wire protocol -
and you're more than welcome to implement your own agent control dashboard.

Agentmon is one possible example of such integrations.

## My take on tmux / cmux / herdr

- tmux is de facto standard of terminal multiplexer - you MUST know to use it
  when you ever ssh into some host.
  - But agent integration can be better with our own addition
- cmux tries to replace your terminal (e.g. Alacritty, Ghostty)
  - Need to replace whole your terminal application stacks with cmux.
- herdr tries to replace tmux with better agent integrations
  - But "agent control" is owned by first party, hard to extend

hmux tries to be a drop-in replacement of tmux - so that you can use it in the
same way in your local / remote in a same way + enjoy the agent integrations.

## See it with agentmon

`agentmon` is the first product built on hmux's agent integration. Its terminal
UI brings agent runs together in one live view, shows which runs need attention,
displays their prompt and transcript context, and switches directly to the
corresponding hmux window. It can also launch new runs in separate Git
worktrees, making parallel agent work easier to start, monitor, revisit, and
clean up.

