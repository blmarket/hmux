Currently `xasprintf` allocates memory outside of Rust owned alloc path. I'd
like some wrapper which can make owned `CString` to be created instead of raw
pointer.

If necessary, use into_raw to avoid deep reasoning of string lifecycles, we can
tackle it later.
