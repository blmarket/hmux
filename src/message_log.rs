use crate::types::timeval;
use core::ffi::CStr;
use std::collections::VecDeque;
use std::ffi::CString;

/// The wall-clock time at which a saved server message was recorded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageLogTime {
    /// Whole seconds since the Unix epoch.
    pub seconds: i64,
    /// Microseconds within `seconds`.
    pub microseconds: i64,
}

impl MessageLogTime {
    pub(crate) fn as_timeval(self) -> timeval {
        timeval {
            tv_sec: self.seconds,
            tv_usec: self.microseconds,
        }
    }
}

/// A borrowed observation of one saved server message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageLogEntryRef<'a> {
    /// Exact saved text.
    pub text: &'a CStr,
    /// The wrapping server-wide sequence number.
    pub number: u32,
    /// When the message was saved.
    pub time: MessageLogTime,
}

/// A store of saved server messages.
pub trait MessageLogStore {
    /// Walks retained messages from newest to oldest.
    fn entries(&self) -> impl Iterator<Item = MessageLogEntryRef<'_>>;

    /// Saves exact text, assigns its time and sequence number, and retains at
    /// most `limit` messages.
    fn add(&mut self, text: &CStr, limit: u32);
}

struct RustMessageLogEntry {
    msg: CString,
    msg_num: u32,
    msg_time: MessageLogTime,
}

/// The saved-message implementation used by hmux.
pub struct RustMessageLog {
    entries: VecDeque<RustMessageLogEntry>,
    next: u32,
}

impl RustMessageLog {
    /// Makes an independent empty log.
    pub fn new() -> Self {
        Self::empty()
    }

    pub(crate) const fn empty() -> Self {
        Self {
            entries: VecDeque::new(),
            next: 0,
        }
    }
}

impl Default for RustMessageLog {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageLogStore for RustMessageLog {
    fn entries(&self) -> impl Iterator<Item = MessageLogEntryRef<'_>> {
        self.entries.iter().rev().map(|entry| MessageLogEntryRef {
            text: &entry.msg,
            number: entry.msg_num,
            time: entry.msg_time,
        })
    }

    fn add(&mut self, text: &CStr, limit: u32) {
        let now = timeval::now();
        let number = self.next;
        self.next = self.next.wrapping_add(1);
        self.entries.push_back(RustMessageLogEntry {
            msg: text.to_owned(),
            msg_num: number,
            msg_time: MessageLogTime {
                seconds: now.tv_sec,
                microseconds: now.tv_usec,
            },
        });
        while self
            .entries
            .front()
            .is_some_and(|entry| entry.msg_num.wrapping_add(limit) < self.next)
        {
            self.entries.pop_front();
        }
    }
}

const MESSAGE_LOG_FIELD: crate::server_state::LocalField<
    std::rc::Rc<std::cell::RefCell<RustMessageLog>>,
> = crate::server_state::LocalField::new(|state| &state.message_log);

pub(crate) fn with_message_log<R>(read: impl FnOnce(&RustMessageLog) -> R) -> R {
    let MESSAGE_LOG = MESSAGE_LOG_FIELD.get();

    let log = MESSAGE_LOG.borrow_mut();
    read(&log)
}

pub(crate) fn with_message_log_mut<R>(mutate: impl FnOnce(&mut RustMessageLog) -> R) -> R {
    let MESSAGE_LOG = MESSAGE_LOG_FIELD.get();

    let mut log = MESSAGE_LOG.borrow_mut();
    mutate(&mut log)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(store: &impl MessageLogStore) -> Vec<(Vec<u8>, u32)> {
        store
            .entries()
            .map(|entry| (entry.text.to_bytes().to_vec(), entry.number))
            .collect()
    }

    #[test]
    fn limits_and_numbering_are_observable_through_the_trait() {
        let mut store = RustMessageLog::new();
        store.add(c"discarded", 0);
        assert!(entries(&store).is_empty());
        store.add(c"one", 3);
        store.add(c"two", 3);
        store.add(c"three", 3);
        store.add(c"four", 2);
        assert_eq!(
            entries(&store),
            [(b"four".to_vec(), 4), (b"three".to_vec(), 3)]
        );
    }

    #[test]
    fn stores_are_independent() {
        let mut first = RustMessageLog::new();
        let second = RustMessageLog::new();
        first.add(c"only first", 10);
        assert_eq!(entries(&first), [(b"only first".to_vec(), 0)]);
        assert!(entries(&second).is_empty());
    }
}
