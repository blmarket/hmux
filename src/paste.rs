use crate::src::events::events_fire;
use crate::src::events_payload::{event_payload_create, event_payload_set_string};
use crate::src::ffi::libc::{free, strlcpy, time};
use crate::src::options::options_get_number;
use crate::src::tmux::{clean_name, global_options};
use crate::src::utf8::utf8_strvis;
use crate::src::xmalloc::{xasprintf, xmalloc, xreallocarray, xstrdup};
pub use crate::src::shared::events::{event_payload};
pub use crate::src::shared::options::{options};
pub use crate::src::shared::paste::{paste_buffer, paste_buffer_name_entry, paste_buffer_time_entry};
pub use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
pub use crate::src::shared::tree::{RB_BLACK, RB_INF, RB_NEGINF, RB_RED};
use crate::src::shared::abi::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct paste_time_tree {
    pub rbh_root: *mut paste_buffer,
}
#[derive(Default)]
pub struct paste_name_tree {
    entries: std::collections::BTreeMap<Vec<u8>, *mut paste_buffer>,
}

static mut paste_next_index: u_int = 0;
static mut paste_next_order: u_int = 0;
static mut paste_num_automatic: u_int = 0;
static mut paste_by_name: paste_name_tree = paste_name_tree {
    entries: std::collections::BTreeMap::new(),
};
static mut paste_by_time: paste_time_tree = paste_time_tree {
    rbh_root: ::core::ptr::null::<paste_buffer>() as *mut paste_buffer,
};
unsafe fn paste_name_key(name: *const ::core::ffi::c_char) -> Vec<u8> {
    std::ffi::CStr::from_ptr(name).to_bytes().to_vec()
}

unsafe fn paste_name_tree_find(
    head: *mut paste_name_tree,
    name: *const ::core::ffi::c_char,
) -> *mut paste_buffer {
    (*head)
        .entries
        .get(&paste_name_key(name))
        .copied()
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}

unsafe fn paste_name_tree_insert(
    head: *mut paste_name_tree,
    elm: *mut paste_buffer,
) -> *mut paste_buffer {
    match (*head).entries.entry(paste_name_key((*elm).name)) {
        std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            ::core::ptr::null_mut::<paste_buffer>()
        }
    }
}

