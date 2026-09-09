//! Stable access to a client's identifying process metadata.

use std::ffi::CStr;

/// A client's display name, user name, and process identifier.
pub trait ClientIdentity {
    /// Builds client identity state.
    fn from_client_identity(name: Option<&CStr>, user: Option<&CStr>, process_id: i32) -> Self
    where
        Self: Sized;

    /// Returns the client display name.
    fn client_name(&self) -> Option<&CStr>;

    /// Replaces the client display name.
    fn set_client_name(&mut self, name: Option<&CStr>);

    /// Returns the client user name.
    fn client_user(&self) -> Option<&CStr>;

    /// Replaces the client user name.
    fn set_client_user(&mut self, user: Option<&CStr>);

    /// Returns the client process identifier.
    fn client_process_id(&self) -> i32;

    /// Sets the client process identifier.
    fn set_client_process_id(&mut self, process_id: i32);
}

impl ClientIdentity for crate::types::client {
    fn from_client_identity(name: Option<&CStr>, user: Option<&CStr>, process_id: i32) -> Self {
        let mut client = Self::default();
        client.name = name.map(CStr::to_owned);
        client.user = user.map(CStr::to_owned);
        client.pid = process_id;
        client
    }
    fn client_name(&self) -> Option<&CStr> {
        self.name.as_deref()
    }
    fn set_client_name(&mut self, name: Option<&CStr>) {
        self.name = name.map(CStr::to_owned);
    }
    fn client_user(&self) -> Option<&CStr> {
        self.user.as_deref()
    }
    fn set_client_user(&mut self, user: Option<&CStr>) {
        self.user = user.map(CStr::to_owned);
    }
    fn client_process_id(&self) -> i32 {
        self.pid
    }
    fn set_client_process_id(&mut self, process_id: i32) {
        self.pid = process_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::client;

    #[test]
    fn identity_round_trips_and_updates() {
        let mut identity = client::from_client_identity(Some(c"one"), Some(c"alice"), 42);
        assert_eq!(identity.client_name(), Some(c"one"));
        assert_eq!(identity.client_user(), Some(c"alice"));
        assert_eq!(identity.client_process_id(), 42);
        identity.set_client_name(Some(c"two"));
        identity.set_client_user(None);
        identity.set_client_process_id(84);
        assert_eq!(identity.client_name(), Some(c"two"));
        assert_eq!(identity.client_user(), None);
        assert_eq!(identity.client_process_id(), 84);
    }
}
