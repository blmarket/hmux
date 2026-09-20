use crate::src::shared::abi::*;
extern "C" {
    pub type evbuffer;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn strtoll(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_longlong;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn evbuffer_new() -> *mut evbuffer;
    fn evbuffer_free(buf: *mut evbuffer);
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn evbuffer_add(
        buf: *mut evbuffer,
        data: *const ::core::ffi::c_void,
        datlen: size_t,
    ) -> ::core::ffi::c_int;
    fn evbuffer_add_printf(
        buf: *mut evbuffer,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(_: *mut ::core::ffi::c_void, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xstrndup(_: *const ::core::ffi::c_char, _: size_t) -> *mut ::core::ffi::c_char;
    fn xmemdup(_: *const ::core::ffi::c_void, _: size_t) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}
pub type __int64_t = i64;
pub type ssize_t = isize;
pub type int64_t = __int64_t;
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2RustUnnamed = 8;
pub const _ISpunct: C2RustUnnamed = 4;
pub const _IScntrl: C2RustUnnamed = 2;
pub const _ISblank: C2RustUnnamed = 1;
pub const _ISgraph: C2RustUnnamed = 32768;
pub const _ISprint: C2RustUnnamed = 16384;
pub const _ISspace: C2RustUnnamed = 8192;
pub const _ISxdigit: C2RustUnnamed = 4096;
pub const _ISdigit: C2RustUnnamed = 2048;
pub const _ISalpha: C2RustUnnamed = 1024;
pub const _ISlower: C2RustUnnamed = 512;
pub const _ISupper: C2RustUnnamed = 256;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_node {
    pub type_0: json_node_type,
    pub key: *mut ::core::ffi::c_char,
    pub parent: *mut json_node,
    pub c2rust_unnamed: C2RustUnnamed_2,
    pub oentry: C2RustUnnamed_1,
    pub aentry: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub tqe_next: *mut json_node,
    pub tqe_prev: *mut *mut json_node,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {
    pub rbe_left: *mut json_node,
    pub rbe_right: *mut json_node,
    pub rbe_parent: *mut json_node,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_2 {
    pub str_0: *mut ::core::ffi::c_char,
    pub num: int64_t,
    pub boolean: ::core::ffi::c_int,
    pub fields: json_fields,
    pub members: json_members,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_members {
    pub tqh_first: *mut json_node,
    pub tqh_last: *mut *mut json_node,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_fields {
    pub rbh_root: *mut json_node,
}
pub type json_node_type = ::core::ffi::c_uint;
pub const NODE_ARRAY: json_node_type = 4;
pub const NODE_OBJECT: json_node_type = 3;
pub const NODE_BOOLEAN: json_node_type = 2;
pub const NODE_NUMBER: json_node_type = 1;
pub const NODE_STRING: json_node_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_parse_ctx {
    pub input: *const ::core::ffi::c_char,
    pub cause: *mut *mut ::core::ffi::c_char,
    pub depth: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_tokens {
    pub size: ::core::ffi::c_int,
    pub capacity: ::core::ffi::c_int,
    pub toks: *mut json_token,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_token {
    pub type_0: json_token_type,
    pub offset: ::core::ffi::c_int,
    pub len: ::core::ffi::c_int,
}
pub type json_token_type = ::core::ffi::c_uint;
pub const TOK_EOF: json_token_type = 8;
pub const TOK_VALUE: json_token_type = 7;
pub const TOK_QUOTE: json_token_type = 6;
pub const TOK_COLON: json_token_type = 5;
pub const TOK_COMMA: json_token_type = 4;
pub const TOK_CLOSEARRAY: json_token_type = 3;
pub const TOK_OPENARRAY: json_token_type = 2;
pub const TOK_CLOSEOBJECT: json_token_type = 1;
pub const TOK_OPENOBJECT: json_token_type = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const ERROR_CTX_LEN: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const PARSE_DEPTH_MAX: ::core::ffi::c_int = 200 as ::core::ffi::c_int;
unsafe extern "C" fn json_node_cmp(
    mut a: *mut json_node,
    mut b: *mut json_node,
) -> ::core::ffi::c_int {
    return strcmp((*a).key, (*b).key);
}
unsafe extern "C" fn json_fields_RB_INSERT(
    mut head: *mut json_fields,
    mut elm: *mut json_node,
) -> *mut json_node {
    let mut tmp: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut parent: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = json_node_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).oentry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).oentry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).oentry.rbe_parent = parent;
    (*elm).oentry.rbe_right = ::core::ptr::null_mut::<json_node>();
    (*elm).oentry.rbe_left = (*elm).oentry.rbe_right;
    (*elm).oentry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).oentry.rbe_left = elm;
        } else {
            (*parent).oentry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    json_fields_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<json_node>();
}
unsafe extern "C" fn json_fields_RB_INSERT_COLOR(
    mut head: *mut json_fields,
    mut elm: *mut json_node,
) {
    let mut parent: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut gparent: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut tmp: *mut json_node = ::core::ptr::null_mut::<json_node>();
    loop {
        parent = (*elm).oentry.rbe_parent;
        if !(!parent.is_null() && (*parent).oentry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).oentry.rbe_parent;
        if parent == (*gparent).oentry.rbe_left {
            tmp = (*gparent).oentry.rbe_right;
            if !tmp.is_null() && (*tmp).oentry.rbe_color == RB_RED {
                (*tmp).oentry.rbe_color = RB_BLACK;
                (*parent).oentry.rbe_color = RB_BLACK;
                (*gparent).oentry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).oentry.rbe_right == elm {
                    tmp = (*parent).oentry.rbe_right;
                    (*parent).oentry.rbe_right = (*tmp).oentry.rbe_left;
                    if !(*parent).oentry.rbe_right.is_null() {
                        (*(*tmp).oentry.rbe_left).oentry.rbe_parent = parent;
                    }
                    (*tmp).oentry.rbe_parent = (*parent).oentry.rbe_parent;
                    if !(*tmp).oentry.rbe_parent.is_null() {
                        if parent == (*(*parent).oentry.rbe_parent).oentry.rbe_left {
                            (*(*parent).oentry.rbe_parent).oentry.rbe_left = tmp;
                        } else {
                            (*(*parent).oentry.rbe_parent).oentry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).oentry.rbe_left = parent;
                    (*parent).oentry.rbe_parent = tmp;
                    !(*tmp).oentry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).oentry.rbe_color = RB_BLACK;
                (*gparent).oentry.rbe_color = RB_RED;
                tmp = (*gparent).oentry.rbe_left;
                (*gparent).oentry.rbe_left = (*tmp).oentry.rbe_right;
                if !(*gparent).oentry.rbe_left.is_null() {
                    (*(*tmp).oentry.rbe_right).oentry.rbe_parent = gparent;
                }
                (*tmp).oentry.rbe_parent = (*gparent).oentry.rbe_parent;
                if !(*tmp).oentry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).oentry.rbe_parent).oentry.rbe_left {
                        (*(*gparent).oentry.rbe_parent).oentry.rbe_left = tmp;
                    } else {
                        (*(*gparent).oentry.rbe_parent).oentry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).oentry.rbe_right = gparent;
                (*gparent).oentry.rbe_parent = tmp;
                !(*tmp).oentry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).oentry.rbe_left;
            if !tmp.is_null() && (*tmp).oentry.rbe_color == RB_RED {
                (*tmp).oentry.rbe_color = RB_BLACK;
                (*parent).oentry.rbe_color = RB_BLACK;
                (*gparent).oentry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).oentry.rbe_left == elm {
                    tmp = (*parent).oentry.rbe_left;
                    (*parent).oentry.rbe_left = (*tmp).oentry.rbe_right;
                    if !(*parent).oentry.rbe_left.is_null() {
                        (*(*tmp).oentry.rbe_right).oentry.rbe_parent = parent;
                    }
                    (*tmp).oentry.rbe_parent = (*parent).oentry.rbe_parent;
                    if !(*tmp).oentry.rbe_parent.is_null() {
                        if parent == (*(*parent).oentry.rbe_parent).oentry.rbe_left {
                            (*(*parent).oentry.rbe_parent).oentry.rbe_left = tmp;
                        } else {
                            (*(*parent).oentry.rbe_parent).oentry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).oentry.rbe_right = parent;
                    (*parent).oentry.rbe_parent = tmp;
                    !(*tmp).oentry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).oentry.rbe_color = RB_BLACK;
                (*gparent).oentry.rbe_color = RB_RED;
                tmp = (*gparent).oentry.rbe_right;
                (*gparent).oentry.rbe_right = (*tmp).oentry.rbe_left;
                if !(*gparent).oentry.rbe_right.is_null() {
                    (*(*tmp).oentry.rbe_left).oentry.rbe_parent = gparent;
                }
                (*tmp).oentry.rbe_parent = (*gparent).oentry.rbe_parent;
                if !(*tmp).oentry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).oentry.rbe_parent).oentry.rbe_left {
                        (*(*gparent).oentry.rbe_parent).oentry.rbe_left = tmp;
                    } else {
                        (*(*gparent).oentry.rbe_parent).oentry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).oentry.rbe_left = gparent;
                (*gparent).oentry.rbe_parent = tmp;
                !(*tmp).oentry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).oentry.rbe_color = RB_BLACK;
}
unsafe extern "C" fn json_fields_RB_REMOVE(
    mut head: *mut json_fields,
    mut elm: *mut json_node,
) -> *mut json_node {
    let mut current_block: u64;
    let mut child: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut parent: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut old: *mut json_node = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).oentry.rbe_left.is_null() {
        child = (*elm).oentry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).oentry.rbe_right.is_null() {
        child = (*elm).oentry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut json_node = ::core::ptr::null_mut::<json_node>();
        elm = (*elm).oentry.rbe_right;
        loop {
            left = (*elm).oentry.rbe_left;
            if left.is_null() {
                break;
            }
            elm = left;
        }
        child = (*elm).oentry.rbe_right;
        parent = (*elm).oentry.rbe_parent;
        color = (*elm).oentry.rbe_color;
        if !child.is_null() {
            (*child).oentry.rbe_parent = parent;
        }
        if !parent.is_null() {
            if (*parent).oentry.rbe_left == elm {
                (*parent).oentry.rbe_left = child;
            } else {
                (*parent).oentry.rbe_right = child;
            }
        } else {
            (*head).rbh_root = child;
        }
        if (*elm).oentry.rbe_parent == old {
            parent = elm;
        }
        (*elm).oentry = (*old).oentry;
        if !(*old).oentry.rbe_parent.is_null() {
            if (*(*old).oentry.rbe_parent).oentry.rbe_left == old {
                (*(*old).oentry.rbe_parent).oentry.rbe_left = elm;
            } else {
                (*(*old).oentry.rbe_parent).oentry.rbe_right = elm;
            }
        } else {
            (*head).rbh_root = elm;
        }
        (*(*old).oentry.rbe_left).oentry.rbe_parent = elm;
        if !(*old).oentry.rbe_right.is_null() {
            (*(*old).oentry.rbe_right).oentry.rbe_parent = elm;
        }
        if !parent.is_null() {
            left = parent;
            loop {
                left = (*left).oentry.rbe_parent;
                if left.is_null() {
                    break;
                }
            }
        }
        current_block = 12328454399491550103;
    }
    match current_block {
        7245201122033322888 => {
            parent = (*elm).oentry.rbe_parent;
            color = (*elm).oentry.rbe_color;
            if !child.is_null() {
                (*child).oentry.rbe_parent = parent;
            }
            if !parent.is_null() {
                if (*parent).oentry.rbe_left == elm {
                    (*parent).oentry.rbe_left = child;
                } else {
                    (*parent).oentry.rbe_right = child;
                }
            } else {
                (*head).rbh_root = child;
            }
        }
        _ => {}
    }
    if color == RB_BLACK {
        json_fields_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn json_fields_RB_REMOVE_COLOR(
    mut head: *mut json_fields,
    mut parent: *mut json_node,
    mut elm: *mut json_node,
) {
    let mut tmp: *mut json_node = ::core::ptr::null_mut::<json_node>();
    while (elm.is_null() || (*elm).oentry.rbe_color == RB_BLACK) && elm != (*head).rbh_root {
        if (*parent).oentry.rbe_left == elm {
            tmp = (*parent).oentry.rbe_right;
            if (*tmp).oentry.rbe_color == RB_RED {
                (*tmp).oentry.rbe_color = RB_BLACK;
                (*parent).oentry.rbe_color = RB_RED;
                tmp = (*parent).oentry.rbe_right;
                (*parent).oentry.rbe_right = (*tmp).oentry.rbe_left;
                if !(*parent).oentry.rbe_right.is_null() {
                    (*(*tmp).oentry.rbe_left).oentry.rbe_parent = parent;
                }
                (*tmp).oentry.rbe_parent = (*parent).oentry.rbe_parent;
                if !(*tmp).oentry.rbe_parent.is_null() {
                    if parent == (*(*parent).oentry.rbe_parent).oentry.rbe_left {
                        (*(*parent).oentry.rbe_parent).oentry.rbe_left = tmp;
                    } else {
                        (*(*parent).oentry.rbe_parent).oentry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).oentry.rbe_left = parent;
                (*parent).oentry.rbe_parent = tmp;
                !(*tmp).oentry.rbe_parent.is_null();
                tmp = (*parent).oentry.rbe_right;
            }
            if ((*tmp).oentry.rbe_left.is_null()
                || (*(*tmp).oentry.rbe_left).oentry.rbe_color == RB_BLACK)
                && ((*tmp).oentry.rbe_right.is_null()
                    || (*(*tmp).oentry.rbe_right).oentry.rbe_color == RB_BLACK)
            {
                (*tmp).oentry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).oentry.rbe_parent;
            } else {
                if (*tmp).oentry.rbe_right.is_null()
                    || (*(*tmp).oentry.rbe_right).oentry.rbe_color == RB_BLACK
                {
                    let mut oleft: *mut json_node = ::core::ptr::null_mut::<json_node>();
                    oleft = (*tmp).oentry.rbe_left;
                    if !oleft.is_null() {
                        (*oleft).oentry.rbe_color = RB_BLACK;
                    }
                    (*tmp).oentry.rbe_color = RB_RED;
                    oleft = (*tmp).oentry.rbe_left;
                    (*tmp).oentry.rbe_left = (*oleft).oentry.rbe_right;
                    if !(*tmp).oentry.rbe_left.is_null() {
                        (*(*oleft).oentry.rbe_right).oentry.rbe_parent = tmp;
                    }
                    (*oleft).oentry.rbe_parent = (*tmp).oentry.rbe_parent;
                    if !(*oleft).oentry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).oentry.rbe_parent).oentry.rbe_left {
                            (*(*tmp).oentry.rbe_parent).oentry.rbe_left = oleft;
                        } else {
                            (*(*tmp).oentry.rbe_parent).oentry.rbe_right = oleft;
                        }
                    } else {
                        (*head).rbh_root = oleft;
                    }
                    (*oleft).oentry.rbe_right = tmp;
                    (*tmp).oentry.rbe_parent = oleft;
                    !(*oleft).oentry.rbe_parent.is_null();
                    tmp = (*parent).oentry.rbe_right;
                }
                (*tmp).oentry.rbe_color = (*parent).oentry.rbe_color;
                (*parent).oentry.rbe_color = RB_BLACK;
                if !(*tmp).oentry.rbe_right.is_null() {
                    (*(*tmp).oentry.rbe_right).oentry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).oentry.rbe_right;
                (*parent).oentry.rbe_right = (*tmp).oentry.rbe_left;
                if !(*parent).oentry.rbe_right.is_null() {
                    (*(*tmp).oentry.rbe_left).oentry.rbe_parent = parent;
                }
                (*tmp).oentry.rbe_parent = (*parent).oentry.rbe_parent;
                if !(*tmp).oentry.rbe_parent.is_null() {
                    if parent == (*(*parent).oentry.rbe_parent).oentry.rbe_left {
                        (*(*parent).oentry.rbe_parent).oentry.rbe_left = tmp;
                    } else {
                        (*(*parent).oentry.rbe_parent).oentry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).oentry.rbe_left = parent;
                (*parent).oentry.rbe_parent = tmp;
                !(*tmp).oentry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        } else {
            tmp = (*parent).oentry.rbe_left;
            if (*tmp).oentry.rbe_color == RB_RED {
                (*tmp).oentry.rbe_color = RB_BLACK;
                (*parent).oentry.rbe_color = RB_RED;
                tmp = (*parent).oentry.rbe_left;
                (*parent).oentry.rbe_left = (*tmp).oentry.rbe_right;
                if !(*parent).oentry.rbe_left.is_null() {
                    (*(*tmp).oentry.rbe_right).oentry.rbe_parent = parent;
                }
                (*tmp).oentry.rbe_parent = (*parent).oentry.rbe_parent;
                if !(*tmp).oentry.rbe_parent.is_null() {
                    if parent == (*(*parent).oentry.rbe_parent).oentry.rbe_left {
                        (*(*parent).oentry.rbe_parent).oentry.rbe_left = tmp;
                    } else {
                        (*(*parent).oentry.rbe_parent).oentry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).oentry.rbe_right = parent;
                (*parent).oentry.rbe_parent = tmp;
                !(*tmp).oentry.rbe_parent.is_null();
                tmp = (*parent).oentry.rbe_left;
            }
            if ((*tmp).oentry.rbe_left.is_null()
                || (*(*tmp).oentry.rbe_left).oentry.rbe_color == RB_BLACK)
                && ((*tmp).oentry.rbe_right.is_null()
                    || (*(*tmp).oentry.rbe_right).oentry.rbe_color == RB_BLACK)
            {
                (*tmp).oentry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).oentry.rbe_parent;
            } else {
                if (*tmp).oentry.rbe_left.is_null()
                    || (*(*tmp).oentry.rbe_left).oentry.rbe_color == RB_BLACK
                {
                    let mut oright: *mut json_node = ::core::ptr::null_mut::<json_node>();
                    oright = (*tmp).oentry.rbe_right;
                    if !oright.is_null() {
                        (*oright).oentry.rbe_color = RB_BLACK;
                    }
                    (*tmp).oentry.rbe_color = RB_RED;
                    oright = (*tmp).oentry.rbe_right;
                    (*tmp).oentry.rbe_right = (*oright).oentry.rbe_left;
                    if !(*tmp).oentry.rbe_right.is_null() {
                        (*(*oright).oentry.rbe_left).oentry.rbe_parent = tmp;
                    }
                    (*oright).oentry.rbe_parent = (*tmp).oentry.rbe_parent;
                    if !(*oright).oentry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).oentry.rbe_parent).oentry.rbe_left {
                            (*(*tmp).oentry.rbe_parent).oentry.rbe_left = oright;
                        } else {
                            (*(*tmp).oentry.rbe_parent).oentry.rbe_right = oright;
                        }
                    } else {
                        (*head).rbh_root = oright;
                    }
                    (*oright).oentry.rbe_left = tmp;
                    (*tmp).oentry.rbe_parent = oright;
                    !(*oright).oentry.rbe_parent.is_null();
                    tmp = (*parent).oentry.rbe_left;
                }
                (*tmp).oentry.rbe_color = (*parent).oentry.rbe_color;
                (*parent).oentry.rbe_color = RB_BLACK;
                if !(*tmp).oentry.rbe_left.is_null() {
                    (*(*tmp).oentry.rbe_left).oentry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).oentry.rbe_left;
                (*parent).oentry.rbe_left = (*tmp).oentry.rbe_right;
                if !(*parent).oentry.rbe_left.is_null() {
                    (*(*tmp).oentry.rbe_right).oentry.rbe_parent = parent;
                }
                (*tmp).oentry.rbe_parent = (*parent).oentry.rbe_parent;
                if !(*tmp).oentry.rbe_parent.is_null() {
                    if parent == (*(*parent).oentry.rbe_parent).oentry.rbe_left {
                        (*(*parent).oentry.rbe_parent).oentry.rbe_left = tmp;
                    } else {
                        (*(*parent).oentry.rbe_parent).oentry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).oentry.rbe_right = parent;
                (*parent).oentry.rbe_parent = tmp;
                !(*tmp).oentry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        }
    }
    if !elm.is_null() {
        (*elm).oentry.rbe_color = RB_BLACK;
    }
}
unsafe extern "C" fn json_fields_RB_NEXT(mut elm: *mut json_node) -> *mut json_node {
    if !(*elm).oentry.rbe_right.is_null() {
        elm = (*elm).oentry.rbe_right;
        while !(*elm).oentry.rbe_left.is_null() {
            elm = (*elm).oentry.rbe_left;
        }
    } else if !(*elm).oentry.rbe_parent.is_null()
        && elm == (*(*elm).oentry.rbe_parent).oentry.rbe_left
    {
        elm = (*elm).oentry.rbe_parent;
    } else {
        while !(*elm).oentry.rbe_parent.is_null()
            && elm == (*(*elm).oentry.rbe_parent).oentry.rbe_right
        {
            elm = (*elm).oentry.rbe_parent;
        }
        elm = (*elm).oentry.rbe_parent;
    }
    return elm;
}
unsafe extern "C" fn json_fields_RB_MINMAX(
    mut head: *mut json_fields,
    mut val: ::core::ffi::c_int,
) -> *mut json_node {
    let mut tmp: *mut json_node = (*head).rbh_root;
    let mut parent: *mut json_node = ::core::ptr::null_mut::<json_node>();
    while !tmp.is_null() {
        parent = tmp;
        if val < 0 as ::core::ffi::c_int {
            tmp = (*tmp).oentry.rbe_left;
        } else {
            tmp = (*tmp).oentry.rbe_right;
        }
    }
    return parent;
}
unsafe extern "C" fn json_fields_RB_FIND(
    mut head: *mut json_fields,
    mut elm: *mut json_node,
) -> *mut json_node {
    let mut tmp: *mut json_node = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = json_node_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).oentry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).oentry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<json_node>();
}
#[no_mangle]
pub unsafe extern "C" fn json_parse(
    mut input: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut json_node {
    let mut tokens: *mut json_tokens = ::core::ptr::null_mut::<json_tokens>();
    let mut pctx: json_parse_ctx = json_parse_ctx {
        input: ::core::ptr::null::<::core::ffi::c_char>(),
        cause: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        depth: 0,
    };
    if *input as ::core::ffi::c_int == '\0' as i32 {
        json_error(
            cause,
            b"empty input\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        return ::core::ptr::null_mut::<json_node>();
    }
    tokens = json_tokenize_input(input, cause);
    if tokens.is_null() {
        return ::core::ptr::null_mut::<json_node>();
    }
    pctx.input = input;
    pctx.cause = cause;
    pctx.depth = 0 as ::core::ffi::c_int;
    return json_parse_tokens(&raw mut tokens, &raw mut pctx);
}
#[no_mangle]
pub unsafe extern "C" fn json_find(
    mut jn: *mut json_node,
    mut key: *const ::core::ffi::c_char,
) -> *mut json_node {
    let mut node: *mut json_node = jn;
    let mut tmp: json_node = json_node {
        type_0: NODE_STRING,
        key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        parent: ::core::ptr::null_mut::<json_node>(),
        c2rust_unnamed: C2RustUnnamed_2 {
            str_0: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        oentry: C2RustUnnamed_1 {
            rbe_left: ::core::ptr::null_mut::<json_node>(),
            rbe_right: ::core::ptr::null_mut::<json_node>(),
            rbe_parent: ::core::ptr::null_mut::<json_node>(),
            rbe_color: 0,
        },
        aentry: C2RustUnnamed_0 {
            tqe_next: ::core::ptr::null_mut::<json_node>(),
            tqe_prev: ::core::ptr::null_mut::<*mut json_node>(),
        },
    };
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<json_node>();
    }
    tmp.key = key as *mut ::core::ffi::c_char;
    return json_fields_RB_FIND(&raw mut (*node).c2rust_unnamed.fields, &raw mut tmp);
}
#[no_mangle]
pub unsafe extern "C" fn json_array_first(mut jn: *mut json_node) -> *mut json_node {
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<json_node>();
    }
    return (*jn).c2rust_unnamed.members.tqh_first;
}
#[no_mangle]
pub unsafe extern "C" fn json_array_next(mut member: *mut json_node) -> *mut json_node {
    if member.is_null()
        || (*member).parent.is_null()
        || (*(*member).parent).type_0 as ::core::ffi::c_uint
            != NODE_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<json_node>();
    }
    return (*member).aentry.tqe_next;
}
#[no_mangle]
pub unsafe extern "C" fn json_get_string(
    mut jn: *mut json_node,
    mut s: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *s = (*jn).c2rust_unnamed.str_0;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_get_number(
    mut jn: *mut json_node,
    mut i: *mut int64_t,
) -> ::core::ffi::c_int {
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *i = (*jn).c2rust_unnamed.num;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_get_boolean(
    mut jn: *mut json_node,
    mut b: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_BOOLEAN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *b = (*jn).c2rust_unnamed.boolean;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_get_object(
    mut jn: *mut json_node,
    mut o: *mut *mut json_node,
) -> ::core::ffi::c_int {
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *o = jn;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_get_array(
    mut jn: *mut json_node,
    mut a: *mut *mut json_node,
) -> ::core::ffi::c_int {
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *a = jn;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_find_string(
    mut jn: *mut json_node,
    mut key: *const ::core::ffi::c_char,
    mut out: *mut *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    field = json_find(jn, key);
    if field.is_null() {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"key \"%s\" not found\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    if (*field).type_0 as ::core::ffi::c_uint
        != NODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"key \"%s\" expected a string\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    *out = (*field).c2rust_unnamed.str_0;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_find_number(
    mut jn: *mut json_node,
    mut key: *const ::core::ffi::c_char,
    mut out: *mut int64_t,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    field = json_find(jn, key);
    if field.is_null() {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"key \"%s\" not found\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    if (*field).type_0 as ::core::ffi::c_uint
        != NODE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"key \"%s\" expected a number\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    *out = (*field).c2rust_unnamed.num;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_find_boolean(
    mut jn: *mut json_node,
    mut key: *const ::core::ffi::c_char,
    mut out: *mut ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    field = json_find(jn, key);
    if field.is_null() {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"key \"%s\" not found\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    if (*field).type_0 as ::core::ffi::c_uint
        != NODE_BOOLEAN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"key \"%s\" expected a boolean\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    *out = (*field).c2rust_unnamed.boolean;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_find_object(
    mut jn: *mut json_node,
    mut key: *const ::core::ffi::c_char,
    mut out: *mut *mut json_node,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    field = json_find(jn, key);
    if field.is_null() {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"key \"%s\" not found\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    if (*field).type_0 as ::core::ffi::c_uint
        != NODE_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"key \"%s\" expected an object\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    *out = field;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_find_array(
    mut jn: *mut json_node,
    mut key: *const ::core::ffi::c_char,
    mut out: *mut *mut json_node,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    field = json_find(jn, key);
    if field.is_null() {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"key \"%s\" not found\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    if (*field).type_0 as ::core::ffi::c_uint
        != NODE_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"key \"%s\" expected an array\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    *out = field;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn json_error(
    mut cause: *mut *mut ::core::ffi::c_char,
    mut reason: *const ::core::ffi::c_char,
    mut loc: *const ::core::ffi::c_char,
) {
    let mut ellipsis: *const ::core::ffi::c_char =
        b"...\0" as *const u8 as *const ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0;
    if cause.is_null() {
        return;
    }
    if loc.is_null() || *loc as ::core::ffi::c_int == '\0' as i32 {
        xasprintf(
            cause,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            reason,
        );
        return;
    }
    i = 0 as ::core::ffi::c_int;
    while i < ERROR_CTX_LEN + 1 as ::core::ffi::c_int {
        if *loc.offset(i as isize) as ::core::ffi::c_int == '\0' as i32 {
            ellipsis = b"\0" as *const u8 as *const ::core::ffi::c_char;
            break;
        } else {
            i += 1;
        }
    }
    xasprintf(
        cause,
        b"%s: %.*s%s\0" as *const u8 as *const ::core::ffi::c_char,
        reason,
        ERROR_CTX_LEN,
        loc,
        ellipsis,
    );
}
unsafe extern "C" fn json_tokenize_input(
    mut input: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut json_tokens {
    let mut current_block: u64;
    let mut tokens: *mut json_tokens = ::core::ptr::null_mut::<json_tokens>();
    let mut type_0: json_token_type = TOK_OPENOBJECT;
    let mut loc: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut start: *const ::core::ffi::c_char = input;
    let mut in_string: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut scan: ::core::ffi::c_int = 0;
    tokens = json_create_tokens();
    loop {
        if !(*input as ::core::ffi::c_int != '\0' as i32) {
            current_block = 16924917904204750491;
            break;
        }
        loc = input;
        scan = 1 as ::core::ffi::c_int;
        if in_string != 0 && *input as ::core::ffi::c_int != '"' as i32 {
            type_0 = TOK_VALUE;
        } else {
            match *input as ::core::ffi::c_int {
                32 | 9 | 10 | 13 => {
                    input = input.offset(1);
                    continue;
                }
                123 => {
                    type_0 = TOK_OPENOBJECT;
                }
                125 => {
                    type_0 = TOK_CLOSEOBJECT;
                }
                91 => {
                    type_0 = TOK_OPENARRAY;
                }
                93 => {
                    type_0 = TOK_CLOSEARRAY;
                }
                34 => {
                    type_0 = TOK_QUOTE;
                }
                58 => {
                    type_0 = TOK_COLON;
                }
                44 => {
                    type_0 = TOK_COMMA;
                }
                _ => {
                    type_0 = TOK_VALUE;
                }
            }
        }
        if type_0 as ::core::ffi::c_uint == TOK_VALUE as ::core::ffi::c_int as ::core::ffi::c_uint {
            scan = json_tokenize_value(tokens, loc);
            if scan == -(1 as ::core::ffi::c_int) {
                current_block = 2526103352432062781;
                break;
            }
            input = input.offset((scan - 1 as ::core::ffi::c_int) as isize);
        }
        json_add_token(tokens, type_0, start, loc, scan);
        if type_0 as ::core::ffi::c_uint == TOK_QUOTE as ::core::ffi::c_int as ::core::ffi::c_uint {
            in_string = (in_string == 0) as ::core::ffi::c_int;
        }
        input = input.offset(1);
    }
    match current_block {
        2526103352432062781 => {
            json_error(
                cause,
                b"tokenization error\0" as *const u8 as *const ::core::ffi::c_char,
                loc,
            );
            json_destroy_tokens(tokens);
            return ::core::ptr::null_mut::<json_tokens>();
        }
        _ => {
            json_add_token(tokens, TOK_EOF, start, loc, 0 as ::core::ffi::c_int);
            return tokens;
        }
    };
}
unsafe extern "C" fn json_tokenize_value(
    mut tokens: *mut json_tokens,
    mut loc: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut prev: *mut json_token = ::core::ptr::null_mut::<json_token>();
    let mut i: ::core::ffi::c_int = 0;
    let mut scan: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*tokens).size == 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    prev = (*tokens)
        .toks
        .offset(((*tokens).size - 1 as ::core::ffi::c_int) as isize) as *mut json_token;
    if (*prev).type_0 as ::core::ffi::c_uint
        == TOK_QUOTE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        while *loc.offset(scan as isize) as ::core::ffi::c_int != '"' as i32 {
            if *loc.offset(scan as isize) as ::core::ffi::c_int == '\0' as i32
                || (*loc.offset(scan as isize) as u_char as ::core::ffi::c_int)
                    < 0x20 as ::core::ffi::c_int
            {
                return -(1 as ::core::ffi::c_int);
            }
            if *loc.offset(scan as isize) as ::core::ffi::c_int != '\\' as i32 {
                scan += 1;
            } else {
                scan += 1;
                match *loc.offset(scan as isize) as ::core::ffi::c_int {
                    34 | 92 | 47 | 98 | 102 | 110 | 114 | 116 => {
                        scan += 1;
                    }
                    117 => {
                        i = 1 as ::core::ffi::c_int;
                        while i <= 4 as ::core::ffi::c_int {
                            if *(*__ctype_b_loc())
                                .offset(*loc.offset((scan + i) as isize) as u_char
                                    as ::core::ffi::c_int
                                    as isize) as ::core::ffi::c_int
                                & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                                    as ::core::ffi::c_int
                                == 0
                            {
                                return -(1 as ::core::ffi::c_int);
                            }
                            i += 1;
                        }
                        scan += 5 as ::core::ffi::c_int;
                    }
                    _ => return -(1 as ::core::ffi::c_int),
                }
            }
        }
    } else if (*prev).type_0 as ::core::ffi::c_uint
        == TOK_COLON as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        loop {
            if *loc.offset(scan as isize) as ::core::ffi::c_int == '\0' as i32 {
                return -(1 as ::core::ffi::c_int);
            }
            scan += 1;
            if !(*loc.offset(scan as isize) as ::core::ffi::c_int != ']' as i32
                && *loc.offset(scan as isize) as ::core::ffi::c_int != '}' as i32
                && *loc.offset(scan as isize) as ::core::ffi::c_int != ',' as i32
                && *(*__ctype_b_loc())
                    .offset(*loc.offset(scan as isize) as u_char as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0)
            {
                break;
            }
        }
    } else {
        return -(1 as ::core::ffi::c_int);
    }
    return scan;
}
unsafe extern "C" fn json_create_tokens() -> *mut json_tokens {
    let mut tokens: *mut json_tokens = ::core::ptr::null_mut::<json_tokens>();
    tokens = xmalloc(::core::mem::size_of::<json_tokens>() as size_t) as *mut json_tokens;
    (*tokens).size = 0 as ::core::ffi::c_int;
    (*tokens).capacity = 1024 as ::core::ffi::c_int;
    (*tokens).toks = xmalloc(
        ((*tokens).capacity as size_t).wrapping_mul(::core::mem::size_of::<json_token>() as size_t),
    ) as *mut json_token;
    return tokens;
}
unsafe extern "C" fn json_destroy_tokens(mut tokens: *mut json_tokens) {
    free((*tokens).toks as *mut ::core::ffi::c_void);
    (*tokens).toks = ::core::ptr::null_mut::<json_token>();
    free(tokens as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn json_add_token(
    mut tokens: *mut json_tokens,
    mut type_0: json_token_type,
    mut input: *const ::core::ffi::c_char,
    mut loc: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) {
    let mut tok: *mut json_token = ::core::ptr::null_mut::<json_token>();
    while (*tokens).size >= (*tokens).capacity {
        (*tokens).capacity *= 2 as ::core::ffi::c_int;
        (*tokens).toks = xrealloc(
            (*tokens).toks as *mut ::core::ffi::c_void,
            (::core::mem::size_of::<json_token>() as size_t)
                .wrapping_mul((*tokens).capacity as size_t),
        ) as *mut json_token;
    }
    let fresh0 = (*tokens).size;
    (*tokens).size = (*tokens).size + 1;
    tok = (*tokens).toks.offset(fresh0 as isize) as *mut json_token;
    (*tok).type_0 = type_0;
    (*tok).offset = loc.offset_from(input) as ::core::ffi::c_long as ::core::ffi::c_int;
    (*tok).len = len;
}
unsafe extern "C" fn json_create_node(
    mut parent: *mut json_node,
    mut type_0: json_node_type,
    mut key: *const ::core::ffi::c_char,
    mut val: *mut ::core::ffi::c_void,
) -> *mut json_node {
    let mut node: *mut json_node = ::core::ptr::null_mut::<json_node>();
    node = xcalloc(1 as size_t, ::core::mem::size_of::<json_node>() as size_t) as *mut json_node;
    (*node).parent = parent;
    if !key.is_null() {
        (*node).key = xstrdup(key);
    }
    (*node).type_0 = type_0;
    if type_0 as ::core::ffi::c_uint == NODE_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint {
        (*node).c2rust_unnamed.fields.rbh_root = ::core::ptr::null_mut::<json_node>();
    } else if type_0 as ::core::ffi::c_uint
        == NODE_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*node).c2rust_unnamed.members.tqh_first = ::core::ptr::null_mut::<json_node>();
        (*node).c2rust_unnamed.members.tqh_last = &raw mut (*node).c2rust_unnamed.members.tqh_first;
    }
    if !val.is_null() {
        json_assign_value(node, val);
    }
    return node;
}
#[no_mangle]
pub unsafe extern "C" fn json_destroy_node(mut node: *mut json_node) {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut field1: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut member: *mut json_node = ::core::ptr::null_mut::<json_node>();
    if node.is_null() {
        return;
    }
    match (*node).type_0 as ::core::ffi::c_uint {
        0 => {
            free((*node).c2rust_unnamed.str_0 as *mut ::core::ffi::c_void);
        }
        3 => {
            field = json_fields_RB_MINMAX(&raw mut (*node).c2rust_unnamed.fields, RB_NEGINF);
            while !field.is_null() && {
                field1 = json_fields_RB_NEXT(field);
                1 as ::core::ffi::c_int != 0
            } {
                json_fields_RB_REMOVE(&raw mut (*node).c2rust_unnamed.fields, field);
                json_destroy_node(field);
                field = field1;
            }
        }
        4 => {
            while !(*node).c2rust_unnamed.members.tqh_first.is_null() {
                member = (*node).c2rust_unnamed.members.tqh_first;
                if !(*member).aentry.tqe_next.is_null() {
                    (*(*member).aentry.tqe_next).aentry.tqe_prev = (*member).aentry.tqe_prev;
                } else {
                    (*node).c2rust_unnamed.members.tqh_last = (*member).aentry.tqe_prev;
                }
                *(*member).aentry.tqe_prev = (*member).aentry.tqe_next;
                json_destroy_node(member);
            }
        }
        1 | 2 | _ => {}
    }
    if !(*node).key.is_null() {
        free((*node).key as *mut ::core::ffi::c_void);
    }
    free(node as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn json_assign_value(
    mut node: *mut json_node,
    mut val: *mut ::core::ffi::c_void,
) {
    let mut child: *mut json_node = val as *mut json_node;
    match (*node).type_0 as ::core::ffi::c_uint {
        0 => {
            (*node).c2rust_unnamed.str_0 = val as *mut ::core::ffi::c_char;
        }
        1 => {
            (*node).c2rust_unnamed.num = *(val as *mut int64_t);
        }
        2 => {
            (*node).c2rust_unnamed.boolean = *(val as *mut ::core::ffi::c_int);
        }
        3 => {
            json_fields_RB_INSERT(&raw mut (*node).c2rust_unnamed.fields, child);
        }
        4 => {
            (*child).aentry.tqe_next = ::core::ptr::null_mut::<json_node>();
            (*child).aentry.tqe_prev = (*node).c2rust_unnamed.members.tqh_last;
            *(*node).c2rust_unnamed.members.tqh_last = child;
            (*node).c2rust_unnamed.members.tqh_last = &raw mut (*child).aentry.tqe_next;
        }
        _ => {
            fatalx(b"unknown node type\0" as *const u8 as *const ::core::ffi::c_char);
        }
    };
}
unsafe extern "C" fn json_parse_tokens(
    mut tokens: *mut *mut json_tokens,
    mut pctx: *mut json_parse_ctx,
) -> *mut json_node {
    let mut tok: *mut json_token = (**tokens).toks;
    let mut jn: *mut json_node = ::core::ptr::null_mut::<json_node>();
    if (*tok).type_0 as ::core::ffi::c_uint
        == TOK_OPENOBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        jn = json_parse_object(
            &raw mut tok,
            pctx,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<json_node>(),
        );
        if !jn.is_null() {
            if (*tok).type_0 as ::core::ffi::c_uint
                != TOK_EOF as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                json_error(
                    (*pctx).cause,
                    b"unexpected trailing data\0" as *const u8 as *const ::core::ffi::c_char,
                    (*pctx).input.offset((*tok).offset as isize),
                );
            } else {
                json_destroy_tokens(*tokens);
                *tokens = ::core::ptr::null_mut::<json_tokens>();
                return jn;
            }
        }
    } else {
        json_error(
            (*pctx).cause,
            b"expected object\0" as *const u8 as *const ::core::ffi::c_char,
            (*pctx).input.offset((*tok).offset as isize),
        );
    }
    if !jn.is_null() {
        json_destroy_node(jn);
    }
    json_destroy_tokens(*tokens);
    *tokens = ::core::ptr::null_mut::<json_tokens>();
    return ::core::ptr::null_mut::<json_node>();
}
unsafe extern "C" fn json_parse_key(
    mut tok: *mut *mut json_token,
    mut pctx: *mut json_parse_ctx,
) -> *mut ::core::ffi::c_char {
    let mut len: ::core::ffi::c_int = 0;
    let mut loc: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut start: *const ::core::ffi::c_char = (*pctx).input.offset((**tok).offset as isize);
    let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !((**tok).type_0 as ::core::ffi::c_uint
        != TOK_QUOTE as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        *tok = (*tok).offset(1);
        loc = (*pctx).input.offset((**tok).offset as isize);
        len = (**tok).len;
        if !((**tok).type_0 as ::core::ffi::c_uint
            != TOK_VALUE as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            *tok = (*tok).offset(1);
            if !((**tok).type_0 as ::core::ffi::c_uint
                != TOK_QUOTE as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                key = xstrndup(loc, len as size_t);
                *tok = (*tok).offset(1);
                return key;
            }
        }
    }
    json_error(
        (*pctx).cause,
        b"invalid key\0" as *const u8 as *const ::core::ffi::c_char,
        start,
    );
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn json_parse_object(
    mut tok: *mut *mut json_token,
    mut pctx: *mut json_parse_ctx,
    mut key: *const ::core::ffi::c_char,
    mut parent: *mut json_node,
) -> *mut json_node {
    let mut current_block: u64;
    let mut object: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut fkey: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut valstr: *mut u_char = ::core::ptr::null_mut::<u_char>();
    if (**tok).type_0 as ::core::ffi::c_uint
        != TOK_OPENOBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<json_node>();
    }
    (*pctx).depth += 1;
    if (*pctx).depth > PARSE_DEPTH_MAX {
        json_error(
            (*pctx).cause,
            b"parse depth exceeded\0" as *const u8 as *const ::core::ffi::c_char,
            (*pctx).input.offset((**tok).offset as isize),
        );
        return ::core::ptr::null_mut::<json_node>();
    }
    *tok = (*tok).offset(1);
    object = json_create_node(parent, NODE_OBJECT, key, NULL);
    loop {
        if !((**tok).type_0 as ::core::ffi::c_uint
            != TOK_CLOSEOBJECT as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            current_block = 9853141518545631134;
            break;
        }
        fkey = json_parse_key(tok, pctx);
        if fkey.is_null() {
            current_block = 7971653673408253115;
            break;
        }
        if !json_find(object, fkey).is_null() {
            json_error(
                (*pctx).cause,
                b"duplicate key\0" as *const u8 as *const ::core::ffi::c_char,
                (*pctx).input.offset((**tok).offset as isize),
            );
            current_block = 7971653673408253115;
            break;
        } else if (**tok).type_0 as ::core::ffi::c_uint
            != TOK_COLON as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            json_error(
                (*pctx).cause,
                b"missing colon\0" as *const u8 as *const ::core::ffi::c_char,
                (*pctx).input.offset((**tok).offset as isize),
            );
            current_block = 7971653673408253115;
            break;
        } else {
            *tok = (*tok).offset(1);
            match (**tok).type_0 as ::core::ffi::c_uint {
                6 => {
                    field = json_parse_string(tok, pctx, fkey, object);
                }
                7 => {
                    valstr = (*pctx).input.offset((**tok).offset as isize) as *mut u_char;
                    if *valstr as ::core::ffi::c_int == '-' as i32
                        && *(*__ctype_b_loc())
                            .offset(*valstr.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                                as ::core::ffi::c_int
                            != 0
                        || *(*__ctype_b_loc()).offset(*valstr as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                                as ::core::ffi::c_int
                            != 0
                    {
                        field = json_parse_number(tok, pctx, fkey, object);
                    } else {
                        field = json_parse_boolean(tok, pctx, fkey, object);
                    }
                }
                0 => {
                    field = json_parse_object(tok, pctx, fkey, object);
                }
                2 => {
                    field = json_parse_array(tok, pctx, fkey, object);
                }
                _ => {
                    json_error(
                        (*pctx).cause,
                        b"unexpected value when parsing object\0" as *const u8
                            as *const ::core::ffi::c_char,
                        (*pctx).input.offset((**tok).offset as isize),
                    );
                    current_block = 7971653673408253115;
                    break;
                }
            }
            if field.is_null() {
                current_block = 7971653673408253115;
                break;
            }
            json_assign_value(object, field as *mut ::core::ffi::c_void);
            if (**tok).type_0 as ::core::ffi::c_uint
                == TOK_COMMA as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*(*tok).offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
                    == TOK_CLOSEOBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    json_error(
                        (*pctx).cause,
                        b"invalid object\0" as *const u8 as *const ::core::ffi::c_char,
                        (*pctx).input.offset((**tok).offset as isize),
                    );
                    current_block = 7971653673408253115;
                    break;
                } else {
                    *tok = (*tok).offset(1);
                }
            } else if (**tok).type_0 as ::core::ffi::c_uint
                != TOK_CLOSEOBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                json_error(
                    (*pctx).cause,
                    b"invalid object\0" as *const u8 as *const ::core::ffi::c_char,
                    (*pctx).input.offset((**tok).offset as isize),
                );
                current_block = 7971653673408253115;
                break;
            }
            free(fkey as *mut ::core::ffi::c_void);
        }
    }
    match current_block {
        7971653673408253115 => {
            if !fkey.is_null() {
                free(fkey as *mut ::core::ffi::c_void);
            }
            json_destroy_node(object);
            return ::core::ptr::null_mut::<json_node>();
        }
        _ => {
            *tok = (*tok).offset(1);
            (*pctx).depth -= 1;
            return object;
        }
    };
}
unsafe extern "C" fn json_parse_array(
    mut tok: *mut *mut json_token,
    mut pctx: *mut json_parse_ctx,
    mut key: *const ::core::ffi::c_char,
    mut parent: *mut json_node,
) -> *mut json_node {
    let mut current_block: u64;
    let mut array: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut member: *mut json_node = ::core::ptr::null_mut::<json_node>();
    if (**tok).type_0 as ::core::ffi::c_uint
        != TOK_OPENARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<json_node>();
    }
    *tok = (*tok).offset(1);
    array = json_create_node(parent, NODE_ARRAY, key, NULL);
    loop {
        if !((**tok).type_0 as ::core::ffi::c_uint
            != TOK_CLOSEARRAY as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            current_block = 1054647088692577877;
            break;
        }
        match (**tok).type_0 as ::core::ffi::c_uint {
            0 => {
                member =
                    json_parse_object(tok, pctx, ::core::ptr::null::<::core::ffi::c_char>(), array);
                if member.is_null() {
                    current_block = 15988181977378875837;
                    break;
                }
                json_assign_value(array, member as *mut ::core::ffi::c_void);
                if (**tok).type_0 as ::core::ffi::c_uint
                    == TOK_COMMA as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if (*(*tok).offset(1 as ::core::ffi::c_int as isize)).type_0
                        as ::core::ffi::c_uint
                        == TOK_CLOSEARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        json_error(
                            (*pctx).cause,
                            b"invalid array\0" as *const u8 as *const ::core::ffi::c_char,
                            (*pctx).input.offset((**tok).offset as isize),
                        );
                        current_block = 15988181977378875837;
                        break;
                    } else {
                        *tok = (*tok).offset(1);
                    }
                } else {
                    if !((**tok).type_0 as ::core::ffi::c_uint
                        != TOK_CLOSEARRAY as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        continue;
                    }
                    json_error(
                        (*pctx).cause,
                        b"invalid array\0" as *const u8 as *const ::core::ffi::c_char,
                        (*pctx).input.offset((**tok).offset as isize),
                    );
                    current_block = 15988181977378875837;
                    break;
                }
            }
            _ => {
                json_error(
                    (*pctx).cause,
                    b"invalid array member\0" as *const u8 as *const ::core::ffi::c_char,
                    (*pctx).input.offset((**tok).offset as isize),
                );
                current_block = 15988181977378875837;
                break;
            }
        }
    }
    match current_block {
        15988181977378875837 => {
            json_destroy_node(array);
            return ::core::ptr::null_mut::<json_node>();
        }
        _ => {
            *tok = (*tok).offset(1);
            return array;
        }
    };
}
unsafe extern "C" fn json_parse_string(
    mut tok: *mut *mut json_token,
    mut pctx: *mut json_parse_ctx,
    mut key: *const ::core::ffi::c_char,
    mut parent: *mut json_node,
) -> *mut json_node {
    let mut loc: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut start: *const ::core::ffi::c_char = (*pctx).input.offset((**tok).offset as isize);
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = 0;
    if !((**tok).type_0 as ::core::ffi::c_uint
        != TOK_QUOTE as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        *tok = (*tok).offset(1);
        if !((**tok).type_0 as ::core::ffi::c_uint
            != TOK_VALUE as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            loc = (*pctx).input.offset((**tok).offset as isize);
            len = (**tok).len;
            *tok = (*tok).offset(1);
            if !((**tok).type_0 as ::core::ffi::c_uint
                != TOK_QUOTE as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                *tok = (*tok).offset(1);
                str = xstrndup(loc, len as size_t);
                return json_create_node(parent, NODE_STRING, key, str as *mut ::core::ffi::c_void);
            }
        }
    }
    json_error(
        (*pctx).cause,
        b"invalid string\0" as *const u8 as *const ::core::ffi::c_char,
        start,
    );
    return ::core::ptr::null_mut::<json_node>();
}
unsafe extern "C" fn json_parse_number(
    mut tok: *mut *mut json_token,
    mut pctx: *mut json_parse_ctx,
    mut key: *const ::core::ffi::c_char,
    mut parent: *mut json_node,
) -> *mut json_node {
    let mut start: *const ::core::ffi::c_char = (*pctx).input.offset((**tok).offset as isize);
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut num: int64_t = 0;
    let mut len: ::core::ffi::c_int = (**tok).len;
    if !(*start.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '0' as i32
        && len != 1 as ::core::ffi::c_int
        || *start.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '-' as i32
            && *start.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '0' as i32
            && len != 2 as ::core::ffi::c_int)
    {
        *__errno_location() = 0 as ::core::ffi::c_int;
        num = strtoll(start, &raw mut endptr, 10 as ::core::ffi::c_int) as int64_t;
        if !(*__errno_location() != 0 as ::core::ffi::c_int
            || endptr != start.offset(len as isize) as *mut ::core::ffi::c_char)
        {
            *tok = (*tok).offset(1);
            return json_create_node(
                parent,
                NODE_NUMBER,
                key,
                &raw mut num as *mut ::core::ffi::c_void,
            );
        }
    }
    json_error(
        (*pctx).cause,
        b"invalid number\0" as *const u8 as *const ::core::ffi::c_char,
        start,
    );
    return ::core::ptr::null_mut::<json_node>();
}
unsafe extern "C" fn json_parse_boolean(
    mut tok: *mut *mut json_token,
    mut pctx: *mut json_parse_ctx,
    mut key: *const ::core::ffi::c_char,
    mut parent: *mut json_node,
) -> *mut json_node {
    let mut len: ::core::ffi::c_int = (**tok).len;
    let mut boolean: ::core::ffi::c_int = 0;
    let mut start: *const ::core::ffi::c_char = (*pctx).input.offset((**tok).offset as isize);
    if strncmp(
        start,
        b"true\0" as *const u8 as *const ::core::ffi::c_char,
        len as size_t,
    ) == 0 as ::core::ffi::c_int
        && len == 4 as ::core::ffi::c_int
    {
        boolean = 1 as ::core::ffi::c_int;
    } else if strncmp(
        start,
        b"false\0" as *const u8 as *const ::core::ffi::c_char,
        len as size_t,
    ) == 0 as ::core::ffi::c_int
        && len == 5 as ::core::ffi::c_int
    {
        boolean = 0 as ::core::ffi::c_int;
    } else {
        json_error(
            (*pctx).cause,
            b"invalid boolean\0" as *const u8 as *const ::core::ffi::c_char,
            start,
        );
        return ::core::ptr::null_mut::<json_node>();
    }
    *tok = (*tok).offset(1);
    return json_create_node(
        parent,
        NODE_BOOLEAN,
        key,
        &raw mut boolean as *mut ::core::ffi::c_void,
    );
}
unsafe extern "C" fn json_string_append(mut buffer: *mut evbuffer, mut node: *mut json_node) {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut member: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut comma: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match (*node).type_0 as ::core::ffi::c_uint {
        0 => {
            evbuffer_add_printf(
                buffer,
                b"\"%s\"\0" as *const u8 as *const ::core::ffi::c_char,
                (*node).c2rust_unnamed.str_0,
            );
        }
        1 => {
            evbuffer_add_printf(
                buffer,
                b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                (*node).c2rust_unnamed.num as ::core::ffi::c_longlong,
            );
        }
        2 => {
            if (*node).c2rust_unnamed.boolean != 0 {
                s = b"true\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                s = b"false\0" as *const u8 as *const ::core::ffi::c_char;
            }
            evbuffer_add(buffer, s as *const ::core::ffi::c_void, strlen(s));
        }
        3 => {
            evbuffer_add(
                buffer,
                b"{\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
            field = json_fields_RB_MINMAX(&raw mut (*node).c2rust_unnamed.fields, RB_NEGINF);
            while !field.is_null() {
                if comma != 0 {
                    evbuffer_add(
                        buffer,
                        b",\0" as *const u8 as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        1 as size_t,
                    );
                }
                evbuffer_add_printf(
                    buffer,
                    b"\"%s\":\0" as *const u8 as *const ::core::ffi::c_char,
                    (*field).key,
                );
                json_string_append(buffer, field);
                comma = 1 as ::core::ffi::c_int;
                field = json_fields_RB_NEXT(field);
            }
            evbuffer_add(
                buffer,
                b"}\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
        }
        4 => {
            evbuffer_add(
                buffer,
                b"[\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
            member = (*node).c2rust_unnamed.members.tqh_first;
            while !member.is_null() {
                if comma != 0 {
                    evbuffer_add(
                        buffer,
                        b",\0" as *const u8 as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        1 as size_t,
                    );
                }
                json_string_append(buffer, member);
                comma = 1 as ::core::ffi::c_int;
                member = (*member).aentry.tqe_next;
            }
            evbuffer_add(
                buffer,
                b"]\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn json_to_string(mut node: *mut json_node) -> *mut ::core::ffi::c_char {
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if node.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    json_string_append(buffer, node);
    out = xmemdup(
        evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
            as *const ::core::ffi::c_void,
        evbuffer_get_length(buffer),
    );
    evbuffer_free(buffer);
    return out;
}
