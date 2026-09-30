//! Consumer-side adaptation of the existing push protocol. Producers and their
//! explicit completion/cleanup paths are unchanged. Not polling pauses only
//! consumption: the peer can continue sending and input can continue growing.
use super::*;
use futures_core::Stream;
use hmux_buffer::{Buf, BufMut, SegmentedBuf};
use std::io;
use std::pin::Pin;
use std::task::{Context, LocalWaker, Poll, Waker};

#[derive(Default)]
pub(super) struct ReadState {
    pub(super) active: bool,
    pub(super) input: SegmentedBuf,
    terminal: bool,
    exhausted: bool,
    error: Option<i32>,
    wake: Option<LocalWaker>,
}

/// Read-side polling on the owning local runtime. Exactly one consumer may poll
/// a file: either the existing callback adapter or a direct stream consumer.
/// Callers using Rc<UnsafeCell<_>> must obtain exclusive access for each poll and
/// release it before invoking listeners. Chunks own their bytes across polls.
impl Stream for client_file {
    type Item = io::Result<Vec<u8>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let read = &mut self.get_mut().read;
        if read.exhausted {
            return Poll::Ready(None);
        }
        if !read.active {
            read.exhausted = true;
            return Poll::Ready(Some(Err(io::ErrorKind::InvalidInput.into())));
        }
        if read.input.has_remaining() {
            // Bound each yielded allocation, not total queued input. Keep peer
            // production and buffering semantics compatible with tmux.
            let count = read.input.remaining().min(8192);
            let mut bytes = vec![0; count];
            read.input.copy_to_slice(&mut bytes);
            return Poll::Ready(Some(Ok(bytes)));
        }
        if let Some(error) = read.error.take() {
            read.exhausted = true;
            return Poll::Ready(Some(Err(io::Error::from_raw_os_error(error))));
        }
        if read.terminal {
            read.exhausted = true;
            return Poll::Ready(None);
        }
        read.wake = Some(cx.local_waker().clone());
        Poll::Pending
    }
}

pub(super) fn notify(read: &mut ReadState) {
    if let Some(wake) = read.wake.take() {
        // Wake after the caller releases its model borrow. No listener runs
        // inline from a data append or completion transition.
        defer(move || wake.wake_by_ref());
    }
}

pub(super) fn finish(file: &mut client_file) {
    if file.read.active && !file.read.terminal {
        file.read.terminal = true;
        file.read.error = (file.error != 0).then_some(file.error);
        notify(&mut file.read);
    }
}

