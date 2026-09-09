//! Numeric identity retained by a session.

use crate::types::u_int;

/// Storage for a session's stable numeric identity.
pub trait SessionIdentity {
    /// Returns the session id.
    fn session_id(&self) -> u_int;

    /// Replaces the session id during construction.
    fn set_session_id(&mut self, id: u_int);
}

/// The session identity storage used by hmux.
#[derive(Default)]
pub struct RustSessionIdentity {
    id: u_int,
}

impl SessionIdentity for RustSessionIdentity {
    fn session_id(&self) -> u_int {
        self.id
    }

    fn set_session_id(&mut self, id: u_int) {
        self.id = id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_can_be_assigned_and_replaced() {
        let mut identity = RustSessionIdentity::default();
        assert_eq!(identity.session_id(), 0);
        identity.set_session_id(42);
        assert_eq!(identity.session_id(), 42);
        identity.set_session_id(u_int::MAX);
        assert_eq!(identity.session_id(), u_int::MAX);
    }
}
