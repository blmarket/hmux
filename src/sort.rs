use crate::src::ffi::libc::{strcasecmp, strcmp};
use crate::src::paste::paste_walk;
use crate::src::server::clients;
use crate::src::server_client::Client as _;
use crate::src::session::sessions;
use crate::src::session::sessions_minmax;
use crate::src::session::Session;
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::client::ClientRef;
use crate::src::shared::client::{CLIENT_ATTACHED, CLIENT_UNATTACHEDFLAGS};
use crate::src::shared::key::*;
use crate::src::shared::key::{key_binding, key_table};
use crate::src::shared::pane::window_pane;
use crate::src::shared::paste::{paste_buffer, PasteBufferRef};
use crate::src::shared::session::session;
use crate::src::shared::session::SessionRef;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::window::WindowRef;
use crate::src::shared::window::{window, winlink};
use crate::src::window::Window as _;
use std::cmp::Ordering;

use crate::src::window::{
    window_pane_index, window_pane_next, window_pane_zindex, winlinks_minmax, winlinks_next,
};

pub(super) fn sort_ordering(result: ::core::ffi::c_int, reversed: ::core::ffi::c_int) -> Ordering {
    let ordering = result.cmp(&0);
    if reversed != 0 {
        ordering.reverse()
    } else {
        ordering
    }
}

pub(super) fn sort_by_criteria<T>(
    values: &mut [T],
    sort_crit: &sort_criteria,
    mut compare: impl FnMut(&T, &T, &sort_criteria) -> Ordering,
) {
    if values.len() < 2 || sort_crit.order == SORT_END {
        return;
    }
    if sort_crit.order == SORT_ORDER {
        if sort_crit.reversed != 0 {
            values.reverse();
        }
        return;
    }
    // Preserve traversal order for equal keys, as the pinned tmux sort does.
    values.sort_by(|a, b| compare(a, b, sort_crit));
}

