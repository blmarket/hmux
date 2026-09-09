use crate::cmd::CmdqStateRef;
use crate::cmd::cmd_parse_from_string;
use crate::cmd::{CmdqItemRef, cmdq_append};

use crate::fmt_args;
use crate::log::{fatalx, log_debug};
use crate::server::client_walk;
use crate::server::server_client_set_key_table;
use crate::text::{KeyStringCodec, RustKeyStringCodec};
pub use crate::types::*;
use ::core::ffi::CStr;
use ::std::cell::RefCell;
use ::std::collections::BTreeMap;
use ::std::ffi::CString;
/// One key binding: the key, the commands it runs, the note that describes
/// it, the table it belongs to and whether it repeats.
///
/// The fields are the binding's own. A binding is made and replaced through
/// `key_bindings_add`, read through the `key_binding_*` accessors, and
/// changed in place only by the three setters the customize mode needs.
#[derive(Clone)]
#[repr(C)]
pub struct key_binding {
    key: key_code,
    cmdlist: Option<CmdListRef>,
    note: Option<CString>,
    tablename: Option<CString>,
    flags: core::ffi::c_int,
}

/// The bindings of one table, by key, which is the order the C's `tree.h`
/// comparison put them in.
pub type key_bindings = std::collections::BTreeMap<key_code, Box<key_binding>>;

/// One key table: the bindings the user has made, the defaults they were
/// built on, and when a key was last looked up in it.
///
/// The fields are the table's own; callers walk bindings through
/// [`key_table::bindings`] and tables through [`key_tables`].
pub struct key_table {
    name: CString,
    activity_time: timeval,
    key_bindings: key_bindings,
    default_key_bindings: key_bindings,
}

impl key_table {
    /// An empty table under `name`.
    pub fn new(name: CString) -> key_table {
        key_table {
            name,
            activity_time: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            key_bindings: key_bindings::new(),
            default_key_bindings: key_bindings::new(),
        }
    }

    pub(crate) fn binding(&self, key: key_code) -> Option<&key_binding> {
        self.key_bindings.get(&key).map(Box::as_ref)
    }

    pub(crate) fn binding_mut(&mut self, key: key_code) -> Option<&mut key_binding> {
        self.key_bindings.get_mut(&key).map(Box::as_mut)
    }

    pub(crate) fn default_binding(&self, key: key_code) -> Option<&key_binding> {
        self.default_key_bindings.get(&key).map(Box::as_ref)
    }

    /// The table's bindings in key order.
    pub(crate) fn bindings(&self) -> impl Iterator<Item = &key_binding> {
        self.key_bindings.values().map(Box::as_ref)
    }
}

/// The key the binding is for, without the flag bits a lookup carries.
pub fn key_binding_key(bd: &key_binding) -> key_code {
    bd.key
}

/// The note written against the binding, or nothing when it has none.
pub fn key_binding_note(bd: &key_binding) -> Option<&CStr> {
    bd.note.as_deref()
}

/// The binding's flags, of which `KEY_BINDING_REPEAT` is the only one.
pub fn key_binding_flags(bd: &key_binding) -> core::ffi::c_int {
    bd.flags
}

/// The name of the table the binding was made in.
pub fn key_binding_tablename(bd: &key_binding) -> Option<&CStr> {
    bd.tablename.as_deref()
}

/// The commands the key runs, as a borrowed view for running or printing
/// them, or nothing before the binding has been given any.
pub fn key_binding_cmdlist(bd: &key_binding) -> Option<&CmdListRef> {
    bd.cmdlist.as_ref()
}

/// Whether two bindings run the same commands, which is how a binding is
/// found to be the default one already.
pub fn key_binding_same_cmdlist(bd: &key_binding, other: &key_binding) -> bool {
    bd.cmdlist == other.cmdlist
}

/// Points the binding at `cmdlist`, which is what the customize mode does
/// once the command a key runs has been retyped and parsed.
pub fn key_binding_set_cmdlist(bd: &mut key_binding, cmdlist: Option<CmdListRef>) {
    bd.cmdlist = cmdlist;
}

/// Writes `note` against the binding.
pub fn key_binding_set_note(bd: &mut key_binding, note: &CStr) {
    bd.note = Some(note.to_owned());
}

/// Turns the key's repeat on if it was off and off if it was on.
pub fn key_binding_toggle_repeat(bd: &mut key_binding) {
    bd.flags ^= KEY_BINDING_REPEAT;
}

pub use crate::consts::{
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CLIENT_READONLY, CMD_PARSE_ERROR,
    CMD_PARSE_SUCCESS, CMD_READONLY, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP,
    CMD_RETURN_WAIT, CMDQ_STATE_REPEAT, KEY_BINDING_REPEAT, KEYC_MASK_FLAGS, LAYOUT_LEFTRIGHT,
    LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC,
    MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD,
    MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS,
    MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT,
    MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR,
    MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN,
    MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION,
    MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, PANE_LINES_DOUBLE,
    PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES,
    PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL,
    PROGRESS_BAR_PAUSED, PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID,
    PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET, RB_BLACK, RB_NEGINF, RB_RED,
    SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN,
};

thread_local! {
    /// The live key-binding tables for this thread, by name. A client keeps a
    /// strong handle after the table is removed from this registry.
    pub(crate) static key_tables: RefCell<BTreeMap<CString, KeyTableRef>> = const {
        RefCell::new(BTreeMap::new())
    };
}
pub(crate) fn key_bindings_get_table_ref(
    name: &CStr,
    create: core::ffi::c_int,
) -> Option<KeyTableRef> {
    key_tables.with_borrow_mut(|tables| {
        if let Some(table) = tables.get(name) {
            return Some(table.clone());
        }
        if create == 0 {
            return None;
        }
        let name_cstr = name.to_owned();
        let table = KeyTableRef::new(key_table::new(name_cstr.clone()));
        tables.insert(name_cstr, table.clone());
        Some(table)
    })
}

