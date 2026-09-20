//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_BASENAME;
    records.push(format!(
        "src/format.rs::FORMAT_BASENAME {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_CHARACTER;
    records.push(format!(
        "src/format.rs::FORMAT_CHARACTER {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_CLIENTS;
    records.push(format!(
        "src/format.rs::FORMAT_CLIENTS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_CLIENT_ENVIRON;
    records.push(format!(
        "src/format.rs::FORMAT_CLIENT_ENVIRON {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_CLIENT_TERMCAP;
    records.push(format!(
        "src/format.rs::FORMAT_CLIENT_TERMCAP {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_CLIENT_TERMFEAT;
    records.push(format!(
        "src/format.rs::FORMAT_CLIENT_TERMFEAT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_COLOUR;
    records.push(format!(
        "src/format.rs::FORMAT_COLOUR {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_COLOUR_ESC_BG;
    records.push(format!(
        "src/format.rs::FORMAT_COLOUR_ESC_BG {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_COLOUR_ESC_FG;
    records.push(format!(
        "src/format.rs::FORMAT_COLOUR_ESC_FG {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_ulonglong = hmux2::src::format::FORMAT_CYCLE;
    records.push(format!(
        "src/format.rs::FORMAT_CYCLE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_ulonglong>(),
        align_of::<::core::ffi::c_ulonglong>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_CYCLE_PERIOD;
    records.push(format!(
        "src/format.rs::FORMAT_CYCLE_PERIOD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_ulonglong = hmux2::src::format::FORMAT_DIFFERENCE;
    records.push(format!(
        "src/format.rs::FORMAT_DIFFERENCE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_ulonglong>(),
        align_of::<::core::ffi::c_ulonglong>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_DIRNAME;
    records.push(format!(
        "src/format.rs::FORMAT_DIRNAME {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_ulonglong = hmux2::src::format::FORMAT_ENVIRON;
    records.push(format!(
        "src/format.rs::FORMAT_ENVIRON {:?} {} {}",
        v,
        size_of::<::core::ffi::c_ulonglong>(),
        align_of::<::core::ffi::c_ulonglong>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_EXPAND;
    records.push(format!(
        "src/format.rs::FORMAT_EXPAND {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_EXPANDTIME;
    records.push(format!(
        "src/format.rs::FORMAT_EXPANDTIME {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_EXPAND_NOCYCLE;
    records.push(format!(
        "src/format.rs::FORMAT_EXPAND_NOCYCLE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_EXPAND_NOJOBS;
    records.push(format!(
        "src/format.rs::FORMAT_EXPAND_NOJOBS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_EXPAND_TIME;
    records.push(format!(
        "src/format.rs::FORMAT_EXPAND_TIME {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_FORCE;
    records.push(format!(
        "src/format.rs::FORMAT_FORCE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::status::FORMAT_FORCE;
    records.push(format!(
        "src/status.rs::FORMAT_FORCE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_LENGTH;
    records.push(format!(
        "src/format.rs::FORMAT_LENGTH {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_LITERAL;
    records.push(format!(
        "src/format.rs::FORMAT_LITERAL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_LOOP_LIMIT;
    records.push(format!(
        "src/format.rs::FORMAT_LOOP_LIMIT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_MAX_PRECISION;
    records.push(format!(
        "src/format.rs::FORMAT_MAX_PRECISION {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_MAX_REPEAT;
    records.push(format!(
        "src/format.rs::FORMAT_MAX_REPEAT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_MAX_WIDTH;
    records.push(format!(
        "src/format.rs::FORMAT_MAX_WIDTH {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_parse::FORMAT_NOJOBS;
    records.push(format!(
        "src/cmd_parse.rs::FORMAT_NOJOBS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_wait_for::FORMAT_NOJOBS;
    records.push(format!(
        "src/cmd_wait_for.rs::FORMAT_NOJOBS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_NOJOBS;
    records.push(format!(
        "src/format.rs::FORMAT_NOJOBS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::hooks::FORMAT_NOJOBS;
    records.push(format!(
        "src/hooks.rs::FORMAT_NOJOBS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::monitor::FORMAT_NOJOBS;
    records.push(format!(
        "src/monitor.rs::FORMAT_NOJOBS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server_client::FORMAT_NOJOBS;
    records.push(format!(
        "src/server_client.rs::FORMAT_NOJOBS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::style::FORMAT_NOJOBS;
    records.push(format!(
        "src/style.rs::FORMAT_NOJOBS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tty::FORMAT_NOJOBS;
    records.push(format!(
        "src/tty.rs::FORMAT_NOJOBS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_border::FORMAT_NOJOBS;
    records.push(format!(
        "src/window_border.rs::FORMAT_NOJOBS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_display_message::FORMAT_NONE;
    records.push(format!(
        "src/cmd_display_message.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_kill_pane::FORMAT_NONE;
    records.push(format!(
        "src/cmd_kill_pane.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_kill_session::FORMAT_NONE;
    records.push(format!(
        "src/cmd_kill_session.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_kill_window::FORMAT_NONE;
    records.push(format!(
        "src/cmd_kill_window.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_list_buffers::FORMAT_NONE;
    records.push(format!(
        "src/cmd_list_buffers.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_list_clients::FORMAT_NONE;
    records.push(format!(
        "src/cmd_list_clients.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_list_commands::FORMAT_NONE;
    records.push(format!(
        "src/cmd_list_commands.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_list_keys::FORMAT_NONE;
    records.push(format!(
        "src/cmd_list_keys.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_list_panes::FORMAT_NONE;
    records.push(format!(
        "src/cmd_list_panes.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_list_sessions::FORMAT_NONE;
    records.push(format!(
        "src/cmd_list_sessions.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_list_windows::FORMAT_NONE;
    records.push(format!(
        "src/cmd_list_windows.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_parse::FORMAT_NONE;
    records.push(format!(
        "src/cmd_parse.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_pipe_pane::FORMAT_NONE;
    records.push(format!(
        "src/cmd_pipe_pane.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_queue::FORMAT_NONE;
    records.push(format!(
        "src/cmd_queue.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_wait_for::FORMAT_NONE;
    records.push(format!(
        "src/cmd_wait_for.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::control_notify::FORMAT_NONE;
    records.push(format!(
        "src/control_notify.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_NONE;
    records.push(format!(
        "src/format.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::hooks::FORMAT_NONE;
    records.push(format!(
        "src/hooks.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server_client::FORMAT_NONE;
    records.push(format!(
        "src/server_client.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::status::FORMAT_NONE;
    records.push(format!(
        "src/status.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_buffer::FORMAT_NONE;
    records.push(format!(
        "src/window_buffer.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_client::FORMAT_NONE;
    records.push(format!(
        "src/window_client.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_switch::FORMAT_NONE;
    records.push(format!(
        "src/window_switch.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_tree::FORMAT_NONE;
    records.push(format!(
        "src/window_tree.rs::FORMAT_NONE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_NOT;
    records.push(format!(
        "src/format.rs::FORMAT_NOT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_NOT_NOT;
    records.push(format!(
        "src/format.rs::FORMAT_NOT_NOT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_OPTIONS;
    records.push(format!(
        "src/format.rs::FORMAT_OPTIONS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_uint = hmux2::src::format::FORMAT_PANE;
    records.push(format!(
        "src/format.rs::FORMAT_PANE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_uint>(),
        align_of::<::core::ffi::c_uint>()
    ));
    let v: ::core::ffi::c_uint = hmux2::src::tty::FORMAT_PANE;
    records.push(format!(
        "src/tty.rs::FORMAT_PANE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_uint>(),
        align_of::<::core::ffi::c_uint>()
    ));
    let v: ::core::ffi::c_uint = hmux2::src::window_border::FORMAT_PANE;
    records.push(format!(
        "src/window_border.rs::FORMAT_PANE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_uint>(),
        align_of::<::core::ffi::c_uint>()
    ));
    let v: ::core::ffi::c_uint = hmux2::src::window_tree::FORMAT_PANE;
    records.push(format!(
        "src/window_tree.rs::FORMAT_PANE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_uint>(),
        align_of::<::core::ffi::c_uint>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_PANES;
    records.push(format!(
        "src/format.rs::FORMAT_PANES {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_PRETTY;
    records.push(format!(
        "src/format.rs::FORMAT_PRETTY {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_QUOTE_ARGUMENTS;
    records.push(format!(
        "src/format.rs::FORMAT_QUOTE_ARGUMENTS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_QUOTE_SHELL;
    records.push(format!(
        "src/format.rs::FORMAT_QUOTE_SHELL {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_QUOTE_SHELL_SQ;
    records.push(format!(
        "src/format.rs::FORMAT_QUOTE_SHELL_SQ {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_QUOTE_STYLE;
    records.push(format!(
        "src/format.rs::FORMAT_QUOTE_STYLE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_RELATIVE;
    records.push(format!(
        "src/format.rs::FORMAT_RELATIVE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_REPEAT;
    records.push(format!(
        "src/format.rs::FORMAT_REPEAT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_SESSIONS;
    records.push(format!(
        "src/format.rs::FORMAT_SESSIONS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_SESSION_NAME;
    records.push(format!(
        "src/format.rs::FORMAT_SESSION_NAME {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_STATUS;
    records.push(format!(
        "src/format.rs::FORMAT_STATUS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::status::FORMAT_STATUS;
    records.push(format!(
        "src/status.rs::FORMAT_STATUS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window_border::FORMAT_STATUS;
    records.push(format!(
        "src/window_border.rs::FORMAT_STATUS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: hmux2::src::format::format_table_type = hmux2::src::format::FORMAT_TABLE_STRING;
    records.push(format!(
        "src/format.rs::FORMAT_TABLE_STRING {:?} {} {}",
        v,
        size_of::<hmux2::src::format::format_table_type>(),
        align_of::<hmux2::src::format::format_table_type>()
    ));
    let v: hmux2::src::format::format_table_type = hmux2::src::format::FORMAT_TABLE_TIME;
    records.push(format!(
        "src/format.rs::FORMAT_TABLE_TIME {:?} {} {}",
        v,
        size_of::<hmux2::src::format::format_table_type>(),
        align_of::<hmux2::src::format::format_table_type>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_TIMESTRING;
    records.push(format!(
        "src/format.rs::FORMAT_TIMESTRING {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_TIME_LIMIT;
    records.push(format!(
        "src/format.rs::FORMAT_TIME_LIMIT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_TIME_LOOP_CHECK;
    records.push(format!(
        "src/format.rs::FORMAT_TIME_LOOP_CHECK {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: hmux2::src::format::format_type = hmux2::src::format::FORMAT_TYPE_PANE;
    records.push(format!(
        "src/format.rs::FORMAT_TYPE_PANE {:?} {} {}",
        v,
        size_of::<hmux2::src::format::format_type>(),
        align_of::<hmux2::src::format::format_type>()
    ));
    let v: hmux2::src::format::format_type = hmux2::src::format::FORMAT_TYPE_SESSION;
    records.push(format!(
        "src/format.rs::FORMAT_TYPE_SESSION {:?} {} {}",
        v,
        size_of::<hmux2::src::format::format_type>(),
        align_of::<hmux2::src::format::format_type>()
    ));
    let v: hmux2::src::format::format_type = hmux2::src::format::FORMAT_TYPE_UNKNOWN;
    records.push(format!(
        "src/format.rs::FORMAT_TYPE_UNKNOWN {:?} {} {}",
        v,
        size_of::<hmux2::src::format::format_type>(),
        align_of::<hmux2::src::format::format_type>()
    ));
    let v: hmux2::src::format::format_type = hmux2::src::format::FORMAT_TYPE_WINDOW;
    records.push(format!(
        "src/format.rs::FORMAT_TYPE_WINDOW {:?} {} {}",
        v,
        size_of::<hmux2::src::format::format_type>(),
        align_of::<hmux2::src::format::format_type>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_display_message::FORMAT_VERBOSE;
    records.push(format!(
        "src/cmd_display_message.rs::FORMAT_VERBOSE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_VERBOSE;
    records.push(format!(
        "src/format.rs::FORMAT_VERBOSE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_WIDTH;
    records.push(format!(
        "src/format.rs::FORMAT_WIDTH {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_uint = hmux2::src::format::FORMAT_WINDOW;
    records.push(format!(
        "src/format.rs::FORMAT_WINDOW {:?} {} {}",
        v,
        size_of::<::core::ffi::c_uint>(),
        align_of::<::core::ffi::c_uint>()
    ));
    let v: ::core::ffi::c_uint = hmux2::src::names::FORMAT_WINDOW;
    records.push(format!(
        "src/names.rs::FORMAT_WINDOW {:?} {} {}",
        v,
        size_of::<::core::ffi::c_uint>(),
        align_of::<::core::ffi::c_uint>()
    ));
    let v: ::core::ffi::c_uint = hmux2::src::window_border::FORMAT_WINDOW;
    records.push(format!(
        "src/window_border.rs::FORMAT_WINDOW {:?} {} {}",
        v,
        size_of::<::core::ffi::c_uint>(),
        align_of::<::core::ffi::c_uint>()
    ));
    let v: ::core::ffi::c_uint = hmux2::src::window_tree::FORMAT_WINDOW;
    records.push(format!(
        "src/window_tree.rs::FORMAT_WINDOW {:?} {} {}",
        v,
        size_of::<::core::ffi::c_uint>(),
        align_of::<::core::ffi::c_uint>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_WINDOWS;
    records.push(format!(
        "src/format.rs::FORMAT_WINDOWS {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::format::FORMAT_WINDOW_NAME;
    records.push(format!(
        "src/format.rs::FORMAT_WINDOW_NAME {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-format.txt"));
}
