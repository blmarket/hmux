//! The client overlays: the menu and the popup drawn over whatever a client
//! is showing.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod menu;
mod popup;

pub use menu::{
    MenuDataRef, menu_add_item, menu_add_items, menu_check_cb, menu_create, menu_data,
    menu_display, menu_mode_cb,
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
pub(crate) use menu::{menu_add_item_for_client, menu_resize_cb};
#[cfg(test)]
pub(crate) use popup::*;
pub(crate) use popup::{popup_check_cb, popup_mode_cb};

pub(crate) use popup::{
    popup_display_for_client, popup_modify_for_client, popup_present_for_client,
};

pub(crate) use menu::menu_display_for_client;

mod pane_numbers;
pub(crate) use pane_numbers::draw_pane_numbers;
