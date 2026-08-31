//! Sorting for the lists the commands and the tree modes show: a criteria
//! record says which order to put a store's entries in and whether to turn the
//! answer round, and one collector per store walks it into a list held here.
//!
//! The stores themselves are other modules' — the paste buffers, the client
//! list, the session and winlink trees, the key tables — so the collectors
//! keep walking them through those modules' own functions and hold what they
//! find as raw pointers. The lists are the module's own, and so is the
//! sorting: the comparisons are not all consistent orders (the key comparison
//! answers *one* for two bindings that are equal), so which sort runs is
//! observable, and [`merge_sort`] below is the one the module is written
//! against.
//!
//! Coverage exemptions: none. Every line of the module is covered by the tests
//! below and by `test_coverage_alpha`.
use crate::key_bindings::{
    key_binding_key, key_binding_tablename, key_bindings_first, key_bindings_first_table,
    key_bindings_next, key_bindings_next_table,
};
use crate::paste::{paste_buffer_data, paste_buffer_name, paste_buffer_order, paste_walk};
use crate::server::client_walk;
use crate::session::{session_activity_time, session_id, session_name, session_owners};
pub use crate::types::*;
use crate::window::{
    window_pane_index, window_pane_zindex, window_panes_first, window_panes_next, winlinks_after,
    winlinks_first,
};
use ::core::cmp::Ordering;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::iter::successors;
use ::core::ptr::null_mut;
use ::std::ffi::CString;
pub const KEYC_MASK_MODIFIERS: ::core::ffi::c_ulonglong =
    0xff0000000000 as ::core::ffi::c_ulonglong;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const SORT_END: sort_order = 8;
pub const SORT_Z: sort_order = 7;
pub const SORT_SIZE: sort_order = 6;
pub const SORT_ORDER: sort_order = 5;
pub const SORT_NAME: sort_order = 4;
pub const SORT_MODIFIER: sort_order = 3;
pub const SORT_INDEX: sort_order = 2;
pub const SORT_CREATION: sort_order = 1;
pub const SORT_ACTIVITY: sort_order = 0;
pub const CLIENT_EXIT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_ATTACHED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const CLIENT_DEAD: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const CLIENT_UNATTACHEDFLAGS: ::core::ffi::c_int = CLIENT_DEAD | CLIENT_SUSPENDED | CLIENT_EXIT;

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

/// What one of the stores' walks answered, as nothing once it has run out.
fn walked<T>(p: *mut T) -> Option<*mut T> {
    if p.is_null() { None } else { Some(p) }
}

/// Every paste buffer, in the order the store walks them, which is newest
/// first.
fn buffers_in_order() -> impl Iterator<Item = *mut paste_buffer> {
    successors(
        walked(unsafe { paste_walk(null_mut::<paste_buffer>()) }),
        |pb| walked(unsafe { paste_walk(*pb) }),
    )
}

/// Every session the server holds, in the order its tree walks them, read out
/// of the handles that own them.
fn sessions_in_order() -> impl Iterator<Item = *mut session> {
    session_owners().into_iter().map(|s| s.as_ptr())
}

/// Every window linked into `s`, in index order.
unsafe fn winlinks_of(s: *mut session) -> impl Iterator<Item = *mut winlink> {
    successors(
        walked(unsafe { winlinks_first(&raw mut (*s).windows) }),
        |wl| walked(unsafe { winlinks_after(*wl) }),
    )
}

/// Every pane of `w`, in the order the window carries them.
unsafe fn panes_of(w: *mut window) -> impl Iterator<Item = *mut window_pane> {
    successors(walked(unsafe { window_panes_first(w) }), move |wp| {
        walked(unsafe { window_panes_next(w, *wp) })
    })
}

/// Every key table, in the order the server walks them.
fn tables_in_order() -> impl Iterator<Item = *mut key_table> {
    successors(walked(key_bindings_first_table()), |table| {
        walked(unsafe { key_bindings_next_table(*table) })
    })
}