unsafe fn paste_name_tree_remove(
    head: *mut paste_name_tree,
    elm: *mut paste_buffer,
) -> *mut paste_buffer {
    (*head)
        .entries
        .remove(&paste_name_key((*elm).name))
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}
unsafe extern "C" fn paste_time_tree_RB_INSERT(
    mut head: *mut paste_time_tree,
    mut elm: *mut paste_buffer,
) -> *mut paste_buffer {
    let mut tmp: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut parent: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = paste_cmp_times(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).time_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).time_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).time_entry.rbe_parent = parent;
    (*elm).time_entry.rbe_right = ::core::ptr::null_mut::<paste_buffer>();
    (*elm).time_entry.rbe_left = (*elm).time_entry.rbe_right;
    (*elm).time_entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).time_entry.rbe_left = elm;
        } else {
            (*parent).time_entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    paste_time_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<paste_buffer>();
}
unsafe extern "C" fn paste_time_tree_RB_NEXT(mut elm: *mut paste_buffer) -> *mut paste_buffer {
    if !(*elm).time_entry.rbe_right.is_null() {
        elm = (*elm).time_entry.rbe_right;
        while !(*elm).time_entry.rbe_left.is_null() {
            elm = (*elm).time_entry.rbe_left;
        }
    } else if !(*elm).time_entry.rbe_parent.is_null()
        && elm == (*(*elm).time_entry.rbe_parent).time_entry.rbe_left
    {
        elm = (*elm).time_entry.rbe_parent;
    } else {
        while !(*elm).time_entry.rbe_parent.is_null()
            && elm == (*(*elm).time_entry.rbe_parent).time_entry.rbe_right
        {
            elm = (*elm).time_entry.rbe_parent;
        }
        elm = (*elm).time_entry.rbe_parent;
    }
    return elm;
}
unsafe extern "C" fn paste_time_tree_RB_INSERT_COLOR(
    mut head: *mut paste_time_tree,
    mut elm: *mut paste_buffer,
) {
    let mut parent: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut gparent: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut tmp: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    loop {
        parent = (*elm).time_entry.rbe_parent;
        if !(!parent.is_null() && (*parent).time_entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).time_entry.rbe_parent;
        if parent == (*gparent).time_entry.rbe_left {
            tmp = (*gparent).time_entry.rbe_right;
            if !tmp.is_null() && (*tmp).time_entry.rbe_color == RB_RED {
                (*tmp).time_entry.rbe_color = RB_BLACK;
                (*parent).time_entry.rbe_color = RB_BLACK;
                (*gparent).time_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).time_entry.rbe_right == elm {
                    tmp = (*parent).time_entry.rbe_right;
                    (*parent).time_entry.rbe_right = (*tmp).time_entry.rbe_left;
                    if !(*parent).time_entry.rbe_right.is_null() {
                        (*(*tmp).time_entry.rbe_left).time_entry.rbe_parent = parent;
                    }
                    (*tmp).time_entry.rbe_parent = (*parent).time_entry.rbe_parent;
                    if !(*tmp).time_entry.rbe_parent.is_null() {
                        if parent == (*(*parent).time_entry.rbe_parent).time_entry.rbe_left {
                            (*(*parent).time_entry.rbe_parent).time_entry.rbe_left = tmp;
                        } else {
                            (*(*parent).time_entry.rbe_parent).time_entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).time_entry.rbe_left = parent;
                    (*parent).time_entry.rbe_parent = tmp;
                    !(*tmp).time_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).time_entry.rbe_color = RB_BLACK;
                (*gparent).time_entry.rbe_color = RB_RED;
                tmp = (*gparent).time_entry.rbe_left;
                (*gparent).time_entry.rbe_left = (*tmp).time_entry.rbe_right;
                if !(*gparent).time_entry.rbe_left.is_null() {
                    (*(*tmp).time_entry.rbe_right).time_entry.rbe_parent = gparent;
                }
                (*tmp).time_entry.rbe_parent = (*gparent).time_entry.rbe_parent;
                if !(*tmp).time_entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).time_entry.rbe_parent).time_entry.rbe_left {
                        (*(*gparent).time_entry.rbe_parent).time_entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).time_entry.rbe_parent).time_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).time_entry.rbe_right = gparent;
                (*gparent).time_entry.rbe_parent = tmp;
                !(*tmp).time_entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).time_entry.rbe_left;
            if !tmp.is_null() && (*tmp).time_entry.rbe_color == RB_RED {
                (*tmp).time_entry.rbe_color = RB_BLACK;
                (*parent).time_entry.rbe_color = RB_BLACK;
                (*gparent).time_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).time_entry.rbe_left == elm {
                    tmp = (*parent).time_entry.rbe_left;
                    (*parent).time_entry.rbe_left = (*tmp).time_entry.rbe_right;
                    if !(*parent).time_entry.rbe_left.is_null() {
                        (*(*tmp).time_entry.rbe_right).time_entry.rbe_parent = parent;
                    }
                    (*tmp).time_entry.rbe_parent = (*parent).time_entry.rbe_parent;
                    if !(*tmp).time_entry.rbe_parent.is_null() {
                        if parent == (*(*parent).time_entry.rbe_parent).time_entry.rbe_left {
                            (*(*parent).time_entry.rbe_parent).time_entry.rbe_left = tmp;
                        } else {
                            (*(*parent).time_entry.rbe_parent).time_entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).time_entry.rbe_right = parent;
                    (*parent).time_entry.rbe_parent = tmp;
                    !(*tmp).time_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).time_entry.rbe_color = RB_BLACK;
                (*gparent).time_entry.rbe_color = RB_RED;
                tmp = (*gparent).time_entry.rbe_right;
                (*gparent).time_entry.rbe_right = (*tmp).time_entry.rbe_left;
                if !(*gparent).time_entry.rbe_right.is_null() {
                    (*(*tmp).time_entry.rbe_left).time_entry.rbe_parent = gparent;
                }
                (*tmp).time_entry.rbe_parent = (*gparent).time_entry.rbe_parent;
                if !(*tmp).time_entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).time_entry.rbe_parent).time_entry.rbe_left {
                        (*(*gparent).time_entry.rbe_parent).time_entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).time_entry.rbe_parent).time_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).time_entry.rbe_left = gparent;
                (*gparent).time_entry.rbe_parent = tmp;
                !(*tmp).time_entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).time_entry.rbe_color = RB_BLACK;
}
unsafe extern "C" fn paste_time_tree_RB_MINMAX(
    mut head: *mut paste_time_tree,
    mut val: ::core::ffi::c_int,
) -> *mut paste_buffer {
    let mut tmp: *mut paste_buffer = (*head).rbh_root;
    let mut parent: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    while !tmp.is_null() {
        parent = tmp;
        if val < 0 as ::core::ffi::c_int {
            tmp = (*tmp).time_entry.rbe_left;
        } else {
            tmp = (*tmp).time_entry.rbe_right;
        }
    }
    return parent;
}
unsafe extern "C" fn paste_time_tree_RB_PREV(mut elm: *mut paste_buffer) -> *mut paste_buffer {
    if !(*elm).time_entry.rbe_left.is_null() {
        elm = (*elm).time_entry.rbe_left;
        while !(*elm).time_entry.rbe_right.is_null() {
            elm = (*elm).time_entry.rbe_right;
        }
    } else if !(*elm).time_entry.rbe_parent.is_null()
        && elm == (*(*elm).time_entry.rbe_parent).time_entry.rbe_right
    {
        elm = (*elm).time_entry.rbe_parent;
    } else {
        while !(*elm).time_entry.rbe_parent.is_null()
            && elm == (*(*elm).time_entry.rbe_parent).time_entry.rbe_left
        {
            elm = (*elm).time_entry.rbe_parent;
        }
        elm = (*elm).time_entry.rbe_parent;
    }
    return elm;
}
unsafe extern "C" fn paste_time_tree_RB_REMOVE(
    mut head: *mut paste_time_tree,
    mut elm: *mut paste_buffer,
) -> *mut paste_buffer {
    let mut current_block: u64;
    let mut child: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut parent: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut old: *mut paste_buffer = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).time_entry.rbe_left.is_null() {
        child = (*elm).time_entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).time_entry.rbe_right.is_null() {
        child = (*elm).time_entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
        elm = (*elm).time_entry.rbe_right;
        loop {
            left = (*elm).time_entry.rbe_left;
            if left.is_null() {
                break;
            }
            elm = left;
        }
        child = (*elm).time_entry.rbe_right;
        parent = (*elm).time_entry.rbe_parent;
        color = (*elm).time_entry.rbe_color;
        if !child.is_null() {
            (*child).time_entry.rbe_parent = parent;
        }
        if !parent.is_null() {
            if (*parent).time_entry.rbe_left == elm {
                (*parent).time_entry.rbe_left = child;
            } else {
                (*parent).time_entry.rbe_right = child;
            }
        } else {
            (*head).rbh_root = child;
        }
        if (*elm).time_entry.rbe_parent == old {
            parent = elm;
        }
        (*elm).time_entry = (*old).time_entry;
        if !(*old).time_entry.rbe_parent.is_null() {
            if (*(*old).time_entry.rbe_parent).time_entry.rbe_left == old {
                (*(*old).time_entry.rbe_parent).time_entry.rbe_left = elm;
            } else {
                (*(*old).time_entry.rbe_parent).time_entry.rbe_right = elm;
            }
        } else {
            (*head).rbh_root = elm;
        }
        (*(*old).time_entry.rbe_left).time_entry.rbe_parent = elm;
        if !(*old).time_entry.rbe_right.is_null() {
            (*(*old).time_entry.rbe_right).time_entry.rbe_parent = elm;
        }
        if !parent.is_null() {
            left = parent;
            loop {
                left = (*left).time_entry.rbe_parent;
                if left.is_null() {
                    break;
                }
            }
        }
        current_block = 14827960014365851420;
    }
    match current_block {
        7245201122033322888 => {
            parent = (*elm).time_entry.rbe_parent;
            color = (*elm).time_entry.rbe_color;
            if !child.is_null() {
                (*child).time_entry.rbe_parent = parent;
            }
            if !parent.is_null() {
                if (*parent).time_entry.rbe_left == elm {
                    (*parent).time_entry.rbe_left = child;
                } else {
                    (*parent).time_entry.rbe_right = child;
                }
            } else {
                (*head).rbh_root = child;
            }
        }
        _ => {}
    }
    if color == RB_BLACK {
        paste_time_tree_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn paste_time_tree_RB_REMOVE_COLOR(
    mut head: *mut paste_time_tree,
    mut parent: *mut paste_buffer,
    mut elm: *mut paste_buffer,
) {
    let mut tmp: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    while (elm.is_null() || (*elm).time_entry.rbe_color == RB_BLACK) && elm != (*head).rbh_root {
        if (*parent).time_entry.rbe_left == elm {
            tmp = (*parent).time_entry.rbe_right;
            if (*tmp).time_entry.rbe_color == RB_RED {
                (*tmp).time_entry.rbe_color = RB_BLACK;
                (*parent).time_entry.rbe_color = RB_RED;
                tmp = (*parent).time_entry.rbe_right;
                (*parent).time_entry.rbe_right = (*tmp).time_entry.rbe_left;
                if !(*parent).time_entry.rbe_right.is_null() {
                    (*(*tmp).time_entry.rbe_left).time_entry.rbe_parent = parent;
                }
                (*tmp).time_entry.rbe_parent = (*parent).time_entry.rbe_parent;
                if !(*tmp).time_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).time_entry.rbe_parent).time_entry.rbe_left {
                        (*(*parent).time_entry.rbe_parent).time_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).time_entry.rbe_parent).time_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).time_entry.rbe_left = parent;
                (*parent).time_entry.rbe_parent = tmp;
                !(*tmp).time_entry.rbe_parent.is_null();
                tmp = (*parent).time_entry.rbe_right;
            }
            if ((*tmp).time_entry.rbe_left.is_null()
                || (*(*tmp).time_entry.rbe_left).time_entry.rbe_color == RB_BLACK)
                && ((*tmp).time_entry.rbe_right.is_null()
                    || (*(*tmp).time_entry.rbe_right).time_entry.rbe_color == RB_BLACK)
            {
                (*tmp).time_entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).time_entry.rbe_parent;
            } else {
                if (*tmp).time_entry.rbe_right.is_null()
                    || (*(*tmp).time_entry.rbe_right).time_entry.rbe_color == RB_BLACK
                {
                    let mut oleft: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
                    oleft = (*tmp).time_entry.rbe_left;
                    if !oleft.is_null() {
                        (*oleft).time_entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).time_entry.rbe_color = RB_RED;
                    oleft = (*tmp).time_entry.rbe_left;
                    (*tmp).time_entry.rbe_left = (*oleft).time_entry.rbe_right;
                    if !(*tmp).time_entry.rbe_left.is_null() {
                        (*(*oleft).time_entry.rbe_right).time_entry.rbe_parent = tmp;
                    }
                    (*oleft).time_entry.rbe_parent = (*tmp).time_entry.rbe_parent;
                    if !(*oleft).time_entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).time_entry.rbe_parent).time_entry.rbe_left {
                            (*(*tmp).time_entry.rbe_parent).time_entry.rbe_left = oleft;
                        } else {
                            (*(*tmp).time_entry.rbe_parent).time_entry.rbe_right = oleft;
                        }
                    } else {
                        (*head).rbh_root = oleft;
                    }
                    (*oleft).time_entry.rbe_right = tmp;
                    (*tmp).time_entry.rbe_parent = oleft;
                    !(*oleft).time_entry.rbe_parent.is_null();
                    tmp = (*parent).time_entry.rbe_right;
                }
                (*tmp).time_entry.rbe_color = (*parent).time_entry.rbe_color;
                (*parent).time_entry.rbe_color = RB_BLACK;
                if !(*tmp).time_entry.rbe_right.is_null() {
                    (*(*tmp).time_entry.rbe_right).time_entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).time_entry.rbe_right;
                (*parent).time_entry.rbe_right = (*tmp).time_entry.rbe_left;
                if !(*parent).time_entry.rbe_right.is_null() {
                    (*(*tmp).time_entry.rbe_left).time_entry.rbe_parent = parent;
                }
                (*tmp).time_entry.rbe_parent = (*parent).time_entry.rbe_parent;
                if !(*tmp).time_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).time_entry.rbe_parent).time_entry.rbe_left {
                        (*(*parent).time_entry.rbe_parent).time_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).time_entry.rbe_parent).time_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).time_entry.rbe_left = parent;
                (*parent).time_entry.rbe_parent = tmp;
                !(*tmp).time_entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        } else {
            tmp = (*parent).time_entry.rbe_left;
            if (*tmp).time_entry.rbe_color == RB_RED {
                (*tmp).time_entry.rbe_color = RB_BLACK;
                (*parent).time_entry.rbe_color = RB_RED;
                tmp = (*parent).time_entry.rbe_left;
                (*parent).time_entry.rbe_left = (*tmp).time_entry.rbe_right;
                if !(*parent).time_entry.rbe_left.is_null() {
                    (*(*tmp).time_entry.rbe_right).time_entry.rbe_parent = parent;
                }
                (*tmp).time_entry.rbe_parent = (*parent).time_entry.rbe_parent;
                if !(*tmp).time_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).time_entry.rbe_parent).time_entry.rbe_left {
                        (*(*parent).time_entry.rbe_parent).time_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).time_entry.rbe_parent).time_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).time_entry.rbe_right = parent;
                (*parent).time_entry.rbe_parent = tmp;
                !(*tmp).time_entry.rbe_parent.is_null();
                tmp = (*parent).time_entry.rbe_left;
            }
            if ((*tmp).time_entry.rbe_left.is_null()
                || (*(*tmp).time_entry.rbe_left).time_entry.rbe_color == RB_BLACK)
                && ((*tmp).time_entry.rbe_right.is_null()
                    || (*(*tmp).time_entry.rbe_right).time_entry.rbe_color == RB_BLACK)
            {
                (*tmp).time_entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).time_entry.rbe_parent;
            } else {
                if (*tmp).time_entry.rbe_left.is_null()
                    || (*(*tmp).time_entry.rbe_left).time_entry.rbe_color == RB_BLACK
                {
                    let mut oright: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
                    oright = (*tmp).time_entry.rbe_right;
                    if !oright.is_null() {
                        (*oright).time_entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).time_entry.rbe_color = RB_RED;
                    oright = (*tmp).time_entry.rbe_right;
                    (*tmp).time_entry.rbe_right = (*oright).time_entry.rbe_left;
                    if !(*tmp).time_entry.rbe_right.is_null() {
                        (*(*oright).time_entry.rbe_left).time_entry.rbe_parent = tmp;
                    }
                    (*oright).time_entry.rbe_parent = (*tmp).time_entry.rbe_parent;
                    if !(*oright).time_entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).time_entry.rbe_parent).time_entry.rbe_left {
                            (*(*tmp).time_entry.rbe_parent).time_entry.rbe_left = oright;
                        } else {
                            (*(*tmp).time_entry.rbe_parent).time_entry.rbe_right = oright;
                        }
                    } else {
                        (*head).rbh_root = oright;
                    }
                    (*oright).time_entry.rbe_left = tmp;
                    (*tmp).time_entry.rbe_parent = oright;
                    !(*oright).time_entry.rbe_parent.is_null();
                    tmp = (*parent).time_entry.rbe_left;
                }
                (*tmp).time_entry.rbe_color = (*parent).time_entry.rbe_color;
                (*parent).time_entry.rbe_color = RB_BLACK;
                if !(*tmp).time_entry.rbe_left.is_null() {
                    (*(*tmp).time_entry.rbe_left).time_entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).time_entry.rbe_left;
                (*parent).time_entry.rbe_left = (*tmp).time_entry.rbe_right;
                if !(*parent).time_entry.rbe_left.is_null() {
                    (*(*tmp).time_entry.rbe_right).time_entry.rbe_parent = parent;
                }
                (*tmp).time_entry.rbe_parent = (*parent).time_entry.rbe_parent;
                if !(*tmp).time_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).time_entry.rbe_parent).time_entry.rbe_left {
                        (*(*parent).time_entry.rbe_parent).time_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).time_entry.rbe_parent).time_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).time_entry.rbe_right = parent;
                (*parent).time_entry.rbe_parent = tmp;
                !(*tmp).time_entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        }
    }
    if !elm.is_null() {
        (*elm).time_entry.rbe_color = RB_BLACK;
    }
}
unsafe extern "C" fn paste_fire_event(
    mut name: *const ::core::ffi::c_char,
    mut pbname: *const ::core::ffi::c_char,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    ep = event_payload_create();
    event_payload_set_string(
        ep,
        b"paste_buffer\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        pbname,
    );
    events_fire(name, ep);
}
unsafe extern "C" fn paste_cmp_times(
    mut a: *const paste_buffer,
    mut b: *const paste_buffer,
) -> ::core::ffi::c_int {
    if (*a).order > (*b).order {
        return -(1 as ::core::ffi::c_int);
    }
    if (*a).order < (*b).order {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn paste_buffer_name(
    mut pb: *mut paste_buffer,
) -> *const ::core::ffi::c_char {
    return (*pb).name;
}
#[no_mangle]
pub unsafe extern "C" fn paste_buffer_order(mut pb: *mut paste_buffer) -> u_int {
    return (*pb).order;
}
#[no_mangle]
pub unsafe extern "C" fn paste_buffer_created(mut pb: *mut paste_buffer) -> time_t {
    return (*pb).created;
}
#[no_mangle]
pub unsafe extern "C" fn paste_buffer_data(
    mut pb: *mut paste_buffer,
    mut size: *mut size_t,
) -> *const ::core::ffi::c_char {
    if !size.is_null() {
        *size = (*pb).size;
    }
    return (*pb).data;
}
#[no_mangle]
pub unsafe extern "C" fn paste_walk(mut pb: *mut paste_buffer) -> *mut paste_buffer {
    if pb.is_null() {
        return paste_time_tree_RB_MINMAX(&raw mut paste_by_time, RB_NEGINF);
    }
    return paste_time_tree_RB_NEXT(pb);
}
#[no_mangle]
pub unsafe extern "C" fn paste_is_empty() -> ::core::ffi::c_int {
    return (paste_by_time.rbh_root == NULL as *mut paste_buffer) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn paste_get_top(
    mut name: *mut *mut ::core::ffi::c_char,
) -> *mut paste_buffer {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    pb = paste_time_tree_RB_MINMAX(&raw mut paste_by_time, RB_NEGINF);
    while !pb.is_null() && (*pb).automatic == 0 {
        pb = paste_time_tree_RB_NEXT(pb);
    }
    if pb.is_null() {
        return ::core::ptr::null_mut::<paste_buffer>();
    }
    if !name.is_null() {
        *name = xstrdup((*pb).name);
    }
    return pb;
}
#[no_mangle]
pub unsafe extern "C" fn paste_get_name(mut name: *const ::core::ffi::c_char) -> *mut paste_buffer {
    if name.is_null() || *name as ::core::ffi::c_int == '\0' as i32 {
        return ::core::ptr::null_mut::<paste_buffer>();
    }
    return paste_name_tree_find(&raw mut paste_by_name, name);
}
#[no_mangle]
pub unsafe extern "C" fn paste_free(mut pb: *mut paste_buffer) {
    paste_fire_event(
        b"paste-buffer-deleted\0" as *const u8 as *const ::core::ffi::c_char,
        (*pb).name,
    );
    paste_name_tree_remove(&raw mut paste_by_name, pb);
    paste_time_tree_RB_REMOVE(&raw mut paste_by_time, pb);
    if (*pb).automatic != 0 {
        paste_num_automatic = paste_num_automatic.wrapping_sub(1);
    }
    free((*pb).data as *mut ::core::ffi::c_void);
    free((*pb).name as *mut ::core::ffi::c_void);
    free(pb as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn paste_add(
    mut prefix: *const ::core::ffi::c_char,
    mut data: *mut ::core::ffi::c_char,
    mut size: size_t,
) {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut pb1: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut limit: u_int = 0;
    if prefix.is_null() {
        prefix = b"buffer\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if size == 0 as size_t {
        free(data as *mut ::core::ffi::c_void);
        return;
    }
    limit = options_get_number(
        global_options,
        b"buffer-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    pb = paste_time_tree_RB_MINMAX(&raw mut paste_by_time, RB_INF);
    while !pb.is_null() && {
        pb1 = paste_time_tree_RB_PREV(pb);
        1 as ::core::ffi::c_int != 0
    } {
        if paste_num_automatic < limit {
            break;
        }
        if (*pb).automatic != 0 {
            paste_free(pb);
        }
        pb = pb1;
    }
    pb = xmalloc(::core::mem::size_of::<paste_buffer>() as size_t) as *mut paste_buffer;
    (*pb).name = ::core::ptr::null_mut::<::core::ffi::c_char>();
    loop {
        free((*pb).name as *mut ::core::ffi::c_void);
        xasprintf(
            &raw mut (*pb).name,
            b"%s%u\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
            paste_next_index,
        );
        paste_next_index = paste_next_index.wrapping_add(1);
        if paste_get_name((*pb).name).is_null() {
            break;
        }
    }
    (*pb).data = data;
    (*pb).size = size;
    (*pb).automatic = 1 as ::core::ffi::c_int;
    paste_num_automatic = paste_num_automatic.wrapping_add(1);
    (*pb).created = time(::core::ptr::null_mut::<time_t>());
    let fresh0 = paste_next_order;
    paste_next_order = paste_next_order.wrapping_add(1);
    (*pb).order = fresh0;
    paste_name_tree_insert(&raw mut paste_by_name, pb);
    paste_time_tree_RB_INSERT(&raw mut paste_by_time, pb);
    paste_fire_event(
        b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*pb).name,
    );
}
#[no_mangle]
pub unsafe extern "C" fn paste_rename(
    mut oldname: *const ::core::ffi::c_char,
    mut newname: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut pb_new: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !cause.is_null() {
        *cause = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if oldname.is_null() || *oldname as ::core::ffi::c_int == '\0' as i32 {
        if !cause.is_null() {
            *cause = xstrdup(b"no buffer\0" as *const u8 as *const ::core::ffi::c_char);
        }
        return -(1 as ::core::ffi::c_int);
    }
    if newname.is_null() || *newname as ::core::ffi::c_int == '\0' as i32 {
        if !cause.is_null() {
            *cause = xstrdup(b"new name is empty\0" as *const u8 as *const ::core::ffi::c_char);
        }
        return -(1 as ::core::ffi::c_int);
    }
    name = clean_name(newname, 0 as ::core::ffi::c_int);
    if name.is_null() {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"invalid buffer name: %s\0" as *const u8 as *const ::core::ffi::c_char,
                newname,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    pb = paste_get_name(oldname);
    if pb.is_null() {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"no buffer %s\0" as *const u8 as *const ::core::ffi::c_char,
                oldname,
            );
        }
        free(name as *mut ::core::ffi::c_void);
        return -(1 as ::core::ffi::c_int);
    }
    pb_new = paste_get_name(name);
    if pb_new == pb {
        free(name as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if !pb_new.is_null() {
        paste_free(pb_new);
    }
    paste_name_tree_remove(&raw mut paste_by_name, pb);
    free((*pb).name as *mut ::core::ffi::c_void);
    (*pb).name = name;
    if (*pb).automatic != 0 {
        paste_num_automatic = paste_num_automatic.wrapping_sub(1);
    }
    (*pb).automatic = 0 as ::core::ffi::c_int;
    paste_name_tree_insert(&raw mut paste_by_name, pb);
    paste_fire_event(
        b"paste-buffer-deleted\0" as *const u8 as *const ::core::ffi::c_char,
        oldname,
    );
    paste_fire_event(
        b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*pb).name,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn paste_set(
    mut data: *mut ::core::ffi::c_char,
    mut size: size_t,
    mut name: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut old: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut newname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !cause.is_null() {
        *cause = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if size == 0 as size_t {
        free(data as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if name.is_null() {
        paste_add(::core::ptr::null::<::core::ffi::c_char>(), data, size);
        return 0 as ::core::ffi::c_int;
    }
    if *name as ::core::ffi::c_int == '\0' as i32 {
        if !cause.is_null() {
            *cause = xstrdup(b"empty buffer name\0" as *const u8 as *const ::core::ffi::c_char);
        }
        return -(1 as ::core::ffi::c_int);
    }
    newname = clean_name(name, 0 as ::core::ffi::c_int);
    if newname.is_null() {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"invalid buffer name: %s\0" as *const u8 as *const ::core::ffi::c_char,
                name,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    pb = xmalloc(::core::mem::size_of::<paste_buffer>() as size_t) as *mut paste_buffer;
    (*pb).name = newname;
    (*pb).data = data;
    (*pb).size = size;
    (*pb).automatic = 0 as ::core::ffi::c_int;
    let fresh1 = paste_next_order;
    paste_next_order = paste_next_order.wrapping_add(1);
    (*pb).order = fresh1;
    (*pb).created = time(::core::ptr::null_mut::<time_t>());
    old = paste_get_name((*pb).name);
    if !old.is_null() {
        paste_free(old);
    }
    paste_name_tree_insert(&raw mut paste_by_name, pb);
    paste_time_tree_RB_INSERT(&raw mut paste_by_time, pb);
    paste_fire_event(
        b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*pb).name,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn paste_replace(
    mut pb: *mut paste_buffer,
    mut data: *mut ::core::ffi::c_char,
    mut size: size_t,
) {
    free((*pb).data as *mut ::core::ffi::c_void);
    (*pb).data = data;
    (*pb).size = size;
    paste_fire_event(
        b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*pb).name,
    );
}
#[no_mangle]
pub unsafe extern "C" fn paste_make_sample(mut pb: *mut paste_buffer) -> *mut ::core::ffi::c_char {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut used: size_t = 0;
    let flags: ::core::ffi::c_int = VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL;
    let width: size_t = 200 as size_t;
    len = (*pb).size;
    if len > width {
        len = width;
    }
    buf = xreallocarray(
        NULL,
        len,
        (4 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as size_t,
    ) as *mut ::core::ffi::c_char;
    used = utf8_strvis(buf, (*pb).data, len, flags);
    if (*pb).size > width || used > width {
        strlcpy(
            buf.offset(width as isize),
            b"...\0" as *const u8 as *const ::core::ffi::c_char,
            4 as size_t,
        );
    }
    return buf;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn named_buffer(name: &CString) -> Box<paste_buffer> {
        Box::new(paste_buffer {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
            name: name.as_ptr() as *mut ::core::ffi::c_char,
            created: 0,
            automatic: 0,
            order: 0,
            name_entry: paste_buffer_name_entry {
                rbe_left: ::core::ptr::null_mut::<paste_buffer>(),
                rbe_right: ::core::ptr::null_mut::<paste_buffer>(),
                rbe_parent: ::core::ptr::null_mut::<paste_buffer>(),
                rbe_color: 0,
            },
            time_entry: paste_buffer_time_entry {
                rbe_left: ::core::ptr::null_mut::<paste_buffer>(),
                rbe_right: ::core::ptr::null_mut::<paste_buffer>(),
                rbe_parent: ::core::ptr::null_mut::<paste_buffer>(),
                rbe_color: 0,
            },
        })
    }

    #[test]
    fn paste_name_tree_matches_strcmp_order_and_duplicate_semantics() {
        let names = [
            CString::new("z").unwrap(),
            CString::new("a").unwrap(),
            CString::new(vec![b'a', 0xff]).unwrap(),
            CString::new("a0").unwrap(),
        ];
        let mut items = names.iter().map(named_buffer).collect::<Vec<_>>();
        let duplicate_name = CString::new("a").unwrap();
        let mut duplicate = named_buffer(&duplicate_name);
        let mut tree = paste_name_tree::default();

        unsafe {
            let z = items[0].as_mut() as *mut paste_buffer;
            let a = items[1].as_mut() as *mut paste_buffer;
            let a_high = items[2].as_mut() as *mut paste_buffer;
            let a0 = items[3].as_mut() as *mut paste_buffer;
            let duplicate = duplicate.as_mut() as *mut paste_buffer;

            assert!(paste_name_tree_insert(&raw mut tree, z).is_null());
            assert!(paste_name_tree_insert(&raw mut tree, a_high).is_null());
            assert!(paste_name_tree_insert(&raw mut tree, a).is_null());
            assert!(paste_name_tree_insert(&raw mut tree, a0).is_null());
            assert_eq!(
                paste_name_tree_insert(&raw mut tree, duplicate),
                a,
                "duplicate names keep the original item"
            );

            let ordered = tree
                .entries
                .keys()
                .map(Vec::as_slice)
                .collect::<Vec<_>>();
            assert_eq!(
                ordered,
                vec![&b"a"[..], &b"a0"[..], &b"a\xff"[..], &b"z"[..]]
            );
            assert_eq!(paste_name_tree_find(&raw mut tree, names[2].as_ptr()), a_high);
            assert!(paste_name_tree_find(&raw mut tree, b"missing\0".as_ptr().cast()).is_null());

            assert_eq!(paste_name_tree_remove(&raw mut tree, a_high), a_high);
            assert!(paste_name_tree_find(&raw mut tree, names[2].as_ptr()).is_null());
        }
    }
}
