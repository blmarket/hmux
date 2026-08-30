//! The client overlays: the menu and the popup drawn over whatever a client
//! is showing.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod menu;
mod popup;

pub use menu::{
    menu_add_item, menu_add_items, menu_check_cb, menu_create, menu_data, menu_display,
    menu_draw_cb, menu_key_cb, menu_mode_cb,
};
pub use popup::{
    PopupDataRef, PopupDataWeak, popup_data, popup_display, popup_editor, popup_modify,
    popup_present, popup_write,
};

#[cfg(test)]
pub(crate) use menu::{
    BOX_LINES_DEFAULT, BOX_LINES_DOUBLE, BOX_LINES_HEAVY, BOX_LINES_NONE, BOX_LINES_PADDED,
    BOX_LINES_ROUNDED, BOX_LINES_SIMPLE, BOX_LINES_SINGLE, MENU_NOMOUSE, MENU_STAYOPEN, MENU_TAB,
};
pub(crate) use menu::{menu_free_box, menu_resize_cb};
#[cfg(test)]
pub(crate) use popup::*;
pub(crate) use popup::{
    popup_check_cb, popup_draw_cb, popup_free_box, popup_key_cb, popup_mode_cb, popup_resize_cb,
};
