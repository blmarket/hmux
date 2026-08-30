use crate::fmt_args;
use crate::tree::GlobalQueue;
pub use crate::types::*;
use crate::text::utf8_stravis;
use crate::xmalloc::xasprintf;
use ::core::cell::UnsafeCell;
use ::core::ffi::{CStr, c_char, c_int, c_longlong};
use ::std::ffi::CString;
use ::std::rc::{Rc, Weak};

/// One set of OSC 8 hyperlinks, as a screen keeps. Each URI and internal id
/// pair is given an inner number, which is what an extended grid cell stores.
#[repr(C)]
pub struct hyperlinks {
    pub next_inner: u_int,
    pub by_inner: hyperlinks_by_inner_tree,
    pub by_uri: hyperlinks_by_uri_tree,
}

/// A strong owner of a screen's hyperlink set. The raw pointer from [`as_ptr`]
/// is only a borrowed compatibility view; the handle must remain alive for
/// every use of that pointer.
#[derive(Clone)]
pub(crate) struct HyperlinksRef(Rc<UnsafeCell<hyperlinks>>);

/// A non-owning observation of a hyperlink set. A link holds its set this
/// way: the global eviction list reaches across sets, and a set that has gone
/// has already taken its links off that list.
#[derive(Clone)]
pub(crate) struct HyperlinksWeak(Weak<UnsafeCell<hyperlinks>>);

impl HyperlinksRef {
    /// Makes an empty hyperlink set with its first inner number ready to use.
    pub(crate) fn new() -> Self {
        Self(Rc::new(UnsafeCell::new(hyperlinks {
            next_inner: 1,
            by_inner: hyperlinks_by_inner_tree::new(),
            by_uri: hyperlinks_by_uri_tree::new(),
        })))
    }

    /// Returns a temporary raw view while this strong handle remains alive.
    pub(crate) fn as_ptr(&self) -> *mut hyperlinks {
        self.0.get()
    }

    /// Removes all links while retaining the set and its inner-number cursor.
    pub(crate) fn reset(&self) {
        unsafe { hyperlinks_reset(self.as_ptr()) };
    }

    /// Makes a non-owning observation of this set.
    pub(crate) fn downgrade(&self) -> HyperlinksWeak {
        HyperlinksWeak(Rc::downgrade(&self.0))
    }
}

impl HyperlinksWeak {
    /// Upgrades the observation if a strong owner still exists.
    pub(crate) fn upgrade(&self) -> Option<HyperlinksRef> {
        self.0.upgrade().map(HyperlinksRef)
    }

    /// The set this observes, named as an address. The set is only there to
    /// be read while a strong owner remains, but the address still tells two
    /// sets apart, which is what the global list matches on while a set is
    /// being dropped.
    fn names(&self) -> *const hyperlinks {
        self.0.as_ptr() as *const hyperlinks
    }
}

impl Drop for hyperlinks {
    fn drop(&mut self) {
        unsafe { hyperlinks_reset(self as *mut hyperlinks) };
    }
}

/// What a link is filed under in [`hyperlinks_by_uri_tree`]. A link with no
/// internal id is anonymous: it is filed under its inner number so that two
/// anonymous links never match, even for the same URI, since a terminal must
/// not tie them together. Named links sort before anonymous ones.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum hyperlinks_uri_key {
    Named(CString, CString),
    Anonymous(u_int),
}

impl hyperlinks_uri_key {
    /// The key for a link with `internal_id`, `uri` and `inner`.
    unsafe fn new(internal_id: *const c_char, uri: *const c_char, inner: u_int) -> Self {
        unsafe {
            if *internal_id == 0 {
                hyperlinks_uri_key::Anonymous(inner)
            } else {
                hyperlinks_uri_key::Named(
                    CStr::from_ptr(internal_id).to_owned(),
                    CStr::from_ptr(uri).to_owned(),
                )
            }
        }
    }

    /// The key `hlu` hangs under.
    unsafe fn of(hlu: *mut hyperlinks_uri) -> Self {
        unsafe {
            hyperlinks_uri_key::new(
                cstr_ptr(&(*hlu).internal_id),
                cstr_ptr(&(*hlu).uri),
                (*hlu).inner,
            )
        }
    }
}

/// The inner number of each of the set's links, by internal id and URI.
/// [`hyperlinks_by_inner_tree`] is what holds the links themselves.
pub type hyperlinks_by_uri_tree = ::std::collections::BTreeMap<hyperlinks_uri_key, u_int>;

/// The set's links by inner number, which is what holds them.
pub type hyperlinks_by_inner_tree = ::std::collections::BTreeMap<u_int, Box<hyperlinks_uri>>;

pub const RB_BLACK: c_int = 0 as c_int;
pub const RB_RED: c_int = 1 as c_int;
pub const RB_NEGINF: c_int = -(1 as c_int);
pub const VIS_OCTAL: c_int = 0x1 as c_int;
pub const VIS_CSTYLE: c_int = 0x2 as c_int;

