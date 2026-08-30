List all global variables, and see if some has duplicated semantics.

For example, I see session has 2 globals, one for C pointers, and the other for
refcounted owned counter. Ultimately I'd like to see they can be merged with
minimal changes.
