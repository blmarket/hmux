//! Stable value-state capabilities of a session.

use crate::{
    SessionAlertState, SessionAttachmentState, SessionDirectoryState, SessionIdentity,
    SessionNameState, SessionStatusState, SessionTimestampState,
};

/// The stable, independently implementable state of a session.
///
/// Window membership, options, environment, terminal settings, timers, and
/// other server graph resources remain context-specific engine concerns.
pub trait Session:
    SessionIdentity
    + SessionNameState
    + SessionDirectoryState
    + SessionTimestampState
    + SessionStatusState
    + SessionAttachmentState
    + SessionAlertState
{
}

impl<T> Session for T where
    T: SessionIdentity
        + SessionNameState
        + SessionDirectoryState
        + SessionTimestampState
        + SessionStatusState
        + SessionAttachmentState
        + SessionAlertState
{
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_session<T: Session>() {}

    #[test]
    fn server_session_implements_the_aggregate_contract() {
        assert_session::<crate::session::session>();
    }
}
