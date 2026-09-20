use crate::src::shared::abi::*;
use crate::src::shared::grid::*;
use crate::src::shared::style::*;
use crate::src::shared::utf8::*;
extern "C" {
    pub type options;
    pub type cmds;
    pub type options_array_item;
    pub type options_entry;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn __ctype_get_mb_cur_max() -> size_t;
    fn strtoull(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulonglong;
    fn mbtowc(
        __pwc: *mut wchar_t,
        __s: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn wctomb(__s: *mut ::core::ffi::c_char, __wchar: wchar_t) -> ::core::ffi::c_int;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn vis(
        _: *mut ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn utf8proc_wcwidth(_: wchar_t) -> ::core::ffi::c_int;
    fn utf8proc_mbtowc(
        _: *mut wchar_t,
        _: *const ::core::ffi::c_char,
        _: size_t,
    ) -> ::core::ffi::c_int;
    fn utf8proc_wctomb(_: *mut ::core::ffi::c_char, _: wchar_t) -> ::core::ffi::c_int;
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(_: *mut ::core::ffi::c_void, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut global_options: *mut options;
    fn options_get(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_array_first(_: *mut options_entry) -> *mut options_array_item;
    fn options_array_next(_: *mut options_array_item) -> *mut options_array_item;
    fn options_array_item_value(_: *mut options_array_item) -> *mut options_value;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}
pub type ssize_t = isize;
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
pub type wchar_t = ::libc::wchar_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_list {
    pub references: ::core::ffi::c_int,
    pub group: u_int,
    pub list: *mut cmds,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_array {
    pub rbh_root: *mut options_array_item,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union options_value {
    pub string: *mut ::core::ffi::c_char,
    pub number: ::core::ffi::c_longlong,
    pub style: style,
    pub array: options_array,
    pub cmdlist: *mut cmd_list,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct utf8_width_item {
    pub wc: wchar_t,
    pub width: u_int,
    pub allocated: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub rbe_left: *mut utf8_width_item,
    pub rbe_right: *mut utf8_width_item,
    pub rbe_parent: *mut utf8_width_item,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct utf8_width_cache {
    pub rbh_root: *mut utf8_width_item,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct utf8_item {
    pub index_entry: C2RustUnnamed_2,
    pub index: u_int,
    pub data_entry: C2RustUnnamed_1,
    pub data: [::core::ffi::c_char; 32],
    pub size: u_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {
    pub rbe_left: *mut utf8_item,
    pub rbe_right: *mut utf8_item,
    pub rbe_parent: *mut utf8_item,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_2 {
    pub rbe_left: *mut utf8_item,
    pub rbe_right: *mut utf8_item,
    pub rbe_parent: *mut utf8_item,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct utf8_data_tree {
    pub rbh_root: *mut utf8_item,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct utf8_index_tree {
    pub rbh_root: *mut utf8_item,
}
pub const ERANGE: ::core::ffi::c_int = 34 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const __WCHAR_MAX: ::core::ffi::c_int = __WCHAR_MAX__;
pub const ULLONG_MAX: ::core::ffi::c_ulonglong = (__LONG_LONG_MAX__ as ::core::ffi::c_ulonglong)
    .wrapping_mul(2 as ::core::ffi::c_ulonglong)
    .wrapping_add(1 as ::core::ffi::c_ulonglong);
pub const WCHAR_MAX: ::core::ffi::c_int = __WCHAR_MAX;
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const VIS_DQ: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const UTF8_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
unsafe extern "C" fn utf8_width_cache_cmp(
    mut uw1: *mut utf8_width_item,
    mut uw2: *mut utf8_width_item,
) -> ::core::ffi::c_int {
    if (*uw1).wc < (*uw2).wc {
        return -(1 as ::core::ffi::c_int);
    }
    if (*uw1).wc > (*uw2).wc {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn utf8_width_cache_RB_FIND(
    mut head: *mut utf8_width_cache,
    mut elm: *mut utf8_width_item,
) -> *mut utf8_width_item {
    let mut tmp: *mut utf8_width_item = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = utf8_width_cache_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<utf8_width_item>();
}
unsafe extern "C" fn utf8_width_cache_RB_REMOVE_COLOR(
    mut head: *mut utf8_width_cache,
    mut parent: *mut utf8_width_item,
    mut elm: *mut utf8_width_item,
) {
    let mut tmp: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    while (elm.is_null() || (*elm).entry.rbe_color == RB_BLACK) && elm != (*head).rbh_root {
        if (*parent).entry.rbe_left == elm {
            tmp = (*parent).entry.rbe_right;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_right;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK
                {
                    let mut oleft: *mut utf8_width_item =
                        ::core::ptr::null_mut::<utf8_width_item>();
                    oleft = (*tmp).entry.rbe_left;
                    if !oleft.is_null() {
                        (*oleft).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oleft = (*tmp).entry.rbe_left;
                    (*tmp).entry.rbe_left = (*oleft).entry.rbe_right;
                    if !(*tmp).entry.rbe_left.is_null() {
                        (*(*oleft).entry.rbe_right).entry.rbe_parent = tmp;
                    }
                    (*oleft).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oleft).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oleft;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oleft;
                        }
                    } else {
                        (*head).rbh_root = oleft;
                    }
                    (*oleft).entry.rbe_right = tmp;
                    (*tmp).entry.rbe_parent = oleft;
                    !(*oleft).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_right;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        } else {
            tmp = (*parent).entry.rbe_left;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_left;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_left.is_null()
                    || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK
                {
                    let mut oright: *mut utf8_width_item =
                        ::core::ptr::null_mut::<utf8_width_item>();
                    oright = (*tmp).entry.rbe_right;
                    if !oright.is_null() {
                        (*oright).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oright = (*tmp).entry.rbe_right;
                    (*tmp).entry.rbe_right = (*oright).entry.rbe_left;
                    if !(*tmp).entry.rbe_right.is_null() {
                        (*(*oright).entry.rbe_left).entry.rbe_parent = tmp;
                    }
                    (*oright).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oright).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oright;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oright;
                        }
                    } else {
                        (*head).rbh_root = oright;
                    }
                    (*oright).entry.rbe_left = tmp;
                    (*tmp).entry.rbe_parent = oright;
                    !(*oright).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_left;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        }
    }
    if !elm.is_null() {
        (*elm).entry.rbe_color = RB_BLACK;
    }
}
unsafe extern "C" fn utf8_width_cache_RB_INSERT(
    mut head: *mut utf8_width_cache,
    mut elm: *mut utf8_width_item,
) -> *mut utf8_width_item {
    let mut tmp: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut parent: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = utf8_width_cache_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<utf8_width_item>();
    (*elm).entry.rbe_left = (*elm).entry.rbe_right;
    (*elm).entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).entry.rbe_left = elm;
        } else {
            (*parent).entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    utf8_width_cache_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<utf8_width_item>();
}
unsafe extern "C" fn utf8_width_cache_RB_NEXT(
    mut elm: *mut utf8_width_item,
) -> *mut utf8_width_item {
    if !(*elm).entry.rbe_right.is_null() {
        elm = (*elm).entry.rbe_right;
        while !(*elm).entry.rbe_left.is_null() {
            elm = (*elm).entry.rbe_left;
        }
    } else if !(*elm).entry.rbe_parent.is_null() && elm == (*(*elm).entry.rbe_parent).entry.rbe_left
    {
        elm = (*elm).entry.rbe_parent;
    } else {
        while !(*elm).entry.rbe_parent.is_null()
            && elm == (*(*elm).entry.rbe_parent).entry.rbe_right
        {
            elm = (*elm).entry.rbe_parent;
        }
        elm = (*elm).entry.rbe_parent;
    }
    return elm;
}
unsafe extern "C" fn utf8_width_cache_RB_MINMAX(
    mut head: *mut utf8_width_cache,
    mut val: ::core::ffi::c_int,
) -> *mut utf8_width_item {
    let mut tmp: *mut utf8_width_item = (*head).rbh_root;
    let mut parent: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    while !tmp.is_null() {
        parent = tmp;
        if val < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else {
            tmp = (*tmp).entry.rbe_right;
        }
    }
    return parent;
}
unsafe extern "C" fn utf8_width_cache_RB_INSERT_COLOR(
    mut head: *mut utf8_width_cache,
    mut elm: *mut utf8_width_item,
) {
    let mut parent: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut gparent: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut tmp: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    loop {
        parent = (*elm).entry.rbe_parent;
        if !(!parent.is_null() && (*parent).entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).entry.rbe_parent;
        if parent == (*gparent).entry.rbe_left {
            tmp = (*gparent).entry.rbe_right;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_right == elm {
                    tmp = (*parent).entry.rbe_right;
                    (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                    if !(*parent).entry.rbe_right.is_null() {
                        (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_left = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_left;
                (*gparent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*gparent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).entry.rbe_left;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_left == elm {
                    tmp = (*parent).entry.rbe_left;
                    (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                    if !(*parent).entry.rbe_left.is_null() {
                        (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_right = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_right;
                (*gparent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*gparent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).entry.rbe_color = RB_BLACK;
}
unsafe extern "C" fn utf8_width_cache_RB_REMOVE(
    mut head: *mut utf8_width_cache,
    mut elm: *mut utf8_width_item,
) -> *mut utf8_width_item {
    let mut current_block: u64;
    let mut child: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut parent: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut old: *mut utf8_width_item = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
        elm = (*elm).entry.rbe_right;
        loop {
            left = (*elm).entry.rbe_left;
            if left.is_null() {
                break;
            }
            elm = left;
        }
        child = (*elm).entry.rbe_right;
        parent = (*elm).entry.rbe_parent;
        color = (*elm).entry.rbe_color;
        if !child.is_null() {
            (*child).entry.rbe_parent = parent;
        }
        if !parent.is_null() {
            if (*parent).entry.rbe_left == elm {
                (*parent).entry.rbe_left = child;
            } else {
                (*parent).entry.rbe_right = child;
            }
        } else {
            (*head).rbh_root = child;
        }
        if (*elm).entry.rbe_parent == old {
            parent = elm;
        }
        (*elm).entry = (*old).entry;
        if !(*old).entry.rbe_parent.is_null() {
            if (*(*old).entry.rbe_parent).entry.rbe_left == old {
                (*(*old).entry.rbe_parent).entry.rbe_left = elm;
            } else {
                (*(*old).entry.rbe_parent).entry.rbe_right = elm;
            }
        } else {
            (*head).rbh_root = elm;
        }
        (*(*old).entry.rbe_left).entry.rbe_parent = elm;
        if !(*old).entry.rbe_right.is_null() {
            (*(*old).entry.rbe_right).entry.rbe_parent = elm;
        }
        if !parent.is_null() {
            left = parent;
            loop {
                left = (*left).entry.rbe_parent;
                if left.is_null() {
                    break;
                }
            }
        }
        current_block = 9927394827938689620;
    }
    match current_block {
        7245201122033322888 => {
            parent = (*elm).entry.rbe_parent;
            color = (*elm).entry.rbe_color;
            if !child.is_null() {
                (*child).entry.rbe_parent = parent;
            }
            if !parent.is_null() {
                if (*parent).entry.rbe_left == elm {
                    (*parent).entry.rbe_left = child;
                } else {
                    (*parent).entry.rbe_right = child;
                }
            } else {
                (*head).rbh_root = child;
            }
        }
        _ => {}
    }
    if color == RB_BLACK {
        utf8_width_cache_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
static mut utf8_width_cache: utf8_width_cache = utf8_width_cache {
    rbh_root: ::core::ptr::null::<utf8_width_item>() as *mut utf8_width_item,
};
static mut utf8_default_width_cache: [utf8_width_item; 162] = [
    utf8_width_item {
        wc: 0x261d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x26f9 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x270a as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x270b as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x270c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x270d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1e6 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1e7 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1e8 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1e9 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1ea as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1eb as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1ec as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1ed as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1ee as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1ef as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1f0 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1f1 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1f2 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1f3 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1f4 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1f5 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1f6 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1f7 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1f8 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1f9 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1fa as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1fb as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1fc as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1fd as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1fe as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f1ff as wchar_t,
        width: 1 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f385 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f3c2 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f3c3 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f3c4 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f3c7 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f3ca as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f3cb as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f3cc as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f3fb as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f3fc as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f3fd as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f3fe as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f3ff as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f442 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f443 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f446 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f447 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f448 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f449 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f44a as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f44b as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f44c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f44d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f44e as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f44f as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f450 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f466 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f467 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f468 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f469 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f46b as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f46c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f46d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f46e as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f470 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f471 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f472 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f473 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f474 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f475 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f476 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f477 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f478 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f47c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f481 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f482 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f483 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f485 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f486 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f487 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f48f as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f491 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f4aa as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f574 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f575 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f57a as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f590 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f595 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f596 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f645 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f646 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f647 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f64b as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f64c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f64d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f64e as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f64f as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f6a3 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f6b4 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f6b5 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f6b6 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f6c0 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f6cc as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f90c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f90f as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f918 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f919 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f91a as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f91b as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f91c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f91d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f91e as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f91f as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f926 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f930 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f931 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f932 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f933 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f934 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f935 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f936 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f937 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f938 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f939 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f93d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f93e as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f977 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9b5 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9b6 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9b8 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9b9 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9bb as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9cd as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9ce as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9cf as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9d1 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9d2 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9d3 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9d4 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9d5 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9d6 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9d7 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9d8 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9d9 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9da as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9db as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9dc as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1f9dd as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1fac3 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1fac4 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1fac5 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1faf0 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1faf1 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1faf2 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1faf3 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1faf4 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1faf5 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1faf6 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1faf7 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
    utf8_width_item {
        wc: 0x1faf8 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    },
];
unsafe extern "C" fn utf8_data_cmp(
    mut ui1: *mut utf8_item,
    mut ui2: *mut utf8_item,
) -> ::core::ffi::c_int {
    if ((*ui1).size as ::core::ffi::c_int) < (*ui2).size as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    if (*ui1).size as ::core::ffi::c_int > (*ui2).size as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    return memcmp(
        &raw mut (*ui1).data as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
        &raw mut (*ui2).data as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
        (*ui1).size as size_t,
    );
}
unsafe extern "C" fn utf8_data_tree_RB_INSERT_COLOR(
    mut head: *mut utf8_data_tree,
    mut elm: *mut utf8_item,
) {
    let mut parent: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    let mut gparent: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    let mut tmp: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    loop {
        parent = (*elm).data_entry.rbe_parent;
        if !(!parent.is_null() && (*parent).data_entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).data_entry.rbe_parent;
        if parent == (*gparent).data_entry.rbe_left {
            tmp = (*gparent).data_entry.rbe_right;
            if !tmp.is_null() && (*tmp).data_entry.rbe_color == RB_RED {
                (*tmp).data_entry.rbe_color = RB_BLACK;
                (*parent).data_entry.rbe_color = RB_BLACK;
                (*gparent).data_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).data_entry.rbe_right == elm {
                    tmp = (*parent).data_entry.rbe_right;
                    (*parent).data_entry.rbe_right = (*tmp).data_entry.rbe_left;
                    if !(*parent).data_entry.rbe_right.is_null() {
                        (*(*tmp).data_entry.rbe_left).data_entry.rbe_parent = parent;
                    }
                    (*tmp).data_entry.rbe_parent = (*parent).data_entry.rbe_parent;
                    if !(*tmp).data_entry.rbe_parent.is_null() {
                        if parent == (*(*parent).data_entry.rbe_parent).data_entry.rbe_left {
                            (*(*parent).data_entry.rbe_parent).data_entry.rbe_left = tmp;
                        } else {
                            (*(*parent).data_entry.rbe_parent).data_entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).data_entry.rbe_left = parent;
                    (*parent).data_entry.rbe_parent = tmp;
                    !(*tmp).data_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).data_entry.rbe_color = RB_BLACK;
                (*gparent).data_entry.rbe_color = RB_RED;
                tmp = (*gparent).data_entry.rbe_left;
                (*gparent).data_entry.rbe_left = (*tmp).data_entry.rbe_right;
                if !(*gparent).data_entry.rbe_left.is_null() {
                    (*(*tmp).data_entry.rbe_right).data_entry.rbe_parent = gparent;
                }
                (*tmp).data_entry.rbe_parent = (*gparent).data_entry.rbe_parent;
                if !(*tmp).data_entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).data_entry.rbe_parent).data_entry.rbe_left {
                        (*(*gparent).data_entry.rbe_parent).data_entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).data_entry.rbe_parent).data_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).data_entry.rbe_right = gparent;
                (*gparent).data_entry.rbe_parent = tmp;
                !(*tmp).data_entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).data_entry.rbe_left;
            if !tmp.is_null() && (*tmp).data_entry.rbe_color == RB_RED {
                (*tmp).data_entry.rbe_color = RB_BLACK;
                (*parent).data_entry.rbe_color = RB_BLACK;
                (*gparent).data_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).data_entry.rbe_left == elm {
                    tmp = (*parent).data_entry.rbe_left;
                    (*parent).data_entry.rbe_left = (*tmp).data_entry.rbe_right;
                    if !(*parent).data_entry.rbe_left.is_null() {
                        (*(*tmp).data_entry.rbe_right).data_entry.rbe_parent = parent;
                    }
                    (*tmp).data_entry.rbe_parent = (*parent).data_entry.rbe_parent;
                    if !(*tmp).data_entry.rbe_parent.is_null() {
                        if parent == (*(*parent).data_entry.rbe_parent).data_entry.rbe_left {
                            (*(*parent).data_entry.rbe_parent).data_entry.rbe_left = tmp;
                        } else {
                            (*(*parent).data_entry.rbe_parent).data_entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).data_entry.rbe_right = parent;
                    (*parent).data_entry.rbe_parent = tmp;
                    !(*tmp).data_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).data_entry.rbe_color = RB_BLACK;
                (*gparent).data_entry.rbe_color = RB_RED;
                tmp = (*gparent).data_entry.rbe_right;
                (*gparent).data_entry.rbe_right = (*tmp).data_entry.rbe_left;
                if !(*gparent).data_entry.rbe_right.is_null() {
                    (*(*tmp).data_entry.rbe_left).data_entry.rbe_parent = gparent;
                }
                (*tmp).data_entry.rbe_parent = (*gparent).data_entry.rbe_parent;
                if !(*tmp).data_entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).data_entry.rbe_parent).data_entry.rbe_left {
                        (*(*gparent).data_entry.rbe_parent).data_entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).data_entry.rbe_parent).data_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).data_entry.rbe_left = gparent;
                (*gparent).data_entry.rbe_parent = tmp;
                !(*tmp).data_entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).data_entry.rbe_color = RB_BLACK;
}
unsafe extern "C" fn utf8_data_tree_RB_INSERT(
    mut head: *mut utf8_data_tree,
    mut elm: *mut utf8_item,
) -> *mut utf8_item {
    let mut tmp: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    let mut parent: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = utf8_data_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).data_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).data_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).data_entry.rbe_parent = parent;
    (*elm).data_entry.rbe_right = ::core::ptr::null_mut::<utf8_item>();
    (*elm).data_entry.rbe_left = (*elm).data_entry.rbe_right;
    (*elm).data_entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).data_entry.rbe_left = elm;
        } else {
            (*parent).data_entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    utf8_data_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<utf8_item>();
}
unsafe extern "C" fn utf8_data_tree_RB_FIND(
    mut head: *mut utf8_data_tree,
    mut elm: *mut utf8_item,
) -> *mut utf8_item {
    let mut tmp: *mut utf8_item = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = utf8_data_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).data_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).data_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<utf8_item>();
}
static mut utf8_data_tree: utf8_data_tree = utf8_data_tree {
    rbh_root: ::core::ptr::null::<utf8_item>() as *mut utf8_item,
};
unsafe extern "C" fn utf8_index_cmp(
    mut ui1: *mut utf8_item,
    mut ui2: *mut utf8_item,
) -> ::core::ffi::c_int {
    if (*ui1).index < (*ui2).index {
        return -(1 as ::core::ffi::c_int);
    }
    if (*ui1).index > (*ui2).index {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn utf8_index_tree_RB_INSERT_COLOR(
    mut head: *mut utf8_index_tree,
    mut elm: *mut utf8_item,
) {
    let mut parent: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    let mut gparent: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    let mut tmp: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    loop {
        parent = (*elm).index_entry.rbe_parent;
        if !(!parent.is_null() && (*parent).index_entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).index_entry.rbe_parent;
        if parent == (*gparent).index_entry.rbe_left {
            tmp = (*gparent).index_entry.rbe_right;
            if !tmp.is_null() && (*tmp).index_entry.rbe_color == RB_RED {
                (*tmp).index_entry.rbe_color = RB_BLACK;
                (*parent).index_entry.rbe_color = RB_BLACK;
                (*gparent).index_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).index_entry.rbe_right == elm {
                    tmp = (*parent).index_entry.rbe_right;
                    (*parent).index_entry.rbe_right = (*tmp).index_entry.rbe_left;
                    if !(*parent).index_entry.rbe_right.is_null() {
                        (*(*tmp).index_entry.rbe_left).index_entry.rbe_parent = parent;
                    }
                    (*tmp).index_entry.rbe_parent = (*parent).index_entry.rbe_parent;
                    if !(*tmp).index_entry.rbe_parent.is_null() {
                        if parent == (*(*parent).index_entry.rbe_parent).index_entry.rbe_left {
                            (*(*parent).index_entry.rbe_parent).index_entry.rbe_left = tmp;
                        } else {
                            (*(*parent).index_entry.rbe_parent).index_entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).index_entry.rbe_left = parent;
                    (*parent).index_entry.rbe_parent = tmp;
                    !(*tmp).index_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).index_entry.rbe_color = RB_BLACK;
                (*gparent).index_entry.rbe_color = RB_RED;
                tmp = (*gparent).index_entry.rbe_left;
                (*gparent).index_entry.rbe_left = (*tmp).index_entry.rbe_right;
                if !(*gparent).index_entry.rbe_left.is_null() {
                    (*(*tmp).index_entry.rbe_right).index_entry.rbe_parent = gparent;
                }
                (*tmp).index_entry.rbe_parent = (*gparent).index_entry.rbe_parent;
                if !(*tmp).index_entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).index_entry.rbe_parent).index_entry.rbe_left {
                        (*(*gparent).index_entry.rbe_parent).index_entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).index_entry.rbe_parent).index_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).index_entry.rbe_right = gparent;
                (*gparent).index_entry.rbe_parent = tmp;
                !(*tmp).index_entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).index_entry.rbe_left;
            if !tmp.is_null() && (*tmp).index_entry.rbe_color == RB_RED {
                (*tmp).index_entry.rbe_color = RB_BLACK;
                (*parent).index_entry.rbe_color = RB_BLACK;
                (*gparent).index_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).index_entry.rbe_left == elm {
                    tmp = (*parent).index_entry.rbe_left;
                    (*parent).index_entry.rbe_left = (*tmp).index_entry.rbe_right;
                    if !(*parent).index_entry.rbe_left.is_null() {
                        (*(*tmp).index_entry.rbe_right).index_entry.rbe_parent = parent;
                    }
                    (*tmp).index_entry.rbe_parent = (*parent).index_entry.rbe_parent;
                    if !(*tmp).index_entry.rbe_parent.is_null() {
                        if parent == (*(*parent).index_entry.rbe_parent).index_entry.rbe_left {
                            (*(*parent).index_entry.rbe_parent).index_entry.rbe_left = tmp;
                        } else {
                            (*(*parent).index_entry.rbe_parent).index_entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).index_entry.rbe_right = parent;
                    (*parent).index_entry.rbe_parent = tmp;
                    !(*tmp).index_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).index_entry.rbe_color = RB_BLACK;
                (*gparent).index_entry.rbe_color = RB_RED;
                tmp = (*gparent).index_entry.rbe_right;
                (*gparent).index_entry.rbe_right = (*tmp).index_entry.rbe_left;
                if !(*gparent).index_entry.rbe_right.is_null() {
                    (*(*tmp).index_entry.rbe_left).index_entry.rbe_parent = gparent;
                }
                (*tmp).index_entry.rbe_parent = (*gparent).index_entry.rbe_parent;
                if !(*tmp).index_entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).index_entry.rbe_parent).index_entry.rbe_left {
                        (*(*gparent).index_entry.rbe_parent).index_entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).index_entry.rbe_parent).index_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).index_entry.rbe_left = gparent;
                (*gparent).index_entry.rbe_parent = tmp;
                !(*tmp).index_entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).index_entry.rbe_color = RB_BLACK;
}
unsafe extern "C" fn utf8_index_tree_RB_INSERT(
    mut head: *mut utf8_index_tree,
    mut elm: *mut utf8_item,
) -> *mut utf8_item {
    let mut tmp: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    let mut parent: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = utf8_index_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).index_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).index_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).index_entry.rbe_parent = parent;
    (*elm).index_entry.rbe_right = ::core::ptr::null_mut::<utf8_item>();
    (*elm).index_entry.rbe_left = (*elm).index_entry.rbe_right;
    (*elm).index_entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).index_entry.rbe_left = elm;
        } else {
            (*parent).index_entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    utf8_index_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<utf8_item>();
}
unsafe extern "C" fn utf8_index_tree_RB_FIND(
    mut head: *mut utf8_index_tree,
    mut elm: *mut utf8_item,
) -> *mut utf8_item {
    let mut tmp: *mut utf8_item = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = utf8_index_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).index_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).index_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<utf8_item>();
}
static mut utf8_index_tree: utf8_index_tree = utf8_index_tree {
    rbh_root: ::core::ptr::null::<utf8_item>() as *mut utf8_item,
};
static mut utf8_no_width: ::core::ffi::c_int = 0;
static mut utf8_next_index: u_int = 0;
unsafe extern "C" fn utf8_item_by_data(
    mut data: *const u_char,
    mut size: size_t,
) -> *mut utf8_item {
    let mut ui: utf8_item = utf8_item {
        index_entry: C2RustUnnamed_2 {
            rbe_left: ::core::ptr::null_mut::<utf8_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_item>(),
            rbe_color: 0,
        },
        index: 0,
        data_entry: C2RustUnnamed_1 {
            rbe_left: ::core::ptr::null_mut::<utf8_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_item>(),
            rbe_color: 0,
        },
        data: [0; 32],
        size: 0,
    };
    memcpy(
        &raw mut ui.data as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        data as *const ::core::ffi::c_void,
        size,
    );
    ui.size = size as u_char;
    return utf8_data_tree_RB_FIND(&raw mut utf8_data_tree, &raw mut ui);
}
unsafe extern "C" fn utf8_item_by_index(mut index: u_int) -> *mut utf8_item {
    let mut ui: utf8_item = utf8_item {
        index_entry: C2RustUnnamed_2 {
            rbe_left: ::core::ptr::null_mut::<utf8_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_item>(),
            rbe_color: 0,
        },
        index: 0,
        data_entry: C2RustUnnamed_1 {
            rbe_left: ::core::ptr::null_mut::<utf8_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_item>(),
            rbe_color: 0,
        },
        data: [0; 32],
        size: 0,
    };
    ui.index = index;
    return utf8_index_tree_RB_FIND(&raw mut utf8_index_tree, &raw mut ui);
}
unsafe extern "C" fn utf8_find_in_width_cache(mut wc: wchar_t) -> *mut utf8_width_item {
    let mut uw: utf8_width_item = utf8_width_item {
        wc: 0,
        width: 0,
        allocated: 0,
        entry: C2RustUnnamed_0 {
            rbe_left: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_right: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_parent: ::core::ptr::null_mut::<utf8_width_item>(),
            rbe_color: 0,
        },
    };
    uw.wc = wc;
    return utf8_width_cache_RB_FIND(&raw mut utf8_width_cache, &raw mut uw);
}
unsafe extern "C" fn utf8_insert_width_cache(mut wc: wchar_t, mut width: u_int) {
    let mut uw: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut old: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    log_debug(
        b"Unicode width cache: %08X=%u\0" as *const u8 as *const ::core::ffi::c_char,
        wc as u_int,
        width,
    );
    uw = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<utf8_width_item>() as size_t,
    ) as *mut utf8_width_item;
    (*uw).wc = wc;
    (*uw).width = width;
    (*uw).allocated = 1 as ::core::ffi::c_int;
    old = utf8_width_cache_RB_INSERT(&raw mut utf8_width_cache, uw);
    if !old.is_null() {
        utf8_width_cache_RB_REMOVE(&raw mut utf8_width_cache, old);
        if (*old).allocated != 0 {
            free(old as *mut ::core::ffi::c_void);
        }
        utf8_width_cache_RB_INSERT(&raw mut utf8_width_cache, uw);
    }
}
unsafe extern "C" fn utf8_add_to_width_cache(mut s: *const ::core::ffi::c_char) {
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut width: u_int = 0;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ud: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut wc: wchar_t = 0;
    let mut wc_start: wchar_t = 0;
    let mut wc_end: wchar_t = 0;
    let mut n: ::core::ffi::c_ulonglong = 0;
    copy = xstrdup(s);
    cp = strchr(copy, '=' as i32);
    if cp.is_null() {
        free(copy as *mut ::core::ffi::c_void);
        return;
    }
    let fresh0 = cp;
    cp = cp.offset(1);
    *fresh0 = '\0' as i32 as ::core::ffi::c_char;
    width = strtonum(
        cp,
        0 as ::core::ffi::c_longlong,
        2 as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as u_int;
    if !errstr.is_null() {
        free(copy as *mut ::core::ffi::c_void);
        return;
    }
    if strncmp(
        copy,
        b"U+\0" as *const u8 as *const ::core::ffi::c_char,
        2 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        *__errno_location() = 0 as ::core::ffi::c_int;
        n = strtoull(
            copy.offset(2 as ::core::ffi::c_int as isize),
            &raw mut endptr,
            16 as ::core::ffi::c_int,
        );
        if *copy.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
            || n == 0 as ::core::ffi::c_ulonglong
            || n > WCHAR_MAX as ::core::ffi::c_ulonglong
            || *__errno_location() == ERANGE && n == ULLONG_MAX
        {
            free(copy as *mut ::core::ffi::c_void);
            return;
        }
        wc_start = n as wchar_t;
        if *endptr as ::core::ffi::c_int == '-' as i32 {
            endptr = endptr.offset(1);
            if strncmp(
                endptr,
                b"U+\0" as *const u8 as *const ::core::ffi::c_char,
                2 as size_t,
            ) != 0 as ::core::ffi::c_int
            {
                free(copy as *mut ::core::ffi::c_void);
                return;
            }
            *__errno_location() = 0 as ::core::ffi::c_int;
            n = strtoull(
                endptr.offset(2 as ::core::ffi::c_int as isize),
                &raw mut endptr,
                16 as ::core::ffi::c_int,
            );
            if *endptr as ::core::ffi::c_int != '\0' as i32
                || n == 0 as ::core::ffi::c_ulonglong
                || n > WCHAR_MAX as ::core::ffi::c_ulonglong
                || *__errno_location() == ERANGE && n == ULLONG_MAX
                || (n as wchar_t) < wc_start
            {
                free(copy as *mut ::core::ffi::c_void);
                return;
            }
            wc_end = n as wchar_t;
        } else {
            if *endptr as ::core::ffi::c_int != '\0' as i32 {
                free(copy as *mut ::core::ffi::c_void);
                return;
            }
            wc_end = wc_start;
        }
        wc = wc_start;
        while wc <= wc_end {
            utf8_insert_width_cache(wc, width);
            wc = wc.wrapping_add(1);
        }
    } else {
        utf8_no_width = 1 as ::core::ffi::c_int;
        ud = utf8_fromcstr(copy);
        utf8_no_width = 0 as ::core::ffi::c_int;
        if (*ud.offset(0 as ::core::ffi::c_int as isize)).size as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
            || (*ud.offset(1 as ::core::ffi::c_int as isize)).size as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        {
            free(ud as *mut ::core::ffi::c_void);
            free(copy as *mut ::core::ffi::c_void);
            return;
        }
        if utf8proc_mbtowc(
            &raw mut wc,
            &raw mut (*ud.offset(0 as ::core::ffi::c_int as isize)).data as *mut u_char
                as *const ::core::ffi::c_char,
            (*ud.offset(0 as ::core::ffi::c_int as isize)).size as size_t,
        ) <= 0 as ::core::ffi::c_int
        {
            free(ud as *mut ::core::ffi::c_void);
            free(copy as *mut ::core::ffi::c_void);
            return;
        }
        free(ud as *mut ::core::ffi::c_void);
        utf8_insert_width_cache(wc, width);
    }
    free(copy as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn utf8_update_width_cache() {
    let mut uw: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut uw1: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut i: u_int = 0;
    uw = utf8_width_cache_RB_MINMAX(&raw mut utf8_width_cache, RB_NEGINF);
    while !uw.is_null() && {
        uw1 = utf8_width_cache_RB_NEXT(uw);
        1 as ::core::ffi::c_int != 0
    } {
        utf8_width_cache_RB_REMOVE(&raw mut utf8_width_cache, uw);
        if (*uw).allocated != 0 {
            free(uw as *mut ::core::ffi::c_void);
        }
        uw = uw1;
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[utf8_width_item; 162]>() as usize)
            .wrapping_div(::core::mem::size_of::<utf8_width_item>() as usize)
    {
        utf8_width_cache_RB_INSERT(
            &raw mut utf8_width_cache,
            (&raw mut utf8_default_width_cache as *mut utf8_width_item).offset(i as isize)
                as *mut utf8_width_item,
        );
        i = i.wrapping_add(1);
    }
    o = options_get(
        global_options,
        b"codepoint-widths\0" as *const u8 as *const ::core::ffi::c_char,
    );
    a = options_array_first(o);
    while !a.is_null() {
        utf8_add_to_width_cache((*options_array_item_value(a)).string);
        a = options_array_next(a);
    }
}
unsafe extern "C" fn utf8_put_item(
    mut data: *const u_char,
    mut size: size_t,
    mut index: *mut u_int,
) -> ::core::ffi::c_int {
    let mut ui: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    ui = utf8_item_by_data(data, size);
    if !ui.is_null() {
        *index = (*ui).index;
        log_debug(
            b"%s: found %.*s = %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"utf8_put_item\0" as *const u8 as *const ::core::ffi::c_char,
            size as ::core::ffi::c_int,
            data,
            *index,
        );
        return 0 as ::core::ffi::c_int;
    }
    if utf8_next_index == (0xffffff as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as u_int {
        return -(1 as ::core::ffi::c_int);
    }
    ui = xcalloc(1 as size_t, ::core::mem::size_of::<utf8_item>() as size_t) as *mut utf8_item;
    let fresh2 = utf8_next_index;
    utf8_next_index = utf8_next_index.wrapping_add(1);
    (*ui).index = fresh2;
    utf8_index_tree_RB_INSERT(&raw mut utf8_index_tree, ui);
    memcpy(
        &raw mut (*ui).data as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        data as *const ::core::ffi::c_void,
        size,
    );
    (*ui).size = size as u_char;
    utf8_data_tree_RB_INSERT(&raw mut utf8_data_tree, ui);
    *index = (*ui).index;
    log_debug(
        b"%s: added %.*s = %u\0" as *const u8 as *const ::core::ffi::c_char,
        b"utf8_put_item\0" as *const u8 as *const ::core::ffi::c_char,
        size as ::core::ffi::c_int,
        data,
        *index,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_from_data(
    mut ud: *const utf8_data,
    mut uc: *mut utf8_char,
) -> utf8_state {
    let mut current_block: u64;
    let mut index: u_int = 0;
    if (*ud).width as ::core::ffi::c_int > 2 as ::core::ffi::c_int {
        fatalx(
            b"invalid UTF-8 width: %u\0" as *const u8 as *const ::core::ffi::c_char,
            (*ud).width as ::core::ffi::c_int,
        );
    }
    if !((*ud).size as ::core::ffi::c_int > UTF8_SIZE) {
        if (*ud).size as ::core::ffi::c_int <= 3 as ::core::ffi::c_int {
            index = (((*ud).data[2 as ::core::ffi::c_int as usize] as utf8_char)
                << 16 as ::core::ffi::c_int
                | ((*ud).data[1 as ::core::ffi::c_int as usize] as utf8_char)
                    << 8 as ::core::ffi::c_int
                | (*ud).data[0 as ::core::ffi::c_int as usize] as utf8_char)
                as u_int;
            current_block = 11875828834189669668;
        } else if utf8_put_item(
            &raw const (*ud).data as *const u_char,
            (*ud).size as size_t,
            &raw mut index,
        ) != 0 as ::core::ffi::c_int
        {
            current_block = 801095099472899353;
        } else {
            current_block = 11875828834189669668;
        }
        match current_block {
            801095099472899353 => {}
            _ => {
                *uc = (((*ud).size as u_int) << 24 as ::core::ffi::c_int
                    | ((*ud).width as u_int).wrapping_add(1 as u_int) << 29 as ::core::ffi::c_int
                    | index) as utf8_char;
                log_debug(
                    b"%s: (%d %d %.*s) -> %08x\0" as *const u8 as *const ::core::ffi::c_char,
                    b"utf8_from_data\0" as *const u8 as *const ::core::ffi::c_char,
                    (*ud).width as ::core::ffi::c_int,
                    (*ud).size as ::core::ffi::c_int,
                    (*ud).size as ::core::ffi::c_int,
                    &raw const (*ud).data as *const u_char,
                    *uc,
                );
                return UTF8_DONE;
            }
        }
    }
    if (*ud).width as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        *uc = (0 as ::core::ffi::c_int as utf8_char) << 24 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int as utf8_char).wrapping_add(1 as utf8_char)
                << 29 as ::core::ffi::c_int;
    } else if (*ud).width as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        *uc = (1 as ::core::ffi::c_int as utf8_char) << 24 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int as utf8_char).wrapping_add(1 as utf8_char)
                << 29 as ::core::ffi::c_int
            | 0x20 as utf8_char;
    } else {
        *uc = (1 as ::core::ffi::c_int as utf8_char) << 24 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int as utf8_char).wrapping_add(1 as utf8_char)
                << 29 as ::core::ffi::c_int
            | 0x2020 as utf8_char;
    }
    return UTF8_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_to_data(mut uc: utf8_char, mut ud: *mut utf8_data) {
    let mut ui: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    let mut index: u_int = 0;
    memset(
        ud as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<utf8_data>() as size_t,
    );
    (*ud).have = (uc >> 24 as ::core::ffi::c_int & 0x1f as utf8_char) as u_char;
    (*ud).size = (*ud).have;
    (*ud).width = (uc >> 29 as ::core::ffi::c_int).wrapping_sub(1 as utf8_char) as u_char;
    if (*ud).size as ::core::ffi::c_int <= 3 as ::core::ffi::c_int {
        (*ud).data[2 as ::core::ffi::c_int as usize] = (uc >> 16 as ::core::ffi::c_int) as u_char;
        (*ud).data[1 as ::core::ffi::c_int as usize] =
            (uc >> 8 as ::core::ffi::c_int & 0xff as utf8_char) as u_char;
        (*ud).data[0 as ::core::ffi::c_int as usize] = (uc & 0xff as utf8_char) as u_char;
    } else {
        index = (uc & 0xffffff as ::core::ffi::c_int as utf8_char) as u_int;
        ui = utf8_item_by_index(index);
        if ui.is_null() {
            memset(
                &raw mut (*ud).data as *mut u_char as *mut ::core::ffi::c_void,
                ' ' as i32,
                (*ud).size as size_t,
            );
        } else {
            memcpy(
                &raw mut (*ud).data as *mut u_char as *mut ::core::ffi::c_void,
                &raw mut (*ui).data as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                (*ud).size as size_t,
            );
        }
    }
    log_debug(
        b"%s: %08x -> (%d %d %.*s)\0" as *const u8 as *const ::core::ffi::c_char,
        b"utf8_to_data\0" as *const u8 as *const ::core::ffi::c_char,
        uc,
        (*ud).width as ::core::ffi::c_int,
        (*ud).size as ::core::ffi::c_int,
        (*ud).size as ::core::ffi::c_int,
        &raw mut (*ud).data as *mut u_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn utf8_build_one(mut ch: u_char) -> utf8_char {
    return (1 as ::core::ffi::c_int as utf8_char) << 24 as ::core::ffi::c_int
        | (1 as ::core::ffi::c_int as utf8_char).wrapping_add(1 as utf8_char)
            << 29 as ::core::ffi::c_int
        | ch as utf8_char;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_set(mut ud: *mut utf8_data, mut ch: u_char) {
    static mut empty: utf8_data = utf8_data {
        data: [
            0 as ::core::ffi::c_int as u_char,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        have: 1 as u_char,
        size: 1 as u_char,
        width: 1 as u_char,
    };
    memcpy(
        ud as *mut ::core::ffi::c_void,
        &raw const empty as *const ::core::ffi::c_void,
        ::core::mem::size_of::<utf8_data>() as size_t,
    );
    *(&raw mut (*ud).data as *mut u_char) = ch;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_copy(mut to: *mut utf8_data, mut from: *const utf8_data) {
    let mut i: u_int = 0;
    memcpy(
        to as *mut ::core::ffi::c_void,
        from as *const ::core::ffi::c_void,
        ::core::mem::size_of::<utf8_data>() as size_t,
    );
    i = (*to).size as u_int;
    while (i as usize) < ::core::mem::size_of::<[u_char; 32]>() as usize {
        (*to).data[i as usize] = '\0' as i32 as u_char;
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn utf8_width(
    mut ud: *mut utf8_data,
    mut width: *mut ::core::ffi::c_int,
) -> utf8_state {
    let mut uw: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut wc: wchar_t = 0;
    if utf8_towc(ud, &raw mut wc) as ::core::ffi::c_uint
        != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return UTF8_ERROR;
    }
    uw = utf8_find_in_width_cache(wc);
    if !uw.is_null() {
        *width = (*uw).width as ::core::ffi::c_int;
        log_debug(
            b"cached width for %08X is %d\0" as *const u8 as *const ::core::ffi::c_char,
            wc as u_int,
            *width,
        );
        return UTF8_DONE;
    }
    *width = utf8proc_wcwidth(wc);
    log_debug(
        b"utf8proc_wcwidth(%05X) returned %d\0" as *const u8 as *const ::core::ffi::c_char,
        wc as u_int,
        *width,
    );
    if *width >= 0 as ::core::ffi::c_int && *width <= 0xff as ::core::ffi::c_int {
        return UTF8_DONE;
    }
    return UTF8_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_towc(mut ud: *const utf8_data, mut wc: *mut wchar_t) -> utf8_state {
    match utf8proc_mbtowc(
        wc,
        &raw const (*ud).data as *const u_char as *const ::core::ffi::c_char,
        (*ud).size as size_t,
    ) {
        -1 => {
            log_debug(
                b"UTF-8 %.*s, mbtowc() %d\0" as *const u8 as *const ::core::ffi::c_char,
                (*ud).size as ::core::ffi::c_int,
                &raw const (*ud).data as *const u_char,
                *__errno_location(),
            );
            mbtowc(
                ::core::ptr::null_mut::<wchar_t>(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                __ctype_get_mb_cur_max(),
            );
            return UTF8_ERROR;
        }
        0 => return UTF8_ERROR,
        _ => {}
    }
    log_debug(
        b"UTF-8 %.*s is U+%06X\0" as *const u8 as *const ::core::ffi::c_char,
        (*ud).size as ::core::ffi::c_int,
        &raw const (*ud).data as *const u_char,
        *wc as u_int,
    );
    return UTF8_DONE;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_has_whitespace(mut ud: *const utf8_data) -> ::core::ffi::c_int {
    let mut tmp: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut wc: wchar_t = 0;
    let mut offset: u_int = 0 as u_int;
    let mut size: u_int = 0;
    let mut ch: u_char = 0;
    while offset < (*ud).size as u_int {
        ch = (*ud).data[offset as usize];
        if (ch as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
            wc = ch as wchar_t;
            size = 1 as u_int;
        } else {
            if ch as ::core::ffi::c_int >= 0xc2 as ::core::ffi::c_int
                && ch as ::core::ffi::c_int <= 0xdf as ::core::ffi::c_int
            {
                size = 2 as u_int;
            } else if ch as ::core::ffi::c_int >= 0xe0 as ::core::ffi::c_int
                && ch as ::core::ffi::c_int <= 0xef as ::core::ffi::c_int
            {
                size = 3 as u_int;
            } else if ch as ::core::ffi::c_int >= 0xf0 as ::core::ffi::c_int
                && ch as ::core::ffi::c_int <= 0xf4 as ::core::ffi::c_int
            {
                size = 4 as u_int;
            } else {
                return 0 as ::core::ffi::c_int;
            }
            if size > ((*ud).size as u_int).wrapping_sub(offset) {
                return 0 as ::core::ffi::c_int;
            }
            memset(
                &raw mut tmp as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<utf8_data>() as size_t,
            );
            memcpy(
                &raw mut tmp.data as *mut u_char as *mut ::core::ffi::c_void,
                (&raw const (*ud).data as *const u_char).offset(offset as isize)
                    as *const ::core::ffi::c_void,
                size as size_t,
            );
            tmp.have = size as u_char;
            tmp.size = tmp.have;
            if utf8_towc(&raw mut tmp, &raw mut wc) as ::core::ffi::c_uint
                != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return 0 as ::core::ffi::c_int;
            }
        }
        offset = offset.wrapping_add(size);
        match wc {
            9 | 10 | 11 | 12 | 13 | 32 | 133 | 160 | 5760 | 8192 | 8193 | 8194 | 8195 | 8196
            | 8197 | 8198 | 8199 | 8200 | 8201 | 8202 | 8232 | 8233 | 8239 | 8287 | 12288 => {
                return 1 as ::core::ffi::c_int
            }
            _ => {}
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_fromwc(mut wc: wchar_t, mut ud: *mut utf8_data) -> utf8_state {
    let mut size: ::core::ffi::c_int = 0;
    let mut width: ::core::ffi::c_int = 0;
    size = utf8proc_wctomb(
        &raw mut (*ud).data as *mut u_char as *mut ::core::ffi::c_char,
        wc,
    );
    if size < 0 as ::core::ffi::c_int {
        log_debug(
            b"UTF-8 %d, wctomb() %d\0" as *const u8 as *const ::core::ffi::c_char,
            wc,
            *__errno_location(),
        );
        wctomb(::core::ptr::null_mut::<::core::ffi::c_char>(), 0 as wchar_t);
        return UTF8_ERROR;
    }
    if size == 0 as ::core::ffi::c_int {
        return UTF8_ERROR;
    }
    (*ud).have = size as u_char;
    (*ud).size = (*ud).have;
    if utf8_width(ud, &raw mut width) as ::core::ffi::c_uint
        == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*ud).width = width as u_char;
        return UTF8_DONE;
    }
    return UTF8_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_open(mut ud: *mut utf8_data, mut ch: u_char) -> utf8_state {
    memset(
        ud as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<utf8_data>() as size_t,
    );
    if ch as ::core::ffi::c_int >= 0xc2 as ::core::ffi::c_int
        && ch as ::core::ffi::c_int <= 0xdf as ::core::ffi::c_int
    {
        (*ud).size = 2 as u_char;
    } else if ch as ::core::ffi::c_int >= 0xe0 as ::core::ffi::c_int
        && ch as ::core::ffi::c_int <= 0xef as ::core::ffi::c_int
    {
        (*ud).size = 3 as u_char;
    } else if ch as ::core::ffi::c_int >= 0xf0 as ::core::ffi::c_int
        && ch as ::core::ffi::c_int <= 0xf4 as ::core::ffi::c_int
    {
        (*ud).size = 4 as u_char;
    } else {
        return UTF8_ERROR;
    }
    utf8_append(ud, ch);
    return UTF8_MORE;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_append(mut ud: *mut utf8_data, mut ch: u_char) -> utf8_state {
    let mut width: ::core::ffi::c_int = 0;
    if (*ud).have as ::core::ffi::c_int >= (*ud).size as ::core::ffi::c_int {
        fatalx(b"UTF-8 character overflow\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if (*ud).size as usize > ::core::mem::size_of::<[u_char; 32]>() as usize {
        fatalx(b"UTF-8 character size too large\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if (*ud).have as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        && ch as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int != 0x80 as ::core::ffi::c_int
    {
        (*ud).width = 0xff as u_char;
    }
    let fresh1 = (*ud).have;
    (*ud).have = (*ud).have.wrapping_add(1);
    (*ud).data[fresh1 as usize] = ch;
    if (*ud).have as ::core::ffi::c_int != (*ud).size as ::core::ffi::c_int {
        return UTF8_MORE;
    }
    if utf8_no_width == 0 {
        if (*ud).width as ::core::ffi::c_int == 0xff as ::core::ffi::c_int {
            return UTF8_ERROR;
        }
        if utf8_width(ud, &raw mut width) as ::core::ffi::c_uint
            != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return UTF8_ERROR;
        }
        (*ud).width = width as u_char;
    }
    return UTF8_DONE;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_strvis(
    mut dst: *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    mut len: size_t,
    mut flag: ::core::ffi::c_int,
) -> size_t {
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut start: *const ::core::ffi::c_char = dst;
    let mut end: *const ::core::ffi::c_char = src.offset(len as isize);
    let mut more: utf8_state = UTF8_MORE;
    let mut i: size_t = 0;
    while src < end {
        more = utf8_open(&raw mut ud, *src as u_char);
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            loop {
                src = src.offset(1);
                if !(src < end
                    && more as ::core::ffi::c_uint
                        == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                more = utf8_append(&raw mut ud, *src as u_char);
            }
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                i = 0 as size_t;
                while i < ud.size as size_t {
                    let fresh3 = dst;
                    dst = dst.offset(1);
                    *fresh3 = ud.data[i as usize] as ::core::ffi::c_char;
                    i = i.wrapping_add(1);
                }
                continue;
            } else {
                src = src.offset(-(ud.have as ::core::ffi::c_int as isize));
            }
        }
        if flag & VIS_DQ != 0
            && *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '$' as i32
            && src < end.offset(-(1 as ::core::ffi::c_int as isize))
        {
            if *(*__ctype_b_loc()).offset(*src.offset(1 as ::core::ffi::c_int as isize) as u_char
                as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
                || *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '_' as i32
                || *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '{' as i32
            {
                let fresh4 = dst;
                dst = dst.offset(1);
                *fresh4 = '\\' as i32 as ::core::ffi::c_char;
            }
            let fresh5 = dst;
            dst = dst.offset(1);
            *fresh5 = '$' as i32 as ::core::ffi::c_char;
        } else if src < end.offset(-(1 as ::core::ffi::c_int as isize)) {
            dst = vis(
                dst,
                *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                flag,
                *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            );
        } else if src < end {
            dst = vis(
                dst,
                *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                flag,
                '\0' as i32,
            );
        }
        src = src.offset(1);
    }
    *dst = '\0' as i32 as ::core::ffi::c_char;
    return dst.offset_from(start) as ::core::ffi::c_long as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_stravis(
    mut dst: *mut *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    mut flag: ::core::ffi::c_int,
) -> size_t {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    buf = xreallocarray(NULL, 4 as size_t, strlen(src).wrapping_add(1 as size_t))
        as *mut ::core::ffi::c_char;
    len = utf8_strvis(buf, src, strlen(src), flag);
    *dst = xrealloc(
        buf as *mut ::core::ffi::c_void,
        len.wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_stravisx(
    mut dst: *mut *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    mut srclen: size_t,
    mut flag: ::core::ffi::c_int,
) -> size_t {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    buf = xreallocarray(NULL, 4 as size_t, srclen.wrapping_add(1 as size_t))
        as *mut ::core::ffi::c_char;
    len = utf8_strvis(buf, src, srclen, flag);
    *dst = xrealloc(
        buf as *mut ::core::ffi::c_void,
        len.wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_isvalid(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut more: utf8_state = UTF8_MORE;
    end = s.offset(strlen(s) as isize);
    while s < end {
        more = utf8_open(&raw mut ud, *s as u_char);
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            loop {
                s = s.offset(1);
                if !(s < end
                    && more as ::core::ffi::c_uint
                        == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                more = utf8_append(&raw mut ud, *s as u_char);
            }
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                continue;
            }
            return 0 as ::core::ffi::c_int;
        } else {
            if (*s as ::core::ffi::c_int) < 0x20 as ::core::ffi::c_int
                || *s as ::core::ffi::c_int > 0x7e as ::core::ffi::c_int
            {
                return 0 as ::core::ffi::c_int;
            }
            s = s.offset(1);
        }
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_sanitize(
    mut src: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut dst: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: size_t = 0 as size_t;
    let mut more: utf8_state = UTF8_MORE;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut i: u_int = 0;
    while *src as ::core::ffi::c_int != '\0' as i32 {
        dst = xreallocarray(
            dst as *mut ::core::ffi::c_void,
            n.wrapping_add(1 as size_t),
            ::core::mem::size_of::<::core::ffi::c_char>() as size_t,
        ) as *mut ::core::ffi::c_char;
        more = utf8_open(&raw mut ud, *src as u_char);
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            loop {
                src = src.offset(1);
                if !(*src as ::core::ffi::c_int != '\0' as i32
                    && more as ::core::ffi::c_uint
                        == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                more = utf8_append(&raw mut ud, *src as u_char);
            }
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                dst = xreallocarray(
                    dst as *mut ::core::ffi::c_void,
                    n.wrapping_add(ud.width as size_t),
                    ::core::mem::size_of::<::core::ffi::c_char>() as size_t,
                ) as *mut ::core::ffi::c_char;
                i = 0 as u_int;
                while i < ud.width as u_int {
                    let fresh6 = n;
                    n = n.wrapping_add(1);
                    *dst.offset(fresh6 as isize) = '_' as i32 as ::core::ffi::c_char;
                    i = i.wrapping_add(1);
                }
                continue;
            } else {
                src = src.offset(-(ud.have as ::core::ffi::c_int as isize));
            }
        }
        if *src as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
            && (*src as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
        {
            let fresh7 = n;
            n = n.wrapping_add(1);
            *dst.offset(fresh7 as isize) = *src;
        } else {
            let fresh8 = n;
            n = n.wrapping_add(1);
            *dst.offset(fresh8 as isize) = '_' as i32 as ::core::ffi::c_char;
        }
        src = src.offset(1);
    }
    dst = xreallocarray(
        dst as *mut ::core::ffi::c_void,
        n.wrapping_add(1 as size_t),
        ::core::mem::size_of::<::core::ffi::c_char>() as size_t,
    ) as *mut ::core::ffi::c_char;
    *dst.offset(n as isize) = '\0' as i32 as ::core::ffi::c_char;
    return dst;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_strlen(mut s: *const utf8_data) -> size_t {
    let mut i: size_t = 0;
    i = 0 as size_t;
    while (*s.offset(i as isize)).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        i = i.wrapping_add(1);
    }
    return i;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_strwidth(mut s: *const utf8_data, mut n: ssize_t) -> u_int {
    let mut i: ssize_t = 0;
    let mut width: u_int = 0 as u_int;
    i = 0 as ssize_t;
    while (*s.offset(i as isize)).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        if n != -(1 as ::core::ffi::c_int) as ssize_t && n == i {
            break;
        }
        width = width.wrapping_add((*s.offset(i as isize)).width as u_int);
        i += 1;
    }
    return width;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_fromcstr(mut src: *const ::core::ffi::c_char) -> *mut utf8_data {
    let mut dst: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut n: size_t = 0 as size_t;
    let mut more: utf8_state = UTF8_MORE;
    while *src as ::core::ffi::c_int != '\0' as i32 {
        dst = xreallocarray(
            dst as *mut ::core::ffi::c_void,
            n.wrapping_add(1 as size_t),
            ::core::mem::size_of::<utf8_data>() as size_t,
        ) as *mut utf8_data;
        more = utf8_open(dst.offset(n as isize) as *mut utf8_data, *src as u_char);
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            loop {
                src = src.offset(1);
                if !(*src as ::core::ffi::c_int != '\0' as i32
                    && more as ::core::ffi::c_uint
                        == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                more = utf8_append(dst.offset(n as isize) as *mut utf8_data, *src as u_char);
            }
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                n = n.wrapping_add(1);
                continue;
            } else {
                src = src.offset(-((*dst.offset(n as isize)).have as ::core::ffi::c_int as isize));
            }
        }
        utf8_set(dst.offset(n as isize) as *mut utf8_data, *src as u_char);
        n = n.wrapping_add(1);
        src = src.offset(1);
    }
    dst = xreallocarray(
        dst as *mut ::core::ffi::c_void,
        n.wrapping_add(1 as size_t),
        ::core::mem::size_of::<utf8_data>() as size_t,
    ) as *mut utf8_data;
    (*dst.offset(n as isize)).size = 0 as u_char;
    return dst;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_tocstr(mut src: *mut utf8_data) -> *mut ::core::ffi::c_char {
    let mut dst: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: size_t = 0 as size_t;
    while (*src).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        dst = xreallocarray(
            dst as *mut ::core::ffi::c_void,
            n.wrapping_add((*src).size as size_t),
            1 as size_t,
        ) as *mut ::core::ffi::c_char;
        memcpy(
            dst.offset(n as isize) as *mut ::core::ffi::c_void,
            &raw mut (*src).data as *mut u_char as *const ::core::ffi::c_void,
            (*src).size as size_t,
        );
        n = n.wrapping_add((*src).size as size_t);
        src = src.offset(1);
    }
    dst = xreallocarray(
        dst as *mut ::core::ffi::c_void,
        n.wrapping_add(1 as size_t),
        1 as size_t,
    ) as *mut ::core::ffi::c_char;
    *dst.offset(n as isize) = '\0' as i32 as ::core::ffi::c_char;
    return dst;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_cstrwidth(mut s: *const ::core::ffi::c_char) -> u_int {
    let mut tmp: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut width: u_int = 0;
    let mut more: utf8_state = UTF8_MORE;
    width = 0 as u_int;
    while *s as ::core::ffi::c_int != '\0' as i32 {
        more = utf8_open(&raw mut tmp, *s as u_char);
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            loop {
                s = s.offset(1);
                if !(*s as ::core::ffi::c_int != '\0' as i32
                    && more as ::core::ffi::c_uint
                        == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                more = utf8_append(&raw mut tmp, *s as u_char);
            }
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                width = width.wrapping_add(tmp.width as u_int);
                continue;
            } else {
                s = s.offset(-(tmp.have as ::core::ffi::c_int as isize));
            }
        }
        if *s as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
            && *s as ::core::ffi::c_int != 0x7f as ::core::ffi::c_int
        {
            width = width.wrapping_add(1);
        }
        s = s.offset(1);
    }
    return width;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_padcstr(
    mut s: *const ::core::ffi::c_char,
    mut width: u_int,
) -> *mut ::core::ffi::c_char {
    let mut slen: size_t = 0;
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: u_int = 0;
    let mut i: u_int = 0;
    n = utf8_cstrwidth(s);
    if n >= width {
        return xstrdup(s);
    }
    slen = strlen(s);
    out = xmalloc(
        slen.wrapping_add(1 as size_t)
            .wrapping_add(width.wrapping_sub(n) as size_t),
    ) as *mut ::core::ffi::c_char;
    memcpy(
        out as *mut ::core::ffi::c_void,
        s as *const ::core::ffi::c_void,
        slen,
    );
    i = n;
    while i < width {
        let fresh9 = slen;
        slen = slen.wrapping_add(1);
        *out.offset(fresh9 as isize) = ' ' as i32 as ::core::ffi::c_char;
        i = i.wrapping_add(1);
    }
    *out.offset(slen as isize) = '\0' as i32 as ::core::ffi::c_char;
    return out;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_rpadcstr(
    mut s: *const ::core::ffi::c_char,
    mut width: u_int,
) -> *mut ::core::ffi::c_char {
    let mut slen: size_t = 0;
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: u_int = 0;
    let mut i: u_int = 0;
    n = utf8_cstrwidth(s);
    if n >= width {
        return xstrdup(s);
    }
    slen = strlen(s);
    out = xmalloc(
        slen.wrapping_add(1 as size_t)
            .wrapping_add(width.wrapping_sub(n) as size_t),
    ) as *mut ::core::ffi::c_char;
    i = 0 as u_int;
    while i < width.wrapping_sub(n) {
        *out.offset(i as isize) = ' ' as i32 as ::core::ffi::c_char;
        i = i.wrapping_add(1);
    }
    memcpy(
        out.offset(i as isize) as *mut ::core::ffi::c_void,
        s as *const ::core::ffi::c_void,
        slen,
    );
    *out.offset((i as size_t).wrapping_add(slen) as isize) = '\0' as i32 as ::core::ffi::c_char;
    return out;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_cstrhas(
    mut s: *const ::core::ffi::c_char,
    mut ud: *const utf8_data,
) -> ::core::ffi::c_int {
    let mut copy: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut loop_0: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    copy = utf8_fromcstr(s);
    loop_0 = copy;
    while (*loop_0).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        if !((*loop_0).size as ::core::ffi::c_int != (*ud).size as ::core::ffi::c_int) {
            if memcmp(
                &raw mut (*loop_0).data as *mut u_char as *const ::core::ffi::c_void,
                &raw const (*ud).data as *const u_char as *const ::core::ffi::c_void,
                (*loop_0).size as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                found = 1 as ::core::ffi::c_int;
                break;
            }
        }
        loop_0 = loop_0.offset(1);
    }
    free(copy as *mut ::core::ffi::c_void);
    return found;
}
pub const __LONG_LONG_MAX__: ::core::ffi::c_longlong =
    9223372036854775807 as ::core::ffi::c_longlong;
pub const __WCHAR_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
