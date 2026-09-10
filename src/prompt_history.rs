use core::ffi::CStr;
use std::ffi::CString;

/// One of tmux's independent prompt histories.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u32)]
pub enum PromptHistoryType {
    #[default]
    Command = 0,
    Search = 1,
    Target = 2,
    WindowTarget = 3,
}

impl PromptHistoryType {
    /// Every prompt-history type in display and persistence order.
    pub const ALL: [Self; 4] = [
        Self::Command,
        Self::Search,
        Self::Target,
        Self::WindowTarget,
    ];

    /// Parses the name accepted by the prompt-history commands and file.
    pub fn parse(name: &CStr) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == name)
    }

    /// Returns the name used by commands and the history file.
    pub const fn name(self) -> &'static CStr {
        match self {
            Self::Command => c"command",
            Self::Search => c"search",
            Self::Target => c"target",
            Self::WindowTarget => c"window-target",
        }
    }
}

/// A store of command, search, target, and window-target prompt histories.
pub trait PromptHistoryStore {
    /// Walks one history from oldest to newest.
    fn entries(&self, kind: PromptHistoryType) -> impl Iterator<Item = &CStr>;

    /// Adds an entry and trims the oldest entries to `limit`.
    fn add(&mut self, kind: PromptHistoryType, line: &CStr, limit: u32);

    /// Moves one step toward older entries.
    fn older<'a>(&'a self, kind: PromptHistoryType, index: &mut u32) -> Option<&'a CStr>;

    /// Moves one step toward newer entries; `None` is the live blank line.
    fn newer<'a>(&'a self, kind: PromptHistoryType, index: &mut u32) -> Option<&'a CStr>;

    /// Removes every entry of one type.
    fn clear(&mut self, kind: PromptHistoryType);

    /// Removes every entry of every type.
    fn clear_all(&mut self);
}

/// The prompt-history implementation used by hmux.
pub struct RustPromptHistoryStore {
    histories: [Vec<CString>; 4],
}

impl RustPromptHistoryStore {
    /// Makes an independent empty store.
    pub fn new() -> Self {
        Self::empty()
    }

    pub(crate) const fn empty() -> Self {
        Self {
            histories: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        }
    }

    fn history(&self, kind: PromptHistoryType) -> &[CString] {
        &self.histories[kind as usize]
    }

    fn history_mut(&mut self, kind: PromptHistoryType) -> &mut Vec<CString> {
        &mut self.histories[kind as usize]
    }
}

impl Default for RustPromptHistoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptHistoryStore for RustPromptHistoryStore {
    fn entries(&self, kind: PromptHistoryType) -> impl Iterator<Item = &CStr> {
        self.history(kind).iter().map(CString::as_c_str)
    }

    fn add(&mut self, kind: PromptHistoryType, line: &CStr, limit: u32) {
        let history = self.history_mut(kind);
        let is_new = history.last().is_none_or(|last| last != line);
        let new_size = (history.len() as u32)
            .wrapping_add(u32::from(is_new))
            .min(limit) as usize;
        let remove = history.len().saturating_add(usize::from(is_new)) - new_size;
        let remove = remove.min(history.len());
        if remove != 0 {
            history.drain(..remove);
        }
        if is_new && new_size != 0 {
            history.push(line.to_owned());
        }
    }

    fn older<'a>(&'a self, kind: PromptHistoryType, index: &mut u32) -> Option<&'a CStr> {
        let history = self.history(kind);
        if history.is_empty() || *index as usize == history.len() {
            return None;
        }
        *index = index.wrapping_add(1);
        Some(history[history.len() - *index as usize].as_c_str())
    }

    fn newer<'a>(&'a self, kind: PromptHistoryType, index: &mut u32) -> Option<&'a CStr> {
        let history = self.history(kind);
        if history.is_empty() || *index == 0 {
            return None;
        }
        *index = index.wrapping_sub(1);
        if *index == 0 {
            return None;
        }
        Some(history[history.len() - *index as usize].as_c_str())
    }

    fn clear(&mut self, kind: PromptHistoryType) {
        self.history_mut(kind).clear();
    }

    fn clear_all(&mut self) {
        for history in &mut self.histories {
            history.clear();
        }
    }
}