fn sort_buffer_cmp(pa: &paste_buffer, pb: &paste_buffer, sort_crit: &sort_criteria) -> Ordering {
    let order = match sort_crit.order {
        SORT_NAME => pa.name.cmp(&pb.name),
        SORT_CREATION => pb.order.cmp(&pa.order),
        SORT_SIZE => (pa.size().wrapping_sub(pb.size()) as i32).cmp(&0),
        _ => Ordering::Equal,
    }
    .then_with(|| pa.name.cmp(&pb.name));
    if sort_crit.reversed != 0 {
        order.reverse()
    } else {
        order
    }
}
unsafe fn sort_client_cmp(ca: &ClientRef, cb: &ClientRef, sort_crit: &sort_criteria) -> Ordering {
    let order = match sort_crit.order {
        SORT_SIZE => {
            let (a, b) = (ca.terminal_size(), cb.terminal_size());
            (a.0.wrapping_sub(b.0) as i32)
                .cmp(&0)
                .then_with(|| (a.1.wrapping_sub(b.1) as i32).cmp(&0))
        }
        SORT_CREATION => {
            let (a, b) = (ca.creation_time(), cb.creation_time());
            a.cmp(&b)
        }
        SORT_ACTIVITY => {
            let (a, b) = (ca.activity_time(), cb.activity_time());
            b.cmp(&a)
        }
        _ => Ordering::Equal,
    }
    .then_with(|| {
        ca.name()
            .expect("client name")
            .cmp(&cb.name().expect("client name"))
    });
    if sort_crit.reversed != 0 {
        order.reverse()
    } else {
        order
    }
}
unsafe fn sort_pane_cmp(a: &window_pane, b: &window_pane, sort_crit: &sort_criteria) -> Ordering {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let _ai: u_int = 0;
    let _bi: u_int = 0;
    match sort_crit.order as ::core::ffi::c_uint {
        0 => {
            result = (*a).active_point.wrapping_sub((*b).active_point) as ::core::ffi::c_int;
        }
        1 => {
            result = (*a).id.wrapping_sub((*b).id) as ::core::ffi::c_int;
        }
        6 => {
            result = (*a)
                .sx
                .wrapping_mul((*a).sy)
                .wrapping_sub((*b).sx.wrapping_mul((*b).sy))
                as ::core::ffi::c_int;
        }
        2 => {
            let ai = window_pane_index(a);
            let bi = window_pane_index(b);
            result = (ai.is_none(), ai).cmp(&(bi.is_none(), bi)) as ::core::ffi::c_int;
        }
        4 => {
            result = strcmp(
                (*(*a).screen_ptr()).title.as_ptr(),
                (*(*b).screen_ptr()).title.as_ptr(),
            );
        }
        7 => {
            let ai = window_pane_zindex(a);
            let bi = window_pane_zindex(b);
            result = (ai.is_none(), ai).cmp(&(bi.is_none(), bi)) as ::core::ffi::c_int;
        }
        3 | 5 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp(
            (*(*a).screen_ptr()).title.as_ptr(),
            (*(*b).screen_ptr()).title.as_ptr(),
        );
    }
    return sort_ordering(result, sort_crit.reversed);
}
unsafe fn sort_winlink_cmp(
    wla: refbox::Weak<winlink>,
    wlb: refbox::Weak<winlink>,
    sort_crit: &sort_criteria,
) -> Ordering {
    let a = wla.get_unchecked().window_handle().expect("linked window");
    let b = wlb.get_unchecked().window_handle().expect("linked window");
    let mut result = match sort_crit.order as u32 {
        2 => wla.get_unchecked().idx - wlb.get_unchecked().idx,
        1 => {
            let (a, b) = (a.creation_time(), b.creation_time());
            a.cmp(&b) as i32
        }
        0 => {
            let (a, b) = (a.activity_time(), b.activity_time());
            b.cmp(&a) as i32
        }
        4 => strcmp(a.name().as_ptr(), b.name().as_ptr()),
        6 => {
            let (ax, ay) = a.size();
            let (bx, by) = b.size();
            ax.wrapping_mul(ay).wrapping_sub(bx.wrapping_mul(by)) as i32
        }
        _ => 0,
    };
    if result == 0 {
        result = strcmp(a.name().as_ptr(), b.name().as_ptr());
    }
    sort_ordering(result, sort_crit.reversed)
}
unsafe fn sort_key_binding_cmp(
    a: &key_binding,
    b: &key_binding,
    sort_crit: &sort_criteria,
) -> Ordering {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match sort_crit.order as ::core::ffi::c_uint {
        2 => {
            result = a.key.wrapping_sub(b.key) as ::core::ffi::c_int;
        }
        3 => {
            result = (a.key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS)
                .wrapping_sub(b.key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS)
                as ::core::ffi::c_int;
        }
        4 => {
            result = (strcasecmp(
                a.tablename
                    .as_ref()
                    .map_or(::core::ptr::null(), |s| s.as_ptr()),
                b.tablename
                    .as_ref()
                    .map_or(::core::ptr::null(), |s| s.as_ptr()),
            ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        }
        0 | 1 | 5 | 6 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = (strcasecmp(
            a.tablename
                .as_ref()
                .map_or(::core::ptr::null(), |s| s.as_ptr()),
            b.tablename
                .as_ref()
                .map_or(::core::ptr::null(), |s| s.as_ptr()),
        ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    return sort_ordering(result, sort_crit.reversed);
}
pub unsafe fn sort_next_order(sort_crit: *mut sort_criteria) {
    let criteria = &mut *sort_crit;
    let sequence = criteria.order_seq;
    if sequence.is_empty() {
        return;
    }
    let next = sequence
        .iter()
        .position(|&order| order == criteria.order)
        .map_or(0, |index| (index + 1) % sequence.len());
    criteria.order = sequence[next];
}
pub unsafe fn sort_order_from_string(mut order: *const ::core::ffi::c_char) -> sort_order {
    if !order.is_null() {
        if strcasecmp(
            order,
            b"activity\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            return SORT_ACTIVITY;
        }
        if strcasecmp(
            order,
            b"creation\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            return SORT_CREATION;
        }
        if strcasecmp(order, b"index\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
            || strcasecmp(order, b"key\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
        {
            return SORT_INDEX;
        }
        if strcasecmp(
            order,
            b"modifier\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            return SORT_MODIFIER;
        }
        if strcasecmp(order, b"name\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
            || strcasecmp(order, b"title\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
        {
            return SORT_NAME;
        }
        if strcasecmp(order, b"order\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            return SORT_ORDER;
        }
        if strcasecmp(order, b"size\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            return SORT_SIZE;
        }
        if strcasecmp(order, b"z\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            return SORT_Z;
        }
    }
    return SORT_END;
}
pub unsafe fn sort_order_to_string(mut order: sort_order) -> *const ::core::ffi::c_char {
    if order as ::core::ffi::c_uint == SORT_ACTIVITY as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"activity\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_CREATION as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"creation\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_INDEX as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"index\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_MODIFIER as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"modifier\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_NAME as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"name\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_ORDER as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"order\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_SIZE as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"size\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_Z as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"z\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
pub unsafe fn sort_would_window_tree_swap(
    mut sort_crit: *mut sort_criteria,
    mut wla: refbox::Weak<winlink>,
    mut wlb: refbox::Weak<winlink>,
) -> ::core::ffi::c_int {
    if (*sort_crit).order as ::core::ffi::c_uint
        == SORT_INDEX as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    return (sort_winlink_cmp((wla).clone(), (wlb).clone(), &*sort_crit) != Ordering::Equal)
        as ::core::ffi::c_int;
}
pub fn sort_get_buffers(sort_crit: &sort_criteria) -> Vec<PasteBufferRef> {
    let mut buffers = Vec::new();
    let mut next = paste_walk(None);
    while let Some(pb) = next {
        next = paste_walk(Some(&pb));
        buffers.push(pb);
    }
    sort_by_criteria(&mut buffers, sort_crit, |a, b, criteria| {
        if a == b {
            Ordering::Equal
        } else {
            sort_buffer_cmp(&a.borrow(), &b.borrow(), criteria)
        }
    });
    buffers
}
pub unsafe fn sort_get_clients(sort_crit: *mut sort_criteria) -> Vec<ClientRef> {
    let mut clients_sorted = Vec::new();
    let mut current = clients.first();
    while let Some(owner) = current {
        current = clients.next(&owner);
        let client = &owner;
        if client.flags() & CLIENT_UNATTACHEDFLAGS as uint64_t == 0
            && client.flags() & CLIENT_ATTACHED as uint64_t != 0
        {
            clients_sorted.push(owner);
        }
    }
    sort_by_criteria(&mut clients_sorted, &*sort_crit, |a, b, criteria| unsafe {
        sort_client_cmp(a, b, criteria)
    });
    clients_sorted
}
pub use crate::src::session::sort_get_sessions;
pub unsafe fn sort_get_panes_window(
    w: &WindowRef,
    sort_crit: &sort_criteria,
) -> Vec<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
    let mut panes = w.pane_snapshot();
    sort_by_criteria(&mut panes, sort_crit, |a, b, criteria| unsafe {
        sort_pane_cmp(&*a.get(), &*b.get(), criteria)
    });
    panes
}
pub unsafe fn sort_get_winlinks(sort_crit: *mut sort_criteria) -> Vec<refbox::Weak<winlink>> {
    let mut links = Vec::new();
    let mut s_owner = sessions_minmax(&sessions);
    while s_owner.is_some() {
        let mut wl = s_owner
            .as_ref()
            .expect("session traversal owner")
            .with_winlinks(|links| winlinks_minmax(links, RB_NEGINF));
        while wl.is_alive() {
            links.push(wl.clone());
            wl = winlinks_next(wl.get_unchecked());
        }
        s_owner = s_owner.as_ref().expect("live session").next_session();
    }
    sort_by_criteria(&mut links, &*sort_crit, |a, b, criteria| unsafe {
        sort_winlink_cmp((*a).clone(), (*b).clone(), criteria)
    });
    links
}
pub unsafe fn sort_get_winlinks_session(
    s_owner: &SessionRef,
    sort_crit: *mut sort_criteria,
) -> Vec<refbox::Weak<winlink>> {
    let mut l = Vec::new();
    let mut wl = s_owner.with_winlinks(|links| winlinks_minmax(links, RB_NEGINF));
    while wl.is_alive() {
        l.push(wl.clone());
        wl = winlinks_next(wl.get_unchecked());
    }
    sort_by_criteria(&mut l, &*sort_crit, |a, b, criteria| unsafe {
        sort_winlink_cmp((*a).clone(), (*b).clone(), criteria)
    });
    l
}
pub unsafe fn sort_get_key_bindings<'a>(
    tables: &'a [std::cell::Ref<'_, key_table>],
    sort_crit: &sort_criteria,
) -> Vec<&'a key_binding> {
    let mut bindings: Vec<_> = tables
        .iter()
        .flat_map(|table| table.key_bindings.iter())
        .collect();
    sort_by_criteria(&mut bindings, sort_crit, |a, b, criteria| unsafe {
        sort_key_binding_cmp(a, b, criteria)
    });
    bindings
}

pub unsafe fn sort_get_key_bindings_table<'a>(
    table: Option<&'a key_table>,
    sort_crit: &sort_criteria,
) -> Vec<&'a key_binding> {
    let mut bindings: Vec<_> = table
        .into_iter()
        .flat_map(|table| table.key_bindings.iter())
        .collect();
    sort_by_criteria(&mut bindings, sort_crit, |a, b, criteria| unsafe {
        sort_key_binding_cmp(a, b, criteria)
    });
    bindings
}

#[cfg(test)]
mod sequence_tests {
    use super::*;

    #[test]
    fn equal_keys_preserve_input_order_in_both_directions() {
        for reversed in [0, 1] {
            let criteria = sort_criteria {
                order: SORT_NAME,
                reversed,
                order_seq: &[],
            };
            let mut values: Vec<_> = (0..30).map(|index| (index % 2, index)).collect();
            sort_by_criteria(&mut values, &criteria, |a, b, criteria| {
                sort_ordering(a.0 - b.0, criteria.reversed)
            });
            let first = reversed;
            let expected: Vec<_> = (first..30)
                .step_by(2)
                .chain((1 - first..30).step_by(2))
                .collect();
            assert_eq!(
                values.iter().map(|value| value.1).collect::<Vec<_>>(),
                expected
            );
        }
    }

    #[test]
    fn sort_sequence_cycles_and_recovers_unknown_order() {
        let mut criteria = sort_criteria {
            order: SORT_NAME,
            reversed: 0,
            order_seq: &[SORT_NAME, SORT_SIZE],
        };
        unsafe {
            sort_next_order(&mut criteria);
            assert_eq!(criteria.order, SORT_SIZE);
            sort_next_order(&mut criteria);
            assert_eq!(criteria.order, SORT_NAME);
            criteria.order = SORT_ACTIVITY;
            sort_next_order(&mut criteria);
            assert_eq!(criteria.order, SORT_NAME);
            criteria.order_seq = &[];
            sort_next_order(&mut criteria);
            assert_eq!(criteria.order, SORT_NAME);
            criteria.order_seq = &[SORT_SIZE];
            sort_next_order(&mut criteria);
            sort_next_order(&mut criteria);
            assert_eq!(criteria.order, SORT_SIZE);
        }
    }
}
