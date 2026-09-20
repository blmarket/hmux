//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: usize = hmux2::src::client::IMSG_HEADER_SIZE;
    records.push(format!(
        "src/client.rs::IMSG_HEADER_SIZE {:?} {} {}",
        v,
        size_of::<usize>(),
        align_of::<usize>()
    ));
    let v: usize = hmux2::src::compat::imsg::IMSG_HEADER_SIZE;
    records.push(format!(
        "src/compat/imsg.rs::IMSG_HEADER_SIZE {:?} {} {}",
        v,
        size_of::<usize>(),
        align_of::<usize>()
    ));
    let v: usize = hmux2::src::file::IMSG_HEADER_SIZE;
    records.push(format!(
        "src/file.rs::IMSG_HEADER_SIZE {:?} {} {}",
        v,
        size_of::<usize>(),
        align_of::<usize>()
    ));
    let v: usize = hmux2::src::server_client::IMSG_HEADER_SIZE;
    records.push(format!(
        "src/server_client.rs::IMSG_HEADER_SIZE {:?} {} {}",
        v,
        size_of::<usize>(),
        align_of::<usize>()
    ));
    let v: usize = hmux2::src::server_fn::IMSG_HEADER_SIZE;
    records.push(format!(
        "src/server_fn.rs::IMSG_HEADER_SIZE {:?} {} {}",
        v,
        size_of::<usize>(),
        align_of::<usize>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::client::MAX_IMSGSIZE;
    records.push(format!(
        "src/client.rs::MAX_IMSGSIZE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::compat::imsg::MAX_IMSGSIZE;
    records.push(format!(
        "src/compat/imsg.rs::MAX_IMSGSIZE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::file::MAX_IMSGSIZE;
    records.push(format!(
        "src/file.rs::MAX_IMSGSIZE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server_fn::MAX_IMSGSIZE;
    records.push(format!(
        "src/server_fn.rs::MAX_IMSGSIZE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::client::PROTOCOL_VERSION;
    records.push(format!(
        "src/client.rs::PROTOCOL_VERSION {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::proc::PROTOCOL_VERSION;
    records.push(format!(
        "src/proc.rs::PROTOCOL_VERSION {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    records.push(format!(
        "src/client.rs::imsg_hdr {} {} type_0={} len={} peerid={} pid={}",
        size_of::<hmux2::src::client::imsg_hdr>(),
        align_of::<hmux2::src::client::imsg_hdr>(),
        offset_of!(hmux2::src::client::imsg_hdr, type_0),
        offset_of!(hmux2::src::client::imsg_hdr, len),
        offset_of!(hmux2::src::client::imsg_hdr, peerid),
        offset_of!(hmux2::src::client::imsg_hdr, pid)
    ));
    records.push(format!(
        "src/compat/imsg.rs::imsg_hdr {} {} type_0={} len={} peerid={} pid={}",
        size_of::<hmux2::src::compat::imsg::imsg_hdr>(),
        align_of::<hmux2::src::compat::imsg::imsg_hdr>(),
        offset_of!(hmux2::src::compat::imsg::imsg_hdr, type_0),
        offset_of!(hmux2::src::compat::imsg::imsg_hdr, len),
        offset_of!(hmux2::src::compat::imsg::imsg_hdr, peerid),
        offset_of!(hmux2::src::compat::imsg::imsg_hdr, pid)
    ));
    records.push(format!(
        "src/file.rs::imsg_hdr {} {} type_0={} len={} peerid={} pid={}",
        size_of::<hmux2::src::file::imsg_hdr>(),
        align_of::<hmux2::src::file::imsg_hdr>(),
        offset_of!(hmux2::src::file::imsg_hdr, type_0),
        offset_of!(hmux2::src::file::imsg_hdr, len),
        offset_of!(hmux2::src::file::imsg_hdr, peerid),
        offset_of!(hmux2::src::file::imsg_hdr, pid)
    ));
    records.push(format!(
        "src/proc.rs::imsg_hdr {} {} type_0={} len={} peerid={} pid={}",
        size_of::<hmux2::src::proc::imsg_hdr>(),
        align_of::<hmux2::src::proc::imsg_hdr>(),
        offset_of!(hmux2::src::proc::imsg_hdr, type_0),
        offset_of!(hmux2::src::proc::imsg_hdr, len),
        offset_of!(hmux2::src::proc::imsg_hdr, peerid),
        offset_of!(hmux2::src::proc::imsg_hdr, pid)
    ));
    records.push(format!(
        "src/server_client.rs::imsg_hdr {} {} type_0={} len={} peerid={} pid={}",
        size_of::<hmux2::src::server_client::imsg_hdr>(),
        align_of::<hmux2::src::server_client::imsg_hdr>(),
        offset_of!(hmux2::src::server_client::imsg_hdr, type_0),
        offset_of!(hmux2::src::server_client::imsg_hdr, len),
        offset_of!(hmux2::src::server_client::imsg_hdr, peerid),
        offset_of!(hmux2::src::server_client::imsg_hdr, pid)
    ));
    records.push(format!(
        "src/server_fn.rs::imsg_hdr {} {} type_0={} len={} peerid={} pid={}",
        size_of::<hmux2::src::server_fn::imsg_hdr>(),
        align_of::<hmux2::src::server_fn::imsg_hdr>(),
        offset_of!(hmux2::src::server_fn::imsg_hdr, type_0),
        offset_of!(hmux2::src::server_fn::imsg_hdr, len),
        offset_of!(hmux2::src::server_fn::imsg_hdr, peerid),
        offset_of!(hmux2::src::server_fn::imsg_hdr, pid)
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-message.txt"));
}
