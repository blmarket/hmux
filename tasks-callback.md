There are callbacks initially migrated from C codebase, where callback function
and arguments are passed separately.

For example, see `popup_editor_t::{cb, arg}`, where arg is always passed to cb.

But as the language was moved to Rust, we can define closures to fold arg to
callback.

Find all similar cases and report ./report-callbacks.md reporting other
callbacks which can be migrated, and any possible blockers for such migrations.
