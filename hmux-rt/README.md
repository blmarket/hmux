# hmux-rt

A single-threaded async runtime layer for the hmux daemon: futures, leaves,
and wake plumbing that embed into a host-owned event loop instead of owning
one themselves.

The public surface is runtime capability only — descriptor readiness
(`AsyncFd`), timers (`sleep`), Unix signal delivery (`Signals`), and the task
machinery (`spawn`, `JoinHandle`, `block_on`). Generic dataflow primitives (notifies, completions, racing and
joining futures) need none of that access and live with the daemon that
composes them.