pub(crate) fn key_bindings_get_table(name: &CStr, create: core::ffi::c_int) -> Option<KeyTableRef> {
    key_bindings_get_table_ref(name, create)
}

pub(crate) unsafe fn key_bindings_add(
    name: &CStr,
    key: key_code,
    note: Option<&CStr>,
    repeat: core::ffi::c_int,
    cmdlist: Option<CmdListRef>,
) {
    let table_ref = key_bindings_get_table_ref(name, 1).expect("key table creation requested");
    let key = key & !KEYC_MASK_FLAGS;
    let binding = {
        let mut table = table_ref.borrow_mut();
        if cmdlist.is_none() {
            if let Some(binding) = table.key_bindings.get_mut(&key) {
                if let Some(note) = note {
                    binding.note = Some(note.to_owned());
                }
                if repeat != 0 {
                    binding.flags |= KEY_BINDING_REPEAT;
                }
            }
            return;
        }
        let binding = key_binding {
            key,
            cmdlist,
            note: note.map(CStr::to_owned),
            tablename: Some(table.name.clone()),
            flags: if repeat != 0 { KEY_BINDING_REPEAT } else { 0 },
        };
        table.key_bindings.insert(key, Box::new(binding.clone()));
        binding
    };
    unsafe {
        let commands = (binding.cmdlist.as_ref().unwrap()).print(0);
        let key_name = RustKeyStringCodec.format_key(key, true);
        log_debug(
            c"%s: %#llx %s = %s",
            fmt_args![
                c"key_bindings_add",
                key,
                key_name.as_c_str(),
                commands.as_c_str()
            ],
        );
    }
}
pub unsafe fn key_bindings_remove(name: &CStr, key: key_code) {
    let key = key & !KEYC_MASK_FLAGS;
    let Some(table_ref) = key_bindings_get_table_ref(name, 0) else {
        return;
    };
    let (removed, empty) = {
        let mut table = table_ref.borrow_mut();
        let Some(binding) = table.key_bindings.remove(&key) else {
            return;
        };
        (
            binding,
            table.key_bindings.is_empty() && !table.has_defaults(),
        )
    };
    unsafe {
        let key_name = RustKeyStringCodec.format_key(removed.key, true);
        log_debug(
            c"%s: %#llx %s",
            fmt_args![c"key_bindings_remove", removed.key, key_name.as_c_str()],
        );
    }
    if empty {
        let removed_table = key_tables.with_borrow_mut(|tables| tables.remove(name));
        drop(removed_table);
    }
}
pub unsafe fn key_bindings_reset(name: &CStr, key: key_code) {
    let key = key & !KEYC_MASK_FLAGS;
    let Some(table_ref) = key_bindings_get_table_ref(name, 0) else {
        return;
    };
    {
        let mut table = table_ref.borrow_mut();
        if !table.key_bindings.contains_key(&key) {
            return;
        }
        if let Some(default) = table.default_key_bindings.get(&key).cloned() {
            let binding = table.key_bindings.get_mut(&key).unwrap();
            binding.cmdlist = default.cmdlist;
            binding.note = default.note;
            binding.flags = default.flags;
            return;
        }
    }
    unsafe { key_bindings_remove(name, key) };
}
pub unsafe fn key_bindings_remove_table(name: &CStr) {
    let table_ref = key_tables.with_borrow_mut(|tables| tables.remove(name));
    let Some(table_ref) = table_ref else {
        return;
    };
    for mut c in client_walk() {
        if { c.keytable() }.is_some_and(|table| table.ptr_eq(&table_ref)) {
            unsafe { server_client_set_key_table(c.as_client_mut(), None) };
        }
    }
}

