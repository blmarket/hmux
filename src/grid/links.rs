use crate::fmt_args;
use crate::text::{RustUtf8VisModel, Utf8VisModel};
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use ::core::ffi::{CStr, c_int, c_longlong};
use ::hashlink::LinkedHashMap;
use ::std::cell::RefCell;
use ::std::ffi::CString;
use ::std::rc::{Rc, Weak};

/// A hyperlink store that can be exercised independently of its implementation.
pub trait Hyperlinks: Clone + Sized {
    /// Makes an empty hyperlink store.
    fn new() -> Self;

    /// Stores a URI and optional internal id and returns its inner id.
    fn put(&self, uri: &CStr, internal_id: Option<&CStr>) -> u_int;

    /// Returns the URI, internal id, and external id stored under an inner id.
    fn get(&self, inner: u_int) -> Option<(CString, CString, CString)>;

    /// Removes every hyperlink while retaining the store.
    fn reset(&self);
}

#[repr(C)]
struct hyperlinks_uri {
    inner: u_int,
    internal_id: Option<CString>,
    external_id: Option<CString>,
    uri: Option<CString>,
}

/// One set of OSC 8 hyperlinks, as a screen keeps. Each URI and internal id
/// pair is given an inner number, which is what an extended grid cell stores.
#[repr(C)]
struct hyperlinks {
    id: usize,
    registry: Rc<RefCell<HyperlinkRegistry>>,
    next_inner: u_int,
    by_inner: hyperlinks_by_inner_tree,
    by_uri: hyperlinks_by_uri_tree,
}

/// The Rust hyperlink-store implementation used by hmux.
#[derive(Clone)]
pub struct RustHyperlinks(Rc<RefCell<hyperlinks>>);

/// A non-owning observation of a hyperlink set. A link holds its set this
/// way: the global eviction list reaches across sets, and a set that has gone
/// has already taken its links off that list.
#[derive(Clone)]
pub(crate) struct HyperlinksWeak(Weak<RefCell<hyperlinks>>);

impl RustHyperlinks {
    fn with<R>(&self, read: impl FnOnce(&hyperlinks) -> R) -> R {
        read(&self.0.borrow())
    }

    fn with_mut<R>(&self, mutate: impl FnOnce(&mut hyperlinks) -> R) -> R {
        mutate(&mut self.0.borrow_mut())
    }

    /// Makes a non-owning observation of this set.
    pub(crate) fn downgrade(&self) -> HyperlinksWeak {
        HyperlinksWeak(Rc::downgrade(&self.0))
    }
}

impl Hyperlinks for RustHyperlinks {
    fn new() -> Self {
        let registry = HYPERLINK_REGISTRY.with(Rc::clone);
        let id = {
            let mut state = registry.borrow_mut();
            let id = state.next_set;
            state.next_set = id.checked_add(1).expect("hyperlink set IDs exhausted");
            id
        };
        Self(Rc::new(RefCell::new(hyperlinks {
            id,
            registry,
            next_inner: 1,
            by_inner: hyperlinks_by_inner_tree::new(),
            by_uri: hyperlinks_by_uri_tree::new(),
        })))
    }

    fn put(&self, uri: &CStr, internal_id: Option<&CStr>) -> u_int {
        unsafe { hyperlinks_put(self, uri, internal_id) }
    }

    fn get(&self, inner: u_int) -> Option<(CString, CString, CString)> {
        self.with(|hyperlinks| hyperlinks_get(hyperlinks, inner))
    }

    fn reset(&self) {
        self.with_mut(hyperlinks_reset);
    }
}

impl HyperlinksWeak {
    /// Upgrades the observation if a strong owner still exists.
    fn upgrade(&self) -> Option<RustHyperlinks> {
        self.0.upgrade().map(RustHyperlinks)
    }
}

impl Drop for hyperlinks {
    fn drop(&mut self) {
        hyperlinks_reset(self);
    }
}

/// What a link is filed under in [`hyperlinks_by_uri_tree`]. A link with no
/// internal id is anonymous: it is filed under its inner number so that two
/// anonymous links never match, even for the same URI, since a terminal must
/// not tie them together. Named links sort before anonymous ones.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum hyperlinks_uri_key {
    Named(CString, CString),
    Anonymous(u_int),
}

impl hyperlinks_uri_key {
    /// The key for a link with `internal_id`, `uri` and `inner`.
    fn new(internal_id: &CStr, uri: &CStr, inner: u_int) -> Self {
        if internal_id.to_bytes().is_empty() {
            hyperlinks_uri_key::Anonymous(inner)
        } else {
            hyperlinks_uri_key::Named(internal_id.to_owned(), uri.to_owned())
        }
    }

    /// The key `hlu` hangs under. A link that carries no internal id is
    /// anonymous, which is what an absent one reads as too.
    fn of(hlu: &hyperlinks_uri) -> Self {
        hyperlinks_uri_key::new(
            hlu.internal_id.as_deref().unwrap_or(c""),
            hlu.uri.as_deref().unwrap_or(c""),
            hlu.inner,
        )
    }
}

/// The inner number of each of the set's links, by internal id and URI.
/// [`hyperlinks_by_inner_tree`] is what holds the links themselves.
type hyperlinks_by_uri_tree = std::collections::BTreeMap<hyperlinks_uri_key, u_int>;

/// The set's links by inner number, which is what holds them.
type hyperlinks_by_inner_tree = std::collections::BTreeMap<u_int, Box<hyperlinks_uri>>;

pub(crate) use crate::consts::{VIS_CSTYLE, VIS_OCTAL};

