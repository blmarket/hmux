use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const AUTHORITATIVE: &[(&str, &str, &str)] = &[
    // ABI
    ("src/shared/abi.rs", "type", "__u_char"),
    ("src/shared/abi.rs", "type", "__u_short"),
    ("src/shared/abi.rs", "type", "__u_int"),
    ("src/shared/abi.rs", "type", "__uint8_t"),
    ("src/shared/abi.rs", "type", "__uint64_t"),
    ("src/shared/abi.rs", "type", "__pid_t"),
    ("src/shared/abi.rs", "type", "__time_t"),
    ("src/shared/abi.rs", "type", "__suseconds_t"),
    ("src/shared/abi.rs", "type", "u_char"),
    ("src/shared/abi.rs", "type", "u_short"),
    ("src/shared/abi.rs", "type", "u_int"),
    ("src/shared/abi.rs", "type", "pid_t"),
    ("src/shared/abi.rs", "type", "time_t"),
    ("src/shared/abi.rs", "type", "size_t"),
    ("src/shared/abi.rs", "type", "uint8_t"),
    ("src/shared/abi.rs", "type", "uint64_t"),
    ("src/shared/abi.rs", "type", "bitstr_t"),
    ("src/shared/abi.rs", "type", "cc_t"),
    ("src/shared/abi.rs", "type", "speed_t"),
    ("src/shared/abi.rs", "type", "tcflag_t"),
    ("src/shared/abi.rs", "struct", "timeval"),
    // Message, layout, display, and terminal scalar domains
    ("src/shared/message.rs", "type", "msgtype"),
    ("src/shared/layout.rs", "type", "layout_type"),
    ("src/shared/layout.rs", "type", "box_lines"),
    ("src/shared/layout.rs", "type", "pane_lines"),
    ("src/shared/display.rs", "struct", "progress_bar"),
    ("src/shared/display.rs", "type", "progress_bar_state"),
    ("src/shared/display.rs", "type", "screen_cursor_style"),
    ("src/shared/tty.rs", "type", "tty_code_code"),
    ("src/shared/terminal.rs", "struct", "termios"),
    ("src/shared/terminal.rs", "union", "termios_input_speed"),
    ("src/shared/terminal.rs", "union", "termios_output_speed"),
    // Client status domains
    ("src/shared/client.rs", "type", "client_exit_type"),
    ("src/shared/client.rs", "type", "client_exit_reason"),
    // Command, argument, option, prompt, and sort domains
    ("src/shared/command.rs", "type", "cmd_parse_status"),
    ("src/shared/arguments.rs", "type", "args_type"),
    ("src/shared/arguments.rs", "type", "args_parse_type"),
    ("src/shared/options.rs", "type", "options_table_type"),
    ("src/shared/prompt.rs", "type", "prompt_type"),
    ("src/shared/prompt.rs", "type", "prompt_key_result"),
    ("src/shared/sort.rs", "type", "sort_order"),
    // Grid
    ("src/shared/grid.rs", "struct", "utf8_data"),
    ("src/shared/grid.rs", "struct", "grid_cell"),
    ("src/shared/grid.rs", "struct", "grid"),
    ("src/shared/grid.rs", "struct", "grid_line"),
    ("src/shared/grid.rs", "struct", "osc133_data"),
    ("src/shared/grid.rs", "struct", "grid_extd_entry"),
    ("src/shared/grid.rs", "type", "utf8_char"),
    ("src/shared/grid.rs", "union", "grid_cell_entry_storage"),
    ("src/shared/grid.rs", "struct", "grid_cell_entry_data"),
    ("src/shared/grid.rs", "struct", "grid_cell_entry"),
    // Style
    ("src/shared/style.rs", "struct", "style"),
    ("src/shared/style.rs", "type", "style_default_type"),
    ("src/shared/style.rs", "type", "style_range_type"),
    ("src/shared/style.rs", "type", "style_list"),
    ("src/shared/style.rs", "type", "style_align"),
    ("src/shared/style.rs", "struct", "style_line_entry"),
    ("src/shared/style.rs", "struct", "style_ranges"),
    ("src/shared/style.rs", "struct", "style_range"),
    ("src/shared/style.rs", "struct", "style_range_entry"),
    // Command domains
    ("src/shared/command.rs", "type", "cmd_retval"),
    ("src/shared/command.rs", "type", "cmd_find_type"),
    // Key domains
    ("src/shared/key.rs", "type", "key_code"),
    ("src/shared/key.rs", "type", "key_code_type"),
    ("src/shared/key.rs", "type", "key_code_mouse_location"),
    // Event ABI.  The opaque declarations intentionally stay opaque.
    ("src/shared/event.rs", "opaque", "event_base"),
    ("src/shared/event.rs", "opaque", "evbuffer"),
    ("src/shared/event.rs", "opaque", "bufferevent_ops"),
    ("src/shared/event.rs", "struct", "event"),
    ("src/shared/event.rs", "union", "event_io_or_signal"),
    ("src/shared/event.rs", "struct", "event_signal"),
    ("src/shared/event.rs", "struct", "event_signal_entry"),
    ("src/shared/event.rs", "struct", "event_io"),
    ("src/shared/event.rs", "struct", "event_io_entry"),
    ("src/shared/event.rs", "union", "event_timeout_pos"),
    ("src/shared/event.rs", "struct", "event_timeout_entry"),
    ("src/shared/event.rs", "struct", "event_callback"),
    ("src/shared/event.rs", "union", "event_callback_union"),
    ("src/shared/event.rs", "struct", "event_callback_entry"),
    ("src/shared/event.rs", "struct", "bufferevent"),
    ("src/shared/event.rs", "type", "bufferevent_event_cb"),
    ("src/shared/event.rs", "type", "bufferevent_data_cb"),
    ("src/shared/event.rs", "struct", "event_watermark"),
    // UTF-8 state
    ("src/shared/utf8.rs", "type", "utf8_state"),
    // Colour
    ("src/shared/colour.rs", "type", "client_theme"),
    ("src/shared/colour.rs", "struct", "colour_palette"),
];

