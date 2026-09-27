//! Authoritative mode_tree model declarations.

use super::abi::{size_t, u_int, uint64_t};
use super::client::client;
use super::key::key_code;
use super::menu::menu_item;
use super::pane::window_pane;
use super::prompt::{prompt_free_cb, prompt_key_result, prompt_result, PromptRef};
use super::screen::screen;
use super::screen_write::screen_write_ctx;
use super::sort::sort_criteria;
use crate::src::window_buffer::window_buffer_itemdata;
use crate::src::window_client::window_client_itemdata;
use crate::src::window_customize::window_customize_itemdata;
use crate::src::window_tree::window_tree_itemdata;
use std::cell::{RefCell, UnsafeCell};
use std::rc::{Rc, Weak};

#[derive(Copy, Clone)]
pub struct mode_tree_help_info {
    pub width: u_int,
    pub item: &'static ::std::ffi::CStr,
    pub lines: &'static [&'static ::std::ffi::CStr],
}

pub struct mode_tree_data {
    pub dead: ::core::ffi::c_int,
    pub zoomed: ::core::ffi::c_int,
    pub wp: *mut window_pane,
    pub menu: &'static [menu_item<'static>],
    pub sort_crit: sort_criteria,
    pub view_name: Option<&'static ::std::ffi::CStr>,
    pub buildcb: mode_tree_build_cb,
    pub drawcb: mode_tree_draw_cb,
    pub searchcb: mode_tree_search_cb,
    pub menucb: mode_tree_menu_cb,
    pub heightcb: mode_tree_height_cb,
    pub keycb: mode_tree_key_cb,
    pub swapcb: mode_tree_swap_cb,
    pub sortcb: mode_tree_sort_cb,
    pub helpcb: mode_tree_help_cb,
    pub children: mode_tree_list,
    pub saved: mode_tree_list,
    pub lines: Vec<mode_tree_line>,
    pub depth: u_int,
    pub maxdepth: u_int,
    pub width: u_int,
    pub height: u_int,
    pub offset: u_int,
    pub current: u_int,
    pub screen: screen,
    pub prompt: Option<PromptRef>,
    pub prompt_data: ModeTreePromptWeak,
    pub prompt_cx: u_int,
    pub prompt_top: ::core::ffi::c_int,
    pub preview: ::core::ffi::c_int,
    pub search: Option<std::ffi::CString>,
    pub filter: Option<std::ffi::CString>,
    pub no_matches: ::core::ffi::c_int,
    pub search_dir: mode_tree_search_dir,
    pub search_icase: ::core::ffi::c_int,
    pub help: ::core::ffi::c_int,
}

impl Default for mode_tree_data {
    fn default() -> Self {
        Self {
            dead: 0,
            zoomed: 0,
            wp: std::ptr::null_mut(),
            menu: &[],
            sort_crit: sort_criteria {
                order: 0,
                reversed: 0,
                order_seq: &[],
            },
            view_name: None,
            buildcb: None,
            drawcb: None,
            searchcb: None,
            menucb: None,
            heightcb: None,
            keycb: None,
            swapcb: None,
            sortcb: None,
            helpcb: None,
            children: mode_tree_list::default(),
            saved: mode_tree_list::default(),
            lines: Vec::new(),
            depth: 0,
            maxdepth: 0,
            width: 0,
            height: 0,
            offset: 0,
            current: 0,
            screen: screen::empty(),
            prompt: None,
            prompt_data: None,
            prompt_cx: 0,
            prompt_top: 0,
            preview: 0,
            search: None,
            filter: None,
            no_matches: 0,
            search_dir: 0,
            search_icase: 0,
            help: 0,
        }
    }
}

pub type mode_tree_search_dir = ::core::ffi::c_uint;

/// The prompt cleanup closure owns this record; all other handles observe it.
pub type ModeTreePromptOwner = refbox::RefBox<mode_tree_prompt>;
pub type ModeTreePromptWeak = Option<refbox::Weak<mode_tree_prompt>>;

pub struct mode_tree_prompt {
    /// Taken at logical cleanup before dropping the callback record.
    pub mtd: Option<Rc<UnsafeCell<mode_tree_data>>>,
    pub c: Weak<UnsafeCell<client>>,
    pub inputcb: mode_tree_prompt_input_cb,
    pub freecb: prompt_free_cb,
}

pub type mode_tree_prompt_input_cb = Option<
    Box<
        dyn FnMut(
            Option<std::ptr::NonNull<client>>,
            Option<&std::ffi::CStr>,
            prompt_key_result,
        ) -> prompt_result,
    >,
>;

/// Each row keeps its mode-specific record alive across rebuilds and callbacks.
/// Category rows have no payload. Rows and mode lists share immutable records
/// in their original heap allocations.
#[derive(Clone, Default)]
pub enum ModeTreeItemData {
    #[default]
    None,
    Buffer(Rc<window_buffer_itemdata>),
    Client(Rc<window_client_itemdata>),
    Customize(Rc<window_customize_itemdata>),
    Tree(Rc<window_tree_itemdata>),
}

