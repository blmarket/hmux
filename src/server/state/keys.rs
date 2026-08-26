//! Key bindings and the prompt history that shares their client-facing plumbing.
//!
//! `install_default_key_bindings` is tmux's `key_bindings_init` table, spelled
//! out; the rest is `bind-key`/`unbind-key` maintenance over it and the lookup
//! the attached-client input path performs on every key.

use std::io;
use std::path::PathBuf;

use super::target::pane_not_found;
use super::{DEFAULT_KEY_TABLE, KeyBinding, ServerState};
use crate::server::command::ExecutableCommand;
use crate::server::key::{KeyCode, parse_key_name};
use crate::server::options::OptionsView;

impl ServerState {
    /// tmux 3.7b's `root` mouse table, in the same order and shape as
    /// `key_bindings_init`.
    ///
    /// The guards are the load-bearing part: a pane whose program enabled a
    /// DECSET mouse mode (`#{mouse_any_flag}`) or that is in a mode gets the
    /// report forwarded with `send-keys -M`, and only an inert pane lets the
    /// client act on the click itself.
    pub(super) fn install_default_key_bindings(&mut self) {
        // tmux's `DEFAULT_PANE_MENU`/`DEFAULT_WINDOW_MENU`/`DEFAULT_SESSION_MENU`,
        // spliced into the bindings below. An operand whose name expands empty
        // is a separator line.
        const DEFAULT_PANE_MENU: &str = concat!(
            " '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' 'send-keys -X history-top'",
            " '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>'",
            " 'send-keys -X history-bottom'",
            " ''",
            " '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},",
            "Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' 'paste-buffer'",
            " ''",
            " '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r'",
            " 'if-shell -F \"#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}\" \"copy-mode -t =\" ;",
            " send-keys -X -t = search-backward -- \"#{q:mouse_word}\"'",
            " '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y'",
            " 'copy-mode -q ; send-keys -l -- \"#{q:mouse_word}\"'",
            " '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c'",
            " 'copy-mode -q ; set-buffer -- \"#{q:mouse_word}\"'",
            " '#{?mouse_line,Copy Line,}' 'l' 'copy-mode -q ; set-buffer -- \"#{q:mouse_line}\"'",
            " ''",
            " '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h'",
            " 'copy-mode -q ; send-keys -l -- \"#{q:mouse_hyperlink}\"'",
            " '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h'",
            " 'copy-mode -q ; set-buffer -- \"#{q:mouse_hyperlink}\"'",
            " ''",
            " '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' 'split-window -h'",
            " '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' 'split-window -v'",
            " ''",
            " '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u'",
            " 'swap-pane -U'",
            " '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd'",
            " 'swap-pane -D'",
            " '#{?pane_marked_set,,-}Swap Marked' 's' 'swap-pane'",
            " ''",
            " 'Kill' 'X' 'kill-pane'",
            " 'Respawn' 'R' 'respawn-pane -k'",
            " '#{?pane_marked,Unmark,Mark}' 'm' 'select-pane -m'",
            " '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z'",
            " 'resize-pane -Z'",
        );
        const DEFAULT_WINDOW_MENU: &str = concat!(
            " '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' 'swap-window -t :-1'",
            " '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' 'swap-window -t :+1'",
            " '#{?pane_marked_set,,-}Swap Marked' 's' 'swap-window'",
            " ''",
            " 'Kill' 'X' 'kill-window'",
            " 'Respawn' 'R' 'respawn-window -k'",
            " '#{?pane_marked,Unmark,Mark}' 'm' 'select-pane -m'",
            " 'Rename' 'n'",
            " 'command-prompt -F -I \"#W\" \"rename-window -t #{window_id} -- %%\"'",
            " ''",
            " 'New After' 'w' 'new-window -a'",
            " 'New At End' 'W' 'new-window'",
        );
        const DEFAULT_SESSION_MENU: &str = concat!(
            " 'Next' 'n' 'switch-client -n'",
            " 'Previous' 'p' 'switch-client -p'",
            " ''",
            " 'Renumber' 'N' 'move-window -r'",
            " 'Rename' 'r' 'command-prompt -I \"#S\" \"rename-session -- %%\"'",
            " 'Detach' 'd' 'detach-client'",
            " ''",
            " 'New Session' 's' 'new-session'",
            " 'New Window' 'w' 'new-window'",
        );
        const ROOT_MOUSE_DEFAULTS: &[(&str, &str)] = &[
            ("MouseDown1Pane", "select-pane -t = ; send-keys -M"),
            ("C-MouseDown1Pane", "swap-pane -s ="),
            (
                "MouseDrag1Pane",
                "if-shell -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' \
                 'send-keys -M' 'copy-mode -M'",
            ),
            (
                "WheelUpPane",
                "if-shell -F '#{||:#{alternate_on},#{||:#{pane_in_mode},#{mouse_any_flag}}}' \
                 'send-keys -M' 'copy-mode -e'",
            ),
            (
                "MouseDown2Pane",
                "select-pane -t = ; if-shell -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' \
                 'send-keys -M' 'paste-buffer -p'",
            ),
            (
                "DoubleClick1Pane",
                "select-pane -t = ; if-shell -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' \
                 'send-keys -M' \
                 'copy-mode -H ; send-keys -X select-word ; run-shell -d 0.3 ; \
                 send-keys -X copy-pipe-and-cancel'",
            ),
            (
                "TripleClick1Pane",
                "select-pane -t = ; if-shell -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' \
                 'send-keys -M' \
                 'copy-mode -H ; send-keys -X select-line ; run-shell -d 0.3 ; \
                 send-keys -X copy-pipe-and-cancel'",
            ),
            ("MouseDown1Border", "select-pane -M"),
            ("MouseDrag1Border", "resize-pane -M"),
            ("MouseDown1Status", "switch-client -t ="),
            ("C-MouseDown1Status", "swap-window -t ="),
            ("WheelUpStatus", "previous-window"),
            ("WheelDownStatus", "next-window"),
            ("MouseDown1Control8", "resize-pane -Z"),
            (
                "MouseDown1Control9",
                "display-menu -O -t = -x M -y M -T 'Kill pane #{pane_index}?' \
                 'Yes' 'y' 'kill-pane -t =' 'No' 'n' ''",
            ),
            (
                "MouseDown1ScrollbarUp",
                "if-shell -F -t = '#{pane_in_mode}' 'send-keys -X page-up' 'copy-mode -u'",
            ),
            (
                "MouseDown1ScrollbarDown",
                "if-shell -F -t = '#{pane_in_mode}' 'send-keys -X page-down' 'copy-mode -d'",
            ),
            (
                "MouseDrag1ScrollbarSlider",
                "if-shell -F -t = '#{pane_in_mode}' \
                 'send-keys -X scroll-to-mouse' 'copy-mode -S'",
            ),
            (
                "M-MouseDown3Pane",
                "display-menu -t = -x M -y M \
                 -T '#[align=centre]#{pane_index} (#{pane_id})' {PANE_MENU}",
            ),
            (
                "MouseDown3Status",
                "display-menu -t = -x W -y W \
                 -T '#[align=centre]#{window_index}:#{window_name}' {WINDOW_MENU}",
            ),
            (
                "M-MouseDown3Status",
                "display-menu -t = -x W -y W \
                 -T '#[align=centre]#{window_index}:#{window_name}' {WINDOW_MENU}",
            ),
            (
                "MouseDown3StatusLeft",
                "display-menu -t = -x M -y W -T '#[align=centre]#{session_name}' {SESSION_MENU}",
            ),
            (
                "M-MouseDown3StatusLeft",
                "display-menu -t = -x M -y W -T '#[align=centre]#{session_name}' {SESSION_MENU}",
            ),
        ];
        const DEFAULTS: &[(&str, &str, &[&str])] = &[
            ("prefix", "C-b", &["send-prefix"]),
            ("prefix", "C-z", &["suspend-client"]),
            ("prefix", "d", &["detach-client"]),
            ("prefix", "c", &["new-window"]),
            ("prefix", "\"", &["split-window"]),
            ("prefix", "%", &["split-window", "-h"]),
            ("prefix", "n", &["next-window"]),
            ("prefix", "p", &["previous-window"]),
            ("prefix", "l", &["last-window"]),
            ("prefix", ":", &["command-prompt"]),
            ("prefix", "o", &["select-pane", "-t", ":.+"]),
            ("prefix", "[", &["copy-mode"]),
            ("prefix", "PPage", &["copy-mode", "-u"]),
            ("prefix", "Up", &["select-pane", "-U"]),
            ("prefix", "Down", &["select-pane", "-D"]),
            ("prefix", "Left", &["select-pane", "-L"]),
            ("prefix", "Right", &["select-pane", "-R"]),
            (
                "prefix",
                "&",
                &[
                    "confirm-before",
                    "-p",
                    "kill-window #W? (y/n)",
                    "kill-window",
                ],
            ),
            (
                "prefix",
                "x",
                &["confirm-before", "-p", "kill-pane #P? (y/n)", "kill-pane"],
            ),
            ("copy-mode", "PPage", &["send-keys", "-X", "page-up"]),
            ("copy-mode", "NPage", &["send-keys", "-X", "page-down"]),
            ("copy-mode", "Up", &["send-keys", "-X", "cursor-up"]),
            ("copy-mode", "Down", &["send-keys", "-X", "cursor-down"]),
            ("copy-mode", "q", &["send-keys", "-X", "cancel"]),
            ("copy-mode", "Escape", &["send-keys", "-X", "cancel"]),
            ("copy-mode", "C-c", &["send-keys", "-X", "cancel"]),
            ("copy-mode-vi", "PPage", &["send-keys", "-X", "page-up"]),
            ("copy-mode-vi", "NPage", &["send-keys", "-X", "page-down"]),
            ("copy-mode-vi", "Up", &["send-keys", "-X", "cursor-up"]),
            ("copy-mode-vi", "Down", &["send-keys", "-X", "cursor-down"]),
            ("copy-mode-vi", "k", &["send-keys", "-X", "cursor-up"]),
            ("copy-mode-vi", "j", &["send-keys", "-X", "cursor-down"]),
            ("copy-mode-vi", "g", &["send-keys", "-X", "history-top"]),
            ("copy-mode-vi", "G", &["send-keys", "-X", "history-bottom"]),
            ("copy-mode-vi", "q", &["send-keys", "-X", "cancel"]),
            (
                "copy-mode-vi",
                "Escape",
                &["send-keys", "-X", "clear-selection"],
            ),
            (
                "copy-mode-vi",
                "Enter",
                &["send-keys", "-X", "copy-pipe-and-cancel"],
            ),
            ("copy-mode-vi", "C-c", &["send-keys", "-X", "cancel"]),
            (
                "copy-mode-vi",
                "Space",
                &["send-keys", "-X", "begin-selection"],
            ),
            ("copy-mode-vi", "$", &["send-keys", "-X", "end-of-line"]),
            ("copy-mode-vi", "0", &["send-keys", "-X", "start-of-line"]),
            ("copy-mode-vi", "h", &["send-keys", "-X", "cursor-left"]),
            ("copy-mode-vi", "l", &["send-keys", "-X", "cursor-right"]),
            ("copy-mode-vi", "w", &["send-keys", "-X", "next-word"]),
            ("copy-mode-vi", "b", &["send-keys", "-X", "previous-word"]),
            ("copy-mode-vi", "e", &["send-keys", "-X", "next-word-end"]),
            ("copy-mode-vi", "n", &["send-keys", "-X", "search-again"]),
            ("copy-mode-vi", "N", &["send-keys", "-X", "search-reverse"]),
            ("copy-mode-vi", "o", &["send-keys", "-X", "other-end"]),
            (
                "copy-mode-vi",
                "v",
                &["send-keys", "-X", "rectangle-toggle"],
            ),
            ("copy-mode-vi", "V", &["send-keys", "-X", "select-line"]),
            (
                "copy-mode-vi",
                "0x5e",
                &["send-keys", "-X", "back-to-indentation"],
            ),
            (
                "copy-mode-vi",
                "%",
                &["send-keys", "-X", "next-matching-bracket"],
            ),
            ("copy-mode-vi", "H", &["send-keys", "-X", "top-line"]),
            ("copy-mode-vi", "M", &["send-keys", "-X", "middle-line"]),
            ("copy-mode-vi", "L", &["send-keys", "-X", "bottom-line"]),
            (
                "copy-mode-vi",
                "r",
                &["send-keys", "-X", "refresh-from-pane"],
            ),
            ("copy-mode-vi", "C-d", &["send-keys", "-X", "halfpage-down"]),
            ("copy-mode-vi", "C-u", &["send-keys", "-X", "halfpage-up"]),
            ("copy-mode-vi", "C-f", &["send-keys", "-X", "page-down"]),
            ("copy-mode-vi", "C-b", &["send-keys", "-X", "page-up"]),
            ("copy-mode-vi", "C-e", &["send-keys", "-X", "scroll-down"]),
            ("copy-mode-vi", "C-y", &["send-keys", "-X", "scroll-up"]),
            (
                "copy-mode",
                "C-Space",
                &["send-keys", "-X", "begin-selection"],
            ),
            ("copy-mode", "C-a", &["send-keys", "-X", "start-of-line"]),
            ("copy-mode", "C-[", &["send-keys", "-X", "cancel"]),
            ("copy-mode", "C-e", &["send-keys", "-X", "end-of-line"]),
            ("copy-mode", "C-f", &["send-keys", "-X", "cursor-right"]),
            ("copy-mode", "C-b", &["send-keys", "-X", "cursor-left"]),
            ("copy-mode", "C-g", &["send-keys", "-X", "clear-selection"]),
            (
                "copy-mode",
                "C-l",
                &["send-keys", "-X", "recentre-top-bottom"],
            ),
            ("copy-mode", "C-n", &["send-keys", "-X", "cursor-down"]),
            ("copy-mode", "C-p", &["send-keys", "-X", "cursor-up"]),
            ("copy-mode", "C-v", &["send-keys", "-X", "page-down"]),
            ("copy-mode", "Space", &["send-keys", "-X", "page-down"]),
            ("copy-mode", "n", &["send-keys", "-X", "search-again"]),
            ("copy-mode", "N", &["send-keys", "-X", "search-reverse"]),
            ("copy-mode", "R", &["send-keys", "-X", "rectangle-toggle"]),
            ("copy-mode", "r", &["send-keys", "-X", "refresh-from-pane"]),
            ("copy-mode", "Home", &["send-keys", "-X", "start-of-line"]),
            ("copy-mode", "End", &["send-keys", "-X", "end-of-line"]),
            (
                "copy-mode",
                "MouseDrag1Pane",
                &["select-pane", ";", "send-keys", "-X", "begin-selection"],
            ),
            (
                "copy-mode",
                "MouseDragEnd1Pane",
                &["send-keys", "-X", "copy-pipe-and-cancel"],
            ),
            (
                "copy-mode",
                "WheelUpPane",
                &["select-pane", ";", "send-keys", "-N5", "-X", "scroll-up"],
            ),
            (
                "copy-mode",
                "WheelDownPane",
                &["select-pane", ";", "send-keys", "-N5", "-X", "scroll-down"],
            ),
            (
                "copy-mode",
                "C-r",
                &[
                    "command-prompt",
                    "-T",
                    "search",
                    "-i",
                    "-p",
                    "(search up)",
                    "-I",
                    "#{pane_search_string}",
                    "send-keys -X search-backward-incremental -- '%%'",
                ],
            ),
            (
                "copy-mode",
                "C-s",
                &[
                    "command-prompt",
                    "-T",
                    "search",
                    "-i",
                    "-p",
                    "(search down)",
                    "-I",
                    "#{pane_search_string}",
                    "send-keys -X search-forward-incremental -- '%%'",
                ],
            ),
            (
                "copy-mode",
                "f",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump forward)",
                    "send-keys -X jump-forward -- '%%'",
                ],
            ),
            (
                "copy-mode",
                "F",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump backward)",
                    "send-keys -X jump-backward -- '%%'",
                ],
            ),
            (
                "copy-mode",
                "t",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump to forward)",
                    "send-keys -X jump-to-forward -- '%%'",
                ],
            ),
            (
                "copy-mode",
                "T",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump to backward)",
                    "send-keys -X jump-to-backward -- '%%'",
                ],
            ),
            ("copy-mode", ",", &["send-keys", "-X", "jump-reverse"]),
            ("copy-mode", ";", &["send-keys", "-X", "jump-again"]),
            ("copy-mode", "X", &["send-keys", "-X", "set-mark"]),
            ("copy-mode", "M-x", &["send-keys", "-X", "jump-to-mark"]),
            ("copy-mode", "M-b", &["send-keys", "-X", "previous-word"]),
            ("copy-mode", "M-f", &["send-keys", "-X", "next-word-end"]),
            (
                "copy-mode",
                "M-l",
                &["send-keys", "-X", "cursor-centre-horizontal"],
            ),
            (
                "copy-mode",
                "M-m",
                &["send-keys", "-X", "back-to-indentation"],
            ),
            ("copy-mode", "M-v", &["send-keys", "-X", "page-up"]),
            ("copy-mode", "M-Up", &["send-keys", "-X", "halfpage-up"]),
            ("copy-mode", "M-Down", &["send-keys", "-X", "halfpage-down"]),
            ("copy-mode", "C-Up", &["send-keys", "-X", "scroll-up"]),
            ("copy-mode", "C-Down", &["send-keys", "-X", "scroll-down"]),
            ("copy-mode", "M-<", &["send-keys", "-X", "history-top"]),
            ("copy-mode", "M->", &["send-keys", "-X", "history-bottom"]),
            ("copy-mode", "P", &["send-keys", "-X", "toggle-position"]),
            (
                "copy-mode",
                "g",
                &[
                    "command-prompt",
                    "-p",
                    "(goto line)",
                    "send-keys -X goto-line -- '%%'",
                ],
            ),
            ("copy-mode", "MouseDown1Pane", &["select-pane"]),
            (
                "copy-mode",
                "DoubleClick1Pane",
                &[
                    "select-pane",
                    ";",
                    "send-keys",
                    "-X",
                    "select-word",
                    ";",
                    "run-shell",
                    "-d",
                    "0.3",
                    ";",
                    "send-keys",
                    "-X",
                    "copy-pipe-and-cancel",
                ],
            ),
            (
                "copy-mode",
                "TripleClick1Pane",
                &[
                    "select-pane",
                    ";",
                    "send-keys",
                    "-X",
                    "select-line",
                    ";",
                    "run-shell",
                    "-d",
                    "0.3",
                    ";",
                    "send-keys",
                    "-X",
                    "copy-pipe-and-cancel",
                ],
            ),
            (
                "copy-mode-vi",
                "/",
                &[
                    "command-prompt",
                    "-p",
                    "(search down)",
                    "send-keys -X search-forward -- '%%'",
                ],
            ),
            (
                "copy-mode-vi",
                "?",
                &[
                    "command-prompt",
                    "-p",
                    "(search up)",
                    "send-keys -X search-backward -- '%%'",
                ],
            ),
            (
                "copy-mode-vi",
                "*",
                &[
                    "send-keys",
                    "-F",
                    "-X",
                    "search-forward",
                    "--",
                    "#{copy_cursor_word}",
                ],
            ),
            (
                "copy-mode-vi",
                "#",
                &[
                    "send-keys",
                    "-F",
                    "-X",
                    "search-backward",
                    "--",
                    "#{copy_cursor_word}",
                ],
            ),
            (
                "copy-mode-vi",
                "C-[",
                &["send-keys", "-X", "clear-selection"],
            ),
            ("copy-mode-vi", "C-h", &["send-keys", "-X", "cursor-left"]),
            (
                "copy-mode-vi",
                ":",
                &[
                    "command-prompt",
                    "-p",
                    "(goto line)",
                    "send-keys -X goto-line -- '%%'",
                ],
            ),
            (
                "copy-mode-vi",
                "f",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump forward)",
                    "send-keys -X jump-forward -- '%%'",
                ],
            ),
            (
                "copy-mode-vi",
                "F",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump backward)",
                    "send-keys -X jump-backward -- '%%'",
                ],
            ),
            (
                "copy-mode-vi",
                "t",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump to forward)",
                    "send-keys -X jump-to-forward -- '%%'",
                ],
            ),
            (
                "copy-mode-vi",
                "T",
                &[
                    "command-prompt",
                    "-1",
                    "-p",
                    "(jump to backward)",
                    "send-keys -X jump-to-backward -- '%%'",
                ],
            ),
            ("copy-mode-vi", ",", &["send-keys", "-X", "jump-reverse"]),
            ("copy-mode-vi", ";", &["send-keys", "-X", "jump-again"]),
            ("copy-mode-vi", "X", &["send-keys", "-X", "set-mark"]),
            ("copy-mode-vi", "M-x", &["send-keys", "-X", "jump-to-mark"]),
            ("copy-mode-vi", "P", &["send-keys", "-X", "toggle-position"]),
            ("copy-mode-vi", "z", &["send-keys", "-X", "scroll-middle"]),
            ("copy-mode-vi", "B", &["send-keys", "-X", "previous-space"]),
            ("copy-mode-vi", "W", &["send-keys", "-X", "next-space"]),
            ("copy-mode-vi", "E", &["send-keys", "-X", "next-space-end"]),
            ("copy-mode-vi", "J", &["send-keys", "-X", "scroll-down"]),
            ("copy-mode-vi", "K", &["send-keys", "-X", "scroll-up"]),
            (
                "copy-mode-vi",
                "C-v",
                &["send-keys", "-X", "rectangle-toggle"],
            ),
            ("copy-mode-vi", "Left", &["send-keys", "-X", "cursor-left"]),
            (
                "copy-mode-vi",
                "Right",
                &["send-keys", "-X", "cursor-right"],
            ),
            ("copy-mode-vi", "C-Up", &["send-keys", "-X", "scroll-up"]),
            (
                "copy-mode-vi",
                "C-Down",
                &["send-keys", "-X", "scroll-down"],
            ),
            (
                "copy-mode-vi",
                "Home",
                &["send-keys", "-X", "start-of-line"],
            ),
            ("copy-mode-vi", "End", &["send-keys", "-X", "end-of-line"]),
            (
                "copy-mode-vi",
                "BSpace",
                &["send-keys", "-X", "cursor-left"],
            ),
            (
                "copy-mode-vi",
                "{",
                &["send-keys", "-X", "previous-paragraph"],
            ),
            ("copy-mode-vi", "}", &["send-keys", "-X", "next-paragraph"]),
            (
                "copy-mode-vi",
                "C-j",
                &["send-keys", "-X", "copy-pipe-and-cancel"],
            ),
            (
                "copy-mode-vi",
                "D",
                &["send-keys", "-X", "copy-pipe-end-of-line-and-cancel"],
            ),
            (
                "copy-mode-vi",
                "A",
                &["send-keys", "-X", "append-selection-and-cancel"],
            ),
            ("copy-mode", "Left", &["send-keys", "-X", "cursor-left"]),
            ("copy-mode", "Right", &["send-keys", "-X", "cursor-right"]),
            ("copy-mode", "M-R", &["send-keys", "-X", "top-line"]),
            ("copy-mode", "M-r", &["send-keys", "-X", "middle-line"]),
            (
                "copy-mode",
                "C-M-b",
                &["send-keys", "-X", "previous-matching-bracket"],
            ),
            (
                "copy-mode",
                "C-M-f",
                &["send-keys", "-X", "next-matching-bracket"],
            ),
            (
                "copy-mode",
                "M-{",
                &["send-keys", "-X", "previous-paragraph"],
            ),
            ("copy-mode", "M-}", &["send-keys", "-X", "next-paragraph"]),
            (
                "copy-mode",
                "C-k",
                &["send-keys", "-X", "copy-pipe-end-of-line-and-cancel"],
            ),
            (
                "copy-mode",
                "C-w",
                &["send-keys", "-X", "copy-pipe-and-cancel"],
            ),
            (
                "copy-mode",
                "M-w",
                &["send-keys", "-X", "copy-pipe-and-cancel"],
            ),
            (
                "copy-mode-vi",
                "MouseDrag1Pane",
                &["select-pane", ";", "send-keys", "-X", "begin-selection"],
            ),
            ("copy-mode-vi", "MouseDown1Pane", &["select-pane"]),
            (
                "copy-mode-vi",
                "DoubleClick1Pane",
                &[
                    "select-pane",
                    ";",
                    "send-keys",
                    "-X",
                    "select-word",
                    ";",
                    "run-shell",
                    "-d",
                    "0.3",
                    ";",
                    "send-keys",
                    "-X",
                    "copy-pipe-and-cancel",
                ],
            ),
            (
                "copy-mode-vi",
                "TripleClick1Pane",
                &[
                    "select-pane",
                    ";",
                    "send-keys",
                    "-X",
                    "select-line",
                    ";",
                    "run-shell",
                    "-d",
                    "0.3",
                    ";",
                    "send-keys",
                    "-X",
                    "copy-pipe-and-cancel",
                ],
            ),
            (
                "copy-mode-vi",
                "MouseDragEnd1Pane",
                &["send-keys", "-X", "copy-pipe-and-cancel"],
            ),
            (
                "copy-mode-vi",
                "WheelUpPane",
                &["select-pane", ";", "send-keys", "-N5", "-X", "scroll-up"],
            ),
            (
                "copy-mode-vi",
                "WheelDownPane",
                &["select-pane", ";", "send-keys", "-N5", "-X", "scroll-down"],
            ),
        ];
        for &(table, name, command) in DEFAULTS {
            let key =
                parse_key_name(name).unwrap_or_else(|| panic!("invalid default key name: {name}"));
            let command = command
                .iter()
                .map(|word| (*word).to_string())
                .collect::<Vec<_>>();
            self.bind_key(table, key, default_binding(&command), false, None);
        }
        // `MouseDown3Pane` is the one mouse default whose branch is itself a
        // whole `display-menu` line, and the menu text carries quotes of its
        // own, so it cannot be written as a quoted word inside the `if-shell`
        // line the way the others are. tmux writes it as a `{ }` command block;
        // this builds the operand directly instead, so `if-shell` gets exactly
        // its condition and two branches.
        {
            let key = parse_key_name("MouseDown3Pane").expect("mouse key");
            let mut menu = String::from(
                "display-menu -t = -x M -y M \
                 -T \"#[align=centre]#{pane_index} (#{pane_id})\"",
            );
            menu.push_str(DEFAULT_PANE_MENU);
            let command = vec![
                "if-shell".to_string(),
                "-F".to_string(),
                "-t".to_string(),
                "=".to_string(),
                "#{||:#{mouse_any_flag},#{&&:#{pane_in_mode},\
                 #{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}}}"
                    .to_string(),
                "select-pane -t = ; send-keys -M".to_string(),
                menu,
            ];
            self.bind_key(
                DEFAULT_KEY_TABLE,
                key,
                default_binding(&command),
                false,
                None,
            );
        }
        for &(name, command) in ROOT_MOUSE_DEFAULTS {
            let key = parse_key_name(name)
                .unwrap_or_else(|| panic!("invalid default mouse key name: {name}"));
            let command = command
                .replace("{PANE_MENU}", DEFAULT_PANE_MENU)
                .replace("{WINDOW_MENU}", DEFAULT_WINDOW_MENU)
                .replace("{SESSION_MENU}", DEFAULT_SESSION_MENU);
            let command = crate::server::command::binding_words(&command);
            self.bind_key(
                DEFAULT_KEY_TABLE,
                key,
                default_binding(&command),
                false,
                None,
            );
        }
        for index in 0..=9 {
            let key = parse_key_name(&index.to_string()).expect("digit key");
            let select = vec![
                "select-window".to_string(),
                "-t".to_string(),
                format!(":{index}"),
            ];
            self.bind_key("prefix", key, default_binding(&select), false, None);
            if index != 0 {
                let repeat = vec![
                    "command-prompt".to_string(),
                    "-N".to_string(),
                    "-p".to_string(),
                    "(repeat)".to_string(),
                    "-I".to_string(),
                    index.to_string(),
                    "send-keys -N '%%'".to_string(),
                ];
                let repeat = default_binding(&repeat);
                self.bind_key("copy-mode-vi", key, repeat.clone(), false, None);
                let meta =
                    parse_key_name(&format!("M-{index}")).expect("copy-mode emacs repeat key");
                self.bind_key("copy-mode", meta, repeat, false, None);
            }
        }

        // The prefix table is also a user-visible catalog: `list-keys -F`
        // exposes these notes and repeat flags even when a client never
        // exercises the corresponding binding. Keep the catalog complete so
        // listing it has the same shape as tmux's default table.
        const PREFIX_DEFAULTS: &[(&str, bool, &str, &str)] = &[
            ("Space", false, "Select next layout", "next-layout"),
            ("!", false, "Break pane to a new window", "break-pane"),
            ("\"", false, "Split window vertically", "split-window"),
            ("#", false, "List all paste buffers", "list-buffers"),
            (
                "$",
                false,
                "Rename current session",
                "command-prompt -I '#S' \"rename-session -- '%%'\"",
            ),
            ("%", false, "Split window horizontally", "split-window -h"),
            ("&", false, "Kill current window", "kill-window"),
            (
                "'",
                false,
                "Prompt for window index to select",
                "select-window",
            ),
            ("(", false, "Switch to previous client", "switch-client -p"),
            (")", false, "Switch to next client", "switch-client -n"),
            ("*", false, "New floating pane", "new-window"),
            (
                ",",
                false,
                "Rename current window",
                "command-prompt -I '#W' \"rename-window -- '%%'\"",
            ),
            (
                "-",
                false,
                "Delete the most recent paste buffer",
                "delete-buffer",
            ),
            (".", false, "Move the current window", "move-window"),
            ("/", false, "Describe key binding", "list-keys"),
            ("0", false, "Select window 0", "select-window -t :=0"),
            ("1", false, "Select window 1", "select-window -t :=1"),
            ("2", false, "Select window 2", "select-window -t :=2"),
            ("3", false, "Select window 3", "select-window -t :=3"),
            ("4", false, "Select window 4", "select-window -t :=4"),
            ("5", false, "Select window 5", "select-window -t :=5"),
            ("6", false, "Select window 6", "select-window -t :=6"),
            ("7", false, "Select window 7", "select-window -t :=7"),
            ("8", false, "Select window 8", "select-window -t :=8"),
            ("9", false, "Select window 9", "select-window -t :=9"),
            (":", false, "Prompt for a command", "command-prompt"),
            (
                ";",
                false,
                "Move to the previously active pane",
                "last-pane",
            ),
            (
                "<",
                false,
                "Display window menu",
                "display-menu -x W -y W \
                 -T '#[align=centre]#{window_index}:#{window_name}' {WINDOW_MENU}",
            ),
            (
                "=",
                false,
                "Choose a paste buffer from a list",
                "choose-buffer -Z",
            ),
            (
                ">",
                false,
                "Display pane menu",
                "display-menu -x P -y P \
                 -T '#[align=centre]#{pane_index} (#{pane_id})' {PANE_MENU}",
            ),
            ("?", false, "List key bindings", "list-keys -N"),
            ("C", false, "Customize options", "customize-mode -Z"),
            (
                "D",
                false,
                "Choose and detach a client from a list",
                "choose-client -Z",
            ),
            ("E", false, "Spread panes out evenly", "select-layout -E"),
            ("L", false, "Switch to the last client", "switch-client -l"),
            ("M", false, "Clear the marked pane", "select-pane -M"),
            ("[", false, "Enter copy mode", "copy-mode"),
            (
                "]",
                false,
                "Paste the most recent paste buffer",
                "paste-buffer -p",
            ),
            ("c", false, "Create a new window", "new-window"),
            ("d", false, "Detach the current client", "detach-client"),
            (
                "f",
                false,
                "Search for a pane",
                "command-prompt \"find-window -Z -- '%%'\"",
            ),
            ("i", false, "Display window information", "display-message"),
            (
                "l",
                false,
                "Select the previously current window",
                "last-window",
            ),
            ("m", false, "Toggle the marked pane", "select-pane -m"),
            ("n", false, "Select the next window", "next-window"),
            ("o", false, "Select the next pane", "select-pane -t :.+"),
            ("p", false, "Select the previous window", "previous-window"),
            ("q", false, "Display pane numbers", "display-panes"),
            ("r", false, "Redraw the current client", "refresh-client"),
            (
                "s",
                false,
                "Choose a session from a list",
                "choose-tree -Zs",
            ),
            ("t", false, "Show a clock", "clock-mode"),
            ("w", false, "Choose a window from a list", "choose-tree -Zw"),
            ("x", false, "Kill the active pane", "kill-pane"),
            ("z", false, "Zoom the active pane", "resize-pane -Z"),
            (
                "{",
                false,
                "Swap the active pane with the pane above",
                "swap-pane -U",
            ),
            (
                "}",
                false,
                "Swap the active pane with the pane below",
                "swap-pane -D",
            ),
            ("~", false, "Show messages", "show-messages"),
            (
                "DC",
                true,
                "Reset so the visible part of the window follows the cursor",
                "refresh-client -c",
            ),
            (
                "PPage",
                false,
                "Enter copy mode and scroll up",
                "copy-mode -u",
            ),
            (
                "Up",
                true,
                "Select the pane above the active pane",
                "select-pane -U",
            ),
            (
                "Down",
                true,
                "Select the pane below the active pane",
                "select-pane -D",
            ),
            (
                "Left",
                true,
                "Select the pane to the left of the active pane",
                "select-pane -L",
            ),
            (
                "Right",
                true,
                "Select the pane to the right of the active pane",
                "select-pane -R",
            ),
            (
                "M-1",
                false,
                "Set the even-horizontal layout",
                "select-layout even-horizontal",
            ),
            (
                "M-2",
                false,
                "Set the even-vertical layout",
                "select-layout even-vertical",
            ),
            (
                "M-3",
                false,
                "Set the main-horizontal layout",
                "select-layout main-horizontal",
            ),
            (
                "M-4",
                false,
                "Set the main-vertical layout",
                "select-layout main-vertical",
            ),
            (
                "M-5",
                false,
                "Select the tiled layout",
                "select-layout tiled",
            ),
            (
                "M-6",
                false,
                "Set the main-horizontal-mirrored layout",
                "select-layout main-horizontal-mirrored",
            ),
            (
                "M-7",
                false,
                "Set the main-vertical-mirrored layout",
                "select-layout main-vertical-mirrored",
            ),
            (
                "M-n",
                false,
                "Select the next window with an alert",
                "next-window -a",
            ),
            (
                "M-o",
                false,
                "Rotate through the panes in reverse",
                "rotate-window -D",
            ),
            (
                "M-p",
                false,
                "Select the previous window with an alert",
                "previous-window -a",
            ),
            ("M-Up", true, "Resize the pane up by 5", "resize-pane -U 5"),
            (
                "M-Down",
                true,
                "Resize the pane down by 5",
                "resize-pane -D 5",
            ),
            (
                "M-Left",
                true,
                "Resize the pane left by 5",
                "resize-pane -L 5",
            ),
            (
                "M-Right",
                true,
                "Resize the pane right by 5",
                "resize-pane -R 5",
            ),
            ("C-b", false, "Send the prefix key", "send-prefix"),
            ("C-o", false, "Rotate through the panes", "rotate-window"),
            ("C-z", false, "Suspend the current client", "suspend-client"),
            ("C-Up", true, "Resize the pane up", "resize-pane -U"),
            ("C-Down", true, "Resize the pane down", "resize-pane -D"),
            ("C-Left", true, "Resize the pane left", "resize-pane -L"),
            ("C-Right", true, "Resize the pane right", "resize-pane -R"),
            (
                "S-Up",
                true,
                "Move the visible part of the window up",
                "refresh-client -U 10",
            ),
            (
                "S-Down",
                true,
                "Move the visible part of the window down",
                "refresh-client -D 10",
            ),
            (
                "S-Left",
                true,
                "Move the visible part of the window left",
                "refresh-client -L 10",
            ),
            (
                "S-Right",
                true,
                "Move the visible part of the window right",
                "refresh-client -R 10",
            ),
        ];
        for &(name, repeat, note, command) in PREFIX_DEFAULTS {
            let key =
                parse_key_name(name).unwrap_or_else(|| panic!("invalid default key name: {name}"));
            if let Some(binding) = self
                .key_tables
                .get_mut("prefix")
                .and_then(|bindings| bindings.get_mut(&key))
            {
                binding.repeat = repeat;
                binding.note = Some(note.to_string());
            } else {
                let command = command
                    .replace("{PANE_MENU}", DEFAULT_PANE_MENU)
                    .replace("{WINDOW_MENU}", DEFAULT_WINDOW_MENU);
                let command = crate::server::command::binding_words(&command);
                self.bind_key(
                    "prefix",
                    key,
                    default_binding(&command),
                    repeat,
                    Some(note.to_string()),
                );
            }
        }
    }

    /// tmux's `server_client_get_key_table`: the table a client attached to
    /// `target` uses until the prefix key moves it elsewhere.
    pub(crate) fn session_key_table(&self, target: &str) -> String {
        self.option_for_target(target, "key-table")
            .filter(|table| !table.is_empty())
            .unwrap_or(DEFAULT_KEY_TABLE)
            .to_string()
    }

    /// Record the key table an attached client moved into, so
    /// `#{client_key_table}`/`#{client_prefix}` see it. The owning client
    /// repaints its own status line; no other client's view depends on it.
    pub(crate) fn set_client_key_table(&mut self, client: &str, table: &str) {
        self.client_renders.set_client_key_table(client, table);
    }

    pub(crate) fn add_prompt_history(&mut self, prompt_type: &str, value: &str) {
        if value.is_empty() {
            return;
        }
        let limit = self
            .server_options()
            .get("prompt-history-limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(100);
        let history = self
            .prompt_history
            .entry(prompt_type.to_string())
            .or_default();
        if history.last().is_some_and(|last| last == value) {
            return;
        }
        history.push(value.to_string());
        if history.len() > limit {
            history.drain(..history.len() - limit);
        }
    }

    pub(crate) fn prompt_history(&self, prompt_type: &str) -> &[String] {
        self.prompt_history
            .get(prompt_type)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// The prompt types tmux keeps a history for, in the order it writes them
    /// to the history file.
    const PROMPT_TYPES: [&'static str; 4] = ["command", "search", "target", "window-target"];

    /// The file `history-file` names, or `None` when the option is empty or
    /// names something tmux's `status_prompt_find_history_file` refuses: only
    /// an absolute path or one under `~/` is accepted.
    fn prompt_history_file(&self) -> Option<PathBuf> {
        let value = self.server_options().get("history-file")?;
        if value.is_empty() {
            return None;
        }
        if value.starts_with('/') {
            return Some(PathBuf::from(value));
        }
        let rest = value.strip_prefix("~/")?;
        Some(PathBuf::from(std::env::var_os("HOME")?).join(rest))
    }

    /// tmux's `status_prompt_load_history`, run once per history file.
    ///
    /// tmux loads it when the configuration has been read; the hmux daemon has
    /// no configuration interface, so the load happens when the option first
    /// names a file instead.
    pub(crate) fn load_prompt_history(&mut self) {
        let Some(path) = self.prompt_history_file() else {
            return;
        };
        if self.prompt_history_file_loaded.as_ref() == Some(&path) {
            return;
        }
        self.prompt_history_file_loaded = Some(path.clone());
        let Ok(contents) = std::fs::read_to_string(&path) else {
            return;
        };
        for line in contents.lines().filter(|line| !line.is_empty()) {
            // An unknown type is an old-format file, whose lines are all
            // command history.
            match line.split_once(':') {
                Some((prompt_type, value)) if Self::PROMPT_TYPES.contains(&prompt_type) => {
                    self.add_prompt_history(prompt_type, value)
                }
                _ => self.add_prompt_history("command", line),
            }
        }
    }

    /// tmux's `status_prompt_save_history`, run as the server exits.
    pub(crate) fn save_prompt_history(&self) {
        let Some(path) = self.prompt_history_file() else {
            return;
        };
        let mut contents = String::new();
        for prompt_type in Self::PROMPT_TYPES {
            for value in self.prompt_history(prompt_type) {
                contents.push_str(prompt_type);
                contents.push(':');
                contents.push_str(value);
                contents.push('\n');
            }
        }
        let _ = std::fs::write(&path, contents);
    }

    pub(crate) fn clear_prompt_history(&mut self, prompt_type: Option<&str>) {
        if let Some(prompt_type) = prompt_type {
            self.prompt_history.remove(prompt_type);
        } else {
            self.prompt_history.clear();
        }
    }

    /// Install or replace a mutable key-table binding.
    pub(crate) fn bind_key(
        &mut self,
        table: &str,
        key: KeyCode,
        command: ExecutableCommand,
        repeat: bool,
        note: Option<String>,
    ) {
        self.key_tables
            .entry(table.to_string())
            .or_default()
            .insert(
                key,
                KeyBinding {
                    repeat,
                    note,
                    command,
                },
            );
    }

    /// tmux's `key_bindings_add` with a NULL command list: `bind-key key` given
    /// no command only annotates a binding that already exists — `-N` replaces
    /// its note and `-r` makes it repeat — and does nothing at all when there
    /// is none.
    pub(crate) fn annotate_key_binding(
        &mut self,
        table: &str,
        key: KeyCode,
        note: Option<String>,
        repeat: bool,
    ) {
        // tmux's `key_bindings_get_table(name, 1)` creates the table either way,
        // so naming an unknown one here is how an empty table comes into being.
        let bindings = self.key_tables.entry(table.to_string()).or_default();
        let Some(binding) = bindings.get_mut(&key) else {
            return;
        };
        if note.is_some() {
            binding.note = note;
        }
        if repeat {
            binding.repeat = true;
        }
    }

    /// Remove one binding, or all bindings in a table when `all` is set.
    pub(crate) fn unbind_key(&mut self, table: &str, key: Option<KeyCode>, all: bool) {
        if all {
            self.key_tables.remove(table);
        } else if let Some(key) = key {
            if let Some(bindings) = self.key_tables.get_mut(table) {
                bindings.remove(&key);
                if bindings.is_empty() {
                    self.key_tables.remove(table);
                }
            }
        }
    }

    pub(crate) fn key_binding(&self, table: &str, key: KeyCode) -> Option<&KeyBinding> {
        self.key_tables.get(table)?.get(&key)
    }

    /// Resolve a key against the target pane's active mode table.
    ///
    /// The outer `Option` distinguishes a pane with no mode from an active mode
    /// with no binding for `key`; tmux consumes keys in the latter case.
    pub(crate) fn pane_mode_key_binding(
        &self,
        target: &str,
        key: KeyCode,
        vi: bool,
    ) -> io::Result<Option<Option<KeyBinding>>> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let pane = &self.window(resolved.session, resolved.window).panes[resolved.pane];
        if pane.copy.is_none() {
            return Ok(None);
        }
        let table = if vi { "copy-mode-vi" } else { "copy-mode" };
        Ok(Some(self.key_binding(table, key).cloned()))
    }

    pub(crate) fn key_table_exists(&self, table: &str) -> bool {
        self.key_tables.contains_key(table)
    }

    pub(crate) fn key_bindings(&self, table: Option<&str>) -> Vec<(&str, KeyCode, &KeyBinding)> {
        let mut out = Vec::new();
        for (table_name, bindings) in &self.key_tables {
            if table.is_some_and(|wanted| wanted != table_name) {
                continue;
            }
            for (key, binding) in bindings {
                out.push((table_name.as_str(), *key, binding));
            }
        }
        out
    }

    /// tmux's `tty_window_offset1`: where a client's terminal sits over the
    /// window it is showing.
    ///
    /// When the window fits, the client sees all of it at the origin and any
    /// pan is abandoned. When it does not, the client shows a `sx`×`sy`
    /// viewport at `(ox, oy)` — an explicit `refresh-client` pan if one is live
    /// for this window, otherwise a window that follows the cursor so the
    /// active pane's cursor stays on screen.
    /// The `user-keys` array for a target, by index: entry *n* is the sequence
    /// tmux's `tty_keys_user` reads as the key `Usern`.
    pub(crate) fn user_key_sequences(&self, target: &str) -> Vec<String> {
        let Some(resolved) = self.resolve(target) else {
            return Vec::new();
        };
        OptionsView::two(
            self.sessions[resolved.session].option_overrides(),
            self.global_options.server(),
        )
        .array_values("user-keys")
        .into_iter()
        .map(str::to_owned)
        .collect()
    }
}

/// Compile one of the built-in bindings. The default table is source code, so
/// a body that does not compile is a bug in this file rather than user input.
fn default_binding(command: &[String]) -> ExecutableCommand {
    ExecutableCommand::compile_argv(command, &[])
        .unwrap_or_else(|error| panic!("invalid default key binding {command:?}: {error}"))
}
