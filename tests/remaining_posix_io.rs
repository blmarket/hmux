//! Frozen pre-migration sizes, alignments, and every named field offset.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_copies_match() {
    let mut records = Vec::new();
    macro_rules! record {
        ($label:literal, $ty:ty, [$($field:ident),*]) => {
            records.push(format!("{} {} {} {:?}", $label, size_of::<$ty>(), align_of::<$ty>(),
                &[$(offset_of!($ty, $field)),*] as &[usize]));
        };
    }
    record!(
        "src/cmd_source_file.rs::dirent",
        *mut hmux2::src::cmd_source_file::dirent,
        []
    );
    record!(
        "src/compat/getdtablecount.rs::dirent",
        *mut hmux2::src::compat::getdtablecount::dirent,
        []
    );
    record!(
        "src/cmd_source_file.rs::glob_t",
        hmux2::src::cmd_source_file::glob_t,
        [
            gl_pathc,
            gl_pathv,
            gl_offs,
            gl_flags,
            gl_closedir,
            gl_readdir,
            gl_opendir,
            gl_lstat,
            gl_stat
        ]
    );
    record!(
        "src/compat/getdtablecount.rs::glob_t",
        hmux2::src::compat::getdtablecount::glob_t,
        [
            gl_pathc,
            gl_pathv,
            gl_offs,
            gl_flags,
            gl_closedir,
            gl_readdir,
            gl_opendir,
            gl_lstat,
            gl_stat
        ]
    );
    record!(
        "src/compat/imsg.rs::iovec",
        hmux2::src::compat::imsg::iovec,
        [iov_base, iov_len]
    );
    record!(
        "src/compat/imsg_buffer.rs::iovec",
        hmux2::src::compat::imsg_buffer::iovec,
        [iov_base, iov_len]
    );
    record!(
        "src/cmd_source_file.rs::stat",
        *mut hmux2::src::cmd_source_file::stat,
        []
    );
    record!(
        "src/compat/getdtablecount.rs::stat",
        *mut hmux2::src::compat::getdtablecount::stat,
        []
    );
    record!(
        "src/server.rs::stat",
        hmux2::src::server::stat,
        [
            st_dev,
            st_ino,
            st_nlink,
            st_mode,
            st_uid,
            st_gid,
            __pad0,
            st_rdev,
            st_size,
            st_blksize,
            st_blocks,
            st_atim,
            st_mtim,
            st_ctim,
            __glibc_reserved
        ]
    );
    record!(
        "src/tmux.rs::stat",
        hmux2::src::tmux::stat,
        [
            st_dev,
            st_ino,
            st_nlink,
            st_mode,
            st_uid,
            st_gid,
            __pad0,
            st_rdev,
            st_size,
            st_blksize,
            st_blocks,
            st_atim,
            st_mtim,
            st_ctim,
            __glibc_reserved
        ]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/remaining-posix_io.txt"));
}
