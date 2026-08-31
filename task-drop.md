Currently many entities have their own function to clean up itself, which
should be replaced by Rust `Drop`.

For all `free` calls, check:

1. If it's free of array, can we replace that array to be owned array?
2. If it's free of an object, can we replace that object to be owned object
   either directly or via Rc, and implement proper Drop to clean up instead.

Report current status as ./report-drop.md
