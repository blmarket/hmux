pub use crate::src::shared::hyperlinks::{
    hyperlink_inner_entry, hyperlink_list_entry, hyperlink_uri_entry, hyperlinks,
    hyperlinks_by_inner_tree, hyperlinks_by_uri_tree, hyperlinks_list, hyperlinks_uri,
};
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
use crate::src::shared::abi::*;
extern "C" {
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn utf8_stravis(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> size_t;
}
pub const VIS_OCTAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VIS_CSTYLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MAX_HYPERLINKS: ::core::ffi::c_int = 5000 as ::core::ffi::c_int;
pub const MAX_HYPERLINK_URI: ::core::ffi::c_int = 1024 as ::core::ffi::c_int;
static mut hyperlinks_next_external_id: ::core::ffi::c_longlong = 1 as ::core::ffi::c_longlong;
static mut global_hyperlinks_count: u_int = 0;
static mut global_hyperlinks: hyperlinks_list = hyperlinks_list {
    tqh_first: ::core::ptr::null::<hyperlinks_uri>() as *mut hyperlinks_uri,
    tqh_last: ::core::ptr::null::<*mut hyperlinks_uri>() as *mut *mut hyperlinks_uri,
};
unsafe extern "C" fn hyperlinks_by_uri_cmp(
    mut left: *mut hyperlinks_uri,
    mut right: *mut hyperlinks_uri,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    if *(*left).internal_id as ::core::ffi::c_int == '\0' as i32
        || *(*right).internal_id as ::core::ffi::c_int == '\0' as i32
    {
        if *(*left).internal_id as ::core::ffi::c_int != '\0' as i32 {
            return -(1 as ::core::ffi::c_int);
        }
        if *(*right).internal_id as ::core::ffi::c_int != '\0' as i32 {
            return 1 as ::core::ffi::c_int;
        }
        return (*left).inner.wrapping_sub((*right).inner) as ::core::ffi::c_int;
    }
    r = strcmp((*left).internal_id, (*right).internal_id);
    if r != 0 as ::core::ffi::c_int {
        return r;
    }
    return strcmp((*left).uri, (*right).uri);
}
unsafe extern "C" fn hyperlinks_by_uri_tree_RB_REMOVE_COLOR(
    mut head: *mut hyperlinks_by_uri_tree,
    mut parent: *mut hyperlinks_uri,
    mut elm: *mut hyperlinks_uri,
) {
    let mut tmp: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    while (elm.is_null() || (*elm).by_uri_entry.rbe_color == RB_BLACK) && elm != (*head).rbh_root {
        if (*parent).by_uri_entry.rbe_left == elm {
            tmp = (*parent).by_uri_entry.rbe_right;
            if (*tmp).by_uri_entry.rbe_color == RB_RED {
                (*tmp).by_uri_entry.rbe_color = RB_BLACK;
                (*parent).by_uri_entry.rbe_color = RB_RED;
                tmp = (*parent).by_uri_entry.rbe_right;
                (*parent).by_uri_entry.rbe_right = (*tmp).by_uri_entry.rbe_left;
                if !(*parent).by_uri_entry.rbe_right.is_null() {
                    (*(*tmp).by_uri_entry.rbe_left).by_uri_entry.rbe_parent = parent;
                }
                (*tmp).by_uri_entry.rbe_parent = (*parent).by_uri_entry.rbe_parent;
                if !(*tmp).by_uri_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left {
                        (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).by_uri_entry.rbe_left = parent;
                (*parent).by_uri_entry.rbe_parent = tmp;
                !(*tmp).by_uri_entry.rbe_parent.is_null();
                tmp = (*parent).by_uri_entry.rbe_right;
            }
            if ((*tmp).by_uri_entry.rbe_left.is_null()
                || (*(*tmp).by_uri_entry.rbe_left).by_uri_entry.rbe_color == RB_BLACK)
                && ((*tmp).by_uri_entry.rbe_right.is_null()
                    || (*(*tmp).by_uri_entry.rbe_right).by_uri_entry.rbe_color == RB_BLACK)
            {
                (*tmp).by_uri_entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).by_uri_entry.rbe_parent;
            } else {
                if (*tmp).by_uri_entry.rbe_right.is_null()
                    || (*(*tmp).by_uri_entry.rbe_right).by_uri_entry.rbe_color == RB_BLACK
                {
                    let mut oleft: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
                    oleft = (*tmp).by_uri_entry.rbe_left;
                    if !oleft.is_null() {
                        (*oleft).by_uri_entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).by_uri_entry.rbe_color = RB_RED;
                    oleft = (*tmp).by_uri_entry.rbe_left;
                    (*tmp).by_uri_entry.rbe_left = (*oleft).by_uri_entry.rbe_right;
                    if !(*tmp).by_uri_entry.rbe_left.is_null() {
                        (*(*oleft).by_uri_entry.rbe_right).by_uri_entry.rbe_parent = tmp;
                    }
                    (*oleft).by_uri_entry.rbe_parent = (*tmp).by_uri_entry.rbe_parent;
                    if !(*oleft).by_uri_entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).by_uri_entry.rbe_parent).by_uri_entry.rbe_left {
                            (*(*tmp).by_uri_entry.rbe_parent).by_uri_entry.rbe_left = oleft;
                        } else {
                            (*(*tmp).by_uri_entry.rbe_parent).by_uri_entry.rbe_right = oleft;
                        }
                    } else {
                        (*head).rbh_root = oleft;
                    }
                    (*oleft).by_uri_entry.rbe_right = tmp;
                    (*tmp).by_uri_entry.rbe_parent = oleft;
                    !(*oleft).by_uri_entry.rbe_parent.is_null();
                    tmp = (*parent).by_uri_entry.rbe_right;
                }
                (*tmp).by_uri_entry.rbe_color = (*parent).by_uri_entry.rbe_color;
                (*parent).by_uri_entry.rbe_color = RB_BLACK;
                if !(*tmp).by_uri_entry.rbe_right.is_null() {
                    (*(*tmp).by_uri_entry.rbe_right).by_uri_entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).by_uri_entry.rbe_right;
                (*parent).by_uri_entry.rbe_right = (*tmp).by_uri_entry.rbe_left;
                if !(*parent).by_uri_entry.rbe_right.is_null() {
                    (*(*tmp).by_uri_entry.rbe_left).by_uri_entry.rbe_parent = parent;
                }
                (*tmp).by_uri_entry.rbe_parent = (*parent).by_uri_entry.rbe_parent;
                if !(*tmp).by_uri_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left {
                        (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).by_uri_entry.rbe_left = parent;
                (*parent).by_uri_entry.rbe_parent = tmp;
                !(*tmp).by_uri_entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        } else {
            tmp = (*parent).by_uri_entry.rbe_left;
            if (*tmp).by_uri_entry.rbe_color == RB_RED {
                (*tmp).by_uri_entry.rbe_color = RB_BLACK;
                (*parent).by_uri_entry.rbe_color = RB_RED;
                tmp = (*parent).by_uri_entry.rbe_left;
                (*parent).by_uri_entry.rbe_left = (*tmp).by_uri_entry.rbe_right;
                if !(*parent).by_uri_entry.rbe_left.is_null() {
                    (*(*tmp).by_uri_entry.rbe_right).by_uri_entry.rbe_parent = parent;
                }
                (*tmp).by_uri_entry.rbe_parent = (*parent).by_uri_entry.rbe_parent;
                if !(*tmp).by_uri_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left {
                        (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).by_uri_entry.rbe_right = parent;
                (*parent).by_uri_entry.rbe_parent = tmp;
                !(*tmp).by_uri_entry.rbe_parent.is_null();
                tmp = (*parent).by_uri_entry.rbe_left;
            }
            if ((*tmp).by_uri_entry.rbe_left.is_null()
                || (*(*tmp).by_uri_entry.rbe_left).by_uri_entry.rbe_color == RB_BLACK)
                && ((*tmp).by_uri_entry.rbe_right.is_null()
                    || (*(*tmp).by_uri_entry.rbe_right).by_uri_entry.rbe_color == RB_BLACK)
            {
                (*tmp).by_uri_entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).by_uri_entry.rbe_parent;
            } else {
                if (*tmp).by_uri_entry.rbe_left.is_null()
                    || (*(*tmp).by_uri_entry.rbe_left).by_uri_entry.rbe_color == RB_BLACK
                {
                    let mut oright: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
                    oright = (*tmp).by_uri_entry.rbe_right;
                    if !oright.is_null() {
                        (*oright).by_uri_entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).by_uri_entry.rbe_color = RB_RED;
                    oright = (*tmp).by_uri_entry.rbe_right;
                    (*tmp).by_uri_entry.rbe_right = (*oright).by_uri_entry.rbe_left;
                    if !(*tmp).by_uri_entry.rbe_right.is_null() {
                        (*(*oright).by_uri_entry.rbe_left).by_uri_entry.rbe_parent = tmp;
                    }
                    (*oright).by_uri_entry.rbe_parent = (*tmp).by_uri_entry.rbe_parent;
                    if !(*oright).by_uri_entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).by_uri_entry.rbe_parent).by_uri_entry.rbe_left {
                            (*(*tmp).by_uri_entry.rbe_parent).by_uri_entry.rbe_left = oright;
                        } else {
                            (*(*tmp).by_uri_entry.rbe_parent).by_uri_entry.rbe_right = oright;
                        }
                    } else {
                        (*head).rbh_root = oright;
                    }
                    (*oright).by_uri_entry.rbe_left = tmp;
                    (*tmp).by_uri_entry.rbe_parent = oright;
                    !(*oright).by_uri_entry.rbe_parent.is_null();
                    tmp = (*parent).by_uri_entry.rbe_left;
                }
                (*tmp).by_uri_entry.rbe_color = (*parent).by_uri_entry.rbe_color;
                (*parent).by_uri_entry.rbe_color = RB_BLACK;
                if !(*tmp).by_uri_entry.rbe_left.is_null() {
                    (*(*tmp).by_uri_entry.rbe_left).by_uri_entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).by_uri_entry.rbe_left;
                (*parent).by_uri_entry.rbe_left = (*tmp).by_uri_entry.rbe_right;
                if !(*parent).by_uri_entry.rbe_left.is_null() {
                    (*(*tmp).by_uri_entry.rbe_right).by_uri_entry.rbe_parent = parent;
                }
                (*tmp).by_uri_entry.rbe_parent = (*parent).by_uri_entry.rbe_parent;
                if !(*tmp).by_uri_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left {
                        (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).by_uri_entry.rbe_right = parent;
                (*parent).by_uri_entry.rbe_parent = tmp;
                !(*tmp).by_uri_entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        }
    }
    if !elm.is_null() {
        (*elm).by_uri_entry.rbe_color = RB_BLACK;
    }
}
unsafe extern "C" fn hyperlinks_by_uri_tree_RB_REMOVE(
    mut head: *mut hyperlinks_by_uri_tree,
    mut elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    let mut current_block: u64;
    let mut child: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut parent: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut old: *mut hyperlinks_uri = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).by_uri_entry.rbe_left.is_null() {
        child = (*elm).by_uri_entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).by_uri_entry.rbe_right.is_null() {
        child = (*elm).by_uri_entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
        elm = (*elm).by_uri_entry.rbe_right;
        loop {
            left = (*elm).by_uri_entry.rbe_left;
            if left.is_null() {
                break;
            }
            elm = left;
        }
        child = (*elm).by_uri_entry.rbe_right;
        parent = (*elm).by_uri_entry.rbe_parent;
        color = (*elm).by_uri_entry.rbe_color;
        if !child.is_null() {
            (*child).by_uri_entry.rbe_parent = parent;
        }
        if !parent.is_null() {
            if (*parent).by_uri_entry.rbe_left == elm {
                (*parent).by_uri_entry.rbe_left = child;
            } else {
                (*parent).by_uri_entry.rbe_right = child;
            }
        } else {
            (*head).rbh_root = child;
        }
        if (*elm).by_uri_entry.rbe_parent == old {
            parent = elm;
        }
        (*elm).by_uri_entry = (*old).by_uri_entry;
        if !(*old).by_uri_entry.rbe_parent.is_null() {
            if (*(*old).by_uri_entry.rbe_parent).by_uri_entry.rbe_left == old {
                (*(*old).by_uri_entry.rbe_parent).by_uri_entry.rbe_left = elm;
            } else {
                (*(*old).by_uri_entry.rbe_parent).by_uri_entry.rbe_right = elm;
            }
        } else {
            (*head).rbh_root = elm;
        }
        (*(*old).by_uri_entry.rbe_left).by_uri_entry.rbe_parent = elm;
        if !(*old).by_uri_entry.rbe_right.is_null() {
            (*(*old).by_uri_entry.rbe_right).by_uri_entry.rbe_parent = elm;
        }
        if !parent.is_null() {
            left = parent;
            loop {
                left = (*left).by_uri_entry.rbe_parent;
                if left.is_null() {
                    break;
                }
            }
        }
        current_block = 11576451286340114506;
    }
    match current_block {
        7245201122033322888 => {
            parent = (*elm).by_uri_entry.rbe_parent;
            color = (*elm).by_uri_entry.rbe_color;
            if !child.is_null() {
                (*child).by_uri_entry.rbe_parent = parent;
            }
            if !parent.is_null() {
                if (*parent).by_uri_entry.rbe_left == elm {
                    (*parent).by_uri_entry.rbe_left = child;
                } else {
                    (*parent).by_uri_entry.rbe_right = child;
                }
            } else {
                (*head).rbh_root = child;
            }
        }
        _ => {}
    }
    if color == RB_BLACK {
        hyperlinks_by_uri_tree_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn hyperlinks_by_uri_tree_RB_FIND(
    mut head: *mut hyperlinks_by_uri_tree,
    mut elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    let mut tmp: *mut hyperlinks_uri = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = hyperlinks_by_uri_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).by_uri_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).by_uri_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<hyperlinks_uri>();
}
unsafe extern "C" fn hyperlinks_by_uri_tree_RB_INSERT_COLOR(
    mut head: *mut hyperlinks_by_uri_tree,
    mut elm: *mut hyperlinks_uri,
) {
    let mut parent: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut gparent: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut tmp: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    loop {
        parent = (*elm).by_uri_entry.rbe_parent;
        if !(!parent.is_null() && (*parent).by_uri_entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).by_uri_entry.rbe_parent;
        if parent == (*gparent).by_uri_entry.rbe_left {
            tmp = (*gparent).by_uri_entry.rbe_right;
            if !tmp.is_null() && (*tmp).by_uri_entry.rbe_color == RB_RED {
                (*tmp).by_uri_entry.rbe_color = RB_BLACK;
                (*parent).by_uri_entry.rbe_color = RB_BLACK;
                (*gparent).by_uri_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).by_uri_entry.rbe_right == elm {
                    tmp = (*parent).by_uri_entry.rbe_right;
                    (*parent).by_uri_entry.rbe_right = (*tmp).by_uri_entry.rbe_left;
                    if !(*parent).by_uri_entry.rbe_right.is_null() {
                        (*(*tmp).by_uri_entry.rbe_left).by_uri_entry.rbe_parent = parent;
                    }
                    (*tmp).by_uri_entry.rbe_parent = (*parent).by_uri_entry.rbe_parent;
                    if !(*tmp).by_uri_entry.rbe_parent.is_null() {
                        if parent == (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left {
                            (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left = tmp;
                        } else {
                            (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).by_uri_entry.rbe_left = parent;
                    (*parent).by_uri_entry.rbe_parent = tmp;
                    !(*tmp).by_uri_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).by_uri_entry.rbe_color = RB_BLACK;
                (*gparent).by_uri_entry.rbe_color = RB_RED;
                tmp = (*gparent).by_uri_entry.rbe_left;
                (*gparent).by_uri_entry.rbe_left = (*tmp).by_uri_entry.rbe_right;
                if !(*gparent).by_uri_entry.rbe_left.is_null() {
                    (*(*tmp).by_uri_entry.rbe_right).by_uri_entry.rbe_parent = gparent;
                }
                (*tmp).by_uri_entry.rbe_parent = (*gparent).by_uri_entry.rbe_parent;
                if !(*tmp).by_uri_entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left {
                        (*(*gparent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).by_uri_entry.rbe_parent).by_uri_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).by_uri_entry.rbe_right = gparent;
                (*gparent).by_uri_entry.rbe_parent = tmp;
                !(*tmp).by_uri_entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).by_uri_entry.rbe_left;
            if !tmp.is_null() && (*tmp).by_uri_entry.rbe_color == RB_RED {
                (*tmp).by_uri_entry.rbe_color = RB_BLACK;
                (*parent).by_uri_entry.rbe_color = RB_BLACK;
                (*gparent).by_uri_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).by_uri_entry.rbe_left == elm {
                    tmp = (*parent).by_uri_entry.rbe_left;
                    (*parent).by_uri_entry.rbe_left = (*tmp).by_uri_entry.rbe_right;
                    if !(*parent).by_uri_entry.rbe_left.is_null() {
                        (*(*tmp).by_uri_entry.rbe_right).by_uri_entry.rbe_parent = parent;
                    }
                    (*tmp).by_uri_entry.rbe_parent = (*parent).by_uri_entry.rbe_parent;
                    if !(*tmp).by_uri_entry.rbe_parent.is_null() {
                        if parent == (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left {
                            (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left = tmp;
                        } else {
                            (*(*parent).by_uri_entry.rbe_parent).by_uri_entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).by_uri_entry.rbe_right = parent;
                    (*parent).by_uri_entry.rbe_parent = tmp;
                    !(*tmp).by_uri_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).by_uri_entry.rbe_color = RB_BLACK;
                (*gparent).by_uri_entry.rbe_color = RB_RED;
                tmp = (*gparent).by_uri_entry.rbe_right;
                (*gparent).by_uri_entry.rbe_right = (*tmp).by_uri_entry.rbe_left;
                if !(*gparent).by_uri_entry.rbe_right.is_null() {
                    (*(*tmp).by_uri_entry.rbe_left).by_uri_entry.rbe_parent = gparent;
                }
                (*tmp).by_uri_entry.rbe_parent = (*gparent).by_uri_entry.rbe_parent;
                if !(*tmp).by_uri_entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left {
                        (*(*gparent).by_uri_entry.rbe_parent).by_uri_entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).by_uri_entry.rbe_parent).by_uri_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).by_uri_entry.rbe_left = gparent;
                (*gparent).by_uri_entry.rbe_parent = tmp;
                !(*tmp).by_uri_entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).by_uri_entry.rbe_color = RB_BLACK;
}
unsafe extern "C" fn hyperlinks_by_uri_tree_RB_INSERT(
    mut head: *mut hyperlinks_by_uri_tree,
    mut elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    let mut tmp: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut parent: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = hyperlinks_by_uri_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).by_uri_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).by_uri_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).by_uri_entry.rbe_parent = parent;
    (*elm).by_uri_entry.rbe_right = ::core::ptr::null_mut::<hyperlinks_uri>();
    (*elm).by_uri_entry.rbe_left = (*elm).by_uri_entry.rbe_right;
    (*elm).by_uri_entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).by_uri_entry.rbe_left = elm;
        } else {
            (*parent).by_uri_entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    hyperlinks_by_uri_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<hyperlinks_uri>();
}
unsafe extern "C" fn hyperlinks_by_inner_cmp(
    mut left: *mut hyperlinks_uri,
    mut right: *mut hyperlinks_uri,
) -> ::core::ffi::c_int {
    return (*left).inner.wrapping_sub((*right).inner) as ::core::ffi::c_int;
}
unsafe extern "C" fn hyperlinks_by_inner_tree_RB_INSERT(
    mut head: *mut hyperlinks_by_inner_tree,
    mut elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    let mut tmp: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut parent: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = hyperlinks_by_inner_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).by_inner_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).by_inner_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).by_inner_entry.rbe_parent = parent;
    (*elm).by_inner_entry.rbe_right = ::core::ptr::null_mut::<hyperlinks_uri>();
    (*elm).by_inner_entry.rbe_left = (*elm).by_inner_entry.rbe_right;
    (*elm).by_inner_entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).by_inner_entry.rbe_left = elm;
        } else {
            (*parent).by_inner_entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    hyperlinks_by_inner_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<hyperlinks_uri>();
}
unsafe extern "C" fn hyperlinks_by_inner_tree_RB_MINMAX(
    mut head: *mut hyperlinks_by_inner_tree,
    mut val: ::core::ffi::c_int,
) -> *mut hyperlinks_uri {
    let mut tmp: *mut hyperlinks_uri = (*head).rbh_root;
    let mut parent: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    while !tmp.is_null() {
        parent = tmp;
        if val < 0 as ::core::ffi::c_int {
            tmp = (*tmp).by_inner_entry.rbe_left;
        } else {
            tmp = (*tmp).by_inner_entry.rbe_right;
        }
    }
    return parent;
}
unsafe extern "C" fn hyperlinks_by_inner_tree_RB_REMOVE_COLOR(
    mut head: *mut hyperlinks_by_inner_tree,
    mut parent: *mut hyperlinks_uri,
    mut elm: *mut hyperlinks_uri,
) {
    let mut tmp: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    while (elm.is_null() || (*elm).by_inner_entry.rbe_color == RB_BLACK) && elm != (*head).rbh_root
    {
        if (*parent).by_inner_entry.rbe_left == elm {
            tmp = (*parent).by_inner_entry.rbe_right;
            if (*tmp).by_inner_entry.rbe_color == RB_RED {
                (*tmp).by_inner_entry.rbe_color = RB_BLACK;
                (*parent).by_inner_entry.rbe_color = RB_RED;
                tmp = (*parent).by_inner_entry.rbe_right;
                (*parent).by_inner_entry.rbe_right = (*tmp).by_inner_entry.rbe_left;
                if !(*parent).by_inner_entry.rbe_right.is_null() {
                    (*(*tmp).by_inner_entry.rbe_left).by_inner_entry.rbe_parent = parent;
                }
                (*tmp).by_inner_entry.rbe_parent = (*parent).by_inner_entry.rbe_parent;
                if !(*tmp).by_inner_entry.rbe_parent.is_null() {
                    if parent
                        == (*(*parent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_left
                    {
                        (*(*parent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_left = tmp;
                    } else {
                        (*(*parent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).by_inner_entry.rbe_left = parent;
                (*parent).by_inner_entry.rbe_parent = tmp;
                !(*tmp).by_inner_entry.rbe_parent.is_null();
                tmp = (*parent).by_inner_entry.rbe_right;
            }
            if ((*tmp).by_inner_entry.rbe_left.is_null()
                || (*(*tmp).by_inner_entry.rbe_left).by_inner_entry.rbe_color == RB_BLACK)
                && ((*tmp).by_inner_entry.rbe_right.is_null()
                    || (*(*tmp).by_inner_entry.rbe_right).by_inner_entry.rbe_color == RB_BLACK)
            {
                (*tmp).by_inner_entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).by_inner_entry.rbe_parent;
            } else {
                if (*tmp).by_inner_entry.rbe_right.is_null()
                    || (*(*tmp).by_inner_entry.rbe_right).by_inner_entry.rbe_color == RB_BLACK
                {
                    let mut oleft: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
                    oleft = (*tmp).by_inner_entry.rbe_left;
                    if !oleft.is_null() {
                        (*oleft).by_inner_entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).by_inner_entry.rbe_color = RB_RED;
                    oleft = (*tmp).by_inner_entry.rbe_left;
                    (*tmp).by_inner_entry.rbe_left = (*oleft).by_inner_entry.rbe_right;
                    if !(*tmp).by_inner_entry.rbe_left.is_null() {
                        (*(*oleft).by_inner_entry.rbe_right)
                            .by_inner_entry
                            .rbe_parent = tmp;
                    }
                    (*oleft).by_inner_entry.rbe_parent = (*tmp).by_inner_entry.rbe_parent;
                    if !(*oleft).by_inner_entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).by_inner_entry.rbe_parent).by_inner_entry.rbe_left {
                            (*(*tmp).by_inner_entry.rbe_parent).by_inner_entry.rbe_left = oleft;
                        } else {
                            (*(*tmp).by_inner_entry.rbe_parent).by_inner_entry.rbe_right = oleft;
                        }
                    } else {
                        (*head).rbh_root = oleft;
                    }
                    (*oleft).by_inner_entry.rbe_right = tmp;
                    (*tmp).by_inner_entry.rbe_parent = oleft;
                    !(*oleft).by_inner_entry.rbe_parent.is_null();
                    tmp = (*parent).by_inner_entry.rbe_right;
                }
                (*tmp).by_inner_entry.rbe_color = (*parent).by_inner_entry.rbe_color;
                (*parent).by_inner_entry.rbe_color = RB_BLACK;
                if !(*tmp).by_inner_entry.rbe_right.is_null() {
                    (*(*tmp).by_inner_entry.rbe_right).by_inner_entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).by_inner_entry.rbe_right;
                (*parent).by_inner_entry.rbe_right = (*tmp).by_inner_entry.rbe_left;
                if !(*parent).by_inner_entry.rbe_right.is_null() {
                    (*(*tmp).by_inner_entry.rbe_left).by_inner_entry.rbe_parent = parent;
                }
                (*tmp).by_inner_entry.rbe_parent = (*parent).by_inner_entry.rbe_parent;
                if !(*tmp).by_inner_entry.rbe_parent.is_null() {
                    if parent
                        == (*(*parent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_left
                    {
                        (*(*parent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_left = tmp;
                    } else {
                        (*(*parent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).by_inner_entry.rbe_left = parent;
                (*parent).by_inner_entry.rbe_parent = tmp;
                !(*tmp).by_inner_entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        } else {
            tmp = (*parent).by_inner_entry.rbe_left;
            if (*tmp).by_inner_entry.rbe_color == RB_RED {
                (*tmp).by_inner_entry.rbe_color = RB_BLACK;
                (*parent).by_inner_entry.rbe_color = RB_RED;
                tmp = (*parent).by_inner_entry.rbe_left;
                (*parent).by_inner_entry.rbe_left = (*tmp).by_inner_entry.rbe_right;
                if !(*parent).by_inner_entry.rbe_left.is_null() {
                    (*(*tmp).by_inner_entry.rbe_right).by_inner_entry.rbe_parent = parent;
                }
                (*tmp).by_inner_entry.rbe_parent = (*parent).by_inner_entry.rbe_parent;
                if !(*tmp).by_inner_entry.rbe_parent.is_null() {
                    if parent
                        == (*(*parent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_left
                    {
                        (*(*parent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_left = tmp;
                    } else {
                        (*(*parent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).by_inner_entry.rbe_right = parent;
                (*parent).by_inner_entry.rbe_parent = tmp;
                !(*tmp).by_inner_entry.rbe_parent.is_null();
                tmp = (*parent).by_inner_entry.rbe_left;
            }
            if ((*tmp).by_inner_entry.rbe_left.is_null()
                || (*(*tmp).by_inner_entry.rbe_left).by_inner_entry.rbe_color == RB_BLACK)
                && ((*tmp).by_inner_entry.rbe_right.is_null()
                    || (*(*tmp).by_inner_entry.rbe_right).by_inner_entry.rbe_color == RB_BLACK)
            {
                (*tmp).by_inner_entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).by_inner_entry.rbe_parent;
            } else {
                if (*tmp).by_inner_entry.rbe_left.is_null()
                    || (*(*tmp).by_inner_entry.rbe_left).by_inner_entry.rbe_color == RB_BLACK
                {
                    let mut oright: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
                    oright = (*tmp).by_inner_entry.rbe_right;
                    if !oright.is_null() {
                        (*oright).by_inner_entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).by_inner_entry.rbe_color = RB_RED;
                    oright = (*tmp).by_inner_entry.rbe_right;
                    (*tmp).by_inner_entry.rbe_right = (*oright).by_inner_entry.rbe_left;
                    if !(*tmp).by_inner_entry.rbe_right.is_null() {
                        (*(*oright).by_inner_entry.rbe_left)
                            .by_inner_entry
                            .rbe_parent = tmp;
                    }
                    (*oright).by_inner_entry.rbe_parent = (*tmp).by_inner_entry.rbe_parent;
                    if !(*oright).by_inner_entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).by_inner_entry.rbe_parent).by_inner_entry.rbe_left {
                            (*(*tmp).by_inner_entry.rbe_parent).by_inner_entry.rbe_left = oright;
                        } else {
                            (*(*tmp).by_inner_entry.rbe_parent).by_inner_entry.rbe_right = oright;
                        }
                    } else {
                        (*head).rbh_root = oright;
                    }
                    (*oright).by_inner_entry.rbe_left = tmp;
                    (*tmp).by_inner_entry.rbe_parent = oright;
                    !(*oright).by_inner_entry.rbe_parent.is_null();
                    tmp = (*parent).by_inner_entry.rbe_left;
                }
                (*tmp).by_inner_entry.rbe_color = (*parent).by_inner_entry.rbe_color;
                (*parent).by_inner_entry.rbe_color = RB_BLACK;
                if !(*tmp).by_inner_entry.rbe_left.is_null() {
                    (*(*tmp).by_inner_entry.rbe_left).by_inner_entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).by_inner_entry.rbe_left;
                (*parent).by_inner_entry.rbe_left = (*tmp).by_inner_entry.rbe_right;
                if !(*parent).by_inner_entry.rbe_left.is_null() {
                    (*(*tmp).by_inner_entry.rbe_right).by_inner_entry.rbe_parent = parent;
                }
                (*tmp).by_inner_entry.rbe_parent = (*parent).by_inner_entry.rbe_parent;
                if !(*tmp).by_inner_entry.rbe_parent.is_null() {
                    if parent
                        == (*(*parent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_left
                    {
                        (*(*parent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_left = tmp;
                    } else {
                        (*(*parent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).by_inner_entry.rbe_right = parent;
                (*parent).by_inner_entry.rbe_parent = tmp;
                !(*tmp).by_inner_entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        }
    }
    if !elm.is_null() {
        (*elm).by_inner_entry.rbe_color = RB_BLACK;
    }
}
unsafe extern "C" fn hyperlinks_by_inner_tree_RB_INSERT_COLOR(
    mut head: *mut hyperlinks_by_inner_tree,
    mut elm: *mut hyperlinks_uri,
) {
    let mut parent: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut gparent: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut tmp: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    loop {
        parent = (*elm).by_inner_entry.rbe_parent;
        if !(!parent.is_null() && (*parent).by_inner_entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).by_inner_entry.rbe_parent;
        if parent == (*gparent).by_inner_entry.rbe_left {
            tmp = (*gparent).by_inner_entry.rbe_right;
            if !tmp.is_null() && (*tmp).by_inner_entry.rbe_color == RB_RED {
                (*tmp).by_inner_entry.rbe_color = RB_BLACK;
                (*parent).by_inner_entry.rbe_color = RB_BLACK;
                (*gparent).by_inner_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).by_inner_entry.rbe_right == elm {
                    tmp = (*parent).by_inner_entry.rbe_right;
                    (*parent).by_inner_entry.rbe_right = (*tmp).by_inner_entry.rbe_left;
                    if !(*parent).by_inner_entry.rbe_right.is_null() {
                        (*(*tmp).by_inner_entry.rbe_left).by_inner_entry.rbe_parent = parent;
                    }
                    (*tmp).by_inner_entry.rbe_parent = (*parent).by_inner_entry.rbe_parent;
                    if !(*tmp).by_inner_entry.rbe_parent.is_null() {
                        if parent
                            == (*(*parent).by_inner_entry.rbe_parent)
                                .by_inner_entry
                                .rbe_left
                        {
                            (*(*parent).by_inner_entry.rbe_parent)
                                .by_inner_entry
                                .rbe_left = tmp;
                        } else {
                            (*(*parent).by_inner_entry.rbe_parent)
                                .by_inner_entry
                                .rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).by_inner_entry.rbe_left = parent;
                    (*parent).by_inner_entry.rbe_parent = tmp;
                    !(*tmp).by_inner_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).by_inner_entry.rbe_color = RB_BLACK;
                (*gparent).by_inner_entry.rbe_color = RB_RED;
                tmp = (*gparent).by_inner_entry.rbe_left;
                (*gparent).by_inner_entry.rbe_left = (*tmp).by_inner_entry.rbe_right;
                if !(*gparent).by_inner_entry.rbe_left.is_null() {
                    (*(*tmp).by_inner_entry.rbe_right).by_inner_entry.rbe_parent = gparent;
                }
                (*tmp).by_inner_entry.rbe_parent = (*gparent).by_inner_entry.rbe_parent;
                if !(*tmp).by_inner_entry.rbe_parent.is_null() {
                    if gparent
                        == (*(*gparent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_left
                    {
                        (*(*gparent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_left = tmp;
                    } else {
                        (*(*gparent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).by_inner_entry.rbe_right = gparent;
                (*gparent).by_inner_entry.rbe_parent = tmp;
                !(*tmp).by_inner_entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).by_inner_entry.rbe_left;
            if !tmp.is_null() && (*tmp).by_inner_entry.rbe_color == RB_RED {
                (*tmp).by_inner_entry.rbe_color = RB_BLACK;
                (*parent).by_inner_entry.rbe_color = RB_BLACK;
                (*gparent).by_inner_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).by_inner_entry.rbe_left == elm {
                    tmp = (*parent).by_inner_entry.rbe_left;
                    (*parent).by_inner_entry.rbe_left = (*tmp).by_inner_entry.rbe_right;
                    if !(*parent).by_inner_entry.rbe_left.is_null() {
                        (*(*tmp).by_inner_entry.rbe_right).by_inner_entry.rbe_parent = parent;
                    }
                    (*tmp).by_inner_entry.rbe_parent = (*parent).by_inner_entry.rbe_parent;
                    if !(*tmp).by_inner_entry.rbe_parent.is_null() {
                        if parent
                            == (*(*parent).by_inner_entry.rbe_parent)
                                .by_inner_entry
                                .rbe_left
                        {
                            (*(*parent).by_inner_entry.rbe_parent)
                                .by_inner_entry
                                .rbe_left = tmp;
                        } else {
                            (*(*parent).by_inner_entry.rbe_parent)
                                .by_inner_entry
                                .rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).by_inner_entry.rbe_right = parent;
                    (*parent).by_inner_entry.rbe_parent = tmp;
                    !(*tmp).by_inner_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).by_inner_entry.rbe_color = RB_BLACK;
                (*gparent).by_inner_entry.rbe_color = RB_RED;
                tmp = (*gparent).by_inner_entry.rbe_right;
                (*gparent).by_inner_entry.rbe_right = (*tmp).by_inner_entry.rbe_left;
                if !(*gparent).by_inner_entry.rbe_right.is_null() {
                    (*(*tmp).by_inner_entry.rbe_left).by_inner_entry.rbe_parent = gparent;
                }
                (*tmp).by_inner_entry.rbe_parent = (*gparent).by_inner_entry.rbe_parent;
                if !(*tmp).by_inner_entry.rbe_parent.is_null() {
                    if gparent
                        == (*(*gparent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_left
                    {
                        (*(*gparent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_left = tmp;
                    } else {
                        (*(*gparent).by_inner_entry.rbe_parent)
                            .by_inner_entry
                            .rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).by_inner_entry.rbe_left = gparent;
                (*gparent).by_inner_entry.rbe_parent = tmp;
                !(*tmp).by_inner_entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).by_inner_entry.rbe_color = RB_BLACK;
}
unsafe extern "C" fn hyperlinks_by_inner_tree_RB_REMOVE(
    mut head: *mut hyperlinks_by_inner_tree,
    mut elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    let mut current_block: u64;
    let mut child: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut parent: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut old: *mut hyperlinks_uri = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).by_inner_entry.rbe_left.is_null() {
        child = (*elm).by_inner_entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).by_inner_entry.rbe_right.is_null() {
        child = (*elm).by_inner_entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
        elm = (*elm).by_inner_entry.rbe_right;
        loop {
            left = (*elm).by_inner_entry.rbe_left;
            if left.is_null() {
                break;
            }
            elm = left;
        }
        child = (*elm).by_inner_entry.rbe_right;
        parent = (*elm).by_inner_entry.rbe_parent;
        color = (*elm).by_inner_entry.rbe_color;
        if !child.is_null() {
            (*child).by_inner_entry.rbe_parent = parent;
        }
        if !parent.is_null() {
            if (*parent).by_inner_entry.rbe_left == elm {
                (*parent).by_inner_entry.rbe_left = child;
            } else {
                (*parent).by_inner_entry.rbe_right = child;
            }
        } else {
            (*head).rbh_root = child;
        }
        if (*elm).by_inner_entry.rbe_parent == old {
            parent = elm;
        }
        (*elm).by_inner_entry = (*old).by_inner_entry;
        if !(*old).by_inner_entry.rbe_parent.is_null() {
            if (*(*old).by_inner_entry.rbe_parent).by_inner_entry.rbe_left == old {
                (*(*old).by_inner_entry.rbe_parent).by_inner_entry.rbe_left = elm;
            } else {
                (*(*old).by_inner_entry.rbe_parent).by_inner_entry.rbe_right = elm;
            }
        } else {
            (*head).rbh_root = elm;
        }
        (*(*old).by_inner_entry.rbe_left).by_inner_entry.rbe_parent = elm;
        if !(*old).by_inner_entry.rbe_right.is_null() {
            (*(*old).by_inner_entry.rbe_right).by_inner_entry.rbe_parent = elm;
        }
        if !parent.is_null() {
            left = parent;
            loop {
                left = (*left).by_inner_entry.rbe_parent;
                if left.is_null() {
                    break;
                }
            }
        }
        current_block = 17539538398936311875;
    }
    match current_block {
        7245201122033322888 => {
            parent = (*elm).by_inner_entry.rbe_parent;
            color = (*elm).by_inner_entry.rbe_color;
            if !child.is_null() {
                (*child).by_inner_entry.rbe_parent = parent;
            }
            if !parent.is_null() {
                if (*parent).by_inner_entry.rbe_left == elm {
                    (*parent).by_inner_entry.rbe_left = child;
                } else {
                    (*parent).by_inner_entry.rbe_right = child;
                }
            } else {
                (*head).rbh_root = child;
            }
        }
        _ => {}
    }
    if color == RB_BLACK {
        hyperlinks_by_inner_tree_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn hyperlinks_by_inner_tree_RB_NEXT(
    mut elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    if !(*elm).by_inner_entry.rbe_right.is_null() {
        elm = (*elm).by_inner_entry.rbe_right;
        while !(*elm).by_inner_entry.rbe_left.is_null() {
            elm = (*elm).by_inner_entry.rbe_left;
        }
    } else if !(*elm).by_inner_entry.rbe_parent.is_null()
        && elm == (*(*elm).by_inner_entry.rbe_parent).by_inner_entry.rbe_left
    {
        elm = (*elm).by_inner_entry.rbe_parent;
    } else {
        while !(*elm).by_inner_entry.rbe_parent.is_null()
            && elm == (*(*elm).by_inner_entry.rbe_parent).by_inner_entry.rbe_right
        {
            elm = (*elm).by_inner_entry.rbe_parent;
        }
        elm = (*elm).by_inner_entry.rbe_parent;
    }
    return elm;
}
unsafe extern "C" fn hyperlinks_by_inner_tree_RB_FIND(
    mut head: *mut hyperlinks_by_inner_tree,
    mut elm: *mut hyperlinks_uri,
) -> *mut hyperlinks_uri {
    let mut tmp: *mut hyperlinks_uri = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = hyperlinks_by_inner_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).by_inner_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).by_inner_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<hyperlinks_uri>();
}
unsafe extern "C" fn hyperlinks_remove(mut hlu: *mut hyperlinks_uri) {
    let mut hl: *mut hyperlinks = (*hlu).tree;
    if !(*hlu).list_entry.tqe_next.is_null() {
        (*(*hlu).list_entry.tqe_next).list_entry.tqe_prev = (*hlu).list_entry.tqe_prev;
    } else {
        global_hyperlinks.tqh_last = (*hlu).list_entry.tqe_prev;
    }
    *(*hlu).list_entry.tqe_prev = (*hlu).list_entry.tqe_next;
    global_hyperlinks_count = global_hyperlinks_count.wrapping_sub(1);
    hyperlinks_by_inner_tree_RB_REMOVE(&raw mut (*hl).by_inner, hlu);
    hyperlinks_by_uri_tree_RB_REMOVE(&raw mut (*hl).by_uri, hlu);
    free((*hlu).internal_id as *mut ::core::ffi::c_void);
    free((*hlu).external_id as *mut ::core::ffi::c_void);
    free((*hlu).uri as *mut ::core::ffi::c_void);
    free(hlu as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn hyperlinks_put(
    mut hl: *mut hyperlinks,
    mut uri_in: *const ::core::ffi::c_char,
    mut internal_id_in: *const ::core::ffi::c_char,
) -> u_int {
    let mut find: hyperlinks_uri = hyperlinks_uri {
        tree: ::core::ptr::null_mut::<hyperlinks>(),
        inner: 0,
        internal_id: ::core::ptr::null::<::core::ffi::c_char>(),
        external_id: ::core::ptr::null::<::core::ffi::c_char>(),
        uri: ::core::ptr::null::<::core::ffi::c_char>(),
        list_entry: hyperlink_list_entry {
            tqe_next: ::core::ptr::null_mut::<hyperlinks_uri>(),
            tqe_prev: ::core::ptr::null_mut::<*mut hyperlinks_uri>(),
        },
        by_inner_entry: hyperlink_inner_entry {
            rbe_left: ::core::ptr::null_mut::<hyperlinks_uri>(),
            rbe_right: ::core::ptr::null_mut::<hyperlinks_uri>(),
            rbe_parent: ::core::ptr::null_mut::<hyperlinks_uri>(),
            rbe_color: 0,
        },
        by_uri_entry: hyperlink_uri_entry {
            rbe_left: ::core::ptr::null_mut::<hyperlinks_uri>(),
            rbe_right: ::core::ptr::null_mut::<hyperlinks_uri>(),
            rbe_parent: ::core::ptr::null_mut::<hyperlinks_uri>(),
            rbe_color: 0,
        },
    };
    let mut hlu: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut uri: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut internal_id: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut external_id: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if internal_id_in.is_null() {
        internal_id_in = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    utf8_stravis(&raw mut uri, uri_in, VIS_OCTAL | VIS_CSTYLE);
    if strlen(uri) > MAX_HYPERLINK_URI as size_t {
        free(uri as *mut ::core::ffi::c_void);
        return 0 as u_int;
    }
    utf8_stravis(&raw mut internal_id, internal_id_in, VIS_OCTAL | VIS_CSTYLE);
    if *internal_id as ::core::ffi::c_int != '\0' as i32 {
        find.uri = uri;
        find.internal_id = internal_id;
        hlu = hyperlinks_by_uri_tree_RB_FIND(&raw mut (*hl).by_uri, &raw mut find);
        if !hlu.is_null() {
            free(uri as *mut ::core::ffi::c_void);
            free(internal_id as *mut ::core::ffi::c_void);
            return (*hlu).inner;
        }
    }
    let fresh0 = hyperlinks_next_external_id;
    hyperlinks_next_external_id = hyperlinks_next_external_id + 1;
    xasprintf(
        &raw mut external_id,
        b"tmux%llX\0" as *const u8 as *const ::core::ffi::c_char,
        fresh0,
    );
    hlu = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<hyperlinks_uri>() as size_t,
    ) as *mut hyperlinks_uri;
    let fresh1 = (*hl).next_inner;
    (*hl).next_inner = (*hl).next_inner.wrapping_add(1);
    (*hlu).inner = fresh1;
    (*hlu).internal_id = internal_id;
    (*hlu).external_id = external_id;
    (*hlu).uri = uri;
    (*hlu).tree = hl;
    hyperlinks_by_uri_tree_RB_INSERT(&raw mut (*hl).by_uri, hlu);
    hyperlinks_by_inner_tree_RB_INSERT(&raw mut (*hl).by_inner, hlu);
    (*hlu).list_entry.tqe_next = ::core::ptr::null_mut::<hyperlinks_uri>();
    (*hlu).list_entry.tqe_prev = global_hyperlinks.tqh_last;
    *global_hyperlinks.tqh_last = hlu;
    global_hyperlinks.tqh_last = &raw mut (*hlu).list_entry.tqe_next;
    global_hyperlinks_count = global_hyperlinks_count.wrapping_add(1);
    if global_hyperlinks_count == MAX_HYPERLINKS as u_int {
        hyperlinks_remove(global_hyperlinks.tqh_first);
    }
    return (*hlu).inner;
}
#[no_mangle]
pub unsafe extern "C" fn hyperlinks_get(
    mut hl: *mut hyperlinks,
    mut inner: u_int,
    mut uri_out: *mut *const ::core::ffi::c_char,
    mut internal_id_out: *mut *const ::core::ffi::c_char,
    mut external_id_out: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut find: hyperlinks_uri = hyperlinks_uri {
        tree: ::core::ptr::null_mut::<hyperlinks>(),
        inner: 0,
        internal_id: ::core::ptr::null::<::core::ffi::c_char>(),
        external_id: ::core::ptr::null::<::core::ffi::c_char>(),
        uri: ::core::ptr::null::<::core::ffi::c_char>(),
        list_entry: hyperlink_list_entry {
            tqe_next: ::core::ptr::null_mut::<hyperlinks_uri>(),
            tqe_prev: ::core::ptr::null_mut::<*mut hyperlinks_uri>(),
        },
        by_inner_entry: hyperlink_inner_entry {
            rbe_left: ::core::ptr::null_mut::<hyperlinks_uri>(),
            rbe_right: ::core::ptr::null_mut::<hyperlinks_uri>(),
            rbe_parent: ::core::ptr::null_mut::<hyperlinks_uri>(),
            rbe_color: 0,
        },
        by_uri_entry: hyperlink_uri_entry {
            rbe_left: ::core::ptr::null_mut::<hyperlinks_uri>(),
            rbe_right: ::core::ptr::null_mut::<hyperlinks_uri>(),
            rbe_parent: ::core::ptr::null_mut::<hyperlinks_uri>(),
            rbe_color: 0,
        },
    };
    let mut hlu: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    find.inner = inner;
    hlu = hyperlinks_by_inner_tree_RB_FIND(&raw mut (*hl).by_inner, &raw mut find);
    if hlu.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if !internal_id_out.is_null() {
        *internal_id_out = (*hlu).internal_id;
    }
    if !external_id_out.is_null() {
        *external_id_out = (*hlu).external_id;
    }
    *uri_out = (*hlu).uri;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn hyperlinks_init() -> *mut hyperlinks {
    let mut hl: *mut hyperlinks = ::core::ptr::null_mut::<hyperlinks>();
    hl = xcalloc(1 as size_t, ::core::mem::size_of::<hyperlinks>() as size_t) as *mut hyperlinks;
    (*hl).next_inner = 1 as u_int;
    (*hl).by_uri.rbh_root = ::core::ptr::null_mut::<hyperlinks_uri>();
    (*hl).by_inner.rbh_root = ::core::ptr::null_mut::<hyperlinks_uri>();
    (*hl).references = 1 as u_int;
    return hl;
}
#[no_mangle]
pub unsafe extern "C" fn hyperlinks_copy(mut hl: *mut hyperlinks) -> *mut hyperlinks {
    (*hl).references = (*hl).references.wrapping_add(1);
    return hl;
}
#[no_mangle]
pub unsafe extern "C" fn hyperlinks_reset(mut hl: *mut hyperlinks) {
    let mut hlu: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    let mut hlu1: *mut hyperlinks_uri = ::core::ptr::null_mut::<hyperlinks_uri>();
    hlu = hyperlinks_by_inner_tree_RB_MINMAX(&raw mut (*hl).by_inner, RB_NEGINF);
    while !hlu.is_null() && {
        hlu1 = hyperlinks_by_inner_tree_RB_NEXT(hlu);
        1 as ::core::ffi::c_int != 0
    } {
        hyperlinks_remove(hlu);
        hlu = hlu1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn hyperlinks_free(mut hl: *mut hyperlinks) {
    (*hl).references = (*hl).references.wrapping_sub(1);
    if (*hl).references == 0 as u_int {
        hyperlinks_reset(hl);
        free(hl as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn run_static_initializers() {
    global_hyperlinks = hyperlinks_list {
        tqh_first: ::core::ptr::null_mut::<hyperlinks_uri>(),
        tqh_last: &raw mut global_hyperlinks.tqh_first,
    };
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];
