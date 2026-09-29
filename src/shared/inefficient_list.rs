/// Rust does not have efficient intrusive TAILQ implementation, unless using the macros.
/// while deciding possible alternatives, we just use Vec for naive implementation acknowledging
/// inefficient time complexity.
pub struct InefficientList<T> {
    entries: Vec<T>,
}

impl<T> Default for InefficientList<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<T> InefficientList<T> {
    pub(crate) fn first(&self) -> Option<&T> {
        self.entries.first()
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, T> {
        self.entries.iter()
    }

    pub(crate) fn next_after(&self, matches: impl FnMut(&T) -> bool) -> Option<&T> {
        let position = self.entries.iter().position(matches)?;
        self.entries.get(position + 1)
    }

    pub(crate) fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.entries.contains(value)
    }

    pub(crate) fn push_back(&mut self, value: T) {
        self.entries.push(value);
    }

    pub(crate) fn remove_first(&mut self, matches: impl FnMut(&T) -> bool) -> Option<T> {
        let position = self.entries.iter().position(matches)?;
        Some(self.entries.remove(position))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
