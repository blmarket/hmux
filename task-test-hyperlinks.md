See `hyperlinks` here for context.

I'd like to see we can create test environment for it, against tmux codebase.

Ideally, we should vendor tmux source code, create Rust bindings (like
tmux-sys?), where we can interact with tmux implementations.

After then, there might be some common trait interfacing with hyperlinks, let's
say Hyperlinks, and it defines the behavioral functions.

Some functions might be just placeholders (e.g. reset() is not really necessary
with Rust impl, as we can simply std::mem::take inner in order to free the
memory inside the struct, but still we can have it with no-op just to make
traits compatible with C impl)

After then, we can create various use based scenarios to test its behavior, and
compare both implementation matches.

Let's create this prototype and see it works.

## Some guidances down the road

- tmux internal types may be opaque to us.
- tmux-sys should try to provide tmux functions as-is + necessary scaffolding
  required for supporting unit tests.
  - For example, `screen` allocation API was added to keep the struct opaque.