fn key_bindings_init_done(_item: &CmdqItemRef) -> cmd_retval {
    {
        key_tables.with_borrow(|tables| {
            for table in tables.values() {
                table.take_defaults();
            }
        });
        CMD_RETURN_NORMAL
    }
}
pub fn key_bindings_init() {
    unsafe {
        static defaults: &[&CStr] = &[
        c"bind -N 'Send the prefix key' C-b { send-prefix }",
        c"bind -N 'Rotate through the panes' C-o { rotate-window }",
        c"bind -N 'Suspend the current client' C-z { suspend-client }",
        c"bind -N 'Select next layout' Space { next-layout }",
        c"bind -N 'Break pane to a new window' ! { break-pane }",
        c"bind -N 'Split window vertically' '\"' { split-window }",
        c"bind -N 'List all paste buffers' '#' { list-buffers }",
        c"bind -N 'Rename current session' '$' { command-prompt -I'#S' { rename-session -- '%%' } }",
        c"bind -N 'Split window horizontally' % { split-window -h }",
        c"bind -N 'Kill current window' & { confirm-before -p\"kill-window #W? (y/n)\" kill-window }",
        c"bind -N 'Prompt for window index to select' \"'\" { command-prompt -T window-target -pindex { select-window -t ':%%' } }",
        c"bind -N 'New floating pane' * { new-pane }",
        c"bind -N 'Switch to previous client' ( { switch-client -p }",
        c"bind -N 'Switch to next client' ) { switch-client -n }",
        c"bind -N 'Rename current window' , { command-prompt -I'#W' { rename-window -- '%%' } }",
        c"bind -N 'Delete the most recent paste buffer' - { delete-buffer }",
        c"bind -N 'Move the current window' . { command-prompt -T target { move-window -t '%%' } }",
        c"bind -N 'Describe key binding' '/' { command-prompt -kpkey  { list-keys -1N '%%' } }",
        c"bind -N 'Select window 0' 0 { select-window -t:=0 }",
        c"bind -N 'Select window 1' 1 { select-window -t:=1 }",
        c"bind -N 'Select window 2' 2 { select-window -t:=2 }",
        c"bind -N 'Select window 3' 3 { select-window -t:=3 }",
        c"bind -N 'Select window 4' 4 { select-window -t:=4 }",
        c"bind -N 'Select window 5' 5 { select-window -t:=5 }",
        c"bind -N 'Select window 6' 6 { select-window -t:=6 }",
        c"bind -N 'Select window 7' 7 { select-window -t:=7 }",
        c"bind -N 'Select window 8' 8 { select-window -t:=8 }",
        c"bind -N 'Select window 9' 9 { select-window -t:=9 }",
        c"bind -N 'Prompt for a command' : { command-prompt }",
        c"bind -N 'Move to the previously active pane' \\; { last-pane }",
        c"bind -N 'Choose a paste buffer from a list' = { choose-buffer -Z }",
        c"bind -N 'List key bindings' ? { list-keys -N }",
        c"bind -N 'Choose and detach a client from a list' D { choose-client -Z }",
        c"bind -N 'Spread panes out evenly' E { select-layout -E }",
        c"bind -N 'Switch to the last client' L { switch-client -l }",
        c"bind -N 'Clear the marked pane' M { select-pane -M }",
        c"bind -N 'Enter copy mode' [ { copy-mode }",
        c"bind -N 'Paste the most recent paste buffer' ] { paste-buffer -p }",
        c"bind -N 'Create a new window' c { new-window }",
        c"bind -N 'Detach the current client' d { detach-client }",
        c"bind -N 'Search for a pane' f { command-prompt { find-window -Z -- '%%' } }",
        c"bind -N 'Display window information' i { display-message }",
        c"bind -N 'Select the previously current window' l { last-window }",
        c"bind -N 'Toggle the marked pane' m { select-pane -m }",
        c"bind -N 'Select the next window' n { next-window }",
        c"bind -N 'Select the next pane' o { select-pane -t:.+ }",
        c"bind -N 'Customize options' C { customize-mode -Z }",
        c"bind -N 'Select the previous window' p { previous-window }",
        c"bind -N 'Display pane numbers' q { display-panes }",
        c"bind -N 'Redraw the current client' r { refresh-client }",
        c"bind -N 'Choose a session from a list' s { choose-tree -Zs }",
        c"bind -N 'Show a clock' t { clock-mode }",
        c"bind -N 'Choose a window from a list' w { choose-tree -Zw }",
        c"bind -N 'Kill the active pane' x { confirm-before -p\"kill-pane #P? (y/n)\" kill-pane }",
        c"bind -N 'Zoom the active pane' z { resize-pane -Z }",
        c"bind -N 'Swap the active pane with the pane above' '{' { swap-pane -U }",
        c"bind -N 'Swap the active pane with the pane below' '}' { swap-pane -D }",
        c"bind -N 'Show messages' '~' { show-messages }",
        c"bind -N 'Enter copy mode and scroll up' PPage { copy-mode -u }",
        c"bind -N 'Select the pane above the active pane' -r Up { select-pane -U }",
        c"bind -N 'Select the pane below the active pane' -r Down { select-pane -D }",
        c"bind -N 'Select the pane to the left of the active pane' -r Left { select-pane -L }",
        c"bind -N 'Select the pane to the right of the active pane' -r Right { select-pane -R }",
        c"bind -N 'Set the even-horizontal layout' M-1 { select-layout even-horizontal }",
        c"bind -N 'Set the even-vertical layout' M-2 { select-layout even-vertical }",
        c"bind -N 'Set the main-horizontal layout' M-3 { select-layout main-horizontal }",
        c"bind -N 'Set the main-vertical layout' M-4 { select-layout main-vertical }",
        c"bind -N 'Select the tiled layout' M-5 { select-layout tiled }",
        c"bind -N 'Set the main-horizontal-mirrored layout' M-6 { select-layout main-horizontal-mirrored }",
        c"bind -N 'Set the main-vertical-mirrored layout' M-7 { select-layout main-vertical-mirrored }",
        c"bind -N 'Select the next window with an alert' M-n { next-window -a }",
        c"bind -N 'Rotate through the panes in reverse' M-o { rotate-window -D }",
        c"bind -N 'Select the previous window with an alert' M-p { previous-window -a }",
        c"bind -N 'Move the visible part of the window up' -r S-Up { refresh-client -U 10 }",
        c"bind -N 'Move the visible part of the window down' -r S-Down { refresh-client -D 10 }",
        c"bind -N 'Move the visible part of the window left' -r S-Left { refresh-client -L 10 }",
        c"bind -N 'Move the visible part of the window right' -r S-Right { refresh-client -R 10 }",
        c"bind -N 'Reset so the visible part of the window follows the cursor' -r DC { refresh-client -c }",
        c"bind -N 'Resize the pane up by 5' -r M-Up { resize-pane -U 5 }",
        c"bind -N 'Resize the pane down by 5' -r M-Down { resize-pane -D 5 }",
        c"bind -N 'Resize the pane left by 5' -r M-Left { resize-pane -L 5 }",
        c"bind -N 'Resize the pane right by 5' -r M-Right { resize-pane -R 5 }",
        c"bind -N 'Resize the pane up' -r C-Up { resize-pane -U }",
        c"bind -N 'Resize the pane down' -r C-Down { resize-pane -D }",
        c"bind -N 'Resize the pane left' -r C-Left { resize-pane -L }",
        c"bind -N 'Resize the pane right' -r C-Right { resize-pane -R }",
        c"bind -N 'Display window menu' < { display-menu -xW -yW -T '#[align=centre]#{window_index}:#{window_name}'  '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' {swap-window -t:-1} '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' {swap-window -t:+1} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-window} '' 'Kill' 'X' {kill-window} 'Respawn' 'R' {respawn-window -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} 'Rename' 'n' {command-prompt -FI \"#W\" {rename-window -t '#{window_id}' -- '%%'}} '' 'New After' 'w' {new-window -a} 'New At End' 'W' {new-window} }",
        c"bind -N 'Display pane menu' > { display-menu -xP -yP -T '#[align=centre]#{pane_index} (#{pane_id})'  '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' {send -X history-top} '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>' {send -X history-bottom} '' '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' {paste-buffer} '' '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r' {if -F '#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}' 'copy-mode -t='; send -Xt= search-backward -- \"#{q:mouse_word}\"} '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y' {copy-mode -q; send-keys -l -- \"#{q:mouse_word}\"} '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c' {copy-mode -q; set-buffer -- \"#{q:mouse_word}\"} '#{?mouse_line,Copy Line,}' 'l' {copy-mode -q; set-buffer -- \"#{q:mouse_line}\"} '' '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h' {copy-mode -q; send-keys -l -- \"#{q:mouse_hyperlink}\"} '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h' {copy-mode -q; set-buffer -- \"#{q:mouse_hyperlink}\"} '' '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' {split-window -h} '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' {split-window -v} '' '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u' {swap-pane -U} '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd' {swap-pane -D} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-pane} '' 'Kill' 'X' {kill-pane} 'Respawn' 'R' {respawn-pane -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z' {resize-pane -Z} }",
        c"bind -n MouseDown1Pane { select-pane -t=; send -M }",
        c"bind -n C-MouseDown1Pane { swap-pane -s@ }",
        c"bind -n MouseDrag1Pane { if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -M } }",
        c"bind -n WheelUpPane { if -F '#{||:#{alternate_on},#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -e } }",
        c"bind -n MouseDown2Pane { select-pane -t=; if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { paste -p } }",
        c"bind -n DoubleClick1Pane { select-pane -t=; if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -H; send -X select-word; run -d0.3; send -X copy-pipe-and-cancel } }",
        c"bind -n TripleClick1Pane { select-pane -t=; if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -H; send -X select-line; run -d0.3; send -X copy-pipe-and-cancel } }",
        c"bind -n MouseDown1Border { select-pane -M }",
        c"bind -n MouseDrag1Border { resize-pane -M }",
        c"bind -n MouseDown1Status { switch-client -t= }",
        c"bind -n C-MouseDown1Status { swap-window -t@ }",
        c"bind -n MouseDown1Control9 { display-menu -t= -xM -yM -O -T 'Kill pane #{pane_index}?' 'Yes' 'y' { kill-pane -t= } 'No' 'n' {}}",
        c"bind -n MouseDown1Control8 { resize-pane -Z }",
        c"bind -n WheelDownStatus { next-window }",
        c"bind -n WheelUpStatus { previous-window }",
        c"bind -n MouseDown3StatusLeft { display-menu -t= -xM -yW -T '#[align=centre]#{session_name}'  'Next' 'n' {switch-client -n} 'Previous' 'p' {switch-client -p} '' 'Renumber' 'N' {move-window -r} 'Rename' 'r' {command-prompt -I \"#S\" {rename-session -- '%%'}} 'Detach' 'd' {detach-client} '' 'New Session' 's' {new-session} 'New Window' 'w' {new-window} }",
        c"bind -n M-MouseDown3StatusLeft { display-menu -t= -xM -yW -T '#[align=centre]#{session_name}'  'Next' 'n' {switch-client -n} 'Previous' 'p' {switch-client -p} '' 'Renumber' 'N' {move-window -r} 'Rename' 'r' {command-prompt -I \"#S\" {rename-session -- '%%'}} 'Detach' 'd' {detach-client} '' 'New Session' 's' {new-session} 'New Window' 'w' {new-window} }",
        c"bind -n MouseDown3Status { display-menu -t= -xW -yW -T '#[align=centre]#{window_index}:#{window_name}'  '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' {swap-window -t:-1} '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' {swap-window -t:+1} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-window} '' 'Kill' 'X' {kill-window} 'Respawn' 'R' {respawn-window -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} 'Rename' 'n' {command-prompt -FI \"#W\" {rename-window -t '#{window_id}' -- '%%'}} '' 'New After' 'w' {new-window -a} 'New At End' 'W' {new-window}}",
        c"bind -n M-MouseDown3Status { display-menu -t= -xW -yW -T '#[align=centre]#{window_index}:#{window_name}'  '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' {swap-window -t:-1} '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' {swap-window -t:+1} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-window} '' 'Kill' 'X' {kill-window} 'Respawn' 'R' {respawn-window -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} 'Rename' 'n' {command-prompt -FI \"#W\" {rename-window -t '#{window_id}' -- '%%'}} '' 'New After' 'w' {new-window -a} 'New At End' 'W' {new-window}}",
        c"bind -n MouseDown3Pane { if -Ft= '#{||:#{mouse_any_flag},#{&&:#{pane_in_mode},#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}}}' { select-pane -t=; send -M } { display-menu -t= -xM -yM -T '#[align=centre]#{pane_index} (#{pane_id})'  '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' {send -X history-top} '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>' {send -X history-bottom} '' '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' {paste-buffer} '' '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r' {if -F '#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}' 'copy-mode -t='; send -Xt= search-backward -- \"#{q:mouse_word}\"} '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y' {copy-mode -q; send-keys -l -- \"#{q:mouse_word}\"} '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c' {copy-mode -q; set-buffer -- \"#{q:mouse_word}\"} '#{?mouse_line,Copy Line,}' 'l' {copy-mode -q; set-buffer -- \"#{q:mouse_line}\"} '' '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h' {copy-mode -q; send-keys -l -- \"#{q:mouse_hyperlink}\"} '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h' {copy-mode -q; set-buffer -- \"#{q:mouse_hyperlink}\"} '' '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' {split-window -h} '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' {split-window -v} '' '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u' {swap-pane -U} '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd' {swap-pane -D} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-pane} '' 'Kill' 'X' {kill-pane} 'Respawn' 'R' {respawn-pane -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z' {resize-pane -Z} } }",
        c"bind -n M-MouseDown3Pane { display-menu -t= -xM -yM -T '#[align=centre]#{pane_index} (#{pane_id})'  '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' {send -X history-top} '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>' {send -X history-bottom} '' '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' {paste-buffer} '' '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r' {if -F '#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}' 'copy-mode -t='; send -Xt= search-backward -- \"#{q:mouse_word}\"} '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y' {copy-mode -q; send-keys -l -- \"#{q:mouse_word}\"} '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c' {copy-mode -q; set-buffer -- \"#{q:mouse_word}\"} '#{?mouse_line,Copy Line,}' 'l' {copy-mode -q; set-buffer -- \"#{q:mouse_line}\"} '' '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h' {copy-mode -q; send-keys -l -- \"#{q:mouse_hyperlink}\"} '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h' {copy-mode -q; set-buffer -- \"#{q:mouse_hyperlink}\"} '' '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' {split-window -h} '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' {split-window -v} '' '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u' {swap-pane -U} '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd' {swap-pane -D} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-pane} '' 'Kill' 'X' {kill-pane} 'Respawn' 'R' {respawn-pane -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z' {resize-pane -Z} }",
        c"bind -n MouseDown1ScrollbarUp { if -Ft= '#{pane_in_mode}' { send -X page-up } {copy-mode -u } }",
        c"bind -n MouseDown1ScrollbarDown { if -Ft= '#{pane_in_mode}' { send -X page-down } {copy-mode -d } }",
        c"bind -n MouseDrag1ScrollbarSlider { if -Ft= '#{pane_in_mode}' { send -X scroll-to-mouse } { copy-mode -S } }",
        c"bind -Tcopy-mode C-Space { send -X begin-selection }",
        c"bind -Tcopy-mode C-a { send -X start-of-line }",
        c"bind -Tcopy-mode C-c { send -X cancel }",
        c"bind -Tcopy-mode C-e { send -X end-of-line }",
        c"bind -Tcopy-mode C-f { send -X cursor-right }",
        c"bind -Tcopy-mode C-b { send -X cursor-left }",
        c"bind -Tcopy-mode C-g { send -X clear-selection }",
        c"bind -Tcopy-mode C-k { send -X copy-pipe-end-of-line-and-cancel }",
        c"bind -Tcopy-mode C-l { send -X recentre-top-bottom }",
        c"bind -Tcopy-mode M-l { send -X cursor-centre-horizontal }",
        c"bind -Tcopy-mode C-n { send -X cursor-down }",
        c"bind -Tcopy-mode C-p { send -X cursor-up }",
        c"bind -Tcopy-mode C-r { command-prompt -T search -ip'(search up)' -I'#{pane_search_string}' { send -X search-backward-incremental -- '%%' } }",
        c"bind -Tcopy-mode C-s { command-prompt -T search -ip'(search down)' -I'#{pane_search_string}' { send -X search-forward-incremental -- '%%' } }",
        c"bind -Tcopy-mode C-v { send -X page-down }",
        c"bind -Tcopy-mode C-w { send -X copy-pipe-and-cancel }",
        c"bind -Tcopy-mode Escape { send -X cancel }",
        c"bind -Tcopy-mode C-[ { send -X cancel }",
        c"bind -Tcopy-mode Space { send -X page-down }",
        c"bind -Tcopy-mode , { send -X jump-reverse }",
        c"bind -Tcopy-mode \\; { send -X jump-again }",
        c"bind -Tcopy-mode F { command-prompt -1p'(jump backward)' { send -X jump-backward -- '%%' } }",
        c"bind -Tcopy-mode N { send -X search-reverse }",
        c"bind -Tcopy-mode P { send -X toggle-position }",
        c"bind -Tcopy-mode R { send -X rectangle-toggle }",
        c"bind -Tcopy-mode T { command-prompt -1p'(jump to backward)' { send -X jump-to-backward -- '%%' } }",
        c"bind -Tcopy-mode X { send -X set-mark }",
        c"bind -Tcopy-mode f { command-prompt -1p'(jump forward)' { send -X jump-forward -- '%%' } }",
        c"bind -Tcopy-mode g { command-prompt -p'(goto line)' { send -X goto-line -- '%%' } }",
        c"bind -Tcopy-mode n { send -X search-again }",
        c"bind -Tcopy-mode q { send -X cancel }",
        c"bind -Tcopy-mode r { send -X refresh-from-pane }",
        c"bind -Tcopy-mode t { command-prompt -1p'(jump to forward)' { send -X jump-to-forward -- '%%' } }",
        c"bind -Tcopy-mode Home { send -X start-of-line }",
        c"bind -Tcopy-mode End { send -X end-of-line }",
        c"bind -Tcopy-mode MouseDown1Pane select-pane",
        c"bind -Tcopy-mode MouseDrag1Pane { select-pane; send -X begin-selection }",
        c"bind -Tcopy-mode MouseDragEnd1Pane { send -X copy-pipe-and-cancel }",
        c"bind -Tcopy-mode WheelUpPane { select-pane; send -N5 -X scroll-up }",
        c"bind -Tcopy-mode WheelDownPane { select-pane; send -N5 -X scroll-down }",
        c"bind -Tcopy-mode DoubleClick1Pane { select-pane; send -X select-word; run -d0.3; send -X copy-pipe-and-cancel }",
        c"bind -Tcopy-mode TripleClick1Pane { select-pane; send -X select-line; run -d0.3; send -X copy-pipe-and-cancel }",
        c"bind -Tcopy-mode NPage { send -X page-down }",
        c"bind -Tcopy-mode PPage { send -X page-up }",
        c"bind -Tcopy-mode Up { send -X cursor-up }",
        c"bind -Tcopy-mode Down { send -X cursor-down }",
        c"bind -Tcopy-mode Left { send -X cursor-left }",
        c"bind -Tcopy-mode Right { send -X cursor-right }",
        c"bind -Tcopy-mode M-1 { command-prompt -Np'(repeat)' -I1 { send -N '%%' } }",
        c"bind -Tcopy-mode M-2 { command-prompt -Np'(repeat)' -I2 { send -N '%%' } }",
        c"bind -Tcopy-mode M-3 { command-prompt -Np'(repeat)' -I3 { send -N '%%' } }",
        c"bind -Tcopy-mode M-4 { command-prompt -Np'(repeat)' -I4 { send -N '%%' } }",
        c"bind -Tcopy-mode M-5 { command-prompt -Np'(repeat)' -I5 { send -N '%%' } }",
        c"bind -Tcopy-mode M-6 { command-prompt -Np'(repeat)' -I6 { send -N '%%' } }",
        c"bind -Tcopy-mode M-7 { command-prompt -Np'(repeat)' -I7 { send -N '%%' } }",
        c"bind -Tcopy-mode M-8 { command-prompt -Np'(repeat)' -I8 { send -N '%%' } }",
        c"bind -Tcopy-mode M-9 { command-prompt -Np'(repeat)' -I9 { send -N '%%' } }",
        c"bind -Tcopy-mode M-< { send -X history-top }",
        c"bind -Tcopy-mode M-> { send -X history-bottom }",
        c"bind -Tcopy-mode M-R { send -X top-line }",
        c"bind -Tcopy-mode M-b { send -X previous-word }",
        c"bind -Tcopy-mode C-M-b { send -X previous-matching-bracket }",
        c"bind -Tcopy-mode M-f { send -X next-word-end }",
        c"bind -Tcopy-mode C-M-f { send -X next-matching-bracket }",
        c"bind -Tcopy-mode M-m { send -X back-to-indentation }",
        c"bind -Tcopy-mode M-r { send -X middle-line }",
        c"bind -Tcopy-mode M-v { send -X page-up }",
        c"bind -Tcopy-mode M-w { send -X copy-pipe-and-cancel }",
        c"bind -Tcopy-mode M-x { send -X jump-to-mark }",
        c"bind -Tcopy-mode 'M-{' { send -X previous-paragraph }",
        c"bind -Tcopy-mode 'M-}' { send -X next-paragraph }",
        c"bind -Tcopy-mode M-Up { send -X halfpage-up }",
        c"bind -Tcopy-mode M-Down { send -X halfpage-down }",
        c"bind -Tcopy-mode C-Up { send -X scroll-up }",
        c"bind -Tcopy-mode C-Down { send -X scroll-down }",
        c"bind -Tcopy-mode-vi '#' { send -FX search-backward -- '#{copy_cursor_word}' }",
        c"bind -Tcopy-mode-vi * { send -FX search-forward -- '#{copy_cursor_word}' }",
        c"bind -Tcopy-mode-vi C-c { send -X cancel }",
        c"bind -Tcopy-mode-vi C-d { send -X halfpage-down }",
        c"bind -Tcopy-mode-vi C-e { send -X scroll-down }",
        c"bind -Tcopy-mode-vi C-b { send -X page-up }",
        c"bind -Tcopy-mode-vi C-f { send -X page-down }",
        c"bind -Tcopy-mode-vi C-h { send -X cursor-left }",
        c"bind -Tcopy-mode-vi C-j { send -X copy-pipe-and-cancel }",
        c"bind -Tcopy-mode-vi Enter { send -X copy-pipe-and-cancel }",
        c"bind -Tcopy-mode-vi C-u { send -X halfpage-up }",
        c"bind -Tcopy-mode-vi C-v { send -X rectangle-toggle }",
        c"bind -Tcopy-mode-vi C-y { send -X scroll-up }",
        c"bind -Tcopy-mode-vi Escape { send -X clear-selection }",
        c"bind -Tcopy-mode-vi C-[ { send -X clear-selection }",
        c"bind -Tcopy-mode-vi Space { send -X begin-selection }",
        c"bind -Tcopy-mode-vi '$' { send -X end-of-line }",
        c"bind -Tcopy-mode-vi , { send -X jump-reverse }",
        c"bind -Tcopy-mode-vi / { command-prompt -T search -p'(search down)' { send -X search-forward -- '%%' } }",
        c"bind -Tcopy-mode-vi 0 { send -X start-of-line }",
        c"bind -Tcopy-mode-vi 1 { command-prompt -Np'(repeat)' -I1 { send -N '%%' } }",
        c"bind -Tcopy-mode-vi 2 { command-prompt -Np'(repeat)' -I2 { send -N '%%' } }",
        c"bind -Tcopy-mode-vi 3 { command-prompt -Np'(repeat)' -I3 { send -N '%%' } }",
        c"bind -Tcopy-mode-vi 4 { command-prompt -Np'(repeat)' -I4 { send -N '%%' } }",
        c"bind -Tcopy-mode-vi 5 { command-prompt -Np'(repeat)' -I5 { send -N '%%' } }",
        c"bind -Tcopy-mode-vi 6 { command-prompt -Np'(repeat)' -I6 { send -N '%%' } }",
        c"bind -Tcopy-mode-vi 7 { command-prompt -Np'(repeat)' -I7 { send -N '%%' } }",
        c"bind -Tcopy-mode-vi 8 { command-prompt -Np'(repeat)' -I8 { send -N '%%' } }",
        c"bind -Tcopy-mode-vi 9 { command-prompt -Np'(repeat)' -I9 { send -N '%%' } }",
        c"bind -Tcopy-mode-vi : { command-prompt -p'(goto line)' { send -X goto-line -- '%%' } }",
        c"bind -Tcopy-mode-vi \\; { send -X jump-again }",
        c"bind -Tcopy-mode-vi ? { command-prompt -T search -p'(search up)' { send -X search-backward -- '%%' } }",
        c"bind -Tcopy-mode-vi A { send -X append-selection-and-cancel }",
        c"bind -Tcopy-mode-vi B { send -X previous-space }",
        c"bind -Tcopy-mode-vi D { send -X copy-pipe-end-of-line-and-cancel }",
        c"bind -Tcopy-mode-vi E { send -X next-space-end }",
        c"bind -Tcopy-mode-vi F { command-prompt -1p'(jump backward)' { send -X jump-backward -- '%%' } }",
        c"bind -Tcopy-mode-vi G { send -X history-bottom }",
        c"bind -Tcopy-mode-vi H { send -X top-line }",
        c"bind -Tcopy-mode-vi J { send -X scroll-down }",
        c"bind -Tcopy-mode-vi K { send -X scroll-up }",
        c"bind -Tcopy-mode-vi L { send -X bottom-line }",
        c"bind -Tcopy-mode-vi M { send -X middle-line }",
        c"bind -Tcopy-mode-vi N { send -X search-reverse }",
        c"bind -Tcopy-mode-vi P { send -X toggle-position }",
        c"bind -Tcopy-mode-vi T { command-prompt -1p'(jump to backward)' { send -X jump-to-backward -- '%%' } }",
        c"bind -Tcopy-mode-vi V { send -X select-line }",
        c"bind -Tcopy-mode-vi W { send -X next-space }",
        c"bind -Tcopy-mode-vi X { send -X set-mark }",
        c"bind -Tcopy-mode-vi ^ { send -X back-to-indentation }",
        c"bind -Tcopy-mode-vi b { send -X previous-word }",
        c"bind -Tcopy-mode-vi e { send -X next-word-end }",
        c"bind -Tcopy-mode-vi f { command-prompt -1p'(jump forward)' { send -X jump-forward -- '%%' } }",
        c"bind -Tcopy-mode-vi g { send -X history-top }",
        c"bind -Tcopy-mode-vi h { send -X cursor-left }",
        c"bind -Tcopy-mode-vi j { send -X cursor-down }",
        c"bind -Tcopy-mode-vi k { send -X cursor-up }",
        c"bind -Tcopy-mode-vi z { send -X scroll-middle }",
        c"bind -Tcopy-mode-vi l { send -X cursor-right }",
        c"bind -Tcopy-mode-vi n { send -X search-again }",
        c"bind -Tcopy-mode-vi o { send -X other-end }",
        c"bind -Tcopy-mode-vi q { send -X cancel }",
        c"bind -Tcopy-mode-vi r { send -X refresh-from-pane }",
        c"bind -Tcopy-mode-vi t { command-prompt -1p'(jump to forward)' { send -X jump-to-forward -- '%%' } }",
        c"bind -Tcopy-mode-vi v { send -X rectangle-toggle }",
        c"bind -Tcopy-mode-vi w { send -X next-word }",
        c"bind -Tcopy-mode-vi '{' { send -X previous-paragraph }",
        c"bind -Tcopy-mode-vi '}' { send -X next-paragraph }",
        c"bind -Tcopy-mode-vi % { send -X next-matching-bracket }",
        c"bind -Tcopy-mode-vi Home { send -X start-of-line }",
        c"bind -Tcopy-mode-vi End { send -X end-of-line }",
        c"bind -Tcopy-mode-vi MouseDown1Pane { select-pane }",
        c"bind -Tcopy-mode-vi MouseDrag1Pane { select-pane; send -X begin-selection }",
        c"bind -Tcopy-mode-vi MouseDragEnd1Pane { send -X copy-pipe-and-cancel }",
        c"bind -Tcopy-mode-vi WheelUpPane { select-pane; send -N5 -X scroll-up }",
        c"bind -Tcopy-mode-vi WheelDownPane { select-pane; send -N5 -X scroll-down }",
        c"bind -Tcopy-mode-vi DoubleClick1Pane { select-pane; send -X select-word; run -d0.3; send -X copy-pipe-and-cancel }",
        c"bind -Tcopy-mode-vi TripleClick1Pane { select-pane; send -X select-line; run -d0.3; send -X copy-pipe-and-cancel }",
        c"bind -Tcopy-mode-vi BSpace { send -X cursor-left }",
        c"bind -Tcopy-mode-vi NPage { send -X page-down }",
        c"bind -Tcopy-mode-vi PPage { send -X page-up }",
        c"bind -Tcopy-mode-vi Up { send -X cursor-up }",
        c"bind -Tcopy-mode-vi Down { send -X cursor-down }",
        c"bind -Tcopy-mode-vi Left { send -X cursor-left }",
        c"bind -Tcopy-mode-vi Right { send -X cursor-right }",
        c"bind -Tcopy-mode-vi M-x { send -X jump-to-mark }",
        c"bind -Tcopy-mode-vi C-Up { send -X scroll-up }",
        c"bind -Tcopy-mode-vi C-Down { send -X scroll-down }",
    ];
        for &default in defaults {
            let mut pr = cmd_parse_from_string(default, None);
            if pr.status as core::ffi::c_uint
                != CMD_PARSE_SUCCESS as core::ffi::c_int as core::ffi::c_uint
            {
                let error = pr.error.take().unwrap();
                log_debug(c"%s", fmt_args![error.as_ptr()]);
                fatalx(c"bad default key: %s", fmt_args![default]);
            }
            let cmdlist = pr.cmdlist.take().unwrap();
            cmdq_append(None, cmdlist.queue_items(None));
        }
        cmdq_append(
            None,
            CmdqItemRef::callback_items(c"key_bindings_init_done", key_bindings_init_done),
        );
    }
}
fn key_bindings_read_only(item: &CmdqItemRef) -> cmd_retval {
    unsafe {
        let item = item.read();
        item.error(c"client is read-only", fmt_args![]);
        CMD_RETURN_ERROR
    }
}
pub unsafe fn key_bindings_dispatch(
    bd: &key_binding,
    item: Option<&CmdqItemRef>,
    c: Option<&mut client>,
    event: Option<&key_event>,
    fs: Option<&cmd_find_state>,
) -> Option<CmdqItemRef> {
    unsafe {
        let mut flags: core::ffi::c_int = 0 as core::ffi::c_int;
        let readonly: core::ffi::c_int = if c
            .as_deref()
            .is_none_or(|c| !c.flags & CLIENT_READONLY as uint64_t != 0)
        {
            1 as core::ffi::c_int
        } else {
            (bd.cmdlist.as_ref().unwrap()).all_have(CMD_READONLY)
        };
        let queued = if readonly == 0 {
            CmdqItemRef::callback_items(c"key_bindings_read_only", key_bindings_read_only)
        } else {
            if bd.flags & KEY_BINDING_REPEAT != 0 {
                flags |= CMDQ_STATE_REPEAT;
            }
            let new_state = CmdqStateRef::create(fs, event, flags);
            (bd.cmdlist.as_ref().unwrap()).queue_items(Some(&new_state))
        };
        match item {
            Some(after) => Some(after.insert_after(queued)),
            None => cmdq_append(
                c.as_deref().and_then(crate::server::client_ref_of).as_ref(),
                queued,
            ),
        }
    }
}

