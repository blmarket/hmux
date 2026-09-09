//! Stable access to systemd bus-error records.

use core::ffi::{CStr, c_int};

/// An owned snapshot of a systemd D-Bus error and its original ownership marker.
pub trait SystemdBusError {
    /// Builds an error snapshot owning copies of its optional strings.
    fn from_systemd_bus_error(
        name: Option<&CStr>,
        message: Option<&CStr>,
        need_free: c_int,
    ) -> Self
    where
        Self: Sized;

    /// Returns the D-Bus error name.
    fn systemd_bus_error_name(&self) -> Option<&CStr>;

    /// Returns the human-readable error message.
    fn systemd_bus_error_message(&self) -> Option<&CStr>;

    /// Returns the recorded systemd ownership marker.
    fn systemd_bus_error_need_free(&self) -> c_int;

    /// Replaces the recorded marker without changing snapshot storage ownership.
    fn set_systemd_bus_error_need_free(&mut self, need_free: c_int);
}

impl SystemdBusError for crate::compat::sd_bus_error {
    fn from_systemd_bus_error(
        name: Option<&CStr>,
        message: Option<&CStr>,
        need_free: c_int,
    ) -> Self {
        Self {
            name: name.map(CStr::to_owned),
            message: message.map(CStr::to_owned),
            _need_free: need_free,
        }
    }

    fn systemd_bus_error_name(&self) -> Option<&CStr> {
        self.name.as_deref()
    }

    fn systemd_bus_error_message(&self) -> Option<&CStr> {
        self.message.as_deref()
    }

    fn systemd_bus_error_need_free(&self) -> c_int {
        self._need_free
    }

    fn set_systemd_bus_error_need_free(&mut self, need_free: c_int) {
        self._need_free = need_free;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compat::sd_bus_error;

    #[test]
    fn snapshot_strings_survive_their_sources_and_marker_changes() {
        let name = c"name".to_owned();
        let message = c"message".to_owned();
        let mut error = sd_bus_error::from_systemd_bus_error(Some(&name), Some(&message), 1);
        drop((name, message));
        error.set_systemd_bus_error_need_free(0);
        assert_eq!(error.systemd_bus_error_name(), Some(c"name"));
        assert_eq!(error.systemd_bus_error_message(), Some(c"message"));
    }

    #[test]
    fn bus_error_exposes_every_field() {
        let mut error = sd_bus_error::from_systemd_bus_error(Some(c"name"), Some(c"message"), 0);
        assert_eq!(error.systemd_bus_error_name(), Some(c"name"));
        assert_eq!(error.systemd_bus_error_message(), Some(c"message"));
        assert_eq!(error.systemd_bus_error_need_free(), 0);
        error.set_systemd_bus_error_need_free(1);
        assert_eq!(error.systemd_bus_error_need_free(), 1);
    }
}
