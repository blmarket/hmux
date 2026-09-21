# hmux2

c2rust translation of tmux

See [the current architecture contract](docs/architecture.md). Historical
module-migration details remain in [module responsibilities and migration
validation](docs/module-organization.md).

The argument storage/conversion inventory is in
[docs/arguments-conversion.md](docs/arguments-conversion.md).

The application uses the local `hmux-rt` runtime and independent `hmux-buffer`
storage crate. Building hmux2 does not require libevent. See
[runtime migration and validation](docs/runtime-migration.md) for the compatibility
boundary, regression commands, and measured limitations.
