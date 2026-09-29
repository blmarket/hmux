use crate::src::arguments::args_string_percentage_result;
use crate::src::events::events_fire_window;
use crate::src::ffi::libc::{strcmp, strlen, strncmp};
use crate::src::layout::{
    layout_cell_is_tiled, layout_create_cell, layout_fix_offsets, layout_fix_panes,
    layout_make_node, layout_print_cell, layout_resize_adjust, layout_set_size, layout_spread_cell,
    layout_take_leaf, layout_take_leaves,
};
use crate::src::options::options_owner_ptr;
use crate::src::options::{options_get_number, options_get_string};
use crate::src::server_fn::server_redraw_window;
use crate::src::shared::abi::*;
use crate::src::shared::layout::layout_cell;
use crate::src::shared::layout::*;
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_MINIMUM;
use crate::src::shared::window::window;
use crate::src::window::{window_count_panes, window_pane_first, window_pane_next, window_resize};
use std::ffi::CStr;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub name: &'static CStr,
    pub arrange: Option<fn(&std::rc::Rc<std::cell::UnsafeCell<window>>)>,
}
static layout_sets: [C2RustUnnamed_35; 7] = {
    [
        C2RustUnnamed_35 {
            name: c"even-horizontal",
            arrange: Some(layout_set_even_h_callback),
        },
        C2RustUnnamed_35 {
            name: c"even-vertical",
            arrange: Some(layout_set_even_v_callback),
        },
        C2RustUnnamed_35 {
            name: c"main-horizontal",
            arrange: Some(layout_set_main_h_callback),
        },
        C2RustUnnamed_35 {
            name: c"main-horizontal-mirrored",
            arrange: Some(layout_set_main_h_mirrored_callback),
        },
        C2RustUnnamed_35 {
            name: c"main-vertical",
            arrange: Some(layout_set_main_v_callback),
        },
        C2RustUnnamed_35 {
            name: c"main-vertical-mirrored",
            arrange: Some(layout_set_main_v_mirrored_callback),
        },
        C2RustUnnamed_35 {
            name: c"tiled",
            arrange: Some(layout_set_tiled_callback),
        },
    ]
};
pub unsafe fn layout_set_lookup(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    let mut matched: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
    {
        if strcmp(layout_sets[i as usize].name.as_ptr(), name) == 0 as ::core::ffi::c_int {
            return i as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
    {
        if strncmp(layout_sets[i as usize].name.as_ptr(), name, strlen(name))
            == 0 as ::core::ffi::c_int
        {
            if matched != -(1 as ::core::ffi::c_int) {
                return -(1 as ::core::ffi::c_int);
            }
            matched = i as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return matched;
}
pub unsafe fn layout_set_select(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut layout: u_int,
) -> u_int {
    let mut w = w_owner.get();
    if layout as usize
        > (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
            .wrapping_sub(1 as usize)
    {
        layout = (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
            .wrapping_sub(1 as usize) as u_int;
    }
    if layout_sets[layout as usize].arrange.is_some() {
        layout_sets[layout as usize]
            .arrange
            .expect("non-null function pointer")(w_owner);
    }
    (*w).lastlayout = layout as ::core::ffi::c_int;
    return layout;
}
pub unsafe fn layout_set_next(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) -> u_int {
    let mut w = w_owner.get();
    let mut layout: u_int = 0;
    if (*w).lastlayout == -(1 as ::core::ffi::c_int) {
        layout = 0 as u_int;
    } else {
        layout = ((*w).lastlayout + 1 as ::core::ffi::c_int) as u_int;
        if layout as usize
            > (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
                .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
                .wrapping_sub(1 as usize)
        {
            layout = 0 as u_int;
        }
    }
    if layout_sets[layout as usize].arrange.is_some() {
        layout_sets[layout as usize]
            .arrange
            .expect("non-null function pointer")(w_owner);
    }
    (*w).lastlayout = layout as ::core::ffi::c_int;
    return layout;
}
pub unsafe fn layout_set_previous(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) -> u_int {
    let mut w = w_owner.get();
    let mut layout: u_int = 0;
    if (*w).lastlayout == -(1 as ::core::ffi::c_int) {
        layout = (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
            .wrapping_sub(1 as usize) as u_int;
    } else {
        layout = (*w).lastlayout as u_int;
        if layout == 0 as u_int {
            layout = (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
                .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
                .wrapping_sub(1 as usize) as u_int;
        } else {
            layout = layout.wrapping_sub(1);
        }
    }
    if layout_sets[layout as usize].arrange.is_some() {
        layout_sets[layout as usize]
            .arrange
            .expect("non-null function pointer")(w_owner);
    }
    (*w).lastlayout = layout as ::core::ffi::c_int;
    return layout;
}
unsafe fn layout_set_first_tiled(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
    let mut cursor = window_pane_first(Some(&*w_owner.get()));
    while let Some(pane) = cursor {
        let wp = pane.get();
        if !(*wp).layout_cell.is_null()
            && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) != 0
        {
            return Some(pane);
        }
        cursor = window_pane_next(Some(&*wp));
    }
    None
}
unsafe fn layout_set_link_floating(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut lcroot: *mut layout_cell,
    leaves: &mut Vec<Box<layout_cell>>,
) {
    let mut w = w_owner.get();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    wp = window_pane_first(w.as_ref())
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !wp.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
        if layout_cell_is_tiled(lc) == 0 {
            layout_cells_push_back(lcroot, layout_take_leaf(leaves, lc));
        }
        wp = window_pane_next(wp.as_ref())
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}

unsafe fn layout_set_even(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut type_0: layout_type,
) {
    let mut w = w_owner.get();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    layout_print_cell(
        (*w).layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root),
        b"layout_set_even\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(&*w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        sx = n
            .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
            .wrapping_sub(1 as u_int);
        if sx < (*w).sx {
            sx = (*w).sx;
        }
        sy = (*w).sy;
    } else {
        sy = n
            .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
            .wrapping_sub(1 as u_int);
        if sy < (*w).sy {
            sy = (*w).sy;
        }
        sx = (*w).sx;
    }
    let mut leaves = layout_take_leaves((*w).layout_root.take());
    (*w).layout_root = Some(layout_create_cell());
    lcroot = (*w)
        .layout_root_ptr()
        .map_or(std::ptr::null_mut(), |root| root);
    layout_set_size(
        lcroot,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, type_0);
    wp = window_pane_first(w.as_ref())
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !wp.is_null() {
        lcchild = (*wp).layout_cell as *mut layout_cell;
        layout_cells_push_back(lcroot, layout_take_leaf(&mut leaves, lcchild));
        (*lcchild).parent = lcroot;
        if layout_cell_is_tiled(lcchild) != 0 {
            (*lcchild).g.sx = (*w).sx;
            (*lcchild).g.sy = (*w).sy;
        }
        wp = window_pane_next(wp.as_ref())
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    layout_spread_cell(w_owner, lcroot);
    assert!(leaves.is_empty(), "all detached pane cells were reinserted");
    layout_fix_offsets(w_owner);
    layout_fix_panes(w_owner, None);
    layout_print_cell(
        (*w).layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root),
        b"layout_set_even\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        &(*(w)).observer.upgrade().expect("live window"),
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*(w)).observer.upgrade().expect("live window"),
    );
    server_redraw_window(&*(w));
}
unsafe fn layout_set_even_h(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    layout_set_even(w_owner, LAYOUT_LEFTRIGHT);
}
unsafe fn layout_set_even_v(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    layout_set_even(w_owner, LAYOUT_TOPBOTTOM);
}
fn layout_set_even_h_callback(w: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    unsafe { layout_set_even_h(w) };
}
fn layout_set_even_v_callback(w: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    unsafe { layout_set_even_v(w) };
}
fn layout_set_main_h_callback(w: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    unsafe { layout_set_main_h(w) };
}
fn layout_set_main_h_mirrored_callback(w: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    unsafe { layout_set_main_h_mirrored(w) };
}
fn layout_set_main_v_callback(w: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    unsafe { layout_set_main_v(w) };
}
fn layout_set_main_v_mirrored_callback(w: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    unsafe { layout_set_main_v_mirrored(w) };
}
fn layout_set_tiled_callback(w: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    unsafe { layout_set_tiled(w) };
}
unsafe fn layout_set_main_h(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    let mut w = w_owner.get();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpmain: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut mainh: u_int = 0;
    let mut otherh: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    layout_print_cell(
        (*w).layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root),
        b"layout_set_main_h\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(&*w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    n = n.wrapping_sub(1);
    sy = (*w).sy.wrapping_sub(1 as u_int);
    s = options_get_string(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"main-pane-height\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mainh = match args_string_percentage_result(
        (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
        sy as ::core::ffi::c_longlong,
        sy as ::core::ffi::c_longlong,
    ) {
        Ok(value) => value as u_int,
        Err(_) => 24 as u_int,
    };
    if mainh.wrapping_add(PANE_MINIMUM as u_int) >= sy {
        if sy <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
            mainh = PANE_MINIMUM as u_int;
        } else {
            mainh = sy.wrapping_sub(PANE_MINIMUM as u_int);
        }
        otherh = PANE_MINIMUM as u_int;
    } else {
        s = options_get_string(
            options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
            b"other-pane-height\0" as *const u8 as *const ::core::ffi::c_char,
        );
        otherh = match args_string_percentage_result(
            (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
            sy as ::core::ffi::c_longlong,
            sy as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as u_int,
            Err(_) => 0 as u_int,
        };
        if otherh == 0 as u_int || otherh > sy || sy.wrapping_sub(otherh) < mainh {
            otherh = sy.wrapping_sub(mainh);
        } else {
            mainh = sy.wrapping_sub(otherh);
        }
    }
    sx = n
        .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_sub(1 as u_int);
    if sx < (*w).sx {
        sx = (*w).sx;
    }
    let mut leaves = layout_take_leaves((*w).layout_root.take());
    (*w).layout_root = Some(layout_create_cell());
    lcroot = (*w)
        .layout_root_ptr()
        .map_or(std::ptr::null_mut(), |root| root);
    layout_set_size(
        lcroot,
        sx,
        mainh.wrapping_add(otherh).wrapping_add(1 as u_int),
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_TOPBOTTOM);
    let main_pane_owner = layout_set_first_tiled(w_owner);
    wpmain = main_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    lcmain = (*wpmain).layout_cell as *mut layout_cell;
    (*lcmain).parent = lcroot;
    layout_set_size(
        lcmain,
        sx,
        mainh,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_cells_push_back(lcroot, layout_take_leaf(&mut leaves, lcmain));
    if n == 1 as u_int {
        wp = window_pane_next(wpmain.as_ref())
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = window_pane_next(wp.as_ref())
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        layout_cells_push_back(
            lcroot,
            layout_take_leaf(&mut leaves, (*wp).layout_cell as *mut layout_cell),
        );
        (*(*wp).layout_cell).parent = lcroot;
        layout_set_size(
            (*wp).layout_cell as *mut layout_cell,
            sx,
            otherh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_set_link_floating(w_owner, lcroot, &mut leaves);
    } else {
        let mut lcother_owner = layout_create_cell();
        lcother = &mut *lcother_owner;
        layout_set_size(
            lcother,
            sx,
            otherh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_make_node(lcother, LAYOUT_LEFTRIGHT);
        layout_cells_push_back(lcroot, lcother_owner);
        wp = window_pane_first(w.as_ref())
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() {
            if !(wp == wpmain) {
                lcchild = (*wp).layout_cell as *mut layout_cell;
                layout_cells_push_back(lcother, layout_take_leaf(&mut leaves, lcchild));
                (*lcchild).parent = lcother;
                if layout_cell_is_tiled(lcchild) != 0 {
                    layout_set_size(
                        lcchild,
                        PANE_MINIMUM as u_int,
                        otherh,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            wp = window_pane_next(wp.as_ref())
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        layout_spread_cell(w_owner, lcother);
    }
    assert!(leaves.is_empty(), "all detached pane cells were reinserted");
    layout_fix_offsets(w_owner);
    layout_fix_panes(w_owner, None);
    layout_print_cell(
        (*w).layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root),
        b"layout_set_main_h\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        &(*(w)).observer.upgrade().expect("live window"),
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*(w)).observer.upgrade().expect("live window"),
    );
    server_redraw_window(&*(w));
}
unsafe fn layout_set_main_h_mirrored(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    let mut w = w_owner.get();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpmain: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut mainh: u_int = 0;
    let mut otherh: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    layout_print_cell(
        (*w).layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root),
        b"layout_set_main_h_mirrored\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(&*w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    n = n.wrapping_sub(1);
    sy = (*w).sy.wrapping_sub(1 as u_int);
    s = options_get_string(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"main-pane-height\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mainh = match args_string_percentage_result(
        (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
        sy as ::core::ffi::c_longlong,
        sy as ::core::ffi::c_longlong,
    ) {
        Ok(value) => value as u_int,
        Err(_) => 24 as u_int,
    };
    if mainh.wrapping_add(PANE_MINIMUM as u_int) >= sy {
        if sy <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
            mainh = PANE_MINIMUM as u_int;
        } else {
            mainh = sy.wrapping_sub(PANE_MINIMUM as u_int);
        }
        otherh = PANE_MINIMUM as u_int;
    } else {
        s = options_get_string(
            options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
            b"other-pane-height\0" as *const u8 as *const ::core::ffi::c_char,
        );
        otherh = match args_string_percentage_result(
            (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
            sy as ::core::ffi::c_longlong,
            sy as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as u_int,
            Err(_) => 0 as u_int,
        };
        if otherh == 0 as u_int || otherh > sy || sy.wrapping_sub(otherh) < mainh {
            otherh = sy.wrapping_sub(mainh);
        } else {
            mainh = sy.wrapping_sub(otherh);
        }
    }
    sx = n
        .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_sub(1 as u_int);
    if sx < (*w).sx {
        sx = (*w).sx;
    }
    let mut leaves = layout_take_leaves((*w).layout_root.take());
    (*w).layout_root = Some(layout_create_cell());
    lcroot = (*w)
        .layout_root_ptr()
        .map_or(std::ptr::null_mut(), |root| root);
    layout_set_size(
        lcroot,
        sx,
        mainh.wrapping_add(otherh).wrapping_add(1 as u_int),
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_TOPBOTTOM);
    let main_pane_owner = layout_set_first_tiled(w_owner);
    wpmain = main_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    lcmain = (*wpmain).layout_cell as *mut layout_cell;
    (*lcmain).parent = lcroot;
    layout_set_size(
        lcmain,
        sx,
        mainh,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_cells_push_back(lcroot, layout_take_leaf(&mut leaves, lcmain));
    if n == 1 as u_int {
        wp = window_pane_next(wpmain.as_ref())
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = window_pane_next(wp.as_ref())
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        layout_cells_push_front(
            lcroot,
            layout_take_leaf(&mut leaves, (*wp).layout_cell as *mut layout_cell),
        );
        (*(*wp).layout_cell).parent = lcroot;
        layout_set_size(
            (*wp).layout_cell as *mut layout_cell,
            sx,
            otherh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_set_link_floating(w_owner, lcroot, &mut leaves);
    } else {
        let mut lcother_owner = layout_create_cell();
        lcother = &mut *lcother_owner;
        layout_set_size(
            lcother,
            sx,
            otherh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_make_node(lcother, LAYOUT_LEFTRIGHT);
        layout_cells_push_front(lcroot, lcother_owner);
        wp = window_pane_first(w.as_ref())
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() {
            if !(wp == wpmain) {
                lcchild = (*wp).layout_cell as *mut layout_cell;
                layout_cells_push_back(lcother, layout_take_leaf(&mut leaves, lcchild));
                (*lcchild).parent = lcother;
                if layout_cell_is_tiled(lcchild) != 0 {
                    layout_set_size(
                        lcchild,
                        PANE_MINIMUM as u_int,
                        otherh,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            wp = window_pane_next(wp.as_ref())
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        layout_spread_cell(w_owner, lcother);
    }
    assert!(leaves.is_empty(), "all detached pane cells were reinserted");
    layout_fix_offsets(w_owner);
    layout_fix_panes(w_owner, None);
    layout_print_cell(
        (*w).layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root),
        b"layout_set_main_h_mirrored\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        &(*(w)).observer.upgrade().expect("live window"),
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*(w)).observer.upgrade().expect("live window"),
    );
    server_redraw_window(&*(w));
}
unsafe fn layout_set_main_v(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    let mut w = w_owner.get();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpmain: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut mainw: u_int = 0;
    let mut otherw: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    layout_print_cell(
        (*w).layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root),
        b"layout_set_main_v\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(&*w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    n = n.wrapping_sub(1);
    sx = (*w).sx.wrapping_sub(1 as u_int);
    s = options_get_string(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"main-pane-width\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mainw = match args_string_percentage_result(
        (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
        sx as ::core::ffi::c_longlong,
        sx as ::core::ffi::c_longlong,
    ) {
        Ok(value) => value as u_int,
        Err(_) => 80 as u_int,
    };
    if mainw.wrapping_add(PANE_MINIMUM as u_int) >= sx {
        if sx <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
            mainw = PANE_MINIMUM as u_int;
        } else {
            mainw = sx.wrapping_sub(PANE_MINIMUM as u_int);
        }
        otherw = PANE_MINIMUM as u_int;
    } else {
        s = options_get_string(
            options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
            b"other-pane-width\0" as *const u8 as *const ::core::ffi::c_char,
        );
        otherw = match args_string_percentage_result(
            (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
            sx as ::core::ffi::c_longlong,
            sx as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as u_int,
            Err(_) => 0 as u_int,
        };
        if otherw == 0 as u_int || otherw > sx || sx.wrapping_sub(otherw) < mainw {
            otherw = sx.wrapping_sub(mainw);
        } else {
            mainw = sx.wrapping_sub(otherw);
        }
    }
    sy = n
        .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_sub(1 as u_int);
    if sy < (*w).sy {
        sy = (*w).sy;
    }
    let mut leaves = layout_take_leaves((*w).layout_root.take());
    (*w).layout_root = Some(layout_create_cell());
    lcroot = (*w)
        .layout_root_ptr()
        .map_or(std::ptr::null_mut(), |root| root);
    layout_set_size(
        lcroot,
        mainw.wrapping_add(otherw).wrapping_add(1 as u_int),
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_LEFTRIGHT);
    let main_pane_owner = layout_set_first_tiled(w_owner);
    wpmain = main_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    lcmain = (*wpmain).layout_cell as *mut layout_cell;
    (*lcmain).parent = lcroot;
    layout_set_size(
        lcmain,
        mainw,
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_cells_push_back(lcroot, layout_take_leaf(&mut leaves, lcmain));
    if n == 1 as u_int {
        wp = window_pane_next(wpmain.as_ref())
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = window_pane_next(wp.as_ref())
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        layout_cells_push_back(
            lcroot,
            layout_take_leaf(&mut leaves, (*wp).layout_cell as *mut layout_cell),
        );
        (*(*wp).layout_cell).parent = lcroot;
        layout_set_size(
            (*wp).layout_cell as *mut layout_cell,
            otherw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_set_link_floating(w_owner, lcroot, &mut leaves);
    } else {
        let mut lcother_owner = layout_create_cell();
        lcother = &mut *lcother_owner;
        layout_make_node(lcother, LAYOUT_TOPBOTTOM);
        layout_set_size(
            lcother,
            otherw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_cells_push_back(lcroot, lcother_owner);
        wp = window_pane_first(w.as_ref())
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() {
            if !(wp == wpmain) {
                lcchild = (*wp).layout_cell as *mut layout_cell;
                layout_cells_push_back(lcother, layout_take_leaf(&mut leaves, lcchild));
                (*lcchild).parent = lcother;
                if layout_cell_is_tiled(lcchild) != 0 {
                    layout_set_size(
                        lcchild,
                        otherw,
                        PANE_MINIMUM as u_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            wp = window_pane_next(wp.as_ref())
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        layout_spread_cell(w_owner, lcother);
    }
    assert!(leaves.is_empty(), "all detached pane cells were reinserted");
    layout_fix_offsets(w_owner);
    layout_fix_panes(w_owner, None);
    layout_print_cell(
        (*w).layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root),
        b"layout_set_main_v\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        &(*(w)).observer.upgrade().expect("live window"),
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*(w)).observer.upgrade().expect("live window"),
    );
    server_redraw_window(&*(w));
}
unsafe fn layout_set_main_v_mirrored(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    let mut w = w_owner.get();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpmain: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut mainw: u_int = 0;
    let mut otherw: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    layout_print_cell(
        (*w).layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root),
        b"layout_set_main_v_mirrored\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(&*w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    n = n.wrapping_sub(1);
    sx = (*w).sx.wrapping_sub(1 as u_int);
    s = options_get_string(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"main-pane-width\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mainw = match args_string_percentage_result(
        (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
        sx as ::core::ffi::c_longlong,
        sx as ::core::ffi::c_longlong,
    ) {
        Ok(value) => value as u_int,
        Err(_) => 80 as u_int,
    };
    if mainw.wrapping_add(PANE_MINIMUM as u_int) >= sx {
        if sx <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
            mainw = PANE_MINIMUM as u_int;
        } else {
            mainw = sx.wrapping_sub(PANE_MINIMUM as u_int);
        }
        otherw = PANE_MINIMUM as u_int;
    } else {
        s = options_get_string(
            options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
            b"other-pane-width\0" as *const u8 as *const ::core::ffi::c_char,
        );
        otherw = match args_string_percentage_result(
            (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
            sx as ::core::ffi::c_longlong,
            sx as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as u_int,
            Err(_) => 0 as u_int,
        };
        if otherw == 0 as u_int || otherw > sx || sx.wrapping_sub(otherw) < mainw {
            otherw = sx.wrapping_sub(mainw);
        } else {
            mainw = sx.wrapping_sub(otherw);
        }
    }
    sy = n
        .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_sub(1 as u_int);
    if sy < (*w).sy {
        sy = (*w).sy;
    }
    let mut leaves = layout_take_leaves((*w).layout_root.take());
    (*w).layout_root = Some(layout_create_cell());
    lcroot = (*w)
        .layout_root_ptr()
        .map_or(std::ptr::null_mut(), |root| root);
    layout_set_size(
        lcroot,
        mainw.wrapping_add(otherw).wrapping_add(1 as u_int),
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_LEFTRIGHT);
    let main_pane_owner = layout_set_first_tiled(w_owner);
    wpmain = main_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    lcmain = (*wpmain).layout_cell as *mut layout_cell;
    (*lcmain).parent = lcroot;
    layout_set_size(
        lcmain,
        mainw,
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_cells_push_back(lcroot, layout_take_leaf(&mut leaves, lcmain));
    if n == 1 as u_int {
        wp = window_pane_next(wpmain.as_ref())
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = window_pane_next(wp.as_ref())
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        layout_cells_push_front(
            lcroot,
            layout_take_leaf(&mut leaves, (*wp).layout_cell as *mut layout_cell),
        );
        (*(*wp).layout_cell).parent = lcroot;
        layout_set_size(
            (*wp).layout_cell as *mut layout_cell,
            otherw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_set_link_floating(w_owner, lcroot, &mut leaves);
    } else {
        let mut lcother_owner = layout_create_cell();
        lcother = &mut *lcother_owner;
        layout_make_node(lcother, LAYOUT_TOPBOTTOM);
        layout_set_size(
            lcother,
            otherw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_cells_push_front(lcroot, lcother_owner);
        wp = window_pane_first(w.as_ref())
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() {
            if !(wp == wpmain) {
                lcchild = (*wp).layout_cell as *mut layout_cell;
                layout_cells_push_back(lcother, layout_take_leaf(&mut leaves, lcchild));
                (*lcchild).parent = lcother;
                if layout_cell_is_tiled(lcchild) != 0 {
                    layout_set_size(
                        lcchild,
                        otherw,
                        PANE_MINIMUM as u_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            wp = window_pane_next(wp.as_ref())
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        layout_spread_cell(w_owner, lcother);
    }
    assert!(leaves.is_empty(), "all detached pane cells were reinserted");
    layout_fix_offsets(w_owner);
    layout_fix_panes(w_owner, None);
    layout_print_cell(
        (*w).layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root),
        b"layout_set_main_v_mirrored\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        &(*(w)).observer.upgrade().expect("live window"),
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*(w)).observer.upgrade().expect("live window"),
    );
    server_redraw_window(&*(w));
}
unsafe fn layout_set_tiled(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    let mut w = w_owner.get();
    let mut oo: *mut options =
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options);
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcrow: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut width: u_int = 0;
    let mut height: u_int = 0;
    let mut used: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut columns: u_int = 0;
    let mut rows: u_int = 0;
    let mut max_columns: u_int = 0;
    layout_print_cell(
        (*w).layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root),
        b"layout_set_tiled\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(&*w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    max_columns = options_get_number(
        oo,
        b"tiled-layout-max-columns\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    columns = 1 as u_int;
    rows = columns;
    while rows.wrapping_mul(columns) < n {
        rows = rows.wrapping_add(1);
        if rows.wrapping_mul(columns) < n && (max_columns == 0 as u_int || columns < max_columns) {
            columns = columns.wrapping_add(1);
        }
    }
    width = (*w)
        .sx
        .wrapping_sub(columns.wrapping_sub(1 as u_int))
        .wrapping_div(columns);
    if width < PANE_MINIMUM as u_int {
        width = PANE_MINIMUM as u_int;
    }
    height = (*w)
        .sy
        .wrapping_sub(rows.wrapping_sub(1 as u_int))
        .wrapping_div(rows);
    if height < PANE_MINIMUM as u_int {
        height = PANE_MINIMUM as u_int;
    }
    sx = width
        .wrapping_add(1 as u_int)
        .wrapping_mul(columns)
        .wrapping_sub(1 as u_int);
    if sx < (*w).sx {
        sx = (*w).sx;
    }
    sy = height
        .wrapping_add(1 as u_int)
        .wrapping_mul(rows)
        .wrapping_sub(1 as u_int);
    if sy < (*w).sy {
        sy = (*w).sy;
    }
    let mut leaves = layout_take_leaves((*w).layout_root.take());
    (*w).layout_root = Some(layout_create_cell());
    lcroot = (*w)
        .layout_root_ptr()
        .map_or(std::ptr::null_mut(), |root| root);
    layout_set_size(
        lcroot,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_TOPBOTTOM);
    wp = window_pane_first(w.as_ref())
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    j = 0 as u_int;
    while j < rows {
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = window_pane_next(wp.as_ref())
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        if wp.is_null() {
            break;
        }
        lcchild = (*wp).layout_cell as *mut layout_cell;
        if n.wrapping_sub(j.wrapping_mul(columns)) == 1 as u_int || columns == 1 as u_int {
            (*lcchild).parent = lcroot;
            layout_cells_push_back(lcroot, layout_take_leaf(&mut leaves, lcchild));
            layout_set_size(
                lcchild,
                (*w).sx,
                height,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            wp = window_pane_next(wp.as_ref())
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        } else {
            let mut lcrow_owner = layout_create_cell();
            lcrow = &mut *lcrow_owner;
            layout_make_node(lcrow, LAYOUT_LEFTRIGHT);
            layout_set_size(
                lcrow,
                (*w).sx,
                height,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            layout_cells_push_back(lcroot, lcrow_owner);
            i = 0 as u_int;
            while i < columns {
                (*lcchild).parent = lcrow;
                layout_cells_push_back(lcrow, layout_take_leaf(&mut leaves, lcchild));
                layout_set_size(
                    lcchild,
                    width,
                    height,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                wp = window_pane_next(wp.as_ref())
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get());
                while !wp.is_null()
                    && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0
                {
                    wp = window_pane_next(wp.as_ref())
                        .as_ref()
                        .map_or(std::ptr::null_mut(), |owner| owner.get());
                }
                if wp.is_null() {
                    break;
                }
                lcchild = (*wp).layout_cell as *mut layout_cell;
                i = i.wrapping_add(1);
            }
            if i == columns {
                i = i.wrapping_sub(1);
            }
            used = i
                .wrapping_add(1 as u_int)
                .wrapping_mul(width.wrapping_add(1 as u_int))
                .wrapping_sub(1 as u_int);
            if !((*w).sx <= used) {
                lcchild = layout_cells_last(&*lcrow);
                layout_resize_adjust(
                    w_owner,
                    lcchild,
                    LAYOUT_LEFTRIGHT,
                    (*w).sx.wrapping_sub(used) as ::core::ffi::c_int,
                );
            }
        }
        j = j.wrapping_add(1);
    }
    used = rows
        .wrapping_mul(height)
        .wrapping_add(rows)
        .wrapping_sub(1 as u_int);
    if (*w).sy > used {
        lcrow = layout_cells_last(&*lcroot);
        layout_resize_adjust(
            w_owner,
            lcrow,
            LAYOUT_TOPBOTTOM,
            (*w).sy.wrapping_sub(used) as ::core::ffi::c_int,
        );
    }
    layout_set_link_floating(w_owner, lcroot, &mut leaves);
    assert!(leaves.is_empty(), "all detached pane cells were reinserted");
    layout_fix_offsets(w_owner);
    layout_fix_panes(w_owner, None);
    layout_print_cell(
        (*w).layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root),
        b"layout_set_tiled\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        &(*(w)).observer.upgrade().expect("live window"),
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*(w)).observer.upgrade().expect("live window"),
    );
    server_redraw_window(&*(w));
}
