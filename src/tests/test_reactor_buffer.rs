use super::*;
use std::io::Read as _;
use std::os::fd::{FromRawFd, RawFd};

fn socket_pair() -> [RawFd; 2] {
    let mut fds = [0; 2];
    assert_eq!(
        unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) },
        0
    );
    fds
}

#[test]
fn owned_segments_are_kept_without_copying() {
    let owned = Bytes::from_static(b"owned");
    let ptr = owned.as_ptr();
    let mut buf = ByteBuffer::new();
    buf.append_bytes(owned);
    assert_eq!(buf.pullup(5).as_ptr(), ptr);
}

#[test]
fn borrowed_appends_and_cross_segment_consumption_preserve_order() {
    let mut buf = ByteBuffer::new();
    buf.append_bytes(Bytes::from_static(b"one"));
    buf.append_bytes(Bytes::from_static(b"two"));
    buf.append(b"three");
    assert_eq!(buf.copy_to_bytes(11).as_ref(), b"onetwothree");
    assert!(buf.is_empty());
}

#[test]
fn a_slice_shares_owned_segments_without_consuming_the_source() {
    let owned = Bytes::from_static(b"012345");
    let expected = unsafe { owned.as_ptr().add(2) };
    let mut buf = ByteBuffer::new();
    buf.append_bytes(owned);
    let mut slice = buf.slice(2, 3);
    assert_eq!(slice.as_slice(), b"234");
    assert_eq!(slice.as_slice().as_ptr(), expected);
    assert_eq!(buf.as_slice(), b"012345");
}

#[test]
fn split_to_moves_a_prefix_across_segments() {
    let mut buf = ByteBuffer::new();
    buf.append_bytes(Bytes::from_static(b"one"));
    buf.append_bytes(Bytes::from_static(b"two"));
    let mut prefix = buf.split_to(4);
    assert_eq!(prefix.as_slice(), b"onet");
    assert_eq!(buf.as_slice(), b"wo");
}

#[test]
fn mutable_buffer_appends_into_the_tail() {
    let mut buf = ByteBuffer::new();
    buf.put_slice(b"hello");
    assert_eq!(buf.as_slice(), b"hello");
}

#[test]
fn vectored_write_consumes_only_written_bytes() {
    let fds = socket_pair();
    let mut buf = ByteBuffer::new();
    buf.append_bytes(Bytes::from_static(b"one"));
    buf.append_bytes(Bytes::from_static(b"two"));
    assert_eq!(buf.write_to_fd(fds[0]).unwrap(), 6);
    let mut out = [0; 6];
    let mut stream = unsafe { std::os::unix::net::UnixStream::from_raw_fd(fds[1]) };
    stream.read_exact(&mut out).unwrap();
    assert_eq!(&out, b"onetwo");
    unsafe { libc::close(fds[0]) };
}

#[test]
fn read_line_handles_crlf_and_retains_tail() {
    let mut buf = ByteBuffer::new();
    buf.append(b"one\r\ntwo\nrest");
    assert_eq!(buf.read_line().unwrap().as_ref(), b"one");
    assert_eq!(buf.read_line().unwrap().as_ref(), b"two");
    assert_eq!(buf.as_slice(), b"rest");
}

#[test]
fn trait_copies_reject_oversized_lengths_without_consuming_input() {
    for count in [7, usize::MAX] {
        let mut buf = ByteBuffer::new();
        buf.append_bytes(Bytes::from_static(b"one"));
        buf.append(b"two");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            BytesBuf::copy_to_bytes(&mut buf, count)
        }));
        assert!(result.is_err());
        assert_eq!(buf.as_slice(), b"onetwo");
        assert_eq!(buf.copy_to_bytes(count).as_ref(), b"onetwo");
        assert!(buf.is_empty());
    }
}

#[test]
fn fragmented_operations_match_a_contiguous_byte_model() {
    for seed in 1..=32_u64 {
        let mut state = seed;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let mut buf = ByteBuffer::new();
        let mut model = Vec::new();
        for step in 0..512 {
            let operation = next() % 9;
            let count = next() as usize % (model.len() + 1);
            match operation {
                0..=2 => {
                    let bytes: Vec<u8> = (0..next() % 67).map(|_| next() as u8).collect();
                    match operation {
                        0 => buf.append(&bytes),
                        1 => buf.append_bytes(Bytes::copy_from_slice(&bytes)),
                        _ => buf.put_slice(&bytes),
                    }
                    model.extend_from_slice(&bytes);
                }
                3 => {
                    let mut prefix = buf.split_to(count);
                    let expected: Vec<_> = model.drain(..count).collect();
                    assert_eq!(prefix.as_slice(), expected);
                    buf.append_buf(&mut prefix);
                    model.extend_from_slice(&expected);
                    assert!(prefix.is_empty());
                }
                4 => {
                    let size = next() as usize % (model.len() - count + 1);
                    let mut slice = buf.slice(count, size);
                    assert_eq!(slice.as_slice(), &model[count..count + size]);
                    slice.append(b"independent");
                    assert_eq!(&slice.as_slice()[..size], &model[count..count + size]);
                }
                5 => {
                    let bytes = BytesBuf::copy_to_bytes(&mut buf, count);
                    assert_eq!(bytes.as_ref(), &model[..count]);
                    model.drain(..count);
                }
                6 => {
                    let requested = if next() % 2 == 0 { usize::MAX } else { count };
                    buf.drain(requested);
                    model.drain(..requested.min(model.len()));
                }
                7 => {
                    if next() % 2 == 0 {
                        buf.append_bytes(Bytes::from_static(b"\r"));
                        buf.append(b"\n");
                        model.extend_from_slice(b"\r\n");
                    }
                    let expected = model.iter().position(|byte| *byte == b'\n').map(|end| {
                        let line_end = end - usize::from(end > 0 && model[end - 1] == b'\r');
                        let line = model[..line_end].to_vec();
                        model.drain(..=end);
                        line
                    });
                    assert_eq!(buf.read_line().as_deref(), expected.as_deref());
                }
                _ => {
                    let mut snapshot = buf.clone();
                    buf.clear();
                    assert_eq!(snapshot.as_slice(), model);
                    model.clear();
                }
            }
            assert_eq!(buf.len(), model.len(), "seed {seed}, step {step}");
            assert_eq!(buf.is_empty(), model.is_empty());
            assert_eq!(buf.clone().as_slice(), model, "seed {seed}, step {step}");
        }
    }
}

#[test]
fn retained_snapshots_survive_input_mutation_without_consuming_the_buffer() {
    let mut input = ByteBuffer::new();
    assert!(input.snapshot().is_empty());
    input.append_bytes(bytes::Bytes::from_static(b"first"));
    input.append(b"-second");
    let held = input.snapshot();
    assert_eq!(held.as_ref(), b"first-second");
    assert_eq!(input.as_slice(), held.as_ref());
    let another = input.snapshot();
    assert_eq!(held.as_ptr(), another.as_ptr());
    input.drain(6);
    input.append(b"-third");
    assert_eq!(input.as_slice(), b"second-third");
    input.clear();
    input.append(b"replacement");
    drop(input);
    assert_eq!(held.as_ref(), b"first-second");
    assert_eq!(another.as_ref(), b"first-second");
}
