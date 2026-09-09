See ./src/cmd/ for context.

Currently `RustCommandEntry::exec` is unsafe, which I'd like to avoid. Goal is
to remove `unsafe` from exec type.

It has lots of underlying work - and I'd like to see what is the incremental
path to reach that goal. Identify outstanding blockers and suggest which one
can be removed without being blocked.