/// Preserve legacy callback accumulation (source-file/load-buffer) and dispatch
/// timing. Direct stream consumers retain their input until they poll it.
pub(super) fn collect_for_callback(file: &mut client_file) {
    if !file.read.active {
        return;
    }
    let mut cx = Context::from_waker(Waker::noop());
    while let Poll::Ready(Some(item)) = Pin::new(&mut *file).poll_next(&mut cx) {
        if let Ok(bytes) = item {
            file.buffer.as_mut().put(SegmentedBuf::from(bytes));
        }
        // Legacy callbacks receive the existing file.error at terminal dispatch.
    }
    // This adapter is polled synchronously by the existing message/terminal
    // callbacks. It must not leave a no-op waiter or schedule redundant events.
    file.read.wake = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::task::{ContextBuilder, LocalWake};

    fn read_file() -> client_file {
        let mut file = client_file::empty();
        file.read.active = true;
        file
    }
    struct WakeCount(Cell<usize>);
    impl LocalWake for WakeCount {
        fn wake(self: Rc<Self>) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[test]
    fn pending_registers_latest_local_task_and_wakes_after_append() {
        let mut file = read_file();
        let old = Rc::new(WakeCount(Cell::new(0)));
        let current = Rc::new(WakeCount(Cell::new(0)));
        let old_wake = LocalWaker::from(old.clone());
        let current_wake = LocalWaker::from(current.clone());
        let mut old_cx = ContextBuilder::from_waker(Waker::noop())
            .local_waker(&old_wake)
            .build();
        let mut cx = ContextBuilder::from_waker(Waker::noop())
            .local_waker(&current_wake)
            .build();
        assert!(Pin::new(&mut file).poll_next(&mut old_cx).is_pending());
        assert!(Pin::new(&mut file).poll_next(&mut cx).is_pending());
        file.read.input.put_slice(b"data");
        notify(&mut file.read);
        assert_eq!(current.0.get(), 0, "no inline reentry");
        unsafe {
            crate::src::reactor::poll_runtime();
        }
        assert_eq!(old.0.get(), 0);
        assert_eq!(current.0.get(), 1);
        let Poll::Ready(Some(Ok(bytes))) = Pin::new(&mut file).poll_next(&mut cx) else {
            panic!("data ready");
        };
        assert_eq!(bytes, b"data");
        assert!(Pin::new(&mut file).poll_next(&mut cx).is_pending());
        finish(&mut file);
        unsafe {
            crate::src::reactor::poll_runtime();
        }
        assert_eq!(current.0.get(), 2, "EOF also wakes the reader");
        assert!(matches!(
            Pin::new(&mut file).poll_next(&mut cx),
            Poll::Ready(None)
        ));
        crate::src::reactor::shutdown_runtime();
    }

    #[test]
    fn buffered_data_precedes_error_and_terminal_polls_stay_finished() {
        let mut file = read_file();
        let expected = (0..20000).map(|i| (i % 251) as u8).collect::<Vec<_>>();
        // Producer continues appending while the listener is not polling.
        file.read.input.put_slice(&expected[..10000]);
        file.read.input.put_slice(&expected[10000..]);
        file.error = EIO;
        finish(&mut file);
        let mut received = Vec::new();
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            match Pin::new(&mut file).poll_next(&mut cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    assert!(bytes.len() <= 8192);
                    received.extend(bytes);
                }
                Poll::Ready(Some(Err(error))) => {
                    assert_eq!(error.raw_os_error(), Some(EIO));
                    break;
                }
                _ => panic!("data then error"),
            }
        }
        assert_eq!(received, expected);
        file.read
            .input
            .put_slice(b"late data must not restart the stream");
        assert!(matches!(
            Pin::new(&mut file).poll_next(&mut cx),
            Poll::Ready(None)
        ));
        assert!(matches!(
            Pin::new(&mut file).poll_next(&mut cx),
            Poll::Ready(None)
        ));
    }

    #[test]
    fn completion_retires_index_without_consuming_a_direct_readers_bytes() {
        unsafe {
            let mut files = client_files::default();
            let file = file_create_with_peer(std::ptr::null_mut(), &mut files, 7, None);
            (*file.get()).read.active = true;
            (*file.get()).read.input.put_slice(b"retained");
            file_fire_done(&file);
            crate::src::reactor::poll_runtime();
            assert!(client_files_is_empty(&files));
            let mut cx = Context::from_waker(Waker::noop());
            let Poll::Ready(Some(Ok(bytes))) = Pin::new(&mut *file.get()).poll_next(&mut cx) else {
                panic!("retained data");
            };
            assert_eq!(bytes, b"retained");
            assert!(matches!(
                Pin::new(&mut *file.get()).poll_next(&mut cx),
                Poll::Ready(None)
            ));
            drop(file);
            crate::src::reactor::shutdown_runtime();
        }
    }

    #[test]
    fn legacy_progress_and_completion_keep_accumulation_and_dispatch_timing() {
        unsafe {
            let mut files = client_files::default();
            let progress = Rc::new(Cell::new(0));
            let finished = Rc::new(Cell::new(false));
            let progress_cb = progress.clone();
            let finished_cb = finished.clone();
            let file = file_create_with_peer(
                std::ptr::null_mut(),
                &mut files,
                7,
                Some(Box::new(move |event| {
                    let bytes = evbuffer_pullup(event.buffer.unwrap(), -1).unwrap();
                    if event.closed {
                        assert_eq!(bytes, b"onetwo");
                        assert!(!finished_cb.replace(true));
                    } else {
                        progress_cb.set(progress_cb.get() + 1);
                        assert_eq!(
                            bytes,
                            if progress_cb.get() == 1 {
                                &b"one"[..]
                            } else {
                                &b"onetwo"[..]
                            }
                        );
                    }
                })),
            );
            (*file.get()).read.active = true;
            (*file.get()).read.input.put_slice(b"one");
            file_fire_read(&file);
            assert_eq!(progress.get(), 1);
            (*file.get()).read.input.put_slice(b"two");
            file_fire_read(&file);
            assert_eq!(progress.get(), 2);
            file_fire_done(&file);
            assert!(!finished.get(), "completion remains deferred");
            crate::src::reactor::poll_runtime();
            assert!(finished.get());
            assert!(client_files_is_empty(&files));
            drop(file);
            crate::src::reactor::shutdown_runtime();
        }
    }

    #[test]
    fn dropping_a_pending_next_future_does_not_cancel_or_consume_input() {
        use std::future::{poll_fn, Future};
        let mut file = read_file();
        let mut cx = Context::from_waker(Waker::noop());
        {
            let mut next = Box::pin(poll_fn(|cx| Pin::new(&mut file).poll_next(cx)));
            assert!(next.as_mut().poll(&mut cx).is_pending());
        }
        file.read.input.put_slice(b"later");
        let Poll::Ready(Some(Ok(bytes))) = Pin::new(&mut file).poll_next(&mut cx) else {
            panic!("data retained");
        };
        assert_eq!(bytes, b"later");
        assert!(!file.read.terminal);
        assert_eq!(file.closed, 0);
    }

    #[test]
    fn polling_a_write_reports_invalid_input_without_changing_write_state() {
        let mut file = client_file::empty();
        file.buffer.put_slice(b"output");
        let mut cx = Context::from_waker(Waker::noop());
        let Poll::Ready(Some(Err(error))) = Pin::new(&mut file).poll_next(&mut cx) else {
            panic!("not a reader");
        };
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(matches!(
            Pin::new(&mut file).poll_next(&mut cx),
            Poll::Ready(None)
        ));
        assert_eq!(file.buffer.remaining(), 6);
        assert_eq!(file.closed, 0);
    }
}
