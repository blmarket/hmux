See structs / functions are dealing with raw pointers. Wondering we can migrate
to safer field(e.g. owned or lifetime borrowed) where it's safe.

e.g. `copy_of` returns raw pointer, but it can return CString and callers may
use it or call into_raw itself.

find such use cases, and make the migrations where applicable. make `git
commit` per single migration.

