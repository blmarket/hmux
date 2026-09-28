This project was originally implemented in C, and there can be some assumptions
which does not align with Rust safety semantics. Specificaly,

1. all object lifetime is logically reasoned manually. e.g. it's possible some
   Weak pointer is always guaranteed to be alive.
2. destruction should be kept logically reasoned. e.g. if there's some
   dedicated free function, we should keep the guarantee it being called until
   we migrate everything to drop function. Also while such manual Drop exists,
   we should not rely on Drop for resource management.
