//! Authoritative option-table value domains.

use std::collections::BTreeMap;

use super::abi::{time_t, u_int};
use super::command::cmd_list;
use super::session::session;
use super::style::style;
use super::window::window;
pub type options_table_type = ::core::ffi::c_uint;
pub const OPTIONS_TABLE_NUMBER: options_table_type = 1;
pub const OPTIONS_TABLE_KEY: options_table_type = 2;
pub const OPTIONS_TABLE_COLOUR: options_table_type = 3;
pub const OPTIONS_TABLE_FLAG: options_table_type = 4;
pub const OPTIONS_TABLE_CHOICE: options_table_type = 5;
pub const OPTIONS_TABLE_COMMAND: options_table_type = 6;
pub const OPTIONS_TABLE_STRING: options_table_type = 0;

pub const OPTIONS_TABLE_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_WINDOW: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_IS_HOOK: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_SERVER: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_SESSION: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_PANE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_IS_ARRAY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_IS_STYLE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_IS_COLOUR: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_STATUS_FORMAT1: [::core::ffi::c_char; 1547] = unsafe {
    ::core::mem::transmute::<
        [u8; 1547],
        [::core::ffi::c_char; 1547],
    >(
        *b"#[align=left range=left #{E:status-left-style}]#[push-default]#{T;=/#{status-left-length}:status-left}#[pop-default]#[norange default]#[list=on align=#{status-justify}]#[list=left-marker]<#[list=right-marker]>#[list=on]#{W:#[range=window|#{window_index} #{E:window-status-style}#{?#{&&:#{window_last_flag},#{!=:#{E:window-status-last-style},default}}, #{E:window-status-last-style},}#{?#{&&:#{window_bell_flag},#{!=:#{E:window-status-bell-style},default}}, #{E:window-status-bell-style},#{?#{&&:#{||:#{window_activity_flag},#{window_silence_flag}},#{!=:#{E:window-status-activity-style},default}}, #{E:window-status-activity-style},}}]#[push-default]#{T:window-status-format}#[pop-default]#[norange default]#{?loop_last_flag,,#{E:window-status-separator}},#[range=window|#{window_index} list=focus #{?#{!=:#{E:window-status-current-style},default},#{E:window-status-current-style},#{E:window-status-style}}#{?#{&&:#{window_last_flag},#{!=:#{E:window-status-last-style},default}}, #{E:window-status-last-style},}#{?#{&&:#{window_bell_flag},#{!=:#{E:window-status-bell-style},default}}, #{E:window-status-bell-style},#{?#{&&:#{||:#{window_activity_flag},#{window_silence_flag}},#{!=:#{E:window-status-activity-style},default}}, #{E:window-status-activity-style},}}]#[push-default]#{T:window-status-current-format}#[pop-default]#[norange list=on default]#{?loop_last_flag,,#{E:window-status-separator}}}#[nolist align=right range=right #{E:status-right-style}]#[push-default]#{T;=/#{status-right-length}:status-right}#[pop-default]#[norange default]\0",
    )
};
pub const OPTIONS_TABLE_STATUS_FORMAT2: [::core::ffi::c_char; 519] = unsafe {
    ::core::mem::transmute::<
        [u8; 519],
        [::core::ffi::c_char; 519],
    >(
        *b"#[align=left]#{R: ,#{n:#{session_name}}}P: #[norange default]#[list=on align=#{status-justify}]#[list=left-marker]<#[list=right-marker]>#[list=on]#{P:#[range=pane|#{pane_id} #{E:pane-status-style}]#[push-default]#{T:window-pane-status-format}#[pop-default]#[norange list=on default]  ,#[range=pane|#{pane_id} list=focus #{?#{!=:#{E:pane-status-current-style},default},#{E:pane-status-current-style},#{E:pane-status-style}}]#[push-default]#{T:window-pane-current-status-format}#[pop-default]#[norange list=on default] }\0",
    )
};
pub const OPTIONS_TABLE_STATUS_FORMAT3: [::core::ffi::c_char; 512] = unsafe {
    ::core::mem::transmute::<
        [u8; 512],
        [::core::ffi::c_char; 512],
    >(
        *b"#[align=left]#{R: ,#{n:#{session_name}}}S: #[norange default]#[list=on align=#{status-justify}]#[list=left-marker]<#[list=right-marker]>#[list=on]#{S:#[range=session|#{session_id} #{E:session-status-style}]#[push-default]#S#{session_alert}#[pop-default]#[norange list=on default]  ,#[range=session|#{session_id} list=focus #{?#{!=:#{E:session-status-current-style},default},#{E:session-status-current-style},#{E:session-status-style}}]#[push-default]#S*#{session_alert}#[pop-default]#[norange list=on default] }\0",
    )
};

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, size_of};

    #[test]
    fn option_table_domain_matches_translated_c_baseline() {
        assert_eq!(size_of::<options_table_type>(), 4);
        assert_eq!(align_of::<options_table_type>(), 4);
        assert_eq!(OPTIONS_TABLE_STRING, 0);
        assert_eq!(OPTIONS_TABLE_COMMAND, 6);
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct options {
    pub tree: *mut options_storage,
    pub parent: *mut options,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_array_item {
    pub key: *mut ::core::ffi::c_char,
    pub value: options_value,
    pub entry: options_array_item_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_entry {
    pub owner: *mut options,
    pub name: *const ::core::ffi::c_char,
    pub tableentry: *const options_table_entry,
    pub value: options_value,
    pub cached: ::core::ffi::c_int,
    pub style: style,
    pub monitor_data: *mut ::core::ffi::c_void,
    pub fire_count: u_int,
    pub fire_time: time_t,
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
pub struct options_table_entry {
    pub name: *const ::core::ffi::c_char,
    pub alternative_name: *const ::core::ffi::c_char,
    pub type_0: options_table_type,
    pub scope: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
    pub minimum: u_int,
    pub maximum: u_int,
    pub choices: *mut *const ::core::ffi::c_char,
    pub default_str: *const ::core::ffi::c_char,
    pub default_num: ::core::ffi::c_longlong,
    pub default_arr: *mut *const ::core::ffi::c_char,
    pub separator: *const ::core::ffi::c_char,
    pub pattern: *const ::core::ffi::c_char,
    pub text: *const ::core::ffi::c_char,
    pub unit: *const ::core::ffi::c_char,
}

/// Rust-owned name index; entries and their values retain their C allocation contract.
#[derive(Default)]
pub struct options_storage {
    pub(crate) entries: BTreeMap<Vec<u8>, *mut options_entry>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_array_item_entry {
    pub rbe_left: *mut options_array_item,
    pub rbe_right: *mut options_array_item,
    pub rbe_parent: *mut options_array_item,
    pub rbe_color: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_name_map {
    pub from: *const ::core::ffi::c_char,
    pub to: *const ::core::ffi::c_char,
}
