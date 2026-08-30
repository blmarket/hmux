Some entities are implemented using their own Refcount implementation. e.g. see
window.references for example.

Wondering we can replace it with Rc/Weak which can be safer alternatives /
later we can get free free.

Check possible migration path at ./plan-refcount.md
