use crate::src::events::events_fire_window;
use crate::src::ffi::libc::{free, memcpy, memmove, qsort, sscanf, strcmp, strlen};
use crate::src::json::{
    json_array_first, json_array_next, json_destroy_node, json_find, json_find_array,
    json_find_boolean, json_find_number, json_find_object, json_find_string, json_get_object,
    json_get_string, json_parse,
};
use crate::src::layout::{
    layout_cell_has_tiled_child, layout_cell_is_tiled, layout_count_cells, layout_create_cell,
    layout_destroy_cell, layout_fix_offsets, layout_fix_panes, layout_free_cell, layout_make_leaf,
    layout_print_cell, layout_replace_with_node, layout_set_size,
};
use crate::src::resize::recalculate_sizes;
use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{__compar_fn_t, __int64_t, int64_t};
pub use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::control_state;
pub use crate::src::shared::ctype::{
    _ISalnum, _ISalpha, _ISblank, _IScntrl, _ISdigit, _ISgraph, _ISlower, _ISprint, _ISpunct,
    _ISspace, _ISupper, _ISxdigit, ctype_code,
};
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::json::json_node;
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::limits::{__INT_MAX__, INT_MAX};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
    PANE_MAXIMUM, PANE_MINIMUM,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::window::WINDOW_MAXIMUM;
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::window::{
    window_count_panes, window_pane_index, window_pane_is_floating, window_pane_last_index,
    window_pane_stack_push, window_pane_stack_remove, window_pane_zindex, window_resize,
    window_set_active_pane,
};
use crate::src::xmalloc::{
    xasprintf, xcalloc, xmalloc, xmemdup, xreallocarray, xstrdup, xvasprintf,
};

use ::std::ops::{Deref, Index};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_14;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_13;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct layout_string {
    pub dat: *mut ::core::ffi::c_char,
    pub size: size_t,
    pub capacity: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct layout_parse_ctx {
    pub version: int64_t,
    pub num_active: ::core::ffi::c_int,
    pub root: *mut layout_cell,
    pub cause: *mut *mut ::core::ffi::c_char,
    pub size: ::core::ffi::c_int,
    pub capacity: ::core::ffi::c_int,
    pub cctxs: *mut layout_parse_cell_ctx,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct layout_parse_cell_ctx {
    pub lc: *mut layout_cell,
    pub active: ::core::ffi::c_int,
    pub last: ::core::ffi::c_int,
    pub index: ::core::ffi::c_int,
    pub zindex: ::core::ffi::c_int,
}

/// Geometry retained by a parsed custom layout before it is attached to a
/// window.  The parser deliberately does not store pointers to panes or
/// layout cells: those belong to the application phase below.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutDescriptionGeometry {
    pub sx: u32,
    pub sy: u32,
    pub xoff: i32,
    pub yoff: i32,
}

/// An owned byte string backed by the project's xmalloc allocator.
///
/// The trailing NUL is an implementation detail for the C-facing helpers; it
/// is not included in `as_slice`.
pub struct LayoutDescriptionBytes {
    data: *mut u8,
    len: usize,
}

impl LayoutDescriptionBytes {
    pub fn from_slice(value: &[u8]) -> Self {
        unsafe {
            let data = xmalloc(value.len().wrapping_add(1)) as *mut u8;
            if !value.is_empty() {
                memcpy(
                    data as *mut ::core::ffi::c_void,
                    value.as_ptr() as *const ::core::ffi::c_void,
                    value.len(),
                );
            }
            *data.add(value.len()) = 0;
            Self {
                data,
                len: value.len(),
            }
        }
    }

    unsafe fn from_owned_c_string(data: *mut ::core::ffi::c_char) -> Self {
        Self {
            data: data as *mut u8,
            len: strlen(data) as usize,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { ::std::slice::from_raw_parts(self.data, self.len) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Clone for LayoutDescriptionBytes {
    fn clone(&self) -> Self {
        Self::from_slice(self.as_slice())
    }
}

impl ::std::fmt::Debug for LayoutDescriptionBytes {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_tuple("LayoutDescriptionBytes")
            .field(&self.as_slice())
            .finish()
    }
}

impl PartialEq for LayoutDescriptionBytes {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for LayoutDescriptionBytes {}

impl AsRef<[u8]> for LayoutDescriptionBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Deref for LayoutDescriptionBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl Drop for LayoutDescriptionBytes {
    fn drop(&mut self) {
        unsafe {
            free(self.data as *mut ::core::ffi::c_void);
        }
    }
}

/// Children of a detached layout node, stored with the same fatal allocation
/// policy as the rest of this module.
pub struct LayoutDescriptionChildren {
    nodes: *mut LayoutDescriptionNode,
    len: usize,
    capacity: usize,
}

impl LayoutDescriptionChildren {
    pub fn new() -> Self {
        Self {
            nodes: ::core::ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }

    pub fn push(&mut self, node: LayoutDescriptionNode) {
        unsafe {
            if self.len == self.capacity {
                let capacity = if self.capacity == 0 {
                    4
                } else {
                    self.capacity.wrapping_mul(2)
                };
                self.nodes = if self.nodes.is_null() {
                    xcalloc(capacity, ::core::mem::size_of::<LayoutDescriptionNode>())
                } else {
                    xreallocarray(
                        self.nodes as *mut ::core::ffi::c_void,
                        capacity,
                        ::core::mem::size_of::<LayoutDescriptionNode>(),
                    )
                } as *mut LayoutDescriptionNode;
                self.capacity = capacity;
            }
            ::core::ptr::write(self.nodes.add(self.len), node);
            self.len += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> ::std::slice::Iter<'_, LayoutDescriptionNode> {
        unsafe {
            if self.len == 0 {
                (&[] as &[LayoutDescriptionNode]).iter()
            } else {
                ::std::slice::from_raw_parts(self.nodes, self.len).iter()
            }
        }
    }
}

impl Default for LayoutDescriptionChildren {
    fn default() -> Self {
        Self::new()
    }
}

impl ::std::fmt::Debug for LayoutDescriptionChildren {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl Clone for LayoutDescriptionChildren {
    fn clone(&self) -> Self {
        let mut copy = Self::new();
        for index in 0..self.len {
            copy.push(self[index].clone());
        }
        copy
    }
}

impl PartialEq for LayoutDescriptionChildren {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl Eq for LayoutDescriptionChildren {}

impl Index<usize> for LayoutDescriptionChildren {
    type Output = LayoutDescriptionNode;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(
            index < self.len,
            "layout description child index out of bounds"
        );
        unsafe { &*self.nodes.add(index) }
    }
}

impl Drop for LayoutDescriptionChildren {
    fn drop(&mut self) {
        unsafe {
            for index in 0..self.len {
                ::core::ptr::drop_in_place(self.nodes.add(index));
            }
            free(self.nodes as *mut ::core::ffi::c_void);
        }
    }
}

/// Pane metadata carried by a custom-layout leaf.
///
/// `index` is the v2 pane ordering key.  `id` is the optional legacy pane
/// number.  `identifier` retains the v2 `I` value even though the live tree
/// uses the window's pane objects when it is applied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutDescriptionPane {
    pub index: Option<i32>,
    pub id: Option<u32>,
    pub identifier: Option<LayoutDescriptionBytes>,
    pub active: bool,
    pub last: Option<i32>,
    pub zindex: Option<i32>,
}

/// A detached custom-layout node.  This is the intermediate representation
/// shared by the legacy and JSON parsers; it has no live-window ownership.
pub struct LayoutDescriptionNode {
    pub type_0: layout_type,
    pub flags: i32,
    pub geometry: LayoutDescriptionGeometry,
    pub pane: Option<LayoutDescriptionPane>,
    pub children: LayoutDescriptionChildren,
}

/// A fully parsed custom layout, ready for validation and application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutDescription {
    pub version: i64,
    pub root: LayoutDescriptionNode,
}

/// Error returned by the byte-oriented parser API.
pub struct LayoutParseError {
    message: LayoutDescriptionBytes,
}

impl LayoutParseError {
    pub fn as_bytes(&self) -> &[u8] {
        self.message.as_slice()
    }
}

impl ::std::error::Error for LayoutParseError {}

impl Clone for LayoutParseError {
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
        }
    }
}

impl ::std::fmt::Debug for LayoutParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("LayoutParseError")
            .field("message", &self.message)
            .finish()
    }
}

impl PartialEq for LayoutParseError {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message
    }
}

impl Eq for LayoutParseError {}

impl LayoutDescriptionNode {
    fn leaf(geometry: LayoutDescriptionGeometry, pane: LayoutDescriptionPane) -> Self {
        Self {
            type_0: LAYOUT_WINDOWPANE,
            flags: 0,
            geometry,
            pane: Some(pane),
            children: LayoutDescriptionChildren::new(),
        }
    }

    fn node(
        type_0: layout_type,
        geometry: LayoutDescriptionGeometry,
        children: LayoutDescriptionChildren,
    ) -> Self {
        Self {
            type_0,
            flags: 0,
            geometry,
            pane: None,
            children,
        }
    }

    fn has_duplicate_index(
        &self,
        target: &LayoutDescriptionPane,
        target_ptr: *const LayoutDescriptionPane,
    ) -> bool {
        if let Some(pane) = self.pane.as_ref() {
            if !::std::ptr::eq(pane, target_ptr)
                && pane.index.is_some()
                && pane.index == target.index
            {
                return true;
            }
        }
        for index in 0..self.children.len() {
            if self.children[index].has_duplicate_index(target, target_ptr) {
                return true;
            }
        }
        false
    }

    fn has_duplicate_zindex(
        &self,
        target: &LayoutDescriptionPane,
        target_ptr: *const LayoutDescriptionPane,
    ) -> bool {
        if let Some(pane) = self.pane.as_ref() {
            if !::std::ptr::eq(pane, target_ptr)
                && pane.zindex.is_some()
                && pane.zindex == target.zindex
            {
                return true;
            }
        }
        for index in 0..self.children.len() {
            if self.children[index].has_duplicate_zindex(target, target_ptr) {
                return true;
            }
        }
        false
    }

    fn has_duplicate_last(
        &self,
        target: &LayoutDescriptionPane,
        target_ptr: *const LayoutDescriptionPane,
    ) -> bool {
        if let Some(pane) = self.pane.as_ref() {
            if !::std::ptr::eq(pane, target_ptr) && pane.last.is_some() && pane.last == target.last
            {
                return true;
            }
        }
        for index in 0..self.children.len() {
            if self.children[index].has_duplicate_last(target, target_ptr) {
                return true;
            }
        }
        false
    }
}

impl Clone for LayoutDescriptionNode {
    fn clone(&self) -> Self {
        Self {
            type_0: self.type_0,
            flags: self.flags,
            geometry: self.geometry,
            pane: self.pane.clone(),
            children: self.children.clone(),
        }
    }
}

impl ::std::fmt::Debug for LayoutDescriptionNode {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("LayoutDescriptionNode")
            .field("type_0", &self.type_0)
            .field("flags", &self.flags)
            .field("geometry", &self.geometry)
            .field("pane", &self.pane)
            .field("children", &self.children)
            .finish()
    }
}

