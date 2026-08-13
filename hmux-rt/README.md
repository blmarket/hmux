# hmux-rt

A single-threaded async runtime layer for the hmux daemon: futures, leaves,
and wake plumbing that embed into a host-owned event loop instead of owning
one themselves.