/// How many links the server keeps across every set before the oldest goes.
pub const MAX_HYPERLINKS: c_int = 5000 as c_int;

static mut hyperlinks_next_external_id: c_longlong = 1 as c_longlong;

/// One link of one set, named the way the global list names them.
#[derive(Clone)]
pub(crate) struct hyperlinks_listed {
    set: HyperlinksWeak,
    inner: u_int,
}

/// Every link of every set, oldest first, which is the order the cap evicts
/// them in.
static global_hyperlinks: GlobalQueue<hyperlinks_listed> = GlobalQueue::new();

/// Takes the link `inner` of `hl` off the global list, if it is on it.
fn hyperlinks_unlist(hl: *const hyperlinks, inner: u_int) {
    let listed = global_hyperlinks.queue();
    if let Some(at) = listed
        .iter()
        .position(|link| link.set.names() == hl && link.inner == inner)
    {
        listed.remove(at);
    }
}

/// Takes the link `inner` off the global list and out of both of `hl`'s
/// trees, and frees it.
unsafe fn hyperlinks_remove(hl: *mut hyperlinks, inner: u_int) {
    unsafe {
        hyperlinks_unlist(hl, inner);
        let Some(hlu) = (*hl).by_inner.get(&inner) else {
            return;
        };
        let key = hyperlinks_uri_key::of(&raw const **hlu as *mut hyperlinks_uri);
        (*hl).by_uri.remove(&key);
        let _ = (*hl).by_inner.remove(&inner);
    }
}

/// Drops the oldest link of any set, which is reached through the set it
/// belongs to. A set that has gone took its links off the list as it went.
unsafe fn hyperlinks_evict(listed: &hyperlinks_listed) {
    unsafe {
        match listed.set.upgrade() {
            Some(hl) => hyperlinks_remove(hl.as_ptr(), listed.inner),
            None => hyperlinks_unlist(listed.set.names(), listed.inner),
        }
    }
}

/// Stores `uri_in` under `internal_id_in` and answers its inner number, or
/// answers the number a link with that id and URI already has. A null or empty
/// id makes an anonymous link, which is never shared. The oldest link of any
/// set goes once the total reaches [`MAX_HYPERLINKS`].
pub(crate) unsafe fn hyperlinks_put(
    owner: &HyperlinksRef,
    uri_in: *const c_char,
    internal_id_in: *const c_char,
) -> u_int {
    unsafe {
        let hl = owner.as_ptr();
        let internal_id_in = if internal_id_in.is_null() {
            c"".as_ptr()
        } else {
            internal_id_in
        };

        let uri = utf8_stravis(uri_in, VIS_OCTAL | VIS_CSTYLE);
        let internal_id = utf8_stravis(internal_id_in, VIS_OCTAL | VIS_CSTYLE);

        if *internal_id_in != 0 {
            let find = hyperlinks_uri_key::new(internal_id.as_ptr(), uri.as_ptr(), 0);
            if let Some(&inner) = (*hl).by_uri.get(&find) {
                return inner;
            }
        }

        let external_id = xasprintf(c"tmux%llX".as_ptr(), fmt_args![hyperlinks_next_external_id]);
        hyperlinks_next_external_id += 1;

        let inner = (*hl).next_inner;
        (*hl).next_inner = (*hl).next_inner.wrapping_add(1);
        let hlu = Box::new(hyperlinks_uri {
            inner,
            internal_id: Some(internal_id),
            external_id: Some(external_id),
            uri: Some(uri),
        });
        let hlu_ptr = &raw const *hlu as *mut hyperlinks_uri;
        (*hl).by_uri.insert(hyperlinks_uri_key::of(hlu_ptr), inner);
        (*hl).by_inner.insert(inner, hlu);

        global_hyperlinks.queue().push_back(hyperlinks_listed {
            set: owner.downgrade(),
            inner,
        });
        if global_hyperlinks.queue().len() == MAX_HYPERLINKS as usize {
            let oldest = global_hyperlinks.queue()[0].clone();
            hyperlinks_evict(&oldest);
        }

        inner
    }
}

/// The link stored under `inner`, as its URI, internal id and external id.
/// A link is put with all three, so none of them is ever absent; the empty
/// string stands in for one that somehow is.
pub fn hyperlinks_get(hl: &hyperlinks, inner: u_int) -> Option<(&CStr, &CStr, &CStr)> {
    let hlu = hl.by_inner.get(&inner)?;
    fn text(s: &Option<CString>) -> &CStr {
        s.as_deref().unwrap_or(c"")
    }
    Some((
        text(&hlu.uri),
        text(&hlu.internal_id),
        text(&hlu.external_id),
    ))
}

/// Frees every link in the set but not the set itself. The inner counter stays
/// where it was, so a number the grid still holds is never handed out again.
unsafe fn hyperlinks_reset(hl: *mut hyperlinks) {
    unsafe {
        let inners: Vec<u_int> = (*hl).by_inner.keys().copied().collect();
        for inner in inners {
            hyperlinks_remove(hl, inner);
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_hyperlinks.rs"]
mod tests;
