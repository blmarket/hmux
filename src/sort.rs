//! Sorting for the lists the commands and the tree modes show: a criteria
//! record says which order to put a store's entries in and whether to turn the
//! answer round, and one collector per store walks it into a list held here.
//!
//! The stores themselves are other modules' — the paste buffers, the client
//! list, the session and winlink trees, the key tables — so the collectors
//! keep walking them through those modules' own functions. Paste-buffer keys
//! are copied out of the borrowed store; the other entries are mostly held as
//! cloned values or retained owners. The lists are the module's own, and so is
//! the sorting: the comparisons are not all consistent orders (the key comparison
//! answers *one* for two bindings that are equal), so which sort runs is
//! observable, and [`merge_sort`] below is the one the module is written
//! against.
//!
//! Coverage exemptions: none. Every line of the module is covered by the tests
//! below and by `test_coverage_alpha`.
use crate::pane_identity::PaneIdentity;
use crate::window_dimensions::WindowDimensionsState;
use crate::window_name::WindowNameState;
use crate::window_timestamps::WindowTimestampState;

use crate::key_bindings::{
    key_binding as KeyBinding, key_binding_key, key_binding_tablename, key_tables,
};
use crate::pane_activity::PaneActivityState;
use crate::pane_geometry::PaneGeometryState;
use crate::paste::{PasteBufferStore, with_paste_buffers};
use crate::server::with_clients;
use crate::session::SESSIONS;
pub use crate::types::*;
use crate::window::{WinlinkRef, window_pane_index, window_pane_zindex, winlinks_in};
use ::core::cmp::Ordering;
use ::core::ffi::{CStr, c_int};
use std::ffi::CString;

/// A sorting choice independent of the criteria representation.
pub trait SortCriteria: Sized {
    /// Makes criteria with no configured order cycle.
    fn new(order: sort_order, reversed: bool) -> Self;

    /// Maps a case-insensitive name or alias to an order.
    fn parse_order(name: Option<&CStr>) -> sort_order;

    /// Returns an order's canonical name, or none for the end marker.
    fn order_name(order: sort_order) -> Option<&'static CStr>;

    /// Returns the selected order.
    fn order(&self) -> sort_order;

    /// Selects an order without changing the reverse bit.
    fn set_order(&mut self, order: sort_order);

    /// Returns whether comparisons are reversed.
    fn reversed(&self) -> bool;

    /// Changes the reverse bit without changing the selected order.
    fn set_reversed(&mut self, reversed: bool);

    /// Replaces the sequence used by [`advance`](Self::advance).
    fn set_cycle(&mut self, cycle: &[sort_order]);

    /// Returns whether an order cycle has been configured.
    fn has_cycle(&self) -> bool;

    /// Advances to the next configured order, wrapping at the end.
    fn advance(&mut self);
}

/// The sort-criteria implementation used by hmux.
#[derive(Clone)]
pub struct RustSortCriteria {
    order: sort_order,
    reversed: bool,
    cycle: Option<Vec<sort_order>>,
}

impl Default for RustSortCriteria {
    /// Sorting by activity, forwards, with no explicit order sequence.
    fn default() -> Self {
        Self::new(SORT_ACTIVITY, false)
    }
}
pub use crate::consts::{
    CLIENT_ATTACHED, CLIENT_DEAD, CLIENT_EXIT, CLIENT_SUSPENDED, CLIENT_UNATTACHEDFLAGS,
    KEYC_MASK_MODIFIERS, RB_NEGINF, SORT_ACTIVITY, SORT_CREATION, SORT_END, SORT_INDEX, SORT_NAME,
    SORT_ORDER, SORT_SIZE, SORT_Z,
};

pub const SORT_MODIFIER: sort_order = 3;

/// Every name an order goes by, first name first: reading a name back answers
/// the order it belongs to whatever its case, and printing an order answers
/// the first name written against it here.
static ORDER_NAMES: [(&CStr, sort_order); 10] = [
    (c"activity", SORT_ACTIVITY),
    (c"creation", SORT_CREATION),
    (c"index", SORT_INDEX),
    (c"key", SORT_INDEX),
    (c"modifier", SORT_MODIFIER),
    (c"name", SORT_NAME),
    (c"title", SORT_NAME),
    (c"order", SORT_ORDER),
    (c"size", SORT_SIZE),
    (c"z", SORT_Z),
];