impl PartialEq for LayoutDescriptionNode {
    fn eq(&self, other: &Self) -> bool {
        self.type_0 == other.type_0
            && self.flags == other.flags
            && self.geometry == other.geometry
            && self.pane == other.pane
            && self.children == other.children
    }
}

impl Eq for LayoutDescriptionNode {}

impl ::std::fmt::Display for LayoutParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        for &byte in self.message.as_slice() {
            if byte.is_ascii() {
                ::std::fmt::Write::write_char(f, byte as char)?;
            } else {
                ::std::fmt::Write::write_char(f, '\u{fffd}')?;
            }
        }
        Ok(())
    }
}

unsafe extern "C" fn layout_parse_index_cmp(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut cca: *const layout_parse_cell_ctx = a as *const layout_parse_cell_ctx;
    let mut ccb: *const layout_parse_cell_ctx = b as *const layout_parse_cell_ctx;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cca).index < (*ccb).index {
        retval = -(1 as ::core::ffi::c_int);
    }
    if (*cca).index > (*ccb).index {
        retval = 1 as ::core::ffi::c_int;
    }
    return retval;
}
unsafe extern "C" fn layout_parse_zindex_cmp(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut cca: *const layout_parse_cell_ctx = a as *const layout_parse_cell_ctx;
    let mut ccb: *const layout_parse_cell_ctx = b as *const layout_parse_cell_ctx;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cca).zindex > (*ccb).zindex {
        retval = -(1 as ::core::ffi::c_int);
    }
    if (*cca).zindex < (*ccb).zindex {
        retval = 1 as ::core::ffi::c_int;
    }
    return retval;
}
unsafe extern "C" fn layout_parse_last_cmp(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut cca: *const layout_parse_cell_ctx = a as *const layout_parse_cell_ctx;
    let mut ccb: *const layout_parse_cell_ctx = b as *const layout_parse_cell_ctx;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cca).last > (*ccb).last {
        retval = -(1 as ::core::ffi::c_int);
    }
    if (*cca).last < (*ccb).last {
        retval = 1 as ::core::ffi::c_int;
    }
    return retval;
}
unsafe extern "C" fn layout_string_init(mut ls: *mut layout_string) {
    (*ls).capacity = 1024 as size_t;
    (*ls).dat = xmalloc((*ls).capacity) as *mut ::core::ffi::c_char;
    *(*ls).dat.offset(0 as ::core::ffi::c_int as isize) = '\0' as i32 as ::core::ffi::c_char;
    (*ls).size = 0 as size_t;
}
unsafe extern "C" fn layout_string_free(mut ls: *mut layout_string) {
    free((*ls).dat as *mut ::core::ffi::c_void);
    (*ls).dat = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*ls).size = 0 as size_t;
    (*ls).capacity = 0 as size_t;
}
unsafe extern "C" fn layout_string_write(
    mut ls: *mut layout_string,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut slen: ::core::ffi::c_int = 0;
    ap = args.clone();
    slen = xvasprintf(&raw mut s, fmt, ap);
    while (*ls)
        .size
        .wrapping_add(slen as size_t)
        .wrapping_add(1 as size_t)
        > (*ls).capacity
    {
        (*ls).dat = xreallocarray(
            (*ls).dat as *mut ::core::ffi::c_void,
            2 as size_t,
            (*ls).capacity,
        ) as *mut ::core::ffi::c_char;
        (*ls).capacity = (*ls).capacity.wrapping_mul(2 as size_t);
    }
    memcpy(
        (*ls).dat.offset((*ls).size as isize) as *mut ::core::ffi::c_void,
        s as *const ::core::ffi::c_void,
        slen as size_t,
    );
    (*ls).size = (*ls).size.wrapping_add(slen as size_t);
    *(*ls).dat.offset((*ls).size as isize) = '\0' as i32 as ::core::ffi::c_char;
    free(s as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn layout_parse_init_ctx(
    mut pctx: *mut layout_parse_ctx,
    mut cause: *mut *mut ::core::ffi::c_char,
) {
    (*pctx).version = -(1 as ::core::ffi::c_int) as int64_t;
    (*pctx).num_active = 0 as ::core::ffi::c_int;
    (*pctx).root = ::core::ptr::null_mut::<layout_cell>();
    (*pctx).cause = cause;
    (*pctx).size = 0 as ::core::ffi::c_int;
    (*pctx).capacity = 64 as ::core::ffi::c_int;
    (*pctx).cctxs = xcalloc(
        (*pctx).capacity as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
    ) as *mut layout_parse_cell_ctx;
}
unsafe extern "C" fn layout_parse_free_ctx(mut pctx: *mut layout_parse_ctx) {
    layout_free_cell((*pctx).root, 0 as ::core::ffi::c_int);
    (*pctx).root = ::core::ptr::null_mut::<layout_cell>();
    free((*pctx).cctxs as *mut ::core::ffi::c_void);
    (*pctx).cctxs = ::core::ptr::null_mut::<layout_parse_cell_ctx>();
    (*pctx).size = 0 as ::core::ffi::c_int;
    (*pctx).capacity = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_parse_add_cctx(
    mut pctx: *mut layout_parse_ctx,
    mut lc: *mut layout_cell,
    mut active: ::core::ffi::c_int,
    mut last: ::core::ffi::c_int,
    mut index: ::core::ffi::c_int,
    mut zindex: ::core::ffi::c_int,
) {
    let mut cctx: *mut layout_parse_cell_ctx = ::core::ptr::null_mut::<layout_parse_cell_ctx>();
    if (*pctx).size >= (*pctx).capacity {
        (*pctx).capacity *= 2 as ::core::ffi::c_int;
        (*pctx).cctxs = xreallocarray(
            (*pctx).cctxs as *mut ::core::ffi::c_void,
            (*pctx).capacity as size_t,
            ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        ) as *mut layout_parse_cell_ctx;
    }
    let fresh0 = (*pctx).size;
    (*pctx).size = (*pctx).size + 1;
    cctx = (*pctx).cctxs.offset(fresh0 as isize) as *mut layout_parse_cell_ctx;
    (*cctx).lc = lc;
    (*cctx).active = active;
    (*cctx).last = last;
    (*cctx).index = index;
    (*cctx).zindex = zindex;
}
unsafe extern "C" fn layout_parse_remove_cctx(
    mut pctx: *mut layout_parse_ctx,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut cctx: *mut layout_parse_cell_ctx = ::core::ptr::null_mut::<layout_parse_cell_ctx>();
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < (*pctx).size {
        if lc == (*(*pctx).cctxs.offset(i as isize)).lc {
            (*pctx).size -= 1;
            cctx = (*pctx).cctxs.offset((*pctx).size as isize) as *mut layout_parse_cell_ctx;
            memmove(
                (*pctx).cctxs.offset(i as isize) as *mut layout_parse_cell_ctx
                    as *mut ::core::ffi::c_void,
                cctx as *const ::core::ffi::c_void,
                ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
            );
            return 0 as ::core::ffi::c_int;
        }
        i += 1;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn layout_find_bottomright(mut lc: *mut layout_cell) -> *mut layout_cell {
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return lc;
    }
    lc = *(*((*lc).cells.tqh_last as *mut layout_cells)).tqh_last;
    return layout_find_bottomright(lc);
}
unsafe extern "C" fn layout_checksum(mut layout: *const ::core::ffi::c_char) -> u_short {
    let mut csum: u_short = 0;
    csum = 0 as u_short;
    while *layout as ::core::ffi::c_int != '\0' as i32 {
        csum = ((csum as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
            + ((csum as ::core::ffi::c_int & 1 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int))
            as u_short;
        csum = (csum as ::core::ffi::c_int + *layout as ::core::ffi::c_int) as u_short;
        layout = layout.offset(1);
    }
    return csum;
}

unsafe fn layout_description_construct_cell(
    layout: &mut *const ::core::ffi::c_char,
) -> Option<(LayoutDescriptionGeometry, Option<u32>)> {
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut cursor = *layout;

    if !(*cursor as u8).is_ascii_digit()
        || sscanf(
            cursor,
            b"%ux%u,%d,%d\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut sx,
            &raw mut sy,
            &raw mut xoff,
            &raw mut yoff,
        ) != 4
    {
        return None;
    }
    while (*cursor as u8).is_ascii_digit() {
        cursor = cursor.offset(1);
    }
    if *cursor as u8 != b'x' {
        return None;
    }
    cursor = cursor.offset(1);
    while (*cursor as u8).is_ascii_digit() {
        cursor = cursor.offset(1);
    }
    if *cursor as u8 != b',' {
        return None;
    }
    cursor = cursor.offset(1);
    while (*cursor as u8).is_ascii_digit() {
        cursor = cursor.offset(1);
    }
    if *cursor as u8 != b',' {
        return None;
    }
    cursor = cursor.offset(1);
    while (*cursor as u8).is_ascii_digit() {
        cursor = cursor.offset(1);
    }

    let mut pane_id = None;
    if *cursor as u8 == b',' {
        let saved = cursor;
        cursor = cursor.offset(1);
        let id_start = cursor;
        let mut id = 0u32;
        while (*cursor as u8).is_ascii_digit() {
            id = id
                .wrapping_mul(10)
                .wrapping_add(u32::from(*cursor as u8 - b'0'));
            cursor = cursor.offset(1);
        }
        if *cursor as u8 == b'x' {
            cursor = saved;
        } else if cursor != id_start {
            pane_id = Some(id);
        }
    }

    *layout = cursor;
    Some((LayoutDescriptionGeometry { sx, sy, xoff, yoff }, pane_id))
}

unsafe fn layout_description_construct_v1(
    layout: &mut *const ::core::ffi::c_char,
    depth: u_int,
) -> Option<LayoutDescriptionNode> {
    if depth > LAYOUT_V1_MAX_DEPTH as u_int {
        return None;
    }
    let (geometry, pane_id) = layout_description_construct_cell(layout)?;
    let next = **layout as u8;
    if matches!(next, b',' | b'}' | b']' | 0) {
        return Some(LayoutDescriptionNode::leaf(
            geometry,
            LayoutDescriptionPane {
                index: None,
                id: pane_id,
                identifier: None,
                active: false,
                last: None,
                zindex: None,
            },
        ));
    }

    let (type_0, close) = match next {
        b'{' => (LAYOUT_LEFTRIGHT, b'}'),
        b'[' => (LAYOUT_TOPBOTTOM, b']'),
        _ => return None,
    };
    let mut children = LayoutDescriptionChildren::new();
    loop {
        *layout = (*layout).offset(1);
        children.push(layout_description_construct_v1(layout, depth + 1)?);
        if **layout as u8 != b',' {
            break;
        }
    }
    if **layout as u8 != close {
        return None;
    }
    *layout = (*layout).offset(1);
    Some(LayoutDescriptionNode::node(type_0, geometry, children))
}

unsafe fn layout_description_json_number(
    node: *mut json_node,
    key: *const ::core::ffi::c_char,
    cause: *mut *mut ::core::ffi::c_char,
    minimum: int64_t,
    maximum: int64_t,
    label: *const ::core::ffi::c_char,
) -> Option<int64_t> {
    let mut number = 0;
    if json_find_number(node, key, &raw mut number, cause) != 0 {
        return None;
    }
    if number < minimum || number > maximum {
        xasprintf(
            cause,
            b"invalid %s %lld\0" as *const u8 as *const ::core::ffi::c_char,
            label,
            number as ::core::ffi::c_longlong,
        );
        return None;
    }
    Some(number)
}

unsafe fn layout_description_parse_json_cell(
    node: *mut json_node,
    cause: *mut *mut ::core::ffi::c_char,
    active_count: &mut ::core::ffi::c_int,
) -> Option<LayoutDescriptionNode> {
    let mut string = ::core::ptr::null::<::core::ffi::c_char>();
    if json_find_string(
        node,
        b"t\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut string,
        cause,
    ) != 0
    {
        return None;
    }
    let type_0 = if strcmp(string, b"p\0" as *const u8 as *const ::core::ffi::c_char) == 0 {
        LAYOUT_WINDOWPANE
    } else if strcmp(string, b"v\0" as *const u8 as *const ::core::ffi::c_char) == 0 {
        LAYOUT_TOPBOTTOM
    } else if strcmp(string, b"h\0" as *const u8 as *const ::core::ffi::c_char) == 0 {
        LAYOUT_LEFTRIGHT
    } else {
        xasprintf(
            cause,
            b"unknown cell type \"%s\"\0" as *const u8 as *const ::core::ffi::c_char,
            string,
        );
        return None;
    };

    let sx = layout_description_json_number(
        node,
        b"w\0" as *const u8 as *const ::core::ffi::c_char,
        cause,
        PANE_MINIMUM as int64_t,
        PANE_MAXIMUM as int64_t,
        b"width\0" as *const u8 as *const ::core::ffi::c_char,
    )?;
    let sy = layout_description_json_number(
        node,
        b"h\0" as *const u8 as *const ::core::ffi::c_char,
        cause,
        PANE_MINIMUM as int64_t,
        PANE_MAXIMUM as int64_t,
        b"height\0" as *const u8 as *const ::core::ffi::c_char,
    )?;
    let xoff = layout_description_json_number(
        node,
        b"x\0" as *const u8 as *const ::core::ffi::c_char,
        cause,
        -WINDOW_MAXIMUM as int64_t,
        WINDOW_MAXIMUM as int64_t,
        b"x-offset\0" as *const u8 as *const ::core::ffi::c_char,
    )?;
    let yoff = layout_description_json_number(
        node,
        b"y\0" as *const u8 as *const ::core::ffi::c_char,
        cause,
        -WINDOW_MAXIMUM as int64_t,
        WINDOW_MAXIMUM as int64_t,
        b"y-offset\0" as *const u8 as *const ::core::ffi::c_char,
    )?;
    let geometry = LayoutDescriptionGeometry {
        sx: sx as u32,
        sy: sy as u32,
        xoff: xoff as i32,
        yoff: yoff as i32,
    };

    if type_0 == LAYOUT_WINDOWPANE {
        if !json_find(node, b"c\0" as *const u8 as *const ::core::ffi::c_char).is_null() {
            *cause =
                xstrdup(b"panes cannot have children\0" as *const u8 as *const ::core::ffi::c_char);
            return None;
        }
        let index = layout_description_json_number(
            node,
            b"i\0" as *const u8 as *const ::core::ffi::c_char,
            cause,
            0,
            INT_MAX as int64_t,
            b"index\0" as *const u8 as *const ::core::ffi::c_char,
        )? as i32;
        let active_field =
            !json_find(node, b"a\0" as *const u8 as *const ::core::ffi::c_char).is_null();
        let mut active = false;
        if active_field {
            let mut boolean = 0;
            if json_find_boolean(
                node,
                b"a\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut boolean,
                cause,
            ) != 0
            {
                return None;
            }
            active = boolean != 0;
            if active {
                *active_count += 1;
            }
        }
        let last = if !active_field
            && !json_find(node, b"l\0" as *const u8 as *const ::core::ffi::c_char).is_null()
        {
            Some(layout_description_json_number(
                node,
                b"l\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
                0,
                INT_MAX as int64_t,
                b"last\0" as *const u8 as *const ::core::ffi::c_char,
            )? as i32)
        } else {
            None
        };
        let mut flags = 0;
        let zindex =
            if !json_find(node, b"z\0" as *const u8 as *const ::core::ffi::c_char).is_null() {
                let zindex = layout_description_json_number(
                    node,
                    b"z\0" as *const u8 as *const ::core::ffi::c_char,
                    cause,
                    0,
                    (INT_MAX - 1) as int64_t,
                    b"floating zindex\0" as *const u8 as *const ::core::ffi::c_char,
                )? as i32;
                flags |= LAYOUT_CELL_FLOATING;
                Some(zindex)
            } else {
                None
            };
        let identifier = {
            let field = json_find(node, b"I\0" as *const u8 as *const ::core::ffi::c_char);
            if field.is_null() {
                None
            } else {
                let mut value = ::core::ptr::null::<::core::ffi::c_char>();
                if json_get_string(field, &raw mut value) == 0 {
                    let length = strlen(value) as usize;
                    Some(LayoutDescriptionBytes::from_slice(
                        ::std::slice::from_raw_parts(value as *const u8, length),
                    ))
                } else {
                    None
                }
            }
        };
        let mut result = LayoutDescriptionNode::leaf(
            geometry,
            LayoutDescriptionPane {
                index: Some(index),
                id: None,
                identifier,
                active,
                last,
                zindex,
            },
        );
        result.flags = flags;
        Some(result)
    } else {
        let mut array = ::core::ptr::null_mut::<json_node>();
        if json_find_array(
            node,
            b"c\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut array,
            cause,
        ) != 0
        {
            return None;
        }
        let first = json_array_first(array);
        if first.is_null() || json_array_next(first).is_null() {
            *cause = xstrdup(
                b"nodes must have more than one child\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return None;
        }
        let mut children = LayoutDescriptionChildren::new();
        let mut member = first;
        while !member.is_null() {
            children.push(layout_description_parse_json_cell(
                member,
                cause,
                active_count,
            )?);
            member = json_array_next(member);
        }
        Some(LayoutDescriptionNode::node(type_0, geometry, children))
    }
}

unsafe fn layout_description_validate(
    description: &LayoutDescription,
    cause: *mut *mut ::core::ffi::c_char,
) -> bool {
    if description.version != 2 {
        return true;
    }
    let mut validation = LayoutDescriptionValidation {
        pane_count: 0,
        active_count: 0,
        duplicate_index: false,
        duplicate_zindex: false,
        duplicate_last: false,
    };
    layout_description_validate_node(&description.root, &description.root, &mut validation);
    if validation.pane_count == 0 {
        *cause = xstrdup(b"no panes\0" as *const u8 as *const ::core::ffi::c_char);
        return false;
    }
    if validation.active_count > 1 {
        *cause = xstrdup(b"more than one active pane\0" as *const u8 as *const ::core::ffi::c_char);
        return false;
    }
    if validation.duplicate_index {
        *cause = xstrdup(b"duplicate pane index\0" as *const u8 as *const ::core::ffi::c_char);
        return false;
    }
    if validation.duplicate_zindex {
        *cause = xstrdup(b"duplicate pane z-index\0" as *const u8 as *const ::core::ffi::c_char);
        return false;
    }
    if validation.duplicate_last {
        *cause = xstrdup(b"duplicate last pane index\0" as *const u8 as *const ::core::ffi::c_char);
        return false;
    }
    true
}

struct LayoutDescriptionValidation {
    pane_count: usize,
    active_count: usize,
    duplicate_index: bool,
    duplicate_zindex: bool,
    duplicate_last: bool,
}

fn layout_description_validate_node(
    node: &LayoutDescriptionNode,
    root: &LayoutDescriptionNode,
    validation: &mut LayoutDescriptionValidation,
) {
    if let Some(pane) = node.pane.as_ref() {
        validation.pane_count += 1;
        if pane.active {
            validation.active_count += 1;
        }
        let pane_ptr = pane as *const LayoutDescriptionPane;
        validation.duplicate_index |= root.has_duplicate_index(pane, pane_ptr);
        validation.duplicate_zindex |= root.has_duplicate_zindex(pane, pane_ptr);
        validation.duplicate_last |= root.has_duplicate_last(pane, pane_ptr);
    }
    for index in 0..node.children.len() {
        layout_description_validate_node(&node.children[index], root, validation);
    }
}

unsafe fn layout_parse_description_c(
    mut input: *const ::core::ffi::c_char,
    cause: *mut *mut ::core::ffi::c_char,
) -> Option<LayoutDescription> {
    while (*input as u8).is_ascii_whitespace() {
        input = input.offset(1);
    }
    if *input as u8 != b'{' {
        let mut checksum = 0u16;
        let mut header_len = 0;
        if sscanf(
            input,
            b"%hx,%n\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut checksum,
            &raw mut header_len,
        ) != 1
            || header_len != 5
        {
            *cause =
                xstrdup(b"malformed layout header\0" as *const u8 as *const ::core::ffi::c_char);
            return None;
        }
        let body = input.offset(header_len as isize);
        if checksum != layout_checksum(body) {
            *cause =
                xstrdup(b"invalid layout checksum\0" as *const u8 as *const ::core::ffi::c_char);
            return None;
        }
        let mut cursor = body;
        let root = match layout_description_construct_v1(&mut cursor, 0) {
            Some(root) => root,
            None => {
                *cause = xstrdup(b"invalid layout\0" as *const u8 as *const ::core::ffi::c_char);
                return None;
            }
        };
        if *cursor as u8 != 0 {
            *cause = xstrdup(b"trailing data\0" as *const u8 as *const ::core::ffi::c_char);
            return None;
        }
        let description = LayoutDescription { version: 1, root };
        if !layout_description_validate(&description, cause) {
            return None;
        }
        return Some(description);
    }

    let json = json_parse(input, cause);
    if json.is_null() {
        return None;
    }
    let mut object = ::core::ptr::null_mut::<json_node>();
    let mut version = 0;
    let mut layout = ::core::ptr::null_mut::<json_node>();
    let mut active_count = 0;
    let result = if json_get_object(json, &raw mut object) != 0 {
        *cause = xstrdup(b"invalid layout json\0" as *const u8 as *const ::core::ffi::c_char);
        None
    } else if json_find_number(
        object,
        b"V\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut version,
        cause,
    ) != 0
        || json_find_object(
            object,
            b"L\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut layout,
            cause,
        ) != 0
    {
        None
    } else {
        let root = layout_description_parse_json_cell(layout, cause, &mut active_count);
        root.map(|root| LayoutDescription { version, root })
    };
    json_destroy_node(json);
    let description = result?;
    if description.version != 2 {
        *cause = xstrdup(b"version mismatch\0" as *const u8 as *const ::core::ffi::c_char);
        return None;
    }
    if active_count > 1 {
        *cause = xstrdup(b"more than one active pane\0" as *const u8 as *const ::core::ffi::c_char);
        return None;
    }
    if !layout_description_validate(&description, cause) {
        return None;
    }
    Some(description)
}

/// Parse a custom layout without creating or changing any live layout cells.
pub fn parse_layout_description(input: &[u8]) -> Result<LayoutDescription, LayoutParseError> {
    if input.contains(&0) {
        return Err(LayoutParseError {
            message: LayoutDescriptionBytes::from_slice(b"embedded NUL"),
        });
    }
    let owned = unsafe { xmemdup(input.as_ptr() as *const ::core::ffi::c_void, input.len()) };
    let mut cause = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let result =
        unsafe { layout_parse_description_c(owned as *const ::core::ffi::c_char, &raw mut cause) };
    unsafe {
        free(owned as *mut ::core::ffi::c_void);
    }
    if result.is_none() {
        let message = if cause.is_null() {
            LayoutDescriptionBytes::from_slice(b"invalid layout")
        } else {
            let message = unsafe { LayoutDescriptionBytes::from_owned_c_string(cause) };
            cause = ::core::ptr::null_mut();
            message
        };
        unsafe {
            free(cause as *mut ::core::ffi::c_void);
        }
        return Err(LayoutParseError { message });
    }
    unsafe {
        free(cause as *mut ::core::ffi::c_void);
    }
    Ok(result.unwrap())
}

unsafe fn layout_string_append(output: *mut layout_string, value: &[u8]) {
    while (*output)
        .size
        .wrapping_add(value.len() as size_t)
        .wrapping_add(1 as size_t)
        > (*output).capacity
    {
        (*output).dat = xreallocarray(
            (*output).dat as *mut ::core::ffi::c_void,
            2 as size_t,
            (*output).capacity,
        ) as *mut ::core::ffi::c_char;
        (*output).capacity = (*output).capacity.wrapping_mul(2 as size_t);
    }
    if !value.is_empty() {
        memcpy(
            (*output).dat.add((*output).size) as *mut ::core::ffi::c_void,
            value.as_ptr() as *const ::core::ffi::c_void,
            value.len(),
        );
    }
    (*output).size = (*output).size.wrapping_add(value.len() as size_t);
    *(*output).dat.add((*output).size) = 0;
}

unsafe fn layout_description_append_json_string(output: *mut layout_string, value: &[u8]) -> bool {
    let mut index = 0;
    while index < value.len() {
        match value[index] {
            byte if byte < 0x20 || byte == b'"' => return false,
            b'\\' => {
                index += 1;
                if index == value.len() {
                    return false;
                }
                if value[index] == b'u' {
                    if index + 4 >= value.len()
                        || !value[index + 1..index + 5]
                            .iter()
                            .all(|byte| byte.is_ascii_hexdigit())
                    {
                        return false;
                    }
                    index += 4;
                } else if !matches!(
                    value[index],
                    b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't'
                ) {
                    return false;
                }
            }
            _ => {}
        }
        index += 1;
    }
    layout_string_append(output, b"\"");
    layout_string_append(output, value);
    layout_string_append(output, b"\"");
    true
}

unsafe fn layout_description_append_json_node(
    node: &LayoutDescriptionNode,
    output: *mut layout_string,
) -> bool {
    let type_name = match node.type_0 {
        LAYOUT_WINDOWPANE => 'p' as ::core::ffi::c_int,
        LAYOUT_TOPBOTTOM => 'v' as ::core::ffi::c_int,
        LAYOUT_LEFTRIGHT => 'h' as ::core::ffi::c_int,
        _ => return false,
    };
    layout_string_write(
        output,
        b"{\"t\":\"%c\",\"w\":%u,\"h\":%u,\"x\":%d,\"y\":%d\0" as *const u8
            as *const ::core::ffi::c_char,
        type_name,
        node.geometry.sx,
        node.geometry.sy,
        node.geometry.xoff,
        node.geometry.yoff,
    );
    if node.type_0 == LAYOUT_WINDOWPANE {
        if !node.children.is_empty() {
            return false;
        }
        let Some(pane) = node.pane.as_ref() else {
            return false;
        };
        let Some(index) = pane.index else {
            return false;
        };
        layout_string_write(
            output,
            b",\"i\":%d\0" as *const u8 as *const ::core::ffi::c_char,
            index,
        );
        if pane.active {
            layout_string_append(output, b",\"a\":true");
        } else if let Some(last) = pane.last {
            layout_string_write(
                output,
                b",\"l\":%d\0" as *const u8 as *const ::core::ffi::c_char,
                last,
            );
        }
        if let Some(zindex) = pane.zindex {
            layout_string_write(
                output,
                b",\"z\":%d\0" as *const u8 as *const ::core::ffi::c_char,
                zindex,
            );
        }
        if let Some(identifier) = pane.identifier.as_ref() {
            layout_string_append(output, b",\"I\":");
            if !layout_description_append_json_string(output, identifier.as_slice()) {
                return false;
            }
        }
    } else {
        if node.pane.is_some() || node.children.len() < 2 {
            return false;
        }
        layout_string_append(output, b",\"c\":[");
        for index in 0..node.children.len() {
            if index != 0 {
                layout_string_append(output, b",");
            }
            if !layout_description_append_json_node(&node.children[index], output) {
                return false;
            }
        }
        layout_string_append(output, b"]");
    }
    layout_string_append(output, b"}");
    true
}

unsafe fn layout_description_append_v1(
    node: &LayoutDescriptionNode,
    output: *mut layout_string,
) -> bool {
    layout_string_write(
        output,
        b"%ux%u,%d,%d\0" as *const u8 as *const ::core::ffi::c_char,
        node.geometry.sx,
        node.geometry.sy,
        node.geometry.xoff,
        node.geometry.yoff,
    );
    match node.type_0 {
        LAYOUT_WINDOWPANE => {
            if !node.children.is_empty() {
                return false;
            }
            if let Some(id) = node.pane.as_ref().and_then(|pane| pane.id) {
                layout_string_write(
                    output,
                    b",%u\0" as *const u8 as *const ::core::ffi::c_char,
                    id,
                );
            }
        }
        LAYOUT_LEFTRIGHT | LAYOUT_TOPBOTTOM => {
            if node.pane.is_some() || node.children.is_empty() {
                return false;
            }
            layout_string_append(
                output,
                if node.type_0 == LAYOUT_LEFTRIGHT {
                    b"{"
                } else {
                    b"["
                },
            );
            for index in 0..node.children.len() {
                if index != 0 {
                    layout_string_append(output, b",");
                }
                if !layout_description_append_v1(&node.children[index], output) {
                    return false;
                }
            }
            layout_string_append(
                output,
                if node.type_0 == LAYOUT_LEFTRIGHT {
                    b"}"
                } else {
                    b"]"
                },
            );
        }
        _ => return false,
    }
    true
}

fn layout_description_checksum(bytes: &[u8]) -> u16 {
    let mut checksum = 0u16;
    for byte in bytes {
        checksum = (checksum >> 1) | ((checksum & 1) << 15);
        checksum = checksum.wrapping_add((*byte as ::core::ffi::c_char) as i32 as u16);
    }
    checksum
}

unsafe fn layout_description_bytes_from_string(
    output: *mut layout_string,
) -> LayoutDescriptionBytes {
    let result = LayoutDescriptionBytes {
        data: (*output).dat as *mut u8,
        len: (*output).size,
    };
    (*output).dat = ::core::ptr::null_mut();
    (*output).size = 0;
    (*output).capacity = 0;
    result
}

/// Serialize a parsed custom layout without consulting or changing a window.
///
/// The legacy JSON tokenizer deliberately retains string escape bytes instead
/// of decoding them; valid escape sequences are therefore emitted unchanged.
/// This returns `None` for an invalid identifier supplied by a caller that
/// constructs a description manually.
pub fn serialize_layout_description(
    description: &LayoutDescription,
) -> Option<LayoutDescriptionBytes> {
    unsafe {
        match description.version {
            1 => {
                let mut body = layout_string {
                    dat: ::core::ptr::null_mut(),
                    size: 0,
                    capacity: 0,
                };
                let mut output = layout_string {
                    dat: ::core::ptr::null_mut(),
                    size: 0,
                    capacity: 0,
                };
                layout_string_init(&raw mut body);
                if !layout_description_append_v1(&description.root, &raw mut body) {
                    layout_string_free(&raw mut body);
                    return None;
                }
                layout_string_init(&raw mut output);
                layout_string_write(
                    &raw mut output,
                    b"%04hx,\0" as *const u8 as *const ::core::ffi::c_char,
                    layout_description_checksum(::std::slice::from_raw_parts(
                        body.dat as *const u8,
                        body.size,
                    )) as ::core::ffi::c_int,
                );
                layout_string_append(
                    &raw mut output,
                    ::std::slice::from_raw_parts(body.dat as *const u8, body.size),
                );
                layout_string_free(&raw mut body);
                Some(layout_description_bytes_from_string(&raw mut output))
            }
            2 => {
                let mut output = layout_string {
                    dat: ::core::ptr::null_mut(),
                    size: 0,
                    capacity: 0,
                };
                layout_string_init(&raw mut output);
                layout_string_append(&raw mut output, b"{\"V\":2,\"L\":");
                if !layout_description_append_json_node(&description.root, &raw mut output) {
                    layout_string_free(&raw mut output);
                    return None;
                }
                layout_string_append(&raw mut output, b"}");
                Some(layout_description_bytes_from_string(&raw mut output))
            }
            _ => None,
        }
    }
}

unsafe fn layout_description_to_cell(
    node: &LayoutDescriptionNode,
    parent: *mut layout_cell,
    pctx: *mut layout_parse_ctx,
) -> *mut layout_cell {
    let lc = layout_create_cell(parent);
    (*lc).type_0 = node.type_0;
    (*lc).flags = node.flags;
    layout_set_size(
        lc,
        node.geometry.sx,
        node.geometry.sy,
        node.geometry.xoff,
        node.geometry.yoff,
    );
    if let Some(pane) = node.pane.as_ref() {
        if (*pctx).version > 1 {
            layout_parse_add_cctx(
                pctx,
                lc,
                i32::from(pane.active),
                pane.last.unwrap_or(-1),
                pane.index.unwrap_or(-1),
                pane.zindex.unwrap_or(INT_MAX),
            );
            if pane.active {
                (*pctx).num_active += 1;
            }
        }
        return lc;
    }
    for index in 0..node.children.len() {
        let lcchild = layout_description_to_cell(&node.children[index], lc, pctx);
        (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
        (*lcchild).entry.tqe_prev = (*lc).cells.tqh_last;
        *(*lc).cells.tqh_last = lcchild;
        (*lc).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
    }
    lc
}
#[no_mangle]
pub unsafe extern "C" fn layout_dump(
    mut w: *mut window,
    mut lcroot: *mut layout_cell,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut layout_string: layout_string = layout_string {
        dat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
        capacity: 0,
    };
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lcroot.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    layout_string_init(&raw mut layout_string);
    if layout_append(lcroot, &raw mut layout_string, flags) == 0 as ::core::ffi::c_int {
        if flags & LAYOUT_CUSTOM_OLD_FORMAT != 0 {
            xasprintf(
                &raw mut out,
                b"%04hx,%s\0" as *const u8 as *const ::core::ffi::c_char,
                layout_checksum(layout_string.dat) as ::core::ffi::c_int,
                layout_string.dat,
            );
        } else {
            xasprintf(
                &raw mut out,
                b"{\"V\":2,\"L\":%s}\0" as *const u8 as *const ::core::ffi::c_char,
                layout_string.dat,
            );
        }
    }
    layout_string_free(&raw mut layout_string);
    return out;
}
unsafe extern "C" fn layout_append_v2(
    mut lc: *mut layout_cell,
    mut ls: *mut layout_string,
) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
    let mut c: ::core::ffi::c_char = 0;
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    if lc.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    type_0 = (*lc).type_0;
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        c = 'v' as i32 as ::core::ffi::c_char;
    } else if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        c = 'h' as i32 as ::core::ffi::c_char;
    } else if type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        c = 'p' as i32 as ::core::ffi::c_char;
    } else {
        return -(1 as ::core::ffi::c_int);
    }
    layout_string_write(
        ls,
        b"{\"t\":\"%c\",\"w\":%u,\"h\":%u,\"x\":%d,\"y\":%d\0" as *const u8
            as *const ::core::ffi::c_char,
        c as ::core::ffi::c_int,
        (*lc).g.sx,
        (*lc).g.sy,
        (*lc).g.xoff,
        (*lc).g.yoff,
    );
    if type_0 as ::core::ffi::c_uint
        != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        layout_string_write(ls, b",\"c\":[\0" as *const u8 as *const ::core::ffi::c_char);
        n = 0 as u_int;
        lcchild = (*lc).cells.tqh_first;
        while !lcchild.is_null() {
            if layout_append_v2(lcchild, ls) != 0 as ::core::ffi::c_int {
                return -(1 as ::core::ffi::c_int);
            }
            layout_string_write(ls, b",\0" as *const u8 as *const ::core::ffi::c_char);
            n = n.wrapping_add(1);
            lcchild = (*lcchild).entry.tqe_next;
        }
        if n == 0 as u_int {
            return -(1 as ::core::ffi::c_int);
        }
        (*ls).size = (*ls).size.wrapping_sub(1);
        *(*ls).dat.offset((*ls).size as isize) = '\0' as i32 as ::core::ffi::c_char;
        layout_string_write(ls, b"]\0" as *const u8 as *const ::core::ffi::c_char);
    } else {
        wp = (*lc).wp;
        if wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        if wp == (*(*wp).window).active {
            layout_string_write(
                ls,
                b",\"a\":true\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else if window_pane_last_index(wp, &raw mut i) == 0 as ::core::ffi::c_int {
            layout_string_write(
                ls,
                b",\"l\":%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
        }
        if window_pane_index(wp, &raw mut i) != 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        layout_string_write(
            ls,
            b",\"i\":%u\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        if (*lc).flags & LAYOUT_CELL_FLOATING != 0
            && window_pane_zindex(wp, &raw mut i) == 0 as ::core::ffi::c_int
        {
            layout_string_write(
                ls,
                b",\"z\":%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
        }
        layout_string_write(
            ls,
            b",\"I\":\"%%%u\"\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
    }
    layout_string_write(ls, b"}\0" as *const u8 as *const ::core::ffi::c_char);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_append_v1(
    mut lc: *mut layout_cell,
    mut ls: *mut layout_string,
) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut brackets: *const ::core::ffi::c_char =
        b"[]\0" as *const u8 as *const ::core::ffi::c_char;
    if lc.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if !(*lc).wp.is_null() {
        layout_string_write(
            ls,
            b"%ux%u,%d,%d,%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*lc).g.sx,
            (*lc).g.sy,
            (*lc).g.xoff,
            (*lc).g.yoff,
            (*(*lc).wp).id,
        );
    } else {
        layout_string_write(
            ls,
            b"%ux%u,%d,%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*lc).g.sx,
            (*lc).g.sy,
            (*lc).g.xoff,
            (*lc).g.yoff,
        );
    }
    let mut current_block_16: u64;
    match (*lc).type_0 as ::core::ffi::c_uint {
        0 => {
            brackets = b"{}\0" as *const u8 as *const ::core::ffi::c_char;
            current_block_16 = 14129903220312603109;
        }
        1 => {
            current_block_16 = 14129903220312603109;
        }
        2 | _ => {
            current_block_16 = 1054647088692577877;
        }
    }
    match current_block_16 {
        14129903220312603109 => {
            layout_string_write(
                ls,
                b"%c\0" as *const u8 as *const ::core::ffi::c_char,
                *brackets.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            );
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                if layout_append_v1(lcchild, ls) != 0 as ::core::ffi::c_int {
                    return -(1 as ::core::ffi::c_int);
                }
                layout_string_write(ls, b",\0" as *const u8 as *const ::core::ffi::c_char);
                lcchild = (*lcchild).entry.tqe_next;
            }
            (*ls).size = (*ls).size.wrapping_sub(1);
            *(*ls).dat.offset((*ls).size as isize) = '\0' as i32 as ::core::ffi::c_char;
            layout_string_write(
                ls,
                b"%c\0" as *const u8 as *const ::core::ffi::c_char,
                *brackets.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            );
        }
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_custom_copy_layout(mut lc: *mut layout_cell) -> *mut layout_cell {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnewchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lconly: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnew: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*lc).flags & LAYOUT_CELL_FLOATING != 0
    {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    lcnew = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    (*lcnew).type_0 = (*lc).type_0;
    (*lcnew).flags = (*lc).flags;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*lcnew).wp = (*lc).wp;
    }
    layout_set_size(lcnew, (*lc).g.sx, (*lc).g.sy, (*lc).g.xoff, (*lc).g.yoff);
    match (*lc).type_0 as ::core::ffi::c_uint {
        1 | 0 => {
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                lcnewchild = layout_custom_copy_layout(lcchild);
                if !lcnewchild.is_null() {
                    (*lcnewchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
                    (*lcnewchild).entry.tqe_prev = (*lcnew).cells.tqh_last;
                    *(*lcnew).cells.tqh_last = lcnewchild;
                    (*lcnew).cells.tqh_last = &raw mut (*lcnewchild).entry.tqe_next;
                    (*lcnewchild).parent = lcnew;
                }
                lcchild = (*lcchild).entry.tqe_next;
            }
            lconly = (*lcnew).cells.tqh_first;
            if lconly.is_null() {
                layout_free_cell(lcnew, 0 as ::core::ffi::c_int);
                return ::core::ptr::null_mut::<layout_cell>();
            }
            if (*lconly).entry.tqe_next.is_null() {
                if !(*lconly).entry.tqe_next.is_null() {
                    (*(*lconly).entry.tqe_next).entry.tqe_prev = (*lconly).entry.tqe_prev;
                } else {
                    (*lcnew).cells.tqh_last = (*lconly).entry.tqe_prev;
                }
                *(*lconly).entry.tqe_prev = (*lconly).entry.tqe_next;
                (*lconly).parent = ::core::ptr::null_mut::<layout_cell>();
                layout_free_cell(lcnew, 0 as ::core::ffi::c_int);
                return lconly;
            }
        }
        2 | _ => {}
    }
    return lcnew;
}
unsafe extern "C" fn layout_custom_create_compat(mut lcroot: *mut layout_cell) -> *mut layout_cell {
    let mut lccompat: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    lccompat = layout_custom_copy_layout(lcroot);
    if !lccompat.is_null() && layout_cell_is_tiled(lccompat) != 0 {
        (*lccompat).g.xoff = 0 as ::core::ffi::c_int;
        (*lccompat).g.yoff = 0 as ::core::ffi::c_int;
    }
    return lccompat;
}
unsafe extern "C" fn layout_custom_unlink_panes(mut lc: *mut layout_cell) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    match (*lc).type_0 as ::core::ffi::c_uint {
        2 => {
            (*lc).wp = ::core::ptr::null_mut::<window_pane>();
        }
        0 | 1 => {
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                layout_custom_unlink_panes(lcchild);
                lcchild = (*lcchild).entry.tqe_next;
            }
        }
        _ => {}
    };
}
unsafe extern "C" fn layout_custom_free_compat(mut lcroot: *mut layout_cell) {
    if lcroot.is_null() {
        return;
    }
    layout_custom_unlink_panes(lcroot);
    layout_free_cell(lcroot, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn layout_append(
    mut lcroot: *mut layout_cell,
    mut ls: *mut layout_string,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lccompat: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut result: ::core::ffi::c_int = 0;
    if flags & LAYOUT_CUSTOM_OLD_FORMAT != 0 {
        if layout_cell_is_tiled(lcroot) == 0 && layout_cell_has_tiled_child(lcroot) == 0 {
            return -(1 as ::core::ffi::c_int);
        }
        lccompat = layout_custom_create_compat(lcroot);
        result = layout_append_v1(lccompat, ls);
        layout_custom_free_compat(lccompat);
    } else {
        result = layout_append_v2(lcroot, ls);
    }
    return result;
}
unsafe extern "C" fn layout_check(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0 as u_int;
    match (*lc).type_0 as ::core::ffi::c_uint {
        0 => {
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                if !(layout_cell_is_tiled(lcchild) == 0
                    && layout_cell_has_tiled_child(lcchild) == 0)
                {
                    if (*lcchild).g.sy != (*lc).g.sy {
                        return 0 as ::core::ffi::c_int;
                    }
                    if layout_check(lcchild) == 0 {
                        return 0 as ::core::ffi::c_int;
                    }
                    n = n.wrapping_add((*lcchild).g.sx.wrapping_add(1 as u_int));
                }
                lcchild = (*lcchild).entry.tqe_next;
            }
            if n != 0 as u_int && n.wrapping_sub(1 as u_int) != (*lc).g.sx {
                return 0 as ::core::ffi::c_int;
            }
        }
        1 => {
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                if !(layout_cell_is_tiled(lcchild) == 0
                    && layout_cell_has_tiled_child(lcchild) == 0)
                {
                    if (*lcchild).g.sx != (*lc).g.sx {
                        return 0 as ::core::ffi::c_int;
                    }
                    if layout_check(lcchild) == 0 {
                        return 0 as ::core::ffi::c_int;
                    }
                    n = n.wrapping_add((*lcchild).g.sy.wrapping_add(1 as u_int));
                }
                lcchild = (*lcchild).entry.tqe_next;
            }
            if n != 0 as u_int && n.wrapping_sub(1 as u_int) != (*lc).g.sy {
                return 0 as ::core::ffi::c_int;
            }
        }
        2 | _ => {}
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn layout_parse(
    mut w: *mut window,
    mut input: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut pctx: layout_parse_ctx = layout_parse_ctx {
        version: 0,
        num_active: 0,
        root: ::core::ptr::null_mut::<layout_cell>(),
        cause: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        size: 0,
        capacity: 0,
        cctxs: ::core::ptr::null_mut::<layout_parse_cell_ctx>(),
    };
    let mut npanes: u_int = 0;
    let mut ncells: u_int = 0;
    let mut sx: u_int = 0 as u_int;
    let mut sy: u_int = 0 as u_int;
    let mut with_floating: ::core::ffi::c_int = 0;
    layout_parse_init_ctx(&raw mut pctx, cause);
    let description = match layout_parse_description_c(input, cause) {
        Some(description) => description,
        None => {
            layout_parse_free_ctx(&raw mut pctx);
            return -(1 as ::core::ffi::c_int);
        }
    };
    // This conversion creates a detached tree. Pane-count pruning and the
    // geometry check below therefore still happen before any live pane is
    // resized or attached to the window.
    pctx.version = description.version;
    pctx.root =
        layout_description_to_cell(&description.root, ::core::ptr::null_mut(), &raw mut pctx);
    with_floating = (pctx.version > 1 as int64_t) as ::core::ffi::c_int;
    npanes = window_count_panes(w, with_floating);
    if npanes == 0 as u_int {
        xasprintf(
            cause,
            b"window @%u has no panes\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
        );
    } else {
        loop {
            ncells = layout_count_cells(pctx.root, with_floating);
            if npanes > ncells {
                xasprintf(
                    cause,
                    b"have %u panes but need %u\0" as *const u8 as *const ::core::ffi::c_char,
                    npanes,
                    ncells,
                );
                current_block = 4277046812173491162;
                break;
            } else {
                if npanes == ncells {
                    current_block = 15976848397966268834;
                    break;
                }
                lcchild = layout_find_bottomright(pctx.root);
                if pctx.version > 1 as int64_t
                    && layout_parse_remove_cctx(&raw mut pctx, lcchild) != 0 as ::core::ffi::c_int
                {
                    *cause = xstrdup(
                        b"empty/missing layout parse context\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                    current_block = 4277046812173491162;
                    break;
                } else {
                    layout_destroy_cell(
                        ::core::ptr::null_mut::<window>(),
                        lcchild,
                        &raw mut pctx.root,
                    );
                }
            }
        }
        match current_block {
            4277046812173491162 => {}
            _ => {
                lc = pctx.root;
                pctx.root = ::core::ptr::null_mut::<layout_cell>();
                match (*lc).type_0 as ::core::ffi::c_uint {
                    0 => {
                        lcchild = (*lc).cells.tqh_first;
                        while !lcchild.is_null() {
                            if layout_cell_is_tiled(lcchild) != 0
                                || layout_cell_has_tiled_child(lcchild) != 0
                            {
                                sy = (*lcchild).g.sy.wrapping_add(1 as u_int);
                                sx = sx.wrapping_add((*lcchild).g.sx.wrapping_add(1 as u_int));
                            }
                            lcchild = (*lcchild).entry.tqe_next;
                        }
                    }
                    1 => {
                        lcchild = (*lc).cells.tqh_first;
                        while !lcchild.is_null() {
                            if layout_cell_is_tiled(lcchild) != 0
                                || layout_cell_has_tiled_child(lcchild) != 0
                            {
                                sx = (*lcchild).g.sx.wrapping_add(1 as u_int);
                                sy = sy.wrapping_add((*lcchild).g.sy.wrapping_add(1 as u_int));
                            }
                            lcchild = (*lcchild).entry.tqe_next;
                        }
                    }
                    2 | _ => {}
                }
                if (*lc).type_0 as ::core::ffi::c_uint
                    != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
                    && sx != 0 as u_int
                    && sy != 0 as u_int
                    && ((*lc).g.sx != sx || (*lc).g.sy != sy)
                {
                    layout_print_cell(
                        lc,
                        b"layout_parse\0" as *const u8 as *const ::core::ffi::c_char,
                        0 as u_int,
                    );
                    (*lc).g.sx = sx.wrapping_sub(1 as u_int);
                    (*lc).g.sy = sy.wrapping_sub(1 as u_int);
                }
                if layout_check(lc) == 0 {
                    *cause = xstrdup(
                        b"size mismatch after applying layout\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                } else {
                    // Everything above this point only changes the detached
                    // tree. Once the resize starts, the remaining operations
                    // are the existing non-fallible tree/pane commit path.
                    if layout_cell_is_tiled(lc) != 0 || layout_cell_has_tiled_child(lc) != 0 {
                        window_resize(
                            w,
                            (*lc).g.sx,
                            (*lc).g.sy,
                            -(1 as ::core::ffi::c_int),
                            -(1 as ::core::ffi::c_int),
                        );
                    }
                    if pctx.version == 1 as int64_t {
                        wp = (*w).panes.tqh_first;
                        while !wp.is_null() {
                            if !(window_pane_is_floating(wp) == 0) {
                                lcchild = (*wp).layout_cell as *mut layout_cell;
                                if !(*lcchild).entry.tqe_next.is_null() {
                                    (*(*lcchild).entry.tqe_next).entry.tqe_prev =
                                        (*lcchild).entry.tqe_prev;
                                } else {
                                    (*(*lcchild).parent).cells.tqh_last = (*lcchild).entry.tqe_prev;
                                }
                                *(*lcchild).entry.tqe_prev = (*lcchild).entry.tqe_next;
                                (*lcchild).parent = ::core::ptr::null_mut::<layout_cell>();
                            }
                            wp = (*wp).entry.tqe_next;
                        }
                    }
                    layout_free_cell((*w).layout_root, 0 as ::core::ffi::c_int);
                    (*w).layout_root = lc;
                    layout_assign(w, &raw mut pctx);
                    layout_fix_offsets(w);
                    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
                    if pctx.version > 1 as int64_t {
                        layout_parse_apply_ctx(w, &raw mut pctx);
                    }
                    recalculate_sizes();
                    layout_print_cell(
                        lc,
                        b"layout_parse\0" as *const u8 as *const ::core::ffi::c_char,
                        0 as u_int,
                    );
                    if pctx.version == 1 as int64_t {
                        events_fire_window(
                            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
                            w,
                        );
                    }
                    layout_parse_free_ctx(&raw mut pctx);
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
    }
    layout_free_cell(lc, 0 as ::core::ffi::c_int);
    layout_parse_free_ctx(&raw mut pctx);
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn layout_assign_from_ctx(mut w: *mut window, mut pctx: *mut layout_parse_ctx) {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: ::core::ffi::c_int = 0;
    qsort(
        (*pctx).cctxs as *mut ::core::ffi::c_void,
        (*pctx).size as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_index_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    wp = (*w).panes.tqh_first;
    i = 0 as ::core::ffi::c_int;
    while i < (*pctx).size {
        lc = (*(*pctx).cctxs.offset(i as isize)).lc;
        layout_make_leaf(lc, wp);
        wp = (*wp).entry.tqe_next;
        i += 1;
    }
}
unsafe extern "C" fn layout_assign_fallback_tiled(
    mut wp: *mut *mut window_pane,
    mut lc: *mut layout_cell,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if lc.is_null() {
        return;
    }
    match (*lc).type_0 as ::core::ffi::c_uint {
        2 => {
            while !(*wp).is_null() && !(**wp).layout_cell.is_null() {
                *wp = (**wp).entry.tqe_next;
            }
            if (*wp).is_null() {
                return;
            }
            layout_make_leaf(lc, *wp);
            *wp = (**wp).entry.tqe_next;
            return;
        }
        0 | 1 => {
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                layout_assign_fallback_tiled(wp, lcchild);
                lcchild = (*lcchild).entry.tqe_next;
            }
            return;
        }
        _ => {}
    };
}
unsafe extern "C" fn layout_assign_fallback(mut w: *mut window, mut lcroot: *mut layout_cell) {
    let mut wp: *mut window_pane = (*w).panes.tqh_first;
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    layout_assign_fallback_tiled(&raw mut wp, lcroot);
    if window_count_panes(w, 1 as ::core::ffi::c_int) > 1 as u_int
        && (*lcroot).type_0 as ::core::ffi::c_uint
            == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        lcroot = layout_replace_with_node(w, lcroot, LAYOUT_TOPBOTTOM);
    }
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if window_pane_is_floating(wp) != 0 {
            lc = (*wp).layout_cell as *mut layout_cell;
            (*lc).parent = lcroot;
            (*lc).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
            (*lc).entry.tqe_prev = (*lcroot).cells.tqh_last;
            *(*lcroot).cells.tqh_last = lc;
            (*lcroot).cells.tqh_last = &raw mut (*lc).entry.tqe_next;
        }
        wp = (*wp).entry.tqe_next;
    }
}
unsafe extern "C" fn layout_assign(mut w: *mut window, mut pctx: *mut layout_parse_ctx) {
    if (*pctx).size > 0 as ::core::ffi::c_int {
        layout_assign_from_ctx(w, pctx);
    } else {
        layout_assign_fallback(w, (*w).layout_root);
    };
}
unsafe extern "C" fn layout_parse_apply_ctx(mut w: *mut window, mut pctx: *mut layout_parse_ctx) {
    let mut cctx: *mut layout_parse_cell_ctx = ::core::ptr::null_mut::<layout_parse_cell_ctx>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpnext: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: ::core::ffi::c_int = 0;
    wp = (*w).z_index.tqh_first;
    while !wp.is_null() {
        wpnext = (*wp).zentry.tqe_next;
        if window_pane_is_floating(wp) != 0 {
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
            } else {
                (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
            }
            *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
        }
        wp = wpnext;
    }
    qsort(
        (*pctx).cctxs as *mut ::core::ffi::c_void,
        (*pctx).size as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_zindex_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    i = 0 as ::core::ffi::c_int;
    while i < (*pctx).size {
        cctx = (*pctx).cctxs.offset(i as isize) as *mut layout_parse_cell_ctx;
        wp = (*(*cctx).lc).wp;
        if window_pane_is_floating(wp) != 0 {
            (*wp).zentry.tqe_next = (*w).z_index.tqh_first;
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*w).z_index.tqh_first).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
            } else {
                (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
            }
            (*w).z_index.tqh_first = wp;
            (*wp).zentry.tqe_prev = &raw mut (*w).z_index.tqh_first;
        }
        i += 1;
    }
    i = 0 as ::core::ffi::c_int;
    while i < (*pctx).size {
        cctx = (*pctx).cctxs.offset(i as isize) as *mut layout_parse_cell_ctx;
        if (*cctx).active == 1 as ::core::ffi::c_int {
            window_set_active_pane(w, (*(*cctx).lc).wp, 1 as ::core::ffi::c_int);
            break;
        } else {
            i += 1;
        }
    }
    while !(*w).last_panes.tqh_first.is_null() {
        wp = (*w).last_panes.tqh_first;
        window_pane_stack_remove(&raw mut (*w).last_panes, wp);
    }
    qsort(
        (*pctx).cctxs as *mut ::core::ffi::c_void,
        (*pctx).size as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_last_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    i = 0 as ::core::ffi::c_int;
    while i < (*pctx).size {
        cctx = (*pctx).cctxs.offset(i as isize) as *mut layout_parse_cell_ctx;
        wp = (*(*cctx).lc).wp;
        if !((*cctx).last < 0 as ::core::ffi::c_int || (*cctx).active == 1 as ::core::ffi::c_int) {
            window_pane_stack_push(&raw mut (*w).last_panes, wp);
        }
        i += 1;
    }
}
