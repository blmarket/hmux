# Pane mode entry migration

`window_pane_modes.storage` owns an ordered vector of
`refbox::RefBox<window_mode_entry>`. The outer Box owns the vector; each
entry has a stable allocation and a Weak identity. Insertion, active-entry
lookup, stack promotion/removal, the complete `window_mode` callback table,
and all mode implementations now exchange `refbox::Weak<window_mode_entry>`.
There are no remaining raw mode-entry parameters or return values.

The copy, clock, and display-panes timers capture weak identities and retain
their existing checked borrow during dispatch. Expired entries skip dispatch;
conflicting checked borrows remain errors. Synchronous translated readers
use the existing unchecked reference accessors under their manually reasoned
owner/dispatch lifetime guarantees. This is an observer API migration, not a
conversion of all unsafe mode code to checked borrowing.

Explicit free callbacks still run with the detached RefBox owner alive.
Screen-source changes occur before cleanup callbacks and are resolved again
after callbacks. Removal expires weak observers when the retained owner and
any active dispatch borrow finish. No new ownership or guard type was added.

Validation: workspace tests cover stack identity through growth and promotion,
removal and weak expiry, parent lifetime throughout free callbacks, and prompt
callbacks that destroy their own mode. The mode-stack tests also pass Valgrind
with zero errors and zero definitely/indirectly lost bytes (48 possibly lost
bytes from the Rust test harness).
