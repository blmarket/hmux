//! The entities that carry hooks.

use crate::server::command::ExecutableCommand;
use crate::server::options::{
    set_hook_in, unset_hook_in, HookName, OptionSet, PaneHook, SessionHook, WindowHook,
};

use super::{PaneNode, Session, Window};

/// An entity that carries hooks: the catalog of hooks its scope has, and the
/// option table those hooks live in.
///
/// Hooks are options, so this is a facade over the entity's own [`OptionSet`]
/// and not a store of its own. What it adds is the typing: a `Self::Hook` is a
/// catalogued hook of this entity's scope by construction, so attaching one
/// cannot name a hook the entity does not have, and the array rules live in one
/// place instead of at every call site.
///
/// The *effective* bodies a fire needs are deliberately not here. They resolve
/// across the entity's own layer and the global layer behind it — and, for a
/// pane hook, the window layer in between, which belongs to no entity of the
/// pane's own scope. That composition is `OptionsView::array_commands`.
pub(crate) trait Hookable {
    /// The closed catalog of hook names this entity's scope has.
    type Hook: HookName;

    fn hook_store_mut(&mut self) -> &mut OptionSet;

    /// Attach a body to one of this entity's hooks. `append` is `set-option -a`:
    /// without it the hook's array is replaced rather than added to.
    fn set_hook(
        &mut self,
        hook: Self::Hook,
        index: Option<u32>,
        append: bool,
        body: ExecutableCommand,
    ) {
        set_hook_in(self.hook_store_mut(), hook.as_str(), index, append, body);
    }

    /// Remove one member of a hook, or the whole hook when no index names one.
    fn unset_hook(&mut self, hook: Self::Hook, index: Option<u32>) {
        unset_hook_in(self.hook_store_mut(), hook.as_str(), index);
    }
}

impl Hookable for Session {
    type Hook = SessionHook;

    fn hook_store_mut(&mut self) -> &mut OptionSet {
        self.option_overrides_mut()
    }
}

impl Hookable for Window {
    type Hook = WindowHook;

    fn hook_store_mut(&mut self) -> &mut OptionSet {
        self.option_overrides_mut()
    }
}

impl Hookable for PaneNode {
    type Hook = PaneHook;

    fn hook_store_mut(&mut self) -> &mut OptionSet {
        self.option_overrides_mut()
    }
}
