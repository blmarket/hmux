use hmux2::src::shared::abi::timeval;
use hmux2::src::shared::status::message_list;
use std::ffi::CString;

#[test]
fn message_list_owns_entries_trims_oldest_and_reverses_for_display() {
    let mut messages = message_list::new();
    let time = timeval {
        tv_sec: 123,
        tv_usec: 456,
    };

    messages.push_back(CString::new("one").unwrap(), 0, time);
    messages.push_back(CString::new("two").unwrap(), 1, time);
    messages.push_back(CString::new("three").unwrap(), 2, time);
    messages.trim(3, 2);

    let displayed: Vec<_> = messages
        .iter_rev()
        .map(|msg| {
            (
                msg.msg.to_bytes().to_vec(),
                msg.msg_num,
                msg.msg_time.tv_sec,
            )
        })
        .collect();
    assert_eq!(
        displayed,
        [(b"three".to_vec(), 2, 123), (b"two".to_vec(), 1, 123)]
    );

    messages.trim(3, 1);
    let displayed: Vec<_> = messages
        .iter_rev()
        .map(|msg| (msg.msg.to_bytes().to_vec(), msg.msg_num))
        .collect();
    assert_eq!(displayed, [(b"three".to_vec(), 2)]);

    messages.clear();
    assert_eq!(messages.iter_rev().count(), 0);
}
