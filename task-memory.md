Currently code have mixed alloc/deallocs.

Allocation happen by xcalloc (originated from C code), which I'd like to
migrate to owned Box

For shortcuts those migrated Boxes use Box::into_raw - let's ignore them for
now

Deallocation happen by free, which I'd like to migrate to drop of Box -
assuming all allocs are from Box. Sometimes those are coming from raw pointers,
via Box::from_raw

First of all, I'd like to convert all legacy alloc, dealloc to use Box, so that
later we can only track Box::from_raw to identify proper lifecycles.

If it's actually array, we should use Vec instead of Box. In that case do not
migrate.