/// How many links the server keeps across every set before the oldest goes.
pub(crate) const MAX_HYPERLINKS: c_int = 5000 as c_int;

/// One link of one set, named the way the global list names them.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct hyperlinks_listed {
    set: usize,
    inner: u_int,
}

/// The eviction order and identities shared by hyperlink sets on one server
/// thread. Sets retain this owner so their cleanup also works after thread-local
/// teardown has released its reference to the registry.
struct HyperlinkRegistry {
    next_set: usize,
    next_external: c_longlong,
    links: LinkedHashMap<hyperlinks_listed, HyperlinksWeak>,
}

thread_local! {
    static HYPERLINK_REGISTRY: Rc<RefCell<HyperlinkRegistry>> = Rc::new(RefCell::new(
        HyperlinkRegistry {
            next_set: 1,
            next_external: 1,
            links: LinkedHashMap::new(),
        }
    ));
}

/// Takes the link `inner` of `hl` off its registry, if it is on it.
fn hyperlinks_unlist(hl: &hyperlinks, inner: u_int) {
    hl.registry
        .borrow_mut()
        .links
        .remove(&hyperlinks_listed { set: hl.id, inner });
}

/// Takes the link `inner` out of both of `hl`'s trees and frees it.
fn hyperlinks_remove_from_set(hl: &mut hyperlinks, inner: u_int) {
    let Some(hlu) = hl.by_inner.get(&inner) else {
        return;
    };
    let key = hyperlinks_uri_key::of(hlu);
    hl.by_uri.remove(&key);
    let _ = hl.by_inner.remove(&inner);
}

/// Drops the oldest link of any set, which is reached through the set it
/// belongs to. A set that has gone took its links off the list as it went.
fn hyperlinks_evict(listed: hyperlinks_listed, set: HyperlinksWeak) {
    if let Some(hl) = set.upgrade() {
        hl.with_mut(|hl| hyperlinks_remove_from_set(hl, listed.inner));
    }
}

/// Stores `uri_in` under `internal_id_in` and answers its inner number, or
/// answers the number a link with that id and URI already has. An absent or
/// empty id makes an anonymous link, which is never shared. The oldest link
/// of any set goes once the total reaches [`MAX_HYPERLINKS`].
unsafe fn hyperlinks_put(
    owner: &RustHyperlinks,
    uri_in: &CStr,
    internal_id_in: Option<&CStr>,
) -> u_int {
    {
        let internal_id_in = internal_id_in.unwrap_or(c"");

        let uri = RustUtf8VisModel.encode_utf8(uri_in.to_bytes(), VIS_OCTAL | VIS_CSTYLE);
        let internal_id =
            RustUtf8VisModel.encode_utf8(internal_id_in.to_bytes(), VIS_OCTAL | VIS_CSTYLE);

        if !internal_id_in.to_bytes().is_empty() {
            let find = hyperlinks_uri_key::new(&internal_id, &uri, 0);
            if let Some(inner) = owner.with(|hl| hl.by_uri.get(&find).copied()) {
                return inner;
            }
        }

        let registry = owner.with(|hl| Rc::clone(&hl.registry));
        let external = {
            let mut state = registry.borrow_mut();
            let external = state.next_external;
            state.next_external = external
                .checked_add(1)
                .expect("hyperlink external IDs exhausted");
            external
        };
        let external_id = xasprintf(c"tmux%llX", fmt_args![external]);

        let (set, inner) = owner.with_mut(|hl| {
            let inner = hl.next_inner;
            hl.next_inner = hl
                .next_inner
                .checked_add(1)
                .expect("hyperlink inner IDs exhausted");
            let hlu = Box::new(hyperlinks_uri {
                inner,
                internal_id: Some(internal_id),
                external_id: Some(external_id),
                uri: Some(uri),
            });
            hl.by_uri.insert(hyperlinks_uri_key::of(&hlu), inner);
            hl.by_inner.insert(inner, hlu);
            (hl.id, inner)
        });

        let listed = hyperlinks_listed { set, inner };
        let oldest = {
            let mut state = registry.borrow_mut();
            let links = &mut state.links;
            assert!(links.replace(listed, owner.downgrade()).is_none());
            (links.len() == MAX_HYPERLINKS as usize).then(|| {
                links
                    .pop_front()
                    .expect("the hyperlink list reached its limit")
            })
        };
        if let Some((oldest, set)) = oldest {
            hyperlinks_evict(oldest, set);
        }

        inner
    }
}

/// The link stored under `inner`, as its URI, internal id and external id.
/// A link is put with all three, so none of them is ever absent; the empty
/// string stands in for one that somehow is.
fn hyperlinks_get(hl: &hyperlinks, inner: u_int) -> Option<(CString, CString, CString)> {
    let hlu = hl.by_inner.get(&inner)?;
    fn text(s: &Option<CString>) -> CString {
        s.as_deref().unwrap_or(c"").to_owned()
    }
    Some((
        text(&hlu.uri),
        text(&hlu.internal_id),
        text(&hlu.external_id),
    ))
}

/// Frees every link in the set but not the set itself. The inner counter stays
/// where it was, so a number the grid still holds is never handed out again.
fn hyperlinks_reset(hl: &mut hyperlinks) {
    for &inner in hl.by_inner.keys() {
        hyperlinks_unlist(hl, inner);
    }
    hl.by_inner.clear();
    hl.by_uri.clear();
}

#[cfg(test)]
#[path = "../tests/test_hyperlinks.rs"]
mod tests;
