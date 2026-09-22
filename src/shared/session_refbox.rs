//! Trial replacement for `session.windows`, `session.curw`, and `session.lastw`.
//!
//! The ordered map owns each link. The current link and visit history only hold
//! weak references. This is a Rust model of the three fields, not a replacement
//! for the translated `#[repr(C)]` session yet: its raw-pointer call sites and
//! C allocation/free protocol must move together.

use std::collections::{BTreeMap, VecDeque};

use refbox::{BorrowError, RefBox, Weak};

pub struct Winlink {
    pub idx: i32,
}

#[derive(Default)]
pub struct SessionWindows {
    windows: BTreeMap<i32, RefBox<Winlink>>,
    curw: Option<Weak<Winlink>>,
    lastw: VecDeque<Weak<Winlink>>,
}

impl SessionWindows {
    pub fn insert(&mut self, idx: i32) -> bool {
        if self.windows.contains_key(&idx) {
            return false;
        }
        self.windows.insert(idx, RefBox::new(Winlink { idx }));
        true
    }

    pub fn find(&self, idx: i32) -> Option<Weak<Winlink>> {
        self.windows.get(&idx).map(RefBox::downgrade)
    }

    pub fn current(&self) -> Option<&Weak<Winlink>> {
        self.curw.as_ref()
    }

    pub fn select(&mut self, idx: i32) -> bool {
        let Some(next) = self.find(idx) else {
            return false;
        };
        if self.curw.as_ref() == Some(&next) {
            return true;
        }
        self.lastw.retain(|link| link != &next && link.is_alive());
        if let Some(old) = self.curw.take() {
            if old.is_alive() {
                self.lastw.retain(|link| link != &old);
                self.lastw.push_front(old);
            }
        }
        self.curw = Some(next);
        true
    }

    pub fn select_last(&mut self) -> bool {
        while let Some(previous) = self.lastw.pop_front() {
            if previous.is_alive() {
                let idx = match previous.try_access_mut(|link| link.idx) {
                    Ok(idx) => idx,
                    Err(BorrowError::Dropped) => continue,
                    Err(BorrowError::Borrowed) => {
                        self.lastw.push_front(previous);
                        return false;
                    }
                };
                return self.select(idx);
            }
        }
        false
    }

    pub fn remove(&mut self, idx: i32) -> bool {
        let Some(owner) = self.windows.get(&idx) else {
            return false;
        };
        let removed_current = self.curw.as_ref().is_some_and(|link| link.is(owner));
        if removed_current {
            self.curw = None;
        }
        self.lastw.retain(|link| !link.is(owner) && link.is_alive());
        self.windows.remove(&idx);
        if removed_current {
            if !self.select_last() {
                if let Some((&first, _)) = self.windows.first_key_value() {
                    self.select(first);
                }
            }
        }
        true
    }

    pub fn indices(&self) -> impl Iterator<Item = i32> + '_ {
        self.windows.keys().copied()
    }

    pub fn history_indices(&self) -> impl Iterator<Item = i32> + '_ {
        self.lastw
            .iter()
            .filter_map(|link| link.try_borrow_mut().ok().map(|link| link.idx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_and_history_follow_stack_order() {
        let mut session = SessionWindows::default();
        for idx in [1, 2, 3] {
            assert!(session.insert(idx));
        }
        assert!(!session.insert(2));
        assert_eq!(session.indices().collect::<Vec<_>>(), [1, 2, 3]);
        assert!(session.select(1));
        assert!(session.select(2));
        assert!(session.select(3));
        assert_eq!(session.history_indices().collect::<Vec<_>>(), [2, 1]);
        assert!(session.select_last());
        assert_eq!(session.current().unwrap().try_borrow_mut().unwrap().idx, 2);
        assert_eq!(session.history_indices().collect::<Vec<_>>(), [3, 1]);
    }

    #[test]
    fn removing_owner_invalidates_external_weak_reference() {
        let mut session = SessionWindows::default();
        session.insert(1);
        session.insert(2);
        session.select(1);
        session.select(2);
        let old = session.find(2).unwrap();
        assert!(session.remove(2));
        assert_eq!(old.try_borrow_mut().err(), Some(BorrowError::Dropped));
        assert_eq!(session.current().unwrap().try_borrow_mut().unwrap().idx, 1);
        assert!(session.history_indices().next().is_none());
    }

    #[test]
    fn borrow_conflicts_are_reported() {
        let mut session = SessionWindows::default();
        session.insert(1);
        let link = session.find(1).unwrap();
        let borrow = link.try_borrow_mut().unwrap();
        assert_eq!(link.try_borrow_mut().err(), Some(BorrowError::Borrowed));
        drop(borrow);
        assert!(link.try_borrow_mut().is_ok());
    }
}