impl ModeTreeItemData {
    pub fn as_buffer(&self) -> Option<&Rc<window_buffer_itemdata>> {
        match self {
            Self::Buffer(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_client(&self) -> Option<&Rc<window_client_itemdata>> {
        match self {
            Self::Client(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_customize(&self) -> Option<&Rc<window_customize_itemdata>> {
        match self {
            Self::Customize(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_tree(&self) -> Option<&Rc<window_tree_itemdata>> {
        match self {
            Self::Tree(item) => Some(item),
            _ => None,
        }
    }
}

pub type ModeTreeItemRef = Rc<RefCell<mode_tree_item>>;
pub type ModeTreeItemWeak = Weak<RefCell<mode_tree_item>>;

#[derive(Clone)]
pub struct mode_tree_line {
    pub item: ModeTreeItemRef,
    pub depth: u_int,
    pub last: ::core::ffi::c_int,
    pub flat: ::core::ffi::c_int,
}

pub struct mode_tree_item {
    pub parent: ModeTreeItemWeak,
    pub itemdata: ModeTreeItemData,
    pub line: u_int,
    pub key: key_code,
    pub keystr: Option<std::ffi::CString>,
    pub keylen: size_t,
    pub tag: uint64_t,
    pub name: std::ffi::CString,
    pub text: Option<std::ffi::CString>,
    pub expanded: ::core::ffi::c_int,
    pub tagged: ::core::ffi::c_int,
    pub draw_as_parent: ::core::ffi::c_int,
    pub no_tag: ::core::ffi::c_int,
    pub align: ::core::ffi::c_int,
    pub children: mode_tree_list,
}

impl mode_tree_item {
    pub fn empty() -> Self {
        Self {
            parent: Default::default(),
            itemdata: Default::default(),
            line: Default::default(),
            key: Default::default(),
            keystr: Default::default(),
            keylen: Default::default(),
            tag: 0,
            name: std::ffi::CString::default(),
            text: Default::default(),
            expanded: Default::default(),
            tagged: Default::default(),
            draw_as_parent: Default::default(),
            no_tag: Default::default(),
            align: Default::default(),
            children: Default::default(),
        }
    }
}

/// Ordered row owners. Parents are weak, so retaining a selected row cannot
/// keep its ancestors alive after a rebuild or removal.
#[derive(Clone, Default)]
pub struct mode_tree_list {
    pub(crate) items: Vec<ModeTreeItemRef>,
}

impl mode_tree_list {
    pub(crate) fn first(&self) -> Option<ModeTreeItemRef> {
        self.items.first().cloned()
    }

    pub(crate) fn last(&self) -> Option<ModeTreeItemRef> {
        self.items.last().cloned()
    }

    pub(crate) fn position(&self, item: &ModeTreeItemRef) -> usize {
        self.items
            .iter()
            .position(|candidate| Rc::ptr_eq(candidate, item))
            .expect("mode tree item belongs to its parent list")
    }

    pub(crate) fn next(&self, item: &ModeTreeItemRef) -> Option<ModeTreeItemRef> {
        self.items.get(self.position(item) + 1).cloned()
    }

    pub(crate) fn previous(&self, item: &ModeTreeItemRef) -> Option<ModeTreeItemRef> {
        self.position(item)
            .checked_sub(1)
            .and_then(|position| self.items.get(position))
            .cloned()
    }
}

pub type mode_tree_help_cb = Option<fn() -> mode_tree_help_info>;

pub type mode_tree_sort_cb = Option<fn(&mut sort_criteria)>;

pub type mode_tree_swap_cb =
    Option<Box<dyn FnMut(&ModeTreeItemData, &ModeTreeItemData, &mut sort_criteria) -> bool>>;

pub type mode_tree_key_cb = Option<Box<dyn FnMut(&ModeTreeItemData, u_int) -> key_code>>;

pub type mode_tree_height_cb = Option<Box<dyn FnMut(u_int) -> u_int>>;

pub type mode_tree_menu_cb = Option<Box<dyn FnMut(&Rc<UnsafeCell<client>>, key_code)>>;

pub type mode_tree_search_cb =
    Option<Box<dyn FnMut(&ModeTreeItemData, &std::ffi::CStr, bool) -> bool>>;

pub type mode_tree_draw_cb =
    Option<Box<dyn FnMut(&ModeTreeItemData, &mut screen_write_ctx, u_int, u_int)>>;

pub type mode_tree_build_cb = Option<
    Box<
        dyn FnMut(
            &mut sort_criteria,
            Option<uint64_t>,
            Option<&std::ffi::CStr>,
        ) -> Option<uint64_t>,
    >,
>;
