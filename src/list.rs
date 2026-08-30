use ::std::collections::VecDeque;

/// How a walk reads the `at`th node out of the list it is walking. A list
/// either holds the pointers themselves or owns the nodes in boxes, and it is
/// either a [`Vec`] or a [`VecDeque`], which is all the walk needs to know
/// about it.
pub type At<C, T> = fn(&C, usize) -> Option<*mut T>;

#[allow(clippy::ptr_arg)]
fn listed<T>(list: &Vec<*mut T>, at: usize) -> Option<*mut T> {
    list.get(at).copied()
}

#[allow(clippy::ptr_arg)]
fn owned<T>(list: &Vec<Box<T>>, at: usize) -> Option<*mut T> {
    list.get(at).map(|node| &raw const **node as *mut T)
}

fn queued<T>(list: &VecDeque<*mut T>, at: usize) -> Option<*mut T> {
    list.get(at).copied()
}

fn queued_owned<T>(list: &VecDeque<Box<T>>, at: usize) -> Option<*mut T> {
    list.get(at).map(|node| &raw const **node as *mut T)
}

/// Where `want` sits in `list`, which is wherever the body last left it.
fn position<C, T>(list: &C, node: At<C, T>, want: *mut T) -> Option<usize> {
    let mut at = 0;
    while let Some(found) = node(list, at) {
        if found == want {
            return Some(at);
        }
        at += 1;
    }
    None
}

/// Walks the nodes `head` holds the way C's `TAILQ_FOREACH` did: the
/// successor of the node the body has just had is read out of the list
/// afterwards, so a body that pushes onto the end is walked into what it
/// pushed and one that takes the node out is walked on to whatever followed
/// it. Nothing is copied out of the list, so the walk costs no allocation.
///
/// The list may be added to, reordered or taken from while the walk is in
/// progress: each step finds the node it last handed out by identity, so a
/// move neither repeats nor skips a node. Use [`foreach_safe`] where the C
/// used `TAILQ_FOREACH_SAFE`, that is where the body frees the node it has
/// been given.
///
/// # Safety
///
/// `head` must point to a list that stays alive and unmoved for as long as
/// the walk does.
pub unsafe fn foreach<T>(head: *mut Vec<*mut T>) -> Foreach<Vec<*mut T>, T> {
    Foreach {
        head,
        node: listed,
        at: 0,
        last: None,
    }
}

/// [`foreach`] over a container whose nodes are reached through `node`
/// rather than held as plain pointers.
///
/// # Safety
///
/// As [`foreach`].
pub unsafe fn foreach_by<C, T>(head: *mut C, node: At<C, T>) -> Foreach<C, T> {
    Foreach {
        head,
        node,
        at: 0,
        last: None,
    }
}

/// [`foreach`] over a list that owns its nodes rather than pointing at them.
///
/// # Safety
///
/// As [`foreach`].
pub unsafe fn foreach_owned<T>(head: *mut Vec<Box<T>>) -> Foreach<Vec<Box<T>>, T> {
    Foreach {
        head,
        node: owned,
        at: 0,
        last: None,
    }
}

/// The walk [`foreach`] hands back.
pub struct Foreach<C, T> {
    head: *mut C,
    node: At<C, T>,
    at: usize,
    last: Option<*mut T>,
}

impl<C, T> Iterator for Foreach<C, T> {
    type Item = *mut T;

    fn next(&mut self) -> Option<*mut T> {
        let list = unsafe { &*self.head };
        let node = self.node;
        let at = match self.last {
            None => 0,
            Some(last) if node(list, self.at) == Some(last) => self.at + 1,
            Some(last) => match position(list, node, last) {
                Some(moved) => moved + 1,
                None => self.at,
            },
        };
        let item = node(list, at)?;
        self.at = at;
        self.last = Some(item);
        Some(item)
    }
}

