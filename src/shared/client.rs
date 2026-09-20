//! Client exit statuses and exit reasons shared by translated modules.

pub type client_exit_type = ::core::ffi::c_uint;

pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;

pub type client_exit_reason = ::core::ffi::c_uint;

pub const CLIENT_EXIT_MESSAGE_PROVIDED: client_exit_reason = 8;
pub const CLIENT_EXIT_SERVER_EXITED: client_exit_reason = 7;
pub const CLIENT_EXIT_EXITED: client_exit_reason = 6;
pub const CLIENT_EXIT_LOST_SERVER: client_exit_reason = 5;
pub const CLIENT_EXIT_TERMINATED: client_exit_reason = 4;
pub const CLIENT_EXIT_LOST_TTY: client_exit_reason = 3;
pub const CLIENT_EXIT_DETACHED_HUP: client_exit_reason = 2;
pub const CLIENT_EXIT_DETACHED: client_exit_reason = 1;
pub const CLIENT_EXIT_NONE: client_exit_reason = 0;

#[cfg(test)]
mod tests {
    use super::{
        client_exit_reason, client_exit_type, CLIENT_EXIT_DETACH, CLIENT_EXIT_DETACHED,
        CLIENT_EXIT_MESSAGE_PROVIDED, CLIENT_EXIT_NONE,
    };
    use ::core::mem::{align_of, size_of};

    #[test]
    fn client_exit_domains_match_the_translated_c_baseline() {
        assert_eq!(size_of::<client_exit_type>(), 4);
        assert_eq!(align_of::<client_exit_type>(), 4);
        assert_eq!(size_of::<client_exit_reason>(), 4);
        assert_eq!(align_of::<client_exit_reason>(), 4);
        assert_eq!(CLIENT_EXIT_DETACH, 2);
        assert_eq!(CLIENT_EXIT_MESSAGE_PROVIDED, 8);
        assert_eq!(CLIENT_EXIT_DETACHED, 1);
        assert_eq!(CLIENT_EXIT_NONE, 0);
    }
}
