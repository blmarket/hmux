//! Stable access to one operating-system password database entry.

use core::ffi::CStr;
use std::ffi::CString;

/// A user account returned by the operating-system password database.
pub trait UserAccount {
    /// Builds an account owning copies of its optional strings.
    fn from_user_account(
        name: Option<&CStr>,
        user_id: u32,
        group_id: u32,
        gecos: Option<&CStr>,
        home: Option<&CStr>,
        shell: Option<&CStr>,
    ) -> Self
    where
        Self: Sized;

    /// Returns the account name.
    fn account_name(&self) -> Option<&CStr>;

    /// Returns the numeric user identifier.
    fn account_user_id(&self) -> u32;

    /// Returns the numeric group identifier.
    fn account_group_id(&self) -> u32;

    /// Returns the user-information field.
    fn account_gecos(&self) -> Option<&CStr>;

    /// Returns the home directory.
    fn account_home(&self) -> Option<&CStr>;

    /// Returns the login shell.
    fn account_shell(&self) -> Option<&CStr>;
}

/// An owned snapshot of an operating-system account record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserAccountRecord {
    name: Option<CString>,
    user_id: u32,
    group_id: u32,
    gecos: Option<CString>,
    home: Option<CString>,
    shell: Option<CString>,
}

impl UserAccountRecord {
    /// Looks up an account by numeric user identifier using caller-owned storage.
    pub fn lookup_uid(uid: u32) -> Option<Self> {
        unsafe {
            Self::lookup(|record, buffer, result| {
                libc::getpwuid_r(
                    uid,
                    record,
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                    result,
                )
            })
        }
    }

    /// Looks up an account by name using caller-owned storage.
    pub fn lookup_name(name: &CStr) -> Option<Self> {
        unsafe {
            Self::lookup(|record, buffer, result| {
                libc::getpwnam_r(
                    name.as_ptr(),
                    record,
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                    result,
                )
            })
        }
    }

    /// # Safety
    ///
    /// A successful reader must set `result` and populate `record` with optional
    /// C strings that remain valid until the caller's buffer is released.
    unsafe fn lookup(
        mut read: impl FnMut(&mut libc::passwd, &mut [u8], &mut *mut libc::passwd) -> i32,
    ) -> Option<Self> {
        let mut buffer = vec![0; 1024];
        loop {
            let mut record: libc::passwd = unsafe { core::mem::zeroed() };
            let mut result = core::ptr::null_mut();
            let status = read(&mut record, &mut buffer, &mut result);
            if status == libc::ERANGE {
                buffer.resize(buffer.len().checked_mul(2)?, 0);
                continue;
            }
            if status != 0 || result.is_null() {
                return None;
            }
            return Some(unsafe {
                Self::from_user_account(
                    (!record.pw_name.is_null()).then(|| CStr::from_ptr(record.pw_name)),
                    record.pw_uid,
                    record.pw_gid,
                    (!record.pw_gecos.is_null()).then(|| CStr::from_ptr(record.pw_gecos)),
                    (!record.pw_dir.is_null()).then(|| CStr::from_ptr(record.pw_dir)),
                    (!record.pw_shell.is_null()).then(|| CStr::from_ptr(record.pw_shell)),
                )
            });
        }
    }
}

impl UserAccount for UserAccountRecord {
    fn from_user_account(
        name: Option<&CStr>,
        user_id: u32,
        group_id: u32,
        gecos: Option<&CStr>,
        home: Option<&CStr>,
        shell: Option<&CStr>,
    ) -> Self {
        Self {
            name: name.map(CStr::to_owned),
            user_id,
            group_id,
            gecos: gecos.map(CStr::to_owned),
            home: home.map(CStr::to_owned),
            shell: shell.map(CStr::to_owned),
        }
    }
    fn account_name(&self) -> Option<&CStr> {
        self.name.as_deref()
    }
    fn account_user_id(&self) -> u32 {
        self.user_id
    }
    fn account_group_id(&self) -> u32 {
        self.group_id
    }
    fn account_gecos(&self) -> Option<&CStr> {
        self.gecos.as_deref()
    }
    fn account_home(&self) -> Option<&CStr> {
        self.home.as_deref()
    }
    fn account_shell(&self) -> Option<&CStr> {
        self.shell.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_owns_its_strings() {
        let source = c"alice".to_owned();
        let account = UserAccountRecord::from_user_account(
            Some(&source),
            1000,
            100,
            None,
            Some(c"/home/alice"),
            Some(c"/bin/sh"),
        );
        drop(source);
        assert_eq!(account.account_name(), Some(c"alice"));
        assert_eq!(account.account_user_id(), 1000);
        assert_eq!(account.account_group_id(), 100);
        assert_eq!(account.account_gecos(), None);
        assert_eq!(account.account_home(), Some(c"/home/alice"));
        assert_eq!(account.account_shell(), Some(c"/bin/sh"));
    }

    #[test]
    fn account_lookups_retain_independent_snapshots() {
        let uid = unsafe { libc::getuid() };
        let account = UserAccountRecord::lookup_uid(uid).expect("current user has an account");
        let name = account.account_name().expect("current account has a name");
        let named =
            UserAccountRecord::lookup_name(name).expect("current account can be found by name");
        assert_eq!(named.account_user_id(), uid);
        assert_eq!(named.account_name(), Some(name));
        let saved = account.clone();
        let _ = UserAccountRecord::lookup_uid(0);
        assert_eq!(account, saved);
    }

    #[test]
    fn lookup_grows_storage_and_copies_before_it_is_released() {
        let mut calls = 0;
        let account = unsafe {
            UserAccountRecord::lookup(|record, buffer, result| {
                calls += 1;
                if buffer.len() < 2048 {
                    return libc::ERANGE;
                }
                buffer[..6].copy_from_slice(b"alice\0");
                record.pw_name = buffer.as_mut_ptr().cast();
                record.pw_uid = 1000;
                *result = record;
                0
            })
        }
        .unwrap();
        assert_eq!(calls, 2);
        assert_eq!(account.account_name(), Some(c"alice"));
        assert_eq!(account.account_user_id(), 1000);
    }

    #[test]
    fn missing_accounts_and_lookup_errors_return_none() {
        assert!(unsafe { UserAccountRecord::lookup(|_, _, _| 0) }.is_none());
        assert!(unsafe { UserAccountRecord::lookup(|_, _, _| libc::EIO) }.is_none());
    }
}
