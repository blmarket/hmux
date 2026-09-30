//! Authoritative environment declarations.

use std::collections::BTreeMap;

pub const ENVIRON_HIDDEN: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

/// Ordered environment storage. Entries keep their allocation until removed;
/// customization rows use explicit identities rather than these addresses.
#[derive(Default)]
#[repr(C)]
pub struct environ {
    pub(crate) entries: BTreeMap<Vec<u8>, Box<environ_entry>>,
}

impl Clone for environ {
    fn clone(&self) -> Self {
        // A copied environment owns different records. Unlike an entry snapshot,
        // installing this copy must not revive rows from the previous owner.
        Self {
            entries: self
                .entries
                .iter()
                .map(|(name, entry)| {
                    (
                        name.clone(),
                        Box::new(environ_entry::new(
                            entry.name.clone(),
                            entry.flags,
                            entry.value.clone(),
                        )),
                    )
                })
                .collect(),
        }
    }
}

/// Detached entry snapshots preserve the identity of the record they describe.
#[derive(Clone)]
#[repr(C)]
pub struct environ_entry {
    identity: EnvironmentEntryId,
    pub name: std::ffi::CString,
    pub value: Option<std::ffi::CString>,
    pub flags: ::core::ffi::c_int,
}

#[derive(Clone)]
struct EnvironmentEntryId(u64);

impl EnvironmentEntryId {
    fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_ID
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
                // Customization reserves two high bits for the row category and
                // one low bit to distinguish environment and option rows.
                (id < (1 << 61)).then(|| id + 1)
            })
            .expect("environment entry identity exhausted");
        Self(id)
    }
}

impl environ_entry {
    pub(crate) fn new(
        name: std::ffi::CString,
        flags: ::core::ffi::c_int,
        value: Option<std::ffi::CString>,
    ) -> Self {
        Self {
            identity: EnvironmentEntryId::new(),
            name,
            value,
            flags,
        }
    }

    /// Updating or clearing a record preserves its ID; removal retires it.
    pub fn id(&self) -> u64 {
        self.identity.0
    }
}
