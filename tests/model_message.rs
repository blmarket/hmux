//! Frozen sizes, alignments, and public field offsets for message types.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn migrated_message_collection_layout_matches_fixture() {
    let mut records = Vec::new();
    macro_rules! record {
        ($label:literal, $ty:ty, [$($field:ident),*]) => {
            records.push(format!("{} {} {} {:?}", $label, size_of::<$ty>(), align_of::<$ty>(),
                &[$(offset_of!($ty, $field)),*] as &[usize]));
        };
    }
    record!(
        "src/client.rs::ibuf",
        hmux2::src::client::ibuf,
        [buf, size, max, wpos, rpos, fd]
    );
    record!(
        "src/compat/imsg.rs::ibuf",
        hmux2::src::compat::imsg::ibuf,
        [buf, size, max, wpos, rpos, fd]
    );
    record!(
        "src/compat/imsg_buffer.rs::ibuf",
        hmux2::src::compat::imsg_buffer::ibuf,
        [buf, size, max, wpos, rpos, fd]
    );
    record!(
        "src/file.rs::ibuf",
        hmux2::src::file::ibuf,
        [buf, size, max, wpos, rpos, fd]
    );
    record!(
        "src/proc.rs::ibuf",
        hmux2::src::proc::ibuf,
        [buf, size, max, wpos, rpos, fd]
    );
    record!(
        "src/server_client.rs::ibuf",
        hmux2::src::server_client::ibuf,
        [buf, size, max, wpos, rpos, fd]
    );
    record!(
        "src/compat/imsg.rs::ibufqueue",
        *mut hmux2::src::compat::imsg::ibufqueue,
        []
    );
    record!(
        "src/compat/imsg_buffer.rs::ibufqueue",
        hmux2::src::compat::imsg_buffer::ibufqueue,
        [bufs]
    );
    record!(
        "src/compat/imsg_buffer.rs::C2RustUnnamed_1",
        hmux2::src::compat::imsg_buffer::ibufqueue_bufs,
        []
    );
    record!(
        "src/client.rs::imsg",
        hmux2::src::client::imsg,
        [hdr, data, buf]
    );
    record!(
        "src/compat/imsg.rs::imsg",
        hmux2::src::compat::imsg::imsg,
        [hdr, data, buf]
    );
    record!(
        "src/file.rs::imsg",
        hmux2::src::file::imsg,
        [hdr, data, buf]
    );
    record!(
        "src/proc.rs::imsg",
        hmux2::src::proc::imsg,
        [hdr, data, buf]
    );
    record!(
        "src/server_client.rs::imsg",
        hmux2::src::server_client::imsg,
        [hdr, data, buf]
    );
    record!(
        "src/compat/imsg.rs::imsgbuf",
        hmux2::src::compat::imsg::imsgbuf,
        [w, pid, maxsize, fd, flags]
    );
    record!(
        "src/proc.rs::imsgbuf",
        hmux2::src::proc::imsgbuf,
        [w, pid, maxsize, fd, flags]
    );
    record!(
        "src/compat/imsg.rs::msgbuf",
        *mut hmux2::src::compat::imsg::msgbuf,
        []
    );
    record!(
        "src/compat/imsg_buffer.rs::msgbuf",
        hmux2::src::compat::imsg_buffer::msgbuf,
        [bufs, rbufs, rbuf, rpmsg, readhdr, rarg, roff, hdrsize]
    );
    record!("src/proc.rs::msgbuf", *mut hmux2::src::proc::msgbuf, []);
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-message.txt"));
}