/// Walks the nodes `head` holds the way C's `TAILQ_FOREACH_SAFE` did: the
/// successor is taken before the body runs, so the body may take the node it
/// has been given out of the list and free it. A body that pushes onto the
/// end is not walked into what it pushed, and one that takes the successor
/// out ends the walk, both the way the C did.
///
/// # Safety
///
/// `head` must point to a list that stays alive and unmoved for as long as
/// the walk does.
pub unsafe fn foreach_safe<T>(head: *mut Vec<*mut T>) -> ForeachSafe<Vec<*mut T>, T> {
    ForeachSafe {
        head,
        node: listed,
        at: 0,
        next: None,
        started: false,
    }
}

/// [`foreach_safe`] over a container whose nodes are reached through `node`
/// rather than held as plain pointers.
///
/// # Safety
///
/// As [`foreach_safe`].
pub unsafe fn foreach_safe_by<C, T>(head: *mut C, node: At<C, T>) -> ForeachSafe<C, T> {
    ForeachSafe {
        head,
        node,
        at: 0,
        next: None,
        started: false,
    }
}

/// [`foreach_safe`] over a list that owns its nodes rather than pointing at
/// them.
///
/// # Safety
///
/// As [`foreach_safe`].
pub unsafe fn foreach_owned_safe<T>(head: *mut Vec<Box<T>>) -> ForeachSafe<Vec<Box<T>>, T> {
    ForeachSafe {
        head,
        node: owned,
        at: 0,
        next: None,
        started: false,
    }
}

/// [`foreach_safe`] beginning at the node after `after`, the way the C's
/// hand-rolled walks read `TAILQ_NEXT` first and went on from there. The walk
/// is empty when `after` is not in the list.
///
/// # Safety
///
/// As [`foreach_safe`].
pub unsafe fn foreach_queued_safe_after<T>(
    head: *mut VecDeque<*mut T>,
    after: *mut T,
) -> ForeachSafe<VecDeque<*mut T>, T> {
    unsafe { safe_after(head, queued, after) }
}

/// [`foreach_queued_safe_after`] over a queue that owns its nodes rather than
/// pointing at them.
///
/// # Safety
///
/// As [`foreach_safe`].
pub unsafe fn foreach_queued_owned_safe_after<T>(
    head: *mut VecDeque<Box<T>>,
    after: *mut T,
) -> ForeachSafe<VecDeque<Box<T>>, T> {
    unsafe { safe_after(head, queued_owned, after) }
}

/// [`foreach_safe`] over a container whose nodes are reached through `node`,
/// beginning at the node after `after`.
///
/// # Safety
///
/// As [`foreach_safe`].
pub unsafe fn foreach_safe_after_by<C, T>(
    head: *mut C,
    node: At<C, T>,
    after: *mut T,
) -> ForeachSafe<C, T> {
    unsafe { safe_after(head, node, after) }
}

unsafe fn safe_after<C, T>(head: *mut C, node: At<C, T>, after: *mut T) -> ForeachSafe<C, T> {
    let list = unsafe { &*head };
    let at = position(list, node, after);
    ForeachSafe {
        head,
        node,
        at: at.unwrap_or(0),
        next: at.and_then(|at| node(list, at + 1)),
        started: true,
    }
}

/// The walk [`foreach_safe`] hands back.
pub struct ForeachSafe<C, T> {
    head: *mut C,
    node: At<C, T>,
    at: usize,
    next: Option<*mut T>,
    started: bool,
}

impl<C, T> Iterator for ForeachSafe<C, T> {
    type Item = *mut T;

    fn next(&mut self) -> Option<*mut T> {
        let list = unsafe { &*self.head };
        let node = self.node;
        let at = if !self.started {
            self.started = true;
            0
        } else {
            let want = self.next?;
            if node(list, self.at) == Some(want) {
                self.at
            } else if node(list, self.at + 1) == Some(want) {
                self.at + 1
            } else {
                position(list, node, want)?
            }
        };
        let item = node(list, at)?;
        self.at = at;
        self.next = node(list, at + 1);
        Some(item)
    }
}

#[cfg(test)]
#[path = "tests/test_list.rs"]
mod tests;