/// Whether any binding in `l` repeats, which is what the `key_has_repeat`
/// format reports. The C took the array and the count of it to scan; the
/// slice carries both.
pub fn key_bindings_has_repeat(l: &[key_binding]) -> core::ffi::c_int {
    l.iter()
        .any(|bd| key_binding_flags(bd) & KEY_BINDING_REPEAT != 0) as core::ffi::c_int
}

#[cfg(test)]
#[path = "tests/test_key_bindings.rs"]
mod tests;

#[cfg(test)]
pub(crate) use tests::{
    key_binding_cmdlist_ref, key_bindings_get, key_bindings_get_default, key_bindings_reset_table,
};

impl key_table {
    /// The name the table is held under, which is what `#{client_key_table}` and
    /// `list-keys -T` show.
    pub fn name(&self) -> &CStr {
        &self.name
    }
    /// Whether the table has a default tree behind it, which is what decides
    /// whether resetting it puts bindings back or takes the table away.
    pub fn has_defaults(&self) -> bool {
        !self.default_key_bindings.is_empty()
    }
    /// Whether the table holds no bindings of its own, which is what keeps an
    /// empty table out of the customize tree.
    pub fn is_empty(&self) -> bool {
        self.key_bindings.is_empty()
    }
    /// When a key was last looked up in the table, which is what the repeat
    /// timeout is measured from.
    pub fn activity_time(&self) -> timeval {
        self.activity_time
    }
    /// Says a key has just been looked up in the table.
    pub fn set_activity_time(&mut self, at: timeval) {
        self.activity_time = at;
    }
    /// Copies what `table` holds now into its default tree, which is what makes
    /// those bindings the ones `key_bindings_reset` puts back. The server does
    /// this once the bindings it starts with have been parsed.
    pub(crate) fn take_defaults(&mut self) {
        let table = self;
        let bindings: Vec<key_binding> = table
            .key_bindings
            .values()
            .map(|bd| (**bd).clone())
            .collect();
        for bd in bindings {
            let new_bd = Box::new(key_binding {
                key: bd.key,
                cmdlist: bd.cmdlist,
                note: bd.note,
                tablename: None,
                flags: bd.flags,
            });
            table.default_key_bindings.insert(new_bd.key, new_bd);
        }
    }
}

