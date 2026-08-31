Create a new command to setup a new window with envs.

Currently `Ctrl+b c` creates a new window but its env is not consistent.

I'm wondering we can create a new command, or extend existing command which
behave following requirements:

1. it creates a new window at current pane's CWD.
2. it tries to have all env vars same as current pane's ones.

Check what is the requirements, and create ./plan-newwin.md sketching possible
options.
