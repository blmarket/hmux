//! Portable identity state for a session group.

use core::ffi::CStr;

/// The name carried by a group of sessions sharing windows.
pub trait SessionGroupState {
    /// Builds group state with an optional name and no engine-owned members.
    fn from_session_group(name: Option<&CStr>) -> Self
    where
        Self: Sized;

    /// Returns the group name, when assigned.
    fn session_group_name(&self) -> Option<&CStr>;

    /// Replaces or removes the group name.
    fn set_session_group_name(&mut self, name: Option<&CStr>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::session_group;

    #[test]
    fn group_name_round_trips_and_updates() {
        let mut state = session_group::from_session_group(Some(c"workers"));
        assert_eq!(state.session_group_name(), Some(c"workers"));
        state.set_session_group_name(Some(c"editors"));
        assert_eq!(state.session_group_name(), Some(c"editors"));
        state.set_session_group_name(None);
        assert_eq!(state.session_group_name(), None);
    }
}
