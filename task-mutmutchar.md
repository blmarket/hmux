Some code uses `argc: c_int, argv: *mut *mut c_char` which can be migrated to
`Vec<CString>` type. 

Check such cases and migrate them.
