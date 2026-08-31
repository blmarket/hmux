I'm looking for a way to ensure bug-to-bug reproduction for smaller modules as
well.

High level idea - for each traits, can we have 2 impls? one from ours, another
from tmux (for simplicity, let's pull tmux master branch)

Thus it need to have snapshot of tmux exported for comparison, some Rust
wrapper to wire our trait to corresponding tmux implementation.

Design such idea at ./plan-mod-tmux.md