/// Every binding of `table`, in key order.
unsafe fn bindings_of(table: *mut key_table) -> impl Iterator<Item = *mut key_binding> {
    successors(walked(unsafe { key_bindings_first(table) }), move |bd| {
        walked(unsafe { key_bindings_next(table, *bd) })
    })
}

/// Puts `l` in the order `sort_crit` asks for. The end marker is no order at
/// all, so the list is handed back as the walk built it; sorting by order
/// compares nothing either, since the walk is already in that order, and only
/// turns the list round when the criteria are reversed. Everything else goes
/// to [`merge_sort`] behind `cmp`.
unsafe fn sort_list<T>(l: &mut [*mut T], cmp: Compare<T>, sort_crit: &sort_criteria_t) {
    match sort_crit.order {
        SORT_END => {}
        SORT_ORDER => {
            if sort_crit.reversed != 0 {
                l.reverse();
            }
        }
        _ => merge_sort(l, &mut |a, b| unsafe { cmp(a, b, sort_crit) }),
    }
}

/// How two of a store's entries are ordered under some criteria.
type Compare<T> = unsafe fn(*mut T, *mut T, &sort_criteria_t) -> c_int;

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
fn merge_sort<T>(l: &mut [*mut T], cmp: &mut impl FnMut(*mut T, *mut T) -> c_int) {
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
        if cmp(left[a], l[b]) <= 0 {
            l[at] = left[a];
            a += 1;
        } else {
            l[at] = l[b];
            b += 1;
        }
        at += 1;
    }
    l[at..at + (left.len() - a)].copy_from_slice(&left[a..]);
}

