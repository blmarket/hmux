use crate::src::ffi::libc::{strcasecmp, strcmp};
use crate::src::key_bindings::{
    key_bindings_first, key_bindings_first_table, key_bindings_next, key_bindings_next_table,
};
use crate::src::paste::paste_walk;
use crate::src::server::clients;
use crate::src::session::sessions;
use crate::src::session::{sessions_minmax, sessions_next};
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::client::{CLIENT_ATTACHED, CLIENT_UNATTACHEDFLAGS};
use crate::src::shared::key::*;
use crate::src::shared::key::{key_binding, key_table};
use crate::src::shared::pane::window_pane;
use crate::src::shared::paste::{paste_buffer, PasteBufferRef};
use crate::src::shared::session::session;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::window::{window, winlink};
use std::cmp::Ordering;

use crate::src::window::{
    window_pane_first, window_pane_index, window_pane_next, window_pane_zindex, winlinks_minmax,
    winlinks_next,
};

fn sort_ordering(result: ::core::ffi::c_int, reversed: ::core::ffi::c_int) -> Ordering {
    let ordering = result.cmp(&0);
    if reversed != 0 {
        ordering.reverse()
    } else {
        ordering
    }
}

fn sort_by_criteria<T>(
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
        SORT_SIZE => (pa.size.wrapping_sub(pb.size) as i32).cmp(&0),
        _ => Ordering::Equal,
    }
    .then_with(|| pa.name.cmp(&pb.name));
    if sort_crit.reversed != 0 {
        order.reverse()
    } else {
        order
    }
}
unsafe fn sort_client_cmp(ca: *mut client, cb: *mut client, sort_crit: &sort_criteria) -> Ordering {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match sort_crit.order as ::core::ffi::c_uint {
        4 => {
            result = strcmp(
                ((*ca).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                ((*cb).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            );
        }
        6 => {
            result = (*ca).tty.sx.wrapping_sub((*cb).tty.sx) as ::core::ffi::c_int;
            if result == 0 as ::core::ffi::c_int {
                result = (*ca).tty.sy.wrapping_sub((*cb).tty.sy) as ::core::ffi::c_int;
            }
        }
        1 => {
            if if (*ca).creation_time.tv_sec == (*cb).creation_time.tv_sec {
                ((*ca).creation_time.tv_usec > (*cb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*ca).creation_time.tv_sec > (*cb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            } else if if (*ca).creation_time.tv_sec == (*cb).creation_time.tv_sec {
                ((*ca).creation_time.tv_usec < (*cb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*ca).creation_time.tv_sec < (*cb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            }
        }
        0 => {
            if if (*ca).activity_time.tv_sec == (*cb).activity_time.tv_sec {
                ((*ca).activity_time.tv_usec > (*cb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*ca).activity_time.tv_sec > (*cb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            } else if if (*ca).activity_time.tv_sec == (*cb).activity_time.tv_sec {
                ((*ca).activity_time.tv_usec < (*cb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*ca).activity_time.tv_sec < (*cb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            }
        }
        2 | 3 | 5 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp(
            ((*ca).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            ((*cb).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
    }
    return sort_ordering(result, sort_crit.reversed);
}
unsafe fn sort_session_cmp(
    sa: *mut session,
    sb: *mut session,
    sort_crit: &sort_criteria,
) -> Ordering {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match sort_crit.order as ::core::ffi::c_uint {
        2 => {
            result = (*sa).id.wrapping_sub((*sb).id) as ::core::ffi::c_int;
        }
        1 => {
            if if (*sa).creation_time.tv_sec == (*sb).creation_time.tv_sec {
                ((*sa).creation_time.tv_usec > (*sb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*sa).creation_time.tv_sec > (*sb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            } else if if (*sa).creation_time.tv_sec == (*sb).creation_time.tv_sec {
                ((*sa).creation_time.tv_usec < (*sb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*sa).creation_time.tv_sec < (*sb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            }
        }
        0 => {
            if if (*sa).activity_time.tv_sec == (*sb).activity_time.tv_sec {
                ((*sa).activity_time.tv_usec > (*sb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*sa).activity_time.tv_sec > (*sb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            } else if if (*sa).activity_time.tv_sec == (*sb).activity_time.tv_sec {
                ((*sa).activity_time.tv_usec < (*sb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*sa).activity_time.tv_sec < (*sb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            }
        }
        4 => {
            result = strcmp(
                ((*sa).name).as_ptr().cast_mut(),
                ((*sb).name).as_ptr().cast_mut(),
            );
        }
        3 | 5 | 6 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp(
            ((*sa).name).as_ptr().cast_mut(),
            ((*sb).name).as_ptr().cast_mut(),
        );
    }
    return sort_ordering(result, sort_crit.reversed);
}
unsafe fn sort_pane_cmp(
    a: *mut window_pane,
    b: *mut window_pane,
    sort_crit: &sort_criteria,
) -> Ordering {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ai: u_int = 0;
    let mut bi: u_int = 0;
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
            window_pane_index(a, &raw mut ai);
            window_pane_index(b, &raw mut bi);
            result = ai.wrapping_sub(bi) as ::core::ffi::c_int;
        }
        4 => {
            result = strcmp((*(*a).screen).title.as_ptr(), (*(*b).screen).title.as_ptr());
        }
        7 => {
            window_pane_zindex(a, &raw mut ai);
            window_pane_zindex(b, &raw mut bi);
            result = ai.wrapping_sub(bi) as ::core::ffi::c_int;
        }
        3 | 5 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp((*(*a).screen).title.as_ptr(), (*(*b).screen).title.as_ptr());
    }
    return sort_ordering(result, sort_crit.reversed);
}
unsafe fn sort_winlink_cmp(
    wla: *mut winlink,
    wlb: *mut winlink,
    sort_crit: &sort_criteria,
) -> Ordering {
    let mut wa: *mut window = (*wla).window;
    let mut wb: *mut window = (*wlb).window;
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match sort_crit.order as ::core::ffi::c_uint {
        2 => {
            result = (*wla).idx - (*wlb).idx;
        }
        1 => {
            if if (*wa).creation_time.tv_sec == (*wb).creation_time.tv_sec {
                ((*wa).creation_time.tv_usec > (*wb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*wa).creation_time.tv_sec > (*wb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            } else if if (*wa).creation_time.tv_sec == (*wb).creation_time.tv_sec {
                ((*wa).creation_time.tv_usec < (*wb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*wa).creation_time.tv_sec < (*wb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            }
        }
        0 => {
            if if (*wa).activity_time.tv_sec == (*wb).activity_time.tv_sec {
                ((*wa).activity_time.tv_usec > (*wb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*wa).activity_time.tv_sec > (*wb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            } else if if (*wa).activity_time.tv_sec == (*wb).activity_time.tv_sec {
                ((*wa).activity_time.tv_usec < (*wb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*wa).activity_time.tv_sec < (*wb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            }
        }
        4 => {
            result = strcmp((*wa).name.as_ptr(), (*wb).name.as_ptr());
        }
        6 => {
            result = (*wa)
                .sx
                .wrapping_mul((*wa).sy)
                .wrapping_sub((*wb).sx.wrapping_mul((*wb).sy))
                as ::core::ffi::c_int;
        }
        3 | 5 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp((*wa).name.as_ptr(), (*wb).name.as_ptr());
    }
    return sort_ordering(result, sort_crit.reversed);
}
unsafe fn sort_key_binding_cmp(
    a: *mut key_binding,
    b: *mut key_binding,
    sort_crit: &sort_criteria,
) -> Ordering {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match sort_crit.order as ::core::ffi::c_uint {
        2 => {
            result = (*a).key.wrapping_sub((*b).key) as ::core::ffi::c_int;
        }
        3 => {
            result = ((*a).key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS)
                .wrapping_sub((*b).key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS)
                as ::core::ffi::c_int;
        }
        4 => {
            result = (strcasecmp(
                (*a).tablename
                    .as_ref()
                    .map_or(::core::ptr::null(), |s| s.as_ptr()),
                (*b).tablename
                    .as_ref()
                    .map_or(::core::ptr::null(), |s| s.as_ptr()),
            ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        }
        0 | 1 | 5 | 6 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = (strcasecmp(
            (*a).tablename
                .as_ref()
                .map_or(::core::ptr::null(), |s| s.as_ptr()),
            (*b).tablename
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
    mut wla: *mut winlink,
    mut wlb: *mut winlink,
) -> ::core::ffi::c_int {
    if (*sort_crit).order as ::core::ffi::c_uint
        == SORT_INDEX as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    return (sort_winlink_cmp(wla, wlb, &*sort_crit) != Ordering::Equal) as ::core::ffi::c_int;
}
pub fn sort_get_buffers(sort_crit: &sort_criteria) -> Vec<PasteBufferRef> {
    let mut buffers = Vec::new();
    let mut next = paste_walk(None);
    while let Some(pb) = next {
        next = paste_walk(Some(&pb));
        buffers.push(pb);
    }
    sort_by_criteria(&mut buffers, sort_crit, |a, b, criteria| {
        sort_buffer_cmp(&a.borrow(), &b.borrow(), criteria)
    });
    buffers
}
pub unsafe fn sort_get_clients(sort_crit: *mut sort_criteria) -> Vec<*mut client> {
    let mut clients_sorted = Vec::new();
    let mut c = clients.first();
    while !c.is_null() {
        if !((*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !(!(*c).flags & CLIENT_ATTACHED as uint64_t != 0) {
                clients_sorted.push(c);
            }
        }
        c = clients.next(c);
    }
    sort_by_criteria(&mut clients_sorted, &*sort_crit, |a, b, criteria| unsafe {
        sort_client_cmp(*a, *b, criteria)
    });
    clients_sorted
}
pub unsafe fn sort_get_sessions(sort_crit: *mut sort_criteria) -> Vec<*mut session> {
    let mut l = Vec::new();
    let mut s = sessions_minmax(&*std::ptr::addr_of!(sessions));
    while !s.is_null() {
        l.push(s);
        s = sessions_next(&*s);
    }
    sort_by_criteria(&mut l, &*sort_crit, |a, b, criteria| unsafe {
        sort_session_cmp(*a, *b, criteria)
    });
    l
}
pub unsafe fn sort_get_panes_window(
    w: *mut window,
    sort_crit: *mut sort_criteria,
) -> Vec<*mut window_pane> {
    let mut panes = Vec::new();
    let mut wp = window_pane_first(w);
    while !wp.is_null() {
        panes.push(wp);
        wp = window_pane_next(wp);
    }
    sort_by_criteria(&mut panes, &*sort_crit, |a, b, criteria| unsafe {
        sort_pane_cmp(*a, *b, criteria)
    });
    panes
}
pub unsafe fn sort_get_winlinks(sort_crit: *mut sort_criteria) -> Vec<*mut winlink> {
    let mut links = Vec::new();
    let mut s = sessions_minmax(&*std::ptr::addr_of!(sessions));
    while !s.is_null() {
        let mut wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
        while !wl.is_null() {
            links.push(wl);
            wl = winlinks_next(&*wl);
        }
        s = sessions_next(&*s);
    }
    sort_by_criteria(&mut links, &*sort_crit, |a, b, criteria| unsafe {
        sort_winlink_cmp(*a, *b, criteria)
    });
    links
}
pub unsafe fn sort_get_winlinks_session(
    s: *mut session,
    sort_crit: *mut sort_criteria,
) -> Vec<*mut winlink> {
    let mut l = Vec::new();
    let mut wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
    while !wl.is_null() {
        l.push(wl);
        wl = winlinks_next(&*wl);
    }
    sort_by_criteria(&mut l, &*sort_crit, |a, b, criteria| unsafe {
        sort_winlink_cmp(*a, *b, criteria)
    });
    l
}
pub unsafe fn sort_get_key_bindings(sort_crit: *mut sort_criteria) -> Vec<*mut key_binding> {
    let mut bindings = Vec::new();
    let mut table = key_bindings_first_table();
    while !table.is_null() {
        let mut bd = key_bindings_first(table);
        while !bd.is_null() {
            bindings.push(bd);
            bd = key_bindings_next(bd);
        }
        table = key_bindings_next_table(table);
    }
    sort_by_criteria(&mut bindings, &*sort_crit, |a, b, criteria| unsafe {
        sort_key_binding_cmp(*a, *b, criteria)
    });
    bindings
}
pub unsafe fn sort_get_key_bindings_table(
    table: *mut key_table,
    sort_crit: *mut sort_criteria,
) -> Vec<*mut key_binding> {
    let mut bindings = Vec::new();
    if table.is_null() {
        return bindings;
    }
    let mut bd = key_bindings_first(table);
    while !bd.is_null() {
        bindings.push(bd);
        bd = key_bindings_next(bd);
    }
    sort_by_criteria(&mut bindings, &*sort_crit, |a, b, criteria| unsafe {
        sort_key_binding_cmp(*a, *b, criteria)
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