const PROMPT_HISTORY_FIELD: crate::server_state::LocalField<
    std::rc::Rc<std::cell::RefCell<RustPromptHistoryStore>>,
> = crate::server_state::LocalField::new(|state| &state.prompt_history);

pub(crate) fn with_prompt_history<R>(read: impl FnOnce(&RustPromptHistoryStore) -> R) -> R {
    let PROMPT_HISTORY = PROMPT_HISTORY_FIELD.get();

    let history = PROMPT_HISTORY.borrow_mut();
    read(&history)
}

pub(crate) fn with_prompt_history_mut<R>(
    mutate: impl FnOnce(&mut RustPromptHistoryStore) -> R,
) -> R {
    let PROMPT_HISTORY = PROMPT_HISTORY_FIELD.get();

    let mut history = PROMPT_HISTORY.borrow_mut();
    mutate(&mut history)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(store: &impl PromptHistoryStore, kind: PromptHistoryType) -> Vec<Vec<u8>> {
        store
            .entries(kind)
            .map(|entry| entry.to_bytes().to_vec())
            .collect()
    }

    #[test]
    fn type_names_round_trip() {
        for kind in PromptHistoryType::ALL {
            assert_eq!(PromptHistoryType::parse(kind.name()), Some(kind));
        }
        assert_eq!(PromptHistoryType::parse(c"invalid"), None);
    }

    #[test]
    fn add_suppresses_duplicates_and_obeys_the_current_limit() {
        let mut store = RustPromptHistoryStore::new();
        store.add(PromptHistoryType::Command, c"discarded", 0);
        assert!(entries(&store, PromptHistoryType::Command).is_empty());

        for line in [c"one", c"one", c"two", c"three", c"four"] {
            store.add(PromptHistoryType::Command, line, 3);
        }
        assert_eq!(
            entries(&store, PromptHistoryType::Command),
            [b"two".as_slice(), b"three".as_slice(), b"four".as_slice()]
        );

        store.add(PromptHistoryType::Command, c"four", 1);
        assert_eq!(
            entries(&store, PromptHistoryType::Command),
            [b"four".as_slice()]
        );
    }

    #[test]
    fn navigation_uses_zero_for_the_live_blank_line() {
        let mut store = RustPromptHistoryStore::new();
        store.add(PromptHistoryType::Search, c"first", 3);
        store.add(PromptHistoryType::Search, c"second", 3);
        let mut index = 0;

        assert_eq!(
            store.older(PromptHistoryType::Search, &mut index),
            Some(c"second")
        );
        assert_eq!(
            store.older(PromptHistoryType::Search, &mut index),
            Some(c"first")
        );
        assert_eq!(store.older(PromptHistoryType::Search, &mut index), None);
        assert_eq!(
            store.newer(PromptHistoryType::Search, &mut index),
            Some(c"second")
        );
        assert_eq!(store.newer(PromptHistoryType::Search, &mut index), None);
        assert_eq!(store.newer(PromptHistoryType::Search, &mut index), None);
    }

    #[test]
    fn histories_are_independent_and_clear_through_the_store() {
        let mut store = RustPromptHistoryStore::new();
        for kind in PromptHistoryType::ALL {
            store.add(kind, kind.name(), 10);
        }
        store.clear(PromptHistoryType::Target);
        for kind in PromptHistoryType::ALL {
            assert_eq!(
                entries(&store, kind).is_empty(),
                kind == PromptHistoryType::Target
            );
        }
        store.clear_all();
        assert!(
            PromptHistoryType::ALL
                .into_iter()
                .all(|kind| entries(&store, kind).is_empty())
        );
    }
}