impl KeyTableRef {
    /// Copies the table name without retaining its borrow guard.
    pub(crate) fn name(&self) -> CString {
        self.borrow().name().to_owned()
    }

    #[cfg(test)]
    pub(crate) fn has_defaults(&self) -> bool {
        self.borrow().has_defaults()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }

    pub(crate) fn activity_time(&self) -> timeval {
        self.borrow().activity_time()
    }

    pub(crate) fn set_activity_time(&self, at: timeval) {
        self.borrow_mut().set_activity_time(at);
    }

    /// Records the current bindings as the defaults used when the table is reset.
    pub(crate) fn take_defaults(&self) {
        self.borrow_mut().take_defaults();
    }
}

impl ClientRef {
    /// # Safety
    /// Exclude conflicting client access, including reentrant callbacks, during this operation.
    pub(crate) unsafe fn dispatch_binding(
        bd: &key_binding,
        item: Option<&CmdqItemRef>,
        c: Option<&mut Self>,
        event: Option<&key_event>,
        fs: Option<&cmd_find_state>,
    ) -> Option<CmdqItemRef> {
        unsafe { key_bindings_dispatch(bd, item, c.map(|c| c.as_client_mut()), event, fs) }
    }
}

impl KeyTableRef {
    pub(crate) fn binding(&self, key: key_code) -> Option<key_binding> {
        self.borrow().binding(key).cloned()
    }
}