/// The criteria a comparison works by, as the order to compare in and whether
/// the answer is turned round.
fn criteria(crit: &sort_criteria_t) -> (sort_order, c_int) {
    (crit.order, crit.reversed)
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

/// Two of the server's C strings, compared the way `strcmp` orders them.
unsafe fn text_cmp(a: *const c_char, b: *const c_char) -> c_int {
    unsafe { bytes_cmp(CStr::from_ptr(a).to_bytes(), CStr::from_ptr(b).to_bytes()) }
}

/// Two of the names the server owns outright, ordered the same way. A name
/// the server has not set orders as an empty one: every entry a collector
/// gathers carries its name, so that stands where reading the name as a C
/// string would have handed `strcmp` a null pointer.
fn name_cmp(a: &Option<CString>, b: &Option<CString>) -> c_int {
    fn bytes(s: &Option<CString>) -> &[u8] {
        s.as_ref().map_or(&[][..], |s| s.as_bytes())
    }
    bytes_cmp(bytes(a), bytes(b))
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

unsafe fn sort_buffer_cmp(
    a0: *mut paste_buffer,
    b0: *mut paste_buffer,
    crit: &sort_criteria_t,
) -> c_int {
    unsafe {
        let (order, reversed) = criteria(crit);
        let mut result = match order {
            SORT_NAME => bytes_cmp(
                paste_buffer_name(&*a0).to_bytes(),
                paste_buffer_name(&*b0).to_bytes(),
            ),
            SORT_CREATION => match paste_buffer_order(&*a0).cmp(&paste_buffer_order(&*b0)) {
                Ordering::Greater => -1,
                Ordering::Less => 1,
                Ordering::Equal => 0,
            },
            SORT_SIZE => paste_buffer_data(&*a0)
                .len()
                .wrapping_sub(paste_buffer_data(&*b0).len()) as c_int,
            _ => 0,
        };
        if result == 0 {
            result = bytes_cmp(
                paste_buffer_name(&*a0).to_bytes(),
                paste_buffer_name(&*b0).to_bytes(),
            );
        }
        settled(result, reversed)
    }
}

unsafe fn sort_client_cmp(a0: *mut client, b0: *mut client, crit: &sort_criteria_t) -> c_int {
    unsafe {
        let ca = &*a0;
        let cb = &*b0;
        let (order, reversed) = criteria(crit);
        let mut result = match order {
            SORT_NAME => name_cmp(&ca.name, &cb.name),
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
            result = name_cmp(&ca.name, &cb.name);
        }
        settled(result, reversed)
    }
}

unsafe fn sort_session_cmp(a0: *mut session, b0: *mut session, crit: &sort_criteria_t) -> c_int {
    unsafe {
        let sa = &*a0;
        let sb = &*b0;
        let (order, reversed) = criteria(crit);
        let mut result = match order {
            SORT_INDEX => session_id(sa).wrapping_sub(session_id(sb)) as c_int,
            SORT_CREATION => by_creation(&sa.creation_time, &sb.creation_time),
            SORT_ACTIVITY => -by_creation(&session_activity_time(sa), &session_activity_time(sb)),
            SORT_NAME => text_cmp(session_name(sa), session_name(sb)),
            _ => 0,
        };
        if result == 0 {
            result = text_cmp(session_name(sa), session_name(sb));
        }
        settled(result, reversed)
    }
}

unsafe fn sort_pane_cmp(
    wpa: *mut window_pane,
    wpb: *mut window_pane,
    crit: &sort_criteria_t,
) -> c_int {
    unsafe {
        let a = &*wpa;
        let b = &*wpb;
        let (order, reversed) = criteria(crit);
        let mut result = match order {
            SORT_ACTIVITY => a.active_point.wrapping_sub(b.active_point) as c_int,
            SORT_CREATION => a.id.wrapping_sub(b.id) as c_int,
            SORT_SIZE => {
                a.sx.wrapping_mul(a.sy)
                    .wrapping_sub(b.sx.wrapping_mul(b.sy)) as c_int
            }
            SORT_INDEX => {
                let (_, ai) = window_pane_index(wpa);
                let (_, bi) = window_pane_index(wpb);
                ai.wrapping_sub(bi) as c_int
            }
            SORT_NAME => name_cmp(&(*a.screen()).title, &(*b.screen()).title),
            SORT_Z => {
                let (_, ai) = window_pane_zindex(wpa);
                let (_, bi) = window_pane_zindex(wpb);
                ai.wrapping_sub(bi) as c_int
            }
            _ => 0,
        };
        if result == 0 {
            result = name_cmp(&(*a.screen()).title, &(*b.screen()).title);
        }
        settled(result, reversed)
    }
}

unsafe fn sort_winlink_cmp(a0: *mut winlink, b0: *mut winlink, crit: &sort_criteria_t) -> c_int {
    unsafe {
        let wla = &*a0;
        let wlb = &*b0;
        let wa = &*wla.window();
        let wb = &*wlb.window();
        let (order, reversed) = criteria(crit);
        let mut result = match order {
            SORT_INDEX => wla.idx.wrapping_sub(wlb.idx),
            SORT_CREATION => by_creation(&wa.creation_time, &wb.creation_time),
            SORT_ACTIVITY => -by_creation(&wa.activity_time, &wb.activity_time),
            SORT_NAME => name_cmp(&wa.name, &wb.name),
            SORT_SIZE => wa
                .sx
                .wrapping_mul(wa.sy)
                .wrapping_sub(wb.sx.wrapping_mul(wb.sy)) as c_int,
            _ => 0,
        };
        if result == 0 {
            result = name_cmp(&wa.name, &wb.name);
        }
        settled(result, reversed)
    }
}

unsafe fn sort_key_binding_cmp(
    a0: *mut key_binding,
    b0: *mut key_binding,
    crit: &sort_criteria_t,
) -> c_int {
    unsafe {
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

pub fn sort_next_order(sort_crit: &mut sort_criteria_t) {
    let Some(seq) = sort_crit.order_seq else {
        return;
    };
    sort_crit.order = next_in(seq, sort_crit.order);
}

pub fn sort_order_from_string(order: Option<&CStr>) -> sort_order {
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

pub fn sort_order_to_string(order: sort_order) -> Option<&'static CStr> {
    ORDER_NAMES
        .iter()
        .find(|&&(_, named)| named == order)
        .map(|&(text, _)| text)
}

pub unsafe fn sort_would_window_tree_swap(
    sort_crit: &sort_criteria_t,
    wla: *mut winlink,
    wlb: *mut winlink,
) -> c_int {
    unsafe {
        if sort_crit.order == SORT_INDEX {
            return 0;
        }
        (sort_winlink_cmp(wla, wlb, sort_crit) != 0) as c_int
    }
}

pub unsafe fn sort_get_buffers(sort_crit: &sort_criteria_t) -> Vec<*mut paste_buffer> {
    unsafe {
        let mut l: Vec<*mut paste_buffer> = buffers_in_order().collect();
        sort_list(&mut l, sort_buffer_cmp, sort_crit);
        l
    }
}

pub unsafe fn sort_get_clients(sort_crit: &sort_criteria_t) -> Vec<*mut client> {
    unsafe {
        let mut l: Vec<*mut client> = client_walk()
            .filter(|c| {
                (**c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t == 0
                    && (**c).flags & CLIENT_ATTACHED as uint64_t != 0
            })
            .collect();
        sort_list(&mut l, sort_client_cmp, sort_crit);
        l
    }
}

pub unsafe fn sort_get_sessions(sort_crit: &sort_criteria_t) -> Vec<*mut session> {
    unsafe {
        let mut l: Vec<*mut session> = sessions_in_order().collect();
        sort_list(&mut l, sort_session_cmp, sort_crit);
        l
    }
}

pub unsafe fn sort_get_panes(sort_crit: &sort_criteria_t) -> Vec<*mut window_pane> {
    unsafe {
        let mut l: Vec<*mut window_pane> = Vec::new();
        for s in sessions_in_order() {
            for wl in winlinks_of(s) {
                l.extend(panes_of((*wl).window()));
            }
        }
        sort_list(&mut l, sort_pane_cmp, sort_crit);
        l
    }
}

pub unsafe fn sort_get_panes_session(
    s: *mut session,
    sort_crit: &sort_criteria_t,
) -> Vec<*mut window_pane> {
    unsafe {
        let mut l: Vec<*mut window_pane> = Vec::new();
        for wl in winlinks_of(s) {
            l.extend(panes_of((*wl).window()));
        }
        sort_list(&mut l, sort_pane_cmp, sort_crit);
        l
    }
}

pub unsafe fn sort_get_panes_window(
    w: *mut window,
    sort_crit: &sort_criteria_t,
) -> Vec<*mut window_pane> {
    unsafe {
        let mut l: Vec<*mut window_pane> = panes_of(w).collect();
        sort_list(&mut l, sort_pane_cmp, sort_crit);
        l
    }
}

pub unsafe fn sort_get_winlinks(sort_crit: &sort_criteria_t) -> Vec<*mut winlink> {
    unsafe {
        let mut l: Vec<*mut winlink> = Vec::new();
        for s in sessions_in_order() {
            l.extend(winlinks_of(s));
        }
        sort_list(&mut l, sort_winlink_cmp, sort_crit);
        l
    }
}

pub unsafe fn sort_get_winlinks_session(
    s: *mut session,
    sort_crit: &sort_criteria_t,
) -> Vec<*mut winlink> {
    unsafe {
        let mut l: Vec<*mut winlink> = winlinks_of(s).collect();
        sort_list(&mut l, sort_winlink_cmp, sort_crit);
        l
    }
}

pub unsafe fn sort_get_key_bindings(sort_crit: &sort_criteria_t) -> Vec<*mut key_binding> {
    unsafe {
        let mut l: Vec<*mut key_binding> = Vec::new();
        for table in tables_in_order() {
            l.extend(bindings_of(table));
        }
        sort_list(&mut l, sort_key_binding_cmp, sort_crit);
        l
    }
}

pub unsafe fn sort_get_key_bindings_table(
    table: *mut key_table,
    sort_crit: &sort_criteria_t,
) -> Vec<*mut key_binding> {
    unsafe {
        let mut l: Vec<*mut key_binding> = bindings_of(table).collect();
        sort_list(&mut l, sort_key_binding_cmp, sort_crit);
        l
    }
}

#[cfg(test)]
#[path = "tests/test_sort.rs"]
mod tests;