/// Every paste buffer, in the order the store walks them, which is newest
/// first.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SortedPasteBuffer {
    pub(crate) name: CString,
    pub(crate) size: usize,
    pub(crate) order: u_int,
}

/// Every pane of `w`, in the order the window carries them.
fn panes_of(w: &WindowRef) -> impl Iterator<Item = RustWindowPaneWeak> + '_ {
    { w.as_window() }
        .panes
        .iter()
        .map(|pane| pane.downgrade())
        .collect::<Vec<_>>()
        .into_iter()
}

/// Every binding of `table`, in key order.
fn bindings_of(table: &key_table) -> impl Iterator<Item = KeyBinding> + '_ {
    table.bindings().cloned()
}

/// Puts `l` in the order `sort_crit` asks for. The end marker is no order at
/// all, so the list is handed back as the walk built it; sorting by order
/// compares nothing either, since the walk is already in that order, and only
/// turns the list round when the criteria are reversed. Everything else goes
/// to [`merge_sort`] behind `cmp`.
fn sort_list<T: Clone>(
    l: &mut [T],
    mut cmp: impl FnMut(&T, &T, &sort_criteria_t) -> c_int,
    sort_crit: &sort_criteria_t,
) {
    match sort_crit.order() {
        SORT_END => {}
        SORT_ORDER => {
            if sort_crit.reversed() {
                l.reverse();
            }
        }
        _ => merge_sort(l, &mut |a, b| cmp(a, b, sort_crit)),
    }
}

/// The sort the module's answers are written against: split the list in half,
/// sort each half, then merge them taking from the left half for as long as it
/// does not compare greater.
///
/// Which sort runs is observable here, because the comparisons are not all
/// consistent orders — the key comparison answers *one* for two bindings of
/// one table, and both the key and the modifier comparisons cut a `u64` key
/// down to an `int` — and every sort answers such a comparison its own way.
/// This is the sort `qsort` ran for the module before it was written out, so
/// the lists come out as they always did.
fn merge_sort<T: Clone>(l: &mut [T], cmp: &mut impl FnMut(&T, &T) -> c_int) {
    let len = l.len();
    if len <= 1 {
        return;
    }
    let half = len / 2;
    merge_sort(&mut l[..half], cmp);
    merge_sort(&mut l[half..], cmp);

    let left = l[..half].to_vec();
    let (mut a, mut b, mut at) = (0, half, 0);
    while a < left.len() && b < len {
        if cmp(&left[a], &l[b]) <= 0 {
            l[at] = left[a].clone();
            a += 1;
        } else {
            l[at] = l[b].clone();
            b += 1;
        }
        at += 1;
    }
    l[at..at + (left.len() - a)].clone_from_slice(&left[a..]);
}

/// The criteria a comparison works by, as the order to compare in and whether
/// the answer is turned round.
fn criteria(crit: &sort_criteria_t) -> (sort_order, c_int) {
    (crit.order(), c_int::from(crit.reversed()))
}

/// A comparison's answer once the criteria have had their say.
fn settled(result: c_int, reversed: c_int) -> c_int {
    if reversed != 0 {
        result.wrapping_neg()
    } else {
        result
    }
}