const AUTHORITATIVE_CONSTANTS: &[(&str, &str, &str)] = &[
    ("src/shared/grid.rs", "const", "GRID_ATTR_BRIGHT"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_DIM"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_UNDERSCORE"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_BLINK"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_REVERSE"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_HIDDEN"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_ITALICS"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_CHARSET"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_STRIKETHROUGH"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_UNDERSCORE_2"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_UNDERSCORE_3"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_UNDERSCORE_4"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_UNDERSCORE_5"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_OVERLINE"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_NOATTR"),
    ("src/shared/grid.rs", "const", "GRID_ATTR_ALL_UNDERSCORE"),
    ("src/shared/grid.rs", "const", "GRID_FLAG_FG256"),
    ("src/shared/grid.rs", "const", "GRID_FLAG_BG256"),
    ("src/shared/grid.rs", "const", "GRID_FLAG_PADDING"),
    ("src/shared/grid.rs", "const", "GRID_FLAG_EXTENDED"),
    ("src/shared/grid.rs", "const", "GRID_FLAG_SELECTED"),
    ("src/shared/grid.rs", "const", "GRID_FLAG_NOPALETTE"),
    ("src/shared/grid.rs", "const", "GRID_FLAG_CLEARED"),
    ("src/shared/grid.rs", "const", "GRID_FLAG_TAB"),
    ("src/shared/grid.rs", "const", "GRID_LINE_WRAPPED"),
    ("src/shared/grid.rs", "const", "GRID_LINE_EXTENDED"),
    ("src/shared/grid.rs", "const", "GRID_LINE_DEAD"),
    ("src/shared/grid.rs", "const", "GRID_LINE_START_PROMPT"),
    ("src/shared/grid.rs", "const", "GRID_LINE_SECOND_PROMPT"),
    ("src/shared/grid.rs", "const", "GRID_LINE_START_COMMAND"),
    ("src/shared/grid.rs", "const", "GRID_LINE_START_OUTPUT"),
    ("src/shared/grid.rs", "const", "GRID_LINE_END_OUTPUT"),
    ("src/shared/grid.rs", "const", "GRID_LINE_HYPERLINK"),
    ("src/shared/grid.rs", "const", "GRID_LINE_OSC133_FLAGS"),
    ("src/shared/grid.rs", "const", "GRID_STRING_WITH_SEQUENCES"),
    (
        "src/shared/grid.rs",
        "const",
        "GRID_STRING_ESCAPE_SEQUENCES",
    ),
    ("src/shared/grid.rs", "const", "GRID_STRING_TRIM_SPACES"),
    ("src/shared/grid.rs", "const", "GRID_STRING_EMPTY_CELLS"),
    ("src/shared/grid.rs", "const", "GRID_HISTORY"),
    ("src/shared/style.rs", "const", "STYLE_DEFAULT_SET"),
    ("src/shared/style.rs", "const", "STYLE_DEFAULT_POP"),
    ("src/shared/style.rs", "const", "STYLE_DEFAULT_PUSH"),
    ("src/shared/style.rs", "const", "STYLE_DEFAULT_BASE"),
    ("src/shared/style.rs", "const", "STYLE_RANGE_CONTROL"),
    ("src/shared/style.rs", "const", "STYLE_RANGE_USER"),
    ("src/shared/style.rs", "const", "STYLE_RANGE_SESSION"),
    ("src/shared/style.rs", "const", "STYLE_RANGE_WINDOW"),
    ("src/shared/style.rs", "const", "STYLE_RANGE_PANE"),
    ("src/shared/style.rs", "const", "STYLE_RANGE_RIGHT"),
    ("src/shared/style.rs", "const", "STYLE_RANGE_LEFT"),
    ("src/shared/style.rs", "const", "STYLE_RANGE_NONE"),
    ("src/shared/style.rs", "const", "STYLE_LIST_RIGHT_MARKER"),
    ("src/shared/style.rs", "const", "STYLE_LIST_LEFT_MARKER"),
    ("src/shared/style.rs", "const", "STYLE_LIST_FOCUS"),
    ("src/shared/style.rs", "const", "STYLE_LIST_ON"),
    ("src/shared/style.rs", "const", "STYLE_LIST_OFF"),
    (
        "src/shared/style.rs",
        "const",
        "STYLE_ALIGN_ABSOLUTE_CENTRE",
    ),
    ("src/shared/style.rs", "const", "STYLE_ALIGN_RIGHT"),
    ("src/shared/style.rs", "const", "STYLE_ALIGN_CENTRE"),
    ("src/shared/style.rs", "const", "STYLE_ALIGN_LEFT"),
    ("src/shared/style.rs", "const", "STYLE_ALIGN_DEFAULT"),
    ("src/shared/command.rs", "const", "CMD_RETURN_STOP"),
    ("src/shared/command.rs", "const", "CMD_RETURN_WAIT"),
    ("src/shared/command.rs", "const", "CMD_RETURN_NORMAL"),
    ("src/shared/command.rs", "const", "CMD_RETURN_ERROR"),
    ("src/shared/command.rs", "const", "CMD_FIND_SESSION"),
    ("src/shared/command.rs", "const", "CMD_FIND_WINDOW"),
    ("src/shared/command.rs", "const", "CMD_FIND_PANE"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_NOTYPE"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_TRIPLECLICK"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_DOUBLECLICK"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_SECONDCLICK"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_WHEELUP"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_WHEELDOWN"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_MOUSEDRAGEND"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_MOUSEDRAG"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_MOUSEUP"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_MOUSEDOWN"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_MOUSEMOVE"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_FUNCTION"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_USER"),
    ("src/shared/key.rs", "const", "KEYC_TYPE_UNICODE"),
    ("src/shared/utf8.rs", "const", "UTF8_ERROR"),
    ("src/shared/utf8.rs", "const", "UTF8_DONE"),
    ("src/shared/utf8.rs", "const", "UTF8_MORE"),
    ("src/shared/colour.rs", "const", "THEME_DARK"),
    ("src/shared/colour.rs", "const", "THEME_LIGHT"),
    ("src/shared/colour.rs", "const", "THEME_UNKNOWN"),
    ("src/shared/abi.rs", "const", "NULL"),
    ("src/shared/command.rs", "const", "CMD_PARSE_SUCCESS"),
    ("src/shared/command.rs", "const", "CMD_PARSE_ERROR"),
    ("src/shared/arguments.rs", "const", "ARGS_COMMANDS"),
    ("src/shared/arguments.rs", "const", "ARGS_STRING"),
    ("src/shared/arguments.rs", "const", "ARGS_NONE"),
    ("src/shared/arguments.rs", "const", "ARGS_PARSE_COMMANDS"),
    (
        "src/shared/arguments.rs",
        "const",
        "ARGS_PARSE_COMMANDS_OR_STRING",
    ),
    ("src/shared/arguments.rs", "const", "ARGS_PARSE_STRING"),
    ("src/shared/arguments.rs", "const", "ARGS_PARSE_INVALID"),
    ("src/shared/options.rs", "const", "OPTIONS_TABLE_NUMBER"),
    ("src/shared/options.rs", "const", "OPTIONS_TABLE_KEY"),
    ("src/shared/options.rs", "const", "OPTIONS_TABLE_COLOUR"),
    ("src/shared/options.rs", "const", "OPTIONS_TABLE_FLAG"),
    ("src/shared/options.rs", "const", "OPTIONS_TABLE_CHOICE"),
    ("src/shared/options.rs", "const", "OPTIONS_TABLE_COMMAND"),
    ("src/shared/options.rs", "const", "OPTIONS_TABLE_STRING"),
    ("src/shared/prompt.rs", "const", "PROMPT_TYPE_COMMAND"),
    ("src/shared/prompt.rs", "const", "PROMPT_TYPE_INVALID"),
    ("src/shared/prompt.rs", "const", "PROMPT_TYPE_SEARCH"),
    ("src/shared/prompt.rs", "const", "PROMPT_KEY_NOT_HANDLED"),
    ("src/shared/prompt.rs", "const", "PROMPT_KEY_HANDLED"),
    ("src/shared/prompt.rs", "const", "PROMPT_KEY_CLOSE"),
    ("src/shared/prompt.rs", "const", "PROMPT_KEY_MOVE"),
    ("src/shared/sort.rs", "const", "SORT_END"),
    ("src/shared/sort.rs", "const", "SORT_Z"),
    ("src/shared/sort.rs", "const", "SORT_SIZE"),
    ("src/shared/sort.rs", "const", "SORT_ORDER"),
    ("src/shared/sort.rs", "const", "SORT_NAME"),
    ("src/shared/sort.rs", "const", "SORT_MODIFIER"),
    ("src/shared/sort.rs", "const", "SORT_INDEX"),
    ("src/shared/sort.rs", "const", "SORT_CREATION"),
    ("src/shared/sort.rs", "const", "SORT_ACTIVITY"),
    ("src/shared/layout.rs", "const", "LAYOUT_WINDOWPANE"),
    ("src/shared/layout.rs", "const", "LAYOUT_TOPBOTTOM"),
    ("src/shared/layout.rs", "const", "LAYOUT_LEFTRIGHT"),
    ("src/shared/layout.rs", "const", "LAYOUT_CELL_FLOATING"),
    ("src/shared/layout.rs", "const", "LAYOUT_CUSTOM_OLD_FORMAT"),
    ("src/shared/layout.rs", "const", "LAYOUT_V1_MAX_DEPTH"),
    ("src/shared/display.rs", "const", "PROGRESS_BAR_PAUSED"),
    ("src/shared/display.rs", "const", "PROGRESS_BAR_INDETERMINATE"),
    ("src/shared/display.rs", "const", "PROGRESS_BAR_ERROR"),
    ("src/shared/display.rs", "const", "PROGRESS_BAR_NORMAL"),
    ("src/shared/display.rs", "const", "PROGRESS_BAR_HIDDEN"),
    ("src/shared/display.rs", "const", "SCREEN_CURSOR_BAR"),
    ("src/shared/display.rs", "const", "SCREEN_CURSOR_UNDERLINE"),
    ("src/shared/display.rs", "const", "SCREEN_CURSOR_BLOCK"),
    ("src/shared/display.rs", "const", "SCREEN_CURSOR_DEFAULT"),
    ("src/shared/client.rs", "const", "CLIENT_EXIT_DETACH"),
    ("src/shared/client.rs", "const", "CLIENT_EXIT_SHUTDOWN"),
    ("src/shared/client.rs", "const", "CLIENT_EXIT_RETURN"),
    ("src/shared/client.rs", "const", "CLIENT_EXIT_MESSAGE_PROVIDED"),
    ("src/shared/client.rs", "const", "CLIENT_EXIT_SERVER_EXITED"),
    ("src/shared/client.rs", "const", "CLIENT_EXIT_EXITED"),
    ("src/shared/client.rs", "const", "CLIENT_EXIT_LOST_SERVER"),
    ("src/shared/client.rs", "const", "CLIENT_EXIT_TERMINATED"),
    ("src/shared/client.rs", "const", "CLIENT_EXIT_LOST_TTY"),
    ("src/shared/client.rs", "const", "CLIENT_EXIT_DETACHED_HUP"),
    ("src/shared/client.rs", "const", "CLIENT_EXIT_DETACHED"),
    ("src/shared/client.rs", "const", "CLIENT_EXIT_NONE"),
];

// Large generated constant families are guarded by prefix so new members cannot
// be copied back into a translation unit without failing this test.  Prefixes
// are intentionally narrow: other C constants with the same broad subject
// prefix remain private until their type identity has been audited.
const AUTHORITATIVE_CONST_PREFIXES: &[(&str, &str)] = &[
    ("src/shared/key.rs", "KEYC_"),
    ("src/shared/message.rs", "MSG_"),
    ("src/shared/tty.rs", "TTYC_"),
    ("src/shared/layout.rs", "LAYOUT_"),
    ("src/shared/layout.rs", "BOX_LINES_"),
    ("src/shared/layout.rs", "PANE_LINES_"),
    ("src/shared/display.rs", "PROGRESS_BAR_"),
    ("src/shared/display.rs", "SCREEN_CURSOR_"),
    ("src/shared/client.rs", "CLIENT_EXIT_"),
];

fn source_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("read source directory") {
        let path = entry.expect("read source entry").path();
        if path.is_dir() {
            source_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

fn is_declaration(line: &str, kind: &str, name: &str) -> bool {
    let line = line.trim_start();
    match kind {
        "type" => line.starts_with(&format!("pub type {name} =")),
        "opaque" => line == format!("pub type {name};"),
        "struct" => line == format!("pub struct {name} {{"),
        "union" => line == format!("pub union {name} {{"),
        "const" => line.starts_with(&format!("pub const {name}:")),
        _ => false,
    }
}

fn assert_authoritative_constant_prefixes(root: &Path) {
    let mut files = Vec::new();
    source_files(&root.join("src"), &mut files);

    let mut matches: BTreeMap<String, Vec<(PathBuf, usize)>> = BTreeMap::new();
    let mut prefix_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for file in &files {
        let text = fs::read_to_string(file).expect("read Rust source");
        for (line_number, line) in text.lines().enumerate() {
            let line = line.trim_start();
            let Some(rest) = line.strip_prefix("pub const ") else {
                continue;
            };
            let Some(name) = rest.split(':').next() else {
                continue;
            };
            for (_, prefix) in AUTHORITATIVE_CONST_PREFIXES {
                if name.starts_with(prefix) {
                    let relative = file.strip_prefix(root).unwrap().to_owned();
                    matches
                        .entry(name.to_owned())
                        .or_default()
                        .push((relative, line_number + 1));
                    *prefix_counts.entry(prefix).or_default() += 1;
                }
            }
        }
    }

    for (expected_file, prefix) in AUTHORITATIVE_CONST_PREFIXES {
        assert!(
            prefix_counts.get(prefix).copied().unwrap_or_default() > 0,
            "authoritative constant prefix {prefix} has no declarations"
        );
        for (name, locations) in matches.iter().filter(|(name, _)| name.starts_with(prefix)) {
            assert_eq!(
                locations.len(),
                1,
                "constant {name} must have exactly one source definition, found {locations:?}"
            );
            assert_eq!(
                locations[0].0.to_string_lossy(),
                *expected_file,
                "constant {name} moved outside its authoritative subject module"
            );
        }
    }
}

fn assert_authoritative(root: &Path, declarations: &[(&str, &str, &str)]) {
    let mut files = Vec::new();
    source_files(&root.join("src"), &mut files);

    for (expected_file, kind, name) in declarations {
        let mut matches = Vec::new();
        for file in &files {
            let text = fs::read_to_string(file).expect("read Rust source");
            for (line_number, line) in text.lines().enumerate() {
                if is_declaration(line, kind, name) {
                    matches.push((file.strip_prefix(root).unwrap().to_owned(), line_number + 1));
                }
            }
        }
        assert_eq!(
            matches.len(),
            1,
            "{kind} {name} must have exactly one source definition, found {matches:?}"
        );
        assert_eq!(
            matches[0].0.to_string_lossy(),
            *expected_file,
            "{kind} {name} moved outside its authoritative subject module"
        );
    }
}

#[test]
fn migrated_shared_declarations_are_unique_and_authoritative() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_authoritative(root, AUTHORITATIVE);
    assert_authoritative(root, AUTHORITATIVE_CONSTANTS);
    assert_authoritative_constant_prefixes(root);
}
