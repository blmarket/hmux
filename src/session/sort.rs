//! Session registry ordering; comparison keys stay inside the Session owner.
use super::*;
use crate::src::sort::{sort_by_criteria, sort_ordering};
use std::cmp::Ordering;

unsafe fn sort_session_cmp(sa: &session, sb: &session, sort_crit: &sort_criteria) -> Ordering {
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
pub unsafe fn sort_get_sessions(
    sort_crit: &sort_criteria,
) -> Vec<std::rc::Rc<std::cell::UnsafeCell<session>>> {
    let mut sessions_sorted = Vec::new();
    let mut current = sessions_minmax(&sessions);
    while let Some(owner) = current {
        current = sessions_next(&*owner.get());
        sessions_sorted.push(owner);
    }
    sort_by_criteria(&mut sessions_sorted, sort_crit, |a, b, criteria| unsafe {
        sort_session_cmp(&*a.get(), &*b.get(), criteria)
    });
    sessions_sorted
}