/// Two names, compared the way `strcmp` orders the bytes behind them.
fn bytes_cmp(a: &[u8], b: &[u8]) -> c_int {
    match a.cmp(b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

/// Two of the names the server owns outright, ordered the same way. A name
/// the server has not set orders as an empty one: every entry a collector
/// gathers carries its name, so that stands where reading the name as a C
/// string would have handed `strcmp` a null pointer.
fn name_cmp(a: Option<&CStr>, b: Option<&CStr>) -> c_int {
    bytes_cmp(a.map_or(&[], CStr::to_bytes), b.map_or(&[], CStr::to_bytes))
}

/// Whether two bindings sit in one table. This is what the key comparison
/// answers where every other comparison answers how two entries order, so two
/// bindings of a table answer *one* — not the zero that says they are equal —
/// and two bindings of different tables answer zero.
fn same_table(a: Option<&CStr>, b: Option<&CStr>) -> c_int {
    match (a, b) {
        (Some(a), Some(b)) => a.to_bytes().eq_ignore_ascii_case(b.to_bytes()) as c_int,
        (None, None) => 1,
        _ => 0,
    }
}

/// Two times, ordered as the creation orders read them: the older one first,
/// by the seconds unless those are the same and only then by the microseconds
/// inside them. The activity orders read the same times the other way round.
fn by_creation(a: &timeval, b: &timeval) -> c_int {
    match (a.tv_sec, a.tv_usec).cmp(&(b.tv_sec, b.tv_usec)) {
        Ordering::Greater => 1,
        Ordering::Less => -1,
        Ordering::Equal => 0,
    }
}

fn sort_buffer_cmp(
    a0: &SortedPasteBuffer,
    b0: &SortedPasteBuffer,
    crit: &sort_criteria_t,
) -> c_int {
    let (order, reversed) = criteria(crit);
    let mut result = match order {
        SORT_NAME => bytes_cmp(a0.name.to_bytes(), b0.name.to_bytes()),
        SORT_CREATION => match a0.order.cmp(&b0.order) {
            Ordering::Greater => -1,
            Ordering::Less => 1,
            Ordering::Equal => 0,
        },
        SORT_SIZE => a0.size.wrapping_sub(b0.size) as c_int,
        _ => 0,
    };
    if result == 0 {
        result = bytes_cmp(a0.name.to_bytes(), b0.name.to_bytes());
    }
    settled(result, reversed)
}

fn sort_client_cmp(a0: &client, b0: &client, crit: &sort_criteria_t) -> c_int {
    let ca = a0;
    let cb = b0;
    let (order, reversed) = criteria(crit);
    let mut result = match order {
        SORT_NAME => name_cmp(ca.name.as_deref(), cb.name.as_deref()),
        SORT_SIZE => {
            let width = ca.tty.sx.wrapping_sub(cb.tty.sx) as c_int;
            if width == 0 {
                ca.tty.sy.wrapping_sub(cb.tty.sy) as c_int
            } else {
                width
            }
        }
        SORT_CREATION => by_creation(&ca.creation_time, &cb.creation_time),
        SORT_ACTIVITY => -by_creation(&ca.activity_time, &cb.activity_time),
        _ => 0,
    };
    if result == 0 {
        result = name_cmp(ca.name.as_deref(), cb.name.as_deref());
    }
    settled(result, reversed)
}

fn sort_session_cmp(a0: &session, b0: &session, crit: &sort_criteria_t) -> c_int {
    let sa = a0;
    let sb = b0;
    let (order, reversed) = criteria(crit);
    let mut result = match order {
        SORT_INDEX => crate::SessionIdentity::session_id(sa)
            .wrapping_sub(crate::SessionIdentity::session_id(sb)) as c_int,
        SORT_CREATION => by_creation(&sa.creation_time, &sb.creation_time),
        SORT_ACTIVITY => -by_creation(
            &crate::SessionTimestampState::session_timestamps(sa).activity,
            &crate::SessionTimestampState::session_timestamps(sb).activity,
        ),
        SORT_NAME => name_cmp(
            crate::SessionNameState::session_name(sa),
            crate::SessionNameState::session_name(sb),
        ),
        _ => 0,
    };
    if result == 0 {
        result = name_cmp(
            crate::SessionNameState::session_name(sa),
            crate::SessionNameState::session_name(sb),
        );
    }
    settled(result, reversed)
}

fn sort_pane_cmp(
    a_owner: &RustWindowPaneWeak,
    b_owner: &RustWindowPaneWeak,
    crit: &sort_criteria_t,
) -> c_int {
    unsafe {
        let a = a_owner.get().expect("the sorted pane is live");
        let b = b_owner.get().expect("the sorted pane is live");
        let (order, reversed) = criteria(crit);
        let mut result = match order {
            SORT_ACTIVITY => a.activity_point().wrapping_sub(b.activity_point()) as c_int,
            SORT_CREATION => a.pane_id().wrapping_sub(b.pane_id()) as c_int,
            SORT_SIZE => a
                .geometry()
                .width
                .wrapping_mul(a.geometry().height)
                .wrapping_sub(b.geometry().width.wrapping_mul(b.geometry().height))
                as c_int,
            SORT_INDEX => {
                let (_, ai) = a_owner
                    .window()
                    .map_or((-1, 0), |window| window_pane_index(&window.as_window(), a));
                let (_, bi) = b_owner
                    .window()
                    .map_or((-1, 0), |window| window_pane_index(&window.as_window(), b));
                ai.wrapping_sub(bi) as c_int
            }
            SORT_NAME => name_cmp(a.screen_ref().title(), b.screen_ref().title()),
            SORT_Z => {
                let (_, ai) = window_pane_zindex(a_owner);
                let (_, bi) = window_pane_zindex(b_owner);
                ai.wrapping_sub(bi) as c_int
            }
            _ => 0,
        };
        if result == 0 {
            result = name_cmp(a.screen_ref().title(), b.screen_ref().title());
        }
        settled(result, reversed)
    }
}

fn sort_winlink_cmp(a0: &winlink, b0: &winlink, crit: &sort_criteria_t) -> c_int {
    {
        let wla = a0;
        let wlb = b0;
        let wa = wla
            .window_handle()
            .expect("a link has a window")
            .as_window();
        let wb = wlb
            .window_handle()
            .expect("a link has a window")
            .as_window();
        let (order, reversed) = criteria(crit);
        let mut result = match order {
            SORT_INDEX => wla.idx.wrapping_sub(wlb.idx),
            SORT_CREATION => by_creation(&wa.timestamps().creation, &wb.timestamps().creation),
            SORT_ACTIVITY => -by_creation(&wa.timestamps().activity, &wb.timestamps().activity),
            SORT_NAME => name_cmp(wa.window_name(), wb.window_name()),
            SORT_SIZE => wa
                .dimensions()
                .size
                .width
                .wrapping_mul(wa.dimensions().size.height)
                .wrapping_sub(
                    wb.dimensions()
                        .size
                        .width
                        .wrapping_mul(wb.dimensions().size.height),
                ) as c_int,
            _ => 0,
        };
        if result == 0 {
            result = name_cmp(wa.window_name(), wb.window_name());
        }
        settled(result, reversed)
    }
}

fn sort_key_binding_cmp(a0: &key_binding, b0: &key_binding, crit: &sort_criteria_t) -> c_int {
    {
        let (order, reversed) = criteria(crit);
        let tables = || same_table(key_binding_tablename(a0), key_binding_tablename(b0));
        let mut result = match order {
            SORT_INDEX => key_binding_key(a0).wrapping_sub(key_binding_key(b0)) as c_int,
            SORT_MODIFIER => (key_binding_key(a0) & KEYC_MASK_MODIFIERS)
                .wrapping_sub(key_binding_key(b0) & KEYC_MASK_MODIFIERS)
                as c_int,
            SORT_NAME => tables(),
            _ => 0,
        };
        if result == 0 {
            result = tables();
        }
        settled(result, reversed)
    }
}

/// The order that follows `order` in `seq`. The sequence starts again both for
/// its last order and for an order it does not hold at all, and one that holds
/// nothing has no order to answer, so it answers the end marker its only slot
/// carries.
fn next_in(seq: &[sort_order], order: sort_order) -> sort_order {
    let next = match seq.iter().position(|&o| o == order) {
        Some(i) if i + 1 < seq.len() => i + 1,
        _ => 0,
    };
    seq.get(next).copied().unwrap_or(SORT_END)
}

impl SortCriteria for RustSortCriteria {
    fn new(order: sort_order, reversed: bool) -> Self {
        Self {
            order,
            reversed,
            cycle: None,
        }
    }

    fn parse_order(order: Option<&CStr>) -> sort_order {
        let Some(order) = order else {
            return SORT_END;
        };
        let name = order.to_bytes();
        for (text, named) in ORDER_NAMES {
            if name.eq_ignore_ascii_case(text.to_bytes()) {
                return named;
            }
        }
        SORT_END
    }

    fn order_name(order: sort_order) -> Option<&'static CStr> {
        ORDER_NAMES
            .iter()
            .find(|&&(_, named)| named == order)
            .map(|&(text, _)| text)
    }

    fn order(&self) -> sort_order {
        self.order
    }

    fn set_order(&mut self, order: sort_order) {
        self.order = order;
    }

    fn reversed(&self) -> bool {
        self.reversed
    }

    fn set_reversed(&mut self, reversed: bool) {
        self.reversed = reversed;
    }

    fn set_cycle(&mut self, cycle: &[sort_order]) {
        self.cycle = Some(cycle.to_vec());
    }

    fn has_cycle(&self) -> bool {
        self.cycle.is_some()
    }

    fn advance(&mut self) {
        let Some(cycle) = self.cycle.as_deref() else {
            return;
        };
        self.order = next_in(cycle, self.order);
    }
}

pub fn sort_would_window_tree_swap(
    sort_crit: &sort_criteria_t,
    wla: &winlink,
    wlb: &winlink,
) -> c_int {
    if sort_crit.order() == SORT_INDEX {
        return 0;
    }
    (sort_winlink_cmp(wla, wlb, sort_crit) != 0) as c_int
}

pub(crate) fn sort_get_buffers(sort_crit: &sort_criteria_t) -> Vec<SortedPasteBuffer> {
    let mut l = with_paste_buffers(|buffers| {
        buffers
            .buffers()
            .map(|buffer| SortedPasteBuffer {
                name: buffer.name.to_owned(),
                size: buffer.data.len(),
                order: buffer.order,
            })
            .collect::<Vec<_>>()
    });
    sort_list(&mut l, sort_buffer_cmp, sort_crit);
    l
}

pub fn sort_get_clients(sort_crit: &sort_criteria_t) -> Vec<ClientRef> {
    let mut l: Vec<ClientRef> = with_clients(|clients| {
        clients
            .iter()
            .filter(|owner| {
                let c = unsafe { owner.as_client() };
                c.flags & CLIENT_UNATTACHEDFLAGS as uint64_t == 0
                    && c.flags & CLIENT_ATTACHED as uint64_t != 0
            })
            .cloned()
            .collect()
    });
    sort_list(
        &mut l,
        |a, b, crit| unsafe { sort_client_cmp(a.as_client(), b.as_client(), crit) },
        sort_crit,
    );
    l
}

pub fn sort_get_sessions(sort_crit: &sort_criteria_t) -> Vec<SessionRef> {
    let mut l: Vec<SessionRef> = SESSIONS.read().values().cloned().collect();
    sort_list(
        &mut l,
        |a, b, crit| unsafe { sort_session_cmp(a.as_session(), b.as_session(), crit) },
        sort_crit,
    );
    l
}

pub(crate) fn sort_get_winlinks(sort_crit: &sort_criteria_t) -> Vec<WinlinkRef> {
    let mut l: Vec<_> = SESSIONS.read().values().flat_map(winlinks_in).collect();
    sort_list(
        &mut l,
        |a, b, crit| sort_winlink_cmp(a.get().unwrap(), b.get().unwrap(), crit),
        sort_crit,
    );
    l
}

pub unsafe fn sort_get_key_bindings(sort_crit: &sort_criteria_t) -> Vec<KeyBinding> {
    {
        let mut l: Vec<KeyBinding> = Vec::new();
        key_tables.with_borrow(|tables| {
            for table in tables.values() {
                l.extend(bindings_of(&table.borrow()));
            }
        });
        sort_list(&mut l, sort_key_binding_cmp, sort_crit);
        l
    }
}

pub unsafe fn sort_get_key_bindings_table(
    table: &mut key_table,
    sort_crit: &sort_criteria_t,
) -> Vec<KeyBinding> {
    let mut l: Vec<KeyBinding> = bindings_of(&*table).collect();
    sort_list(&mut l, sort_key_binding_cmp, sort_crit);
    l
}

#[cfg(test)]
#[path = "tests/test_sort.rs"]
mod tests;

impl WindowRef {
    pub(crate) unsafe fn sorted_panes(
        &self,
        sort_crit: &sort_criteria_t,
    ) -> Vec<RustWindowPaneWeak> {
        let w = self;

        {
            let mut l: Vec<_> = panes_of(w).collect();
            sort_list(&mut l, sort_pane_cmp, sort_crit);
            l
        }
    }
}

impl SessionRef {
    pub(crate) fn sorted_winlinks(&self, sort_crit: &sort_criteria_t) -> Vec<WinlinkRef> {
        let s = self;

        let mut l: Vec<_> = winlinks_in(s).collect();
        sort_list(
            &mut l,
            |a, b, crit| sort_winlink_cmp(a.get().unwrap(), b.get().unwrap(), crit),
            sort_crit,
        );
        l
    }
}

impl KeyTableRef {
    /// Returns the existing sorted binding snapshot without exposing table storage.
    /// The owned list is needed while commands format and filter the bindings.
    ///
    /// # Safety
    /// Run on the server thread without conflicting key-table access. Sorting
    /// follows the existing criteria and dispatches no callbacks.
    pub(crate) unsafe fn sorted_bindings(&self, criteria: &sort_criteria_t) -> Vec<KeyBinding> {
        unsafe { sort_get_key_bindings_table(&mut self.borrow_mut(), criteria) }
    }
}
