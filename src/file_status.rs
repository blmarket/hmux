//! Stable access to filesystem metadata in the native stat shape.

/// The meaningful values stored in a native file-status record.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FileStatusValues {
    pub device: u64,
    pub inode: u64,
    pub links: u64,
    pub mode: u32,
    pub user: u32,
    pub group: u32,
    pub special_device: u64,
    pub size: i64,
    pub block_size: i64,
    pub blocks: i64,
    pub accessed: (i64, i64),
    pub modified: (i64, i64),
    pub changed: (i64, i64),
}

/// A native file-status record with implementation padding hidden.
pub trait FileStatus: Default {
    /// Builds a record from all meaningful values.
    fn from_file_status(values: FileStatusValues) -> Self
    where
        Self: Sized;

    /// Returns all meaningful values.
    fn file_status_values(&self) -> FileStatusValues;
}

impl FileStatus for crate::types::stat_t {
    fn from_file_status(values: FileStatusValues) -> Self {
        Self {
            st_dev: values.device,
            st_ino: values.inode,
            st_nlink: values.links,
            st_mode: values.mode,
            st_uid: values.user,
            st_gid: values.group,
            st_rdev: values.special_device,
            st_size: values.size,
            st_blksize: values.block_size,
            st_blocks: values.blocks,
            st_atim: crate::types::timespec {
                tv_sec: values.accessed.0,
                tv_nsec: values.accessed.1,
            },
            st_mtim: crate::types::timespec {
                tv_sec: values.modified.0,
                tv_nsec: values.modified.1,
            },
            st_ctim: crate::types::timespec {
                tv_sec: values.changed.0,
                tv_nsec: values.changed.1,
            },
            ..Self::default()
        }
    }
    fn file_status_values(&self) -> FileStatusValues {
        FileStatusValues {
            device: self.st_dev,
            inode: self.st_ino,
            links: self.st_nlink,
            mode: self.st_mode,
            user: self.st_uid,
            group: self.st_gid,
            special_device: self.st_rdev,
            size: self.st_size,
            block_size: self.st_blksize,
            blocks: self.st_blocks,
            accessed: (self.st_atim.tv_sec, self.st_atim.tv_nsec),
            modified: (self.st_mtim.tv_sec, self.st_mtim.tv_nsec),
            changed: (self.st_ctim.tv_sec, self.st_ctim.tv_nsec),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::stat_t;

    fn values() -> FileStatusValues {
        FileStatusValues {
            device: 1,
            inode: 2,
            links: 3,
            mode: 0o100644,
            user: 4,
            group: 5,
            special_device: 6,
            size: 7,
            block_size: 4096,
            blocks: 8,
            accessed: (9, 10),
            modified: (11, 12),
            changed: (13, 14),
        }
    }

    #[test]
    fn meaningful_values_round_trip() {
        assert_eq!(
            stat_t::from_file_status(values()).file_status_values(),
            values()
        );
    }
}
