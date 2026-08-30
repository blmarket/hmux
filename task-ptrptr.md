Some code uses `*mut *mut` as function argument, usually in order to make
function to allocate something and return to caller.

Wondering we can convert those usages to either `&mut Option<Box>` or `&mut
Option<CString>` where callee may can run std::mem::replace to set the value.

1. Check existing code for usage patterns
2. Suggest proper replacement types

Report as ./plan-ptrptr.md sketching migration plan.
