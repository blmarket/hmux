use std::collections::{BTreeSet, VecDeque};

use bytes::Bytes;

use crate::integration::status::StatusHub;

use super::super::key::parse_key_name;
use super::super::state::{
    self, MenuItem, MenuRequest, ModeBindingUpdate, ModeEdit, ModePrompt, ServerState, SharedState,
};
use super::super::term::TerminalCapabilities;
use super::super::{command, format, status};
use super::super::{options, registry};
use super::{ActiveOverlay, append_view_output};

pub(crate) struct CommandPrompt {
    request: PromptRequest,
    pages: PromptPages,
    editor: PromptEditor,
    presentation: PromptPresentation,
    execution: PromptExecution,
}

struct PromptRequest {
    args: Vec<String>,
    tail: Vec<String>,
    spec: command::CommandPromptSpec,
    action: CommandPromptAction,
}

struct PromptPages {
    entries: Vec<PromptPage>,
    current: usize,
    values: Vec<String>,
}

struct PromptPage {
    label: String,
    initial: String,
}

struct PromptEditor {
    buffer: PromptBuffer,
    last: String,
    yank: Option<String>,
    history_index: usize,
    mode: PromptInputMode,
    completion: Option<PromptCompletionMenu>,
}

struct PromptBuffer {
    chars: Vec<char>,
    cursor: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PromptInputMode {
    Insert,
    ViCommand,
    QuoteNext,
}

struct PromptPresentation {
    frozen_frame: Option<Bytes>,
}

struct PromptExecution {
    owner: PromptOwner,
    deferred_incremental: VecDeque<command::DeferredCommand>,
}

enum PromptOwner {
    Attached,
    External(state::ActiveCommandPrompt),
    Resolved,
}

impl PromptPages {
    fn current(&self) -> Option<&PromptPage> {
        self.entries.get(self.current)
    }

    fn advance(&mut self, value: String) -> Option<String> {
        self.values.push(value);
        self.current += 1;
        self.current().map(|page| page.initial.clone())
    }
}

impl PromptEditor {
    fn new(initial: String, last: String) -> Self {
        let cursor = initial.chars().count();
        Self {
            buffer: PromptBuffer {
                chars: initial.chars().collect(),
                cursor,
            },
            last,
            yank: None,
            history_index: 0,
            mode: PromptInputMode::Insert,
            completion: None,
        }
    }

    fn reset(&mut self, initial: String) {
        self.last = initial.clone();
        self.buffer.chars = initial.chars().collect();
        self.buffer.cursor = self.buffer.chars.len();
        self.history_index = 0;
        self.mode = PromptInputMode::Insert;
        self.completion = None;
    }
}

impl PromptBuffer {
    fn text(&self) -> String {
        self.chars.iter().collect()
    }

    fn replace(&mut self, value: &str) {
        self.chars = value.chars().collect();
        self.cursor = self.chars.len();
    }
}

enum CommandPromptAction {
    Command,
    ModeCommand { item_target: String },
    ModeSearch { target: String },
    ModeFilter { target: String },
    ModeEdit { target: String, edit: ModeEdit },
}

struct PromptCompletionMenu {
    items: Vec<PromptCompletionItem>,
    selected: usize,
    start: usize,
    end: usize,
    replace_entire: bool,
}

struct PromptCompletionItem {
    label: String,
    replacement: String,
}

enum PromptCompletion {
    None,
    Replace(String),
    Menu {
        items: Vec<PromptCompletionItem>,
        replace_entire: bool,
    },
}

pub(super) enum CommandPromptInput {
    Continue,
    Finish(command::CommandResult),
    Cancel,
}

impl CommandPrompt {
    pub(super) fn should_freeze(&self) -> bool {
        !self.request.spec.no_freeze
    }

    pub(super) fn captures_literal_key(&self) -> bool {
        self.request.spec.key
    }

    pub(super) fn freeze(&mut self, frame: Bytes) {
        self.presentation.frozen_frame = Some(frame);
    }

    pub(super) fn frozen_frame(&self) -> Option<&Bytes> {
        self.presentation.frozen_frame.as_ref()
    }

    pub(super) fn has_completion(&self) -> bool {
        self.editor.completion.is_some()
    }

    pub(super) fn is_vi_command(&self) -> bool {
        self.editor.mode == PromptInputMode::ViCommand
    }

    pub(super) fn apply_deferred_side_effect(
        &self,
        result: &command::CommandResult,
        state: &SharedState,
    ) {
        if result.exit != 0 {
            return;
        }
        let (CommandPromptAction::ModeEdit { target, edit }, Some(value)) =
            (&self.request.action, self.pages.values.last())
        else {
            return;
        };
        {
            let mut state = state.borrow_mut();
            let _ = state.mode_view_update_edit(target, edit, value);
        }
    }

    pub(super) fn new(
        args: Vec<String>,
        external: Option<state::ActiveCommandPrompt>,
        state: &SharedState,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) -> Result<Self, String> {
        let prompt_end = args.iter().position(|arg| arg == ";");
        let prompt_args = prompt_end.map_or_else(|| args.clone(), |end| args[..end].to_vec());
        let tail = prompt_end
            .and_then(|end| args.get(end + 1..))
            .unwrap_or_default()
            .to_vec();
        let spec = command::command_prompt_spec(&prompt_args)?;
        let agents = hub.snapshot().panes;
        // One borrow for every page: each expansion installs and restores the
        // command's session, so they must not nest.
        let mut st = state.borrow_mut();
        let pages = spec
            .pages
            .iter()
            .map(|page| PromptPage {
                label: command::expand_command_prompt_format(
                    &page.label,
                    &mut st,
                    &agents,
                    context,
                ),
                initial: command::expand_command_prompt_format(
                    &page.initial,
                    &mut st,
                    &agents,
                    context,
                ),
            })
            .collect::<Vec<_>>();
        drop(st);
        let last = pages
            .first()
            .map(|page| page.initial.clone())
            .unwrap_or_default();
        let initial = if spec.incremental {
            String::new()
        } else {
            last.clone()
        };
        Ok(Self {
            request: PromptRequest {
                args: prompt_args,
                tail,
                spec,
                action: CommandPromptAction::Command,
            },
            pages: PromptPages {
                entries: pages,
                current: 0,
                values: Vec::new(),
            },
            editor: PromptEditor::new(initial, last),
            presentation: PromptPresentation { frozen_frame: None },
            execution: PromptExecution {
                owner: external.map_or(PromptOwner::Attached, PromptOwner::External),
                deferred_incremental: VecDeque::new(),
            },
        })
    }

    pub(super) fn for_mode(
        request: ModePrompt,
        target: &str,
        state: &SharedState,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) -> Result<Self, String> {
        let (args, action) = match request {
            ModePrompt::Search => (
                vec![
                    "command-prompt".to_string(),
                    "-T".to_string(),
                    "search".to_string(),
                    "-p".to_string(),
                    "(search)".to_string(),
                ],
                CommandPromptAction::ModeSearch {
                    target: target.to_string(),
                },
            ),
            ModePrompt::Filter { initial } => (
                vec![
                    "command-prompt".to_string(),
                    "-T".to_string(),
                    "search".to_string(),
                    "-I".to_string(),
                    initial,
                    "-p".to_string(),
                    "(filter)".to_string(),
                ],
                CommandPromptAction::ModeFilter {
                    target: target.to_string(),
                },
            ),
            ModePrompt::Command { item_target } => (
                vec![
                    "command-prompt".to_string(),
                    "-p".to_string(),
                    "(current)".to_string(),
                ],
                CommandPromptAction::ModeCommand { item_target },
            ),
            ModePrompt::Edit(edit) => (
                vec![
                    "command-prompt".to_string(),
                    "-I".to_string(),
                    edit.initial().to_string(),
                    "-p".to_string(),
                    edit.prompt(),
                ],
                CommandPromptAction::ModeEdit {
                    target: target.to_string(),
                    edit,
                },
            ),
        };
        let mut prompt = Self::new(args, None, state, hub, context)?;
        prompt.request.action = action;
        Ok(prompt)
    }

    fn label(&self) -> &str {
        self.pages
            .current()
            .map(|page| page.label.as_str())
            .unwrap_or(":")
    }

    pub(super) fn formatted_display(
        &self,
        state: &ServerState,
        target: &str,
        cols: usize,
    ) -> (String, usize) {
        let input = self.editor.buffer.text();
        let prefix = if let Some(message_format) = state.option_for_target(target, "message-format")
        {
            let mut vars = format::Vars::new();
            vars.set("message", self.label().to_string())
                .set("prompt-input", input.clone())
                .set(
                    "command_prompt",
                    if self.editor.mode == PromptInputMode::ViCommand {
                        "1"
                    } else {
                        "0"
                    },
                );
            format::expand(message_format, &vars)
        } else {
            self.label().to_string()
        };
        let prefix = clip_prompt_display(&prefix, 0, cols);
        let prefix_width = format::display_width(&prefix);
        let available = cols.saturating_sub(prefix_width);
        let mut rendered_input =
            render_prompt_input(&self.editor.buffer.chars[..self.editor.buffer.cursor]);
        rendered_input.push_str(&render_prompt_input(
            &self.editor.buffer.chars[self.editor.buffer.cursor..],
        ));
        let cursor_width =
            prompt_input_width(&self.editor.buffer.chars[..self.editor.buffer.cursor]);
        let offset = if cursor_width >= available && available != 0 {
            cursor_width - available + 1
        } else {
            0
        };
        let visible_input = clip_prompt_display(&rendered_input, offset, available);
        let cursor = prefix_width + cursor_width.saturating_sub(offset);
        (format!("{prefix}{visible_input}"), cursor.min(cols))
    }

    fn run(
        &self,
        values: &[String],
        state: &SharedState,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) -> command::CommandResult {
        if let Some(value) = values.last() {
            match &self.request.action {
                CommandPromptAction::ModeSearch { target } => {
                    return match state.borrow_mut().mode_view_search(target, value) {
                        Ok(()) => command::CommandResult::ok(""),
                        Err(error) => command::CommandResult::err(format!("{error}\n")),
                    };
                }
                CommandPromptAction::ModeFilter { target } => {
                    return match state.borrow_mut().mode_view_filter(target, value) {
                        Ok(()) => command::CommandResult::ok(""),
                        Err(error) => command::CommandResult::err(format!("{error}\n")),
                    };
                }
                CommandPromptAction::ModeEdit { target, edit } => {
                    return run_mode_edit(edit, value, target, state);
                }
                CommandPromptAction::ModeCommand { item_target } => {
                    return run_mode_command(value, item_target);
                }
                CommandPromptAction::Command => {}
            }
        }
        let template = command::command_prompt_template(
            &self.request.args,
            values,
            state,
            &hub.snapshot().panes,
            context,
        );
        let mut result = command::CommandResult::ok("");
        if !template.trim().is_empty() || !self.request.tail.is_empty() {
            result
                .deferred_commands
                .push(command::DeferredCommand::Line {
                    line: template,
                    tail: self.request.tail.clone(),
                });
        }
        result
    }

    /// Resolve the prompt's owner with the command's result. An external
    /// requester receives the whole result in its completion; for the
    /// attached client stdout goes to the view, and a failure comes back as
    /// the error text to show on the status line — the attached client's
    /// only error channel.
    pub(super) fn complete(
        &mut self,
        result: &command::CommandResult,
        state: &SharedState,
        context: &command::ClientContext,
    ) -> Option<String> {
        match std::mem::replace(&mut self.execution.owner, PromptOwner::Resolved) {
            PromptOwner::External(external) => {
                external.complete(state::PromptCompletion {
                    stdout: result.stdout.clone(),
                    stderr: result.stderr.clone(),
                    exit: result.exit,
                    inserted: true,
                });
                None
            }
            PromptOwner::Attached => {
                if !result.stdout_data().is_empty() {
                    if let Some(session_id) = context.current_session_id {
                        append_view_output(state, &format!("${session_id}"), result.stdout_data());
                    }
                }
                (result.exit != 0).then(|| result.stderr.clone())
            }
            PromptOwner::Resolved => None,
        }
    }

    pub(super) fn cancel_external(&mut self) {
        if let PromptOwner::External(external) =
            std::mem::replace(&mut self.execution.owner, PromptOwner::Resolved)
        {
            external.cancel();
        }
    }

    pub(super) fn initial_incremental(
        &mut self,
        state: &SharedState,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) {
        if self.request.spec.incremental {
            let mut values = self.pages.values.clone();
            values.push("=".to_string());
            let mut result = self.run(&values, state, hub, context);
            if let Some(source) = take_deferred_attach_command(&mut result) {
                self.execution.deferred_incremental.push_back(source);
            }
        }
    }

    fn changed(
        &mut self,
        prefix: char,
        state: &SharedState,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) {
        if self.request.spec.incremental {
            let mut values = self.pages.values.clone();
            values.push(format!("{prefix}{}", self.editor.buffer.text()));
            let mut result = self.run(&values, state, hub, context);
            if let Some(source) = take_deferred_attach_command(&mut result) {
                self.execution.deferred_incremental.push_back(source);
            }
        }
    }

    pub(super) fn take_deferred_incremental(&mut self) -> Option<command::DeferredCommand> {
        self.execution.deferred_incremental.pop_front()
    }

    fn finish_page(
        &mut self,
        state: &SharedState,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) -> CommandPromptInput {
        let input = self.editor.buffer.text();
        if !input.is_empty() {
            {
                let mut st = state.borrow_mut();
                st.add_prompt_history(&self.request.spec.prompt_type, &input);
            }
        }
        if self.request.spec.incremental {
            return CommandPromptInput::Cancel;
        }
        if let Some(initial) = self.pages.advance(input) {
            self.editor.reset(initial);
            return CommandPromptInput::Continue;
        }
        CommandPromptInput::Finish(self.run(&self.pages.values, state, hub, context))
    }

    fn delete_previous_word(&mut self, separators: &str) {
        if self.editor.buffer.cursor == 0 {
            self.editor.yank = Some(String::new());
            return;
        }
        let class = |character: char| {
            if character == ' ' {
                0
            } else if separators.contains(character) {
                1
            } else {
                2
            }
        };
        let mut start = self.editor.buffer.cursor;
        while start > 0 && self.editor.buffer.chars[start - 1] == ' ' {
            start -= 1;
        }
        if start > 0 {
            let wanted = class(self.editor.buffer.chars[start - 1]);
            while start > 0 && class(self.editor.buffer.chars[start - 1]) == wanted {
                start -= 1;
            }
        }
        self.editor.yank = Some(
            self.editor.buffer.chars[start..self.editor.buffer.cursor]
                .iter()
                .collect(),
        );
        self.editor
            .buffer
            .chars
            .drain(start..self.editor.buffer.cursor);
        self.editor.buffer.cursor = start;
    }

    fn move_word_forward(&mut self, vi: bool, separators: &str) {
        let size = self.editor.buffer.chars.len();
        let mut index = self.editor.buffer.cursor;
        if !vi {
            while index != size && self.editor.buffer.chars[index] == ' ' {
                index += 1;
            }
        }
        if index == size {
            self.editor.buffer.cursor = index;
            return;
        }
        let separator = separators.contains(self.editor.buffer.chars[index])
            && self.editor.buffer.chars[index] != ' ';
        loop {
            index += 1;
            if index == size {
                break;
            }
            if self.editor.buffer.chars[index] == ' ' {
                if vi {
                    while index != size && self.editor.buffer.chars[index] == ' ' {
                        index += 1;
                    }
                }
                break;
            }
            if separator != separators.contains(self.editor.buffer.chars[index]) {
                break;
            }
        }
        self.editor.buffer.cursor = index;
    }

    fn move_word_end(&mut self, separators: &str) {
        let size = self.editor.buffer.chars.len();
        let mut index = self.editor.buffer.cursor;
        if index == size {
            return;
        }
        loop {
            index += 1;
            if index == size {
                self.editor.buffer.cursor = index;
                return;
            }
            if self.editor.buffer.chars[index] != ' ' {
                break;
            }
        }
        let separator = separators.contains(self.editor.buffer.chars[index]);
        loop {
            index += 1;
            if index == size
                || self.editor.buffer.chars[index] == ' '
                || separator != separators.contains(self.editor.buffer.chars[index])
            {
                break;
            }
        }
        self.editor.buffer.cursor = index.saturating_sub(1);
    }

    fn move_word_backward(&mut self, separators: &str) {
        let mut index = self.editor.buffer.cursor;
        while index != 0 {
            index -= 1;
            if self.editor.buffer.chars[index] != ' ' {
                break;
            }
        }
        let separator = self
            .editor
            .buffer
            .chars
            .get(index)
            .is_some_and(|character| separators.contains(*character));
        while index != 0 {
            index -= 1;
            if self.editor.buffer.chars[index] == ' '
                || separator != separators.contains(self.editor.buffer.chars[index])
            {
                index += 1;
                break;
            }
        }
        self.editor.buffer.cursor = index;
    }

    fn paste(&mut self, state: &SharedState) {
        let source = self.editor.yank.clone().unwrap_or_else(|| {
            state
                .borrow_mut()
                .buffer(None)
                .map(prompt_paste_text)
                .unwrap_or_default()
        });
        let inserted = source.chars().collect::<Vec<_>>();
        self.editor.buffer.chars.splice(
            self.editor.buffer.cursor..self.editor.buffer.cursor,
            inserted.iter().copied(),
        );
        self.editor.buffer.cursor += inserted.len();
    }

    fn replace_completion(&mut self, start: usize, end: usize, replacement: &str) {
        self.editor
            .buffer
            .chars
            .splice(start..end, replacement.chars());
        self.editor.buffer.cursor = start + replacement.chars().count();
    }

    fn complete_word(&mut self, state: &SharedState, context: &command::ClientContext) -> bool {
        let Some((start, end)) =
            prompt_word_range(&self.editor.buffer.chars, self.editor.buffer.cursor)
        else {
            return false;
        };
        let word = self.editor.buffer.chars[start..end]
            .iter()
            .collect::<String>();
        if word.len() >= 64 {
            return false;
        }
        let completion = command_prompt_completion(
            &state.borrow_mut(),
            context,
            &self.request.spec.prompt_type,
            &word,
            start == 0,
        );
        match completion {
            PromptCompletion::None => false,
            PromptCompletion::Replace(replacement) => {
                self.replace_completion(start, end, &replacement);
                true
            }
            PromptCompletion::Menu {
                items,
                replace_entire,
            } => {
                self.editor.completion = Some(PromptCompletionMenu {
                    items,
                    selected: 0,
                    start,
                    end,
                    replace_entire,
                });
                false
            }
        }
    }

    fn handle_completion_key(&mut self, key: &str) -> bool {
        let Some(menu) = self.editor.completion.as_mut() else {
            return false;
        };
        let last = menu.items.len().saturating_sub(1);
        let mut choose = None;
        let mut close = false;
        if let Some(index) = key
            .chars()
            .next()
            .filter(|_| key.chars().count() == 1)
            .and_then(|character| character.to_digit(10))
            .map(|index| index as usize)
            .filter(|index| *index < menu.items.len())
        {
            choose = Some(index);
            close = true;
        } else {
            match key {
                "BTab" | "Up" | "k" => {
                    menu.selected = if menu.selected == 0 {
                        last
                    } else {
                        menu.selected - 1
                    };
                }
                "Tab" => {
                    if menu.selected == last {
                        close = true;
                    } else {
                        menu.selected += 1;
                    }
                }
                "Down" | "j" => {
                    menu.selected = if menu.selected == last {
                        0
                    } else {
                        menu.selected + 1
                    };
                }
                "PPage" | "C-b" => menu.selected = menu.selected.saturating_sub(5),
                "NPage" => menu.selected = (menu.selected + 5).min(last),
                "Home" | "g" => menu.selected = 0,
                "End" | "G" => menu.selected = last,
                "Enter" | "C-m" => {
                    choose = Some(menu.selected);
                    close = true;
                }
                "BSpace" | "Escape" | "C-[" | "C-c" | "C-g" | "q" => close = true,
                _ => {}
            }
        }
        if close {
            let menu = self
                .editor
                .completion
                .take()
                .expect("completion menu checked");
            if let Some(item) = choose.and_then(|index| menu.items.get(index)) {
                if menu.replace_entire {
                    self.editor.buffer.replace(&item.replacement);
                } else {
                    self.replace_completion(menu.start, menu.end, &item.replacement);
                }
            }
        }
        true
    }

    pub(super) fn handle_key(
        &mut self,
        key: &str,
        state: &SharedState,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) -> CommandPromptInput {
        if self.handle_completion_key(key) {
            return CommandPromptInput::Continue;
        }
        if self.request.spec.key {
            self.editor.buffer.replace(key);
            return self.finish_page(state, hub, context);
        }
        if self.request.spec.numeric {
            if key.chars().count() == 1
                && key
                    .chars()
                    .next()
                    .is_some_and(|character| character.is_ascii_digit())
            {
                self.editor
                    .buffer
                    .chars
                    .push(key.chars().next().expect("numeric key checked"));
                self.editor.buffer.cursor = self.editor.buffer.chars.len();
                return CommandPromptInput::Continue;
            }
            return self.finish_page(state, hub, context);
        }
        if self.request.spec.single || self.editor.mode == PromptInputMode::QuoteNext {
            self.editor.mode = PromptInputMode::Insert;
            let character = match key {
                "C-Space" => Some('\0'),
                "BSpace" => Some('\u{7f}'),
                "Space" => Some(' '),
                _ if key.starts_with("C-") && key.chars().count() == 3 => key
                    .chars()
                    .nth(2)
                    .map(|character| ((character.to_ascii_lowercase() as u8) & 0x1f) as char),
                _ if key.chars().count() == 1 => key.chars().next(),
                _ => None,
            };
            if let Some(character) = character {
                self.editor
                    .buffer
                    .chars
                    .insert(self.editor.buffer.cursor, character);
                self.editor.buffer.cursor += 1;
                self.changed('=', state, hub, context);
                if self.request.spec.single {
                    return if self.editor.buffer.chars.len() == 1 {
                        self.finish_page(state, hub, context)
                    } else {
                        CommandPromptInput::Cancel
                    };
                }
            }
            return CommandPromptInput::Continue;
        }

        let target = context
            .current_session_id
            .map(|id| format!("${id}"))
            .unwrap_or_default();
        let separators = state
            .borrow_mut()
            .option_for_target(&target, "word-separators")
            .map(str::to_string)
            .unwrap_or_else(|| " !\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~".to_string());
        let vi_keys = state.borrow_mut().option_for_target(&target, "status-keys") == Some("vi");
        if self.editor.mode == PromptInputMode::ViCommand {
            match key {
                "i" => {
                    self.editor.mode = PromptInputMode::Insert;
                    return CommandPromptInput::Continue;
                }
                "a" => {
                    self.editor.buffer.cursor =
                        (self.editor.buffer.cursor + 1).min(self.editor.buffer.chars.len());
                    self.editor.mode = PromptInputMode::Insert;
                    return CommandPromptInput::Continue;
                }
                "A" => {
                    self.editor.buffer.cursor = self.editor.buffer.chars.len();
                    self.editor.mode = PromptInputMode::Insert;
                    return CommandPromptInput::Continue;
                }
                "I" => {
                    self.editor.buffer.cursor = 0;
                    self.editor.mode = PromptInputMode::Insert;
                    return CommandPromptInput::Continue;
                }
                "C" => {
                    self.editor.mode = PromptInputMode::Insert;
                    if self.editor.buffer.cursor < self.editor.buffer.chars.len() {
                        self.editor.buffer.chars.truncate(self.editor.buffer.cursor);
                        self.changed('=', state, hub, context);
                    }
                    return CommandPromptInput::Continue;
                }
                "s" => {
                    self.editor.mode = PromptInputMode::Insert;
                    if self.editor.buffer.cursor < self.editor.buffer.chars.len() {
                        self.editor.buffer.chars.remove(self.editor.buffer.cursor);
                        self.changed('=', state, hub, context);
                    }
                    return CommandPromptInput::Continue;
                }
                "S" => {
                    self.editor.mode = PromptInputMode::Insert;
                    self.editor.buffer.chars.clear();
                    self.editor.buffer.cursor = 0;
                    self.changed('=', state, hub, context);
                    return CommandPromptInput::Continue;
                }
                "Escape" | "C-[" => return CommandPromptInput::Continue,
                "$" => self.editor.buffer.cursor = self.editor.buffer.chars.len(),
                "0" | "^" => self.editor.buffer.cursor = 0,
                "h" | "BSpace" => {
                    self.editor.buffer.cursor = self.editor.buffer.cursor.saturating_sub(1)
                }
                "l" | "Right" => {
                    self.editor.buffer.cursor =
                        (self.editor.buffer.cursor + 1).min(self.editor.buffer.chars.len())
                }
                "x" | "DC" if self.editor.buffer.cursor < self.editor.buffer.chars.len() => {
                    self.editor.buffer.chars.remove(self.editor.buffer.cursor);
                    self.changed('=', state, hub, context);
                }
                "X" | "C-h" => {
                    if self.editor.buffer.chars.is_empty() && self.request.spec.backspace_exit {
                        return CommandPromptInput::Cancel;
                    }
                    if self.editor.buffer.cursor > 0 {
                        self.editor.buffer.cursor -= 1;
                        self.editor.buffer.chars.remove(self.editor.buffer.cursor);
                        self.changed('=', state, hub, context);
                    }
                }
                "D" if self.editor.buffer.cursor < self.editor.buffer.chars.len() => {
                    self.editor.buffer.chars.truncate(self.editor.buffer.cursor);
                    self.changed('=', state, hub, context);
                }
                "d" => {
                    self.editor.buffer.chars.clear();
                    self.editor.buffer.cursor = 0;
                    self.changed('=', state, hub, context);
                }
                "b" => {
                    self.move_word_backward(&separators);
                    self.changed('=', state, hub, context);
                }
                "B" => {
                    self.move_word_backward("");
                    self.changed('=', state, hub, context);
                }
                "e" => {
                    self.move_word_end(&separators);
                    self.changed('=', state, hub, context);
                }
                "E" => {
                    self.move_word_end("");
                    self.changed('=', state, hub, context);
                }
                "w" => {
                    self.move_word_forward(true, &separators);
                    self.changed('=', state, hub, context);
                }
                "W" => {
                    self.move_word_forward(true, "");
                    self.changed('=', state, hub, context);
                }
                "p" => {
                    self.paste(state);
                    self.changed('=', state, hub, context);
                }
                "Up" | "k" => {
                    {
                        let st = state.borrow_mut();
                        let history = st.prompt_history(&self.request.spec.prompt_type);
                        if self.editor.history_index < history.len() {
                            self.editor.history_index += 1;
                            self.editor
                                .buffer
                                .replace(&history[history.len() - self.editor.history_index]);
                        }
                    }
                    self.changed('=', state, hub, context);
                }
                "Down" | "j" if self.editor.history_index > 0 => {
                    self.editor.history_index -= 1;
                    {
                        let st = state.borrow_mut();
                        let history = st.prompt_history(&self.request.spec.prompt_type);
                        let value = if self.editor.history_index == 0 {
                            ""
                        } else {
                            &history[history.len() - self.editor.history_index]
                        };
                        self.editor.buffer.replace(value);
                    }
                    self.changed('=', state, hub, context);
                }
                "q" | "C-c" => return CommandPromptInput::Cancel,
                "Enter" | "C-m" => return self.finish_page(state, hub, context),
                _ => {}
            }
            return CommandPromptInput::Continue;
        }
        if vi_keys
            && !matches!(
                key,
                "C-a"
                    | "C-c"
                    | "C-e"
                    | "C-g"
                    | "C-h"
                    | "Tab"
                    | "C-k"
                    | "C-n"
                    | "C-p"
                    | "C-t"
                    | "C-u"
                    | "C-v"
                    | "C-w"
                    | "C-y"
                    | "Space"
                    | "Enter"
                    | "C-m"
                    | "C-Left"
                    | "C-Right"
                    | "BSpace"
                    | "DC"
                    | "Down"
                    | "End"
                    | "Home"
                    | "Left"
                    | "Right"
                    | "Up"
                    | "Escape"
                    | "C-["
            )
            && (key.chars().count() != 1 || key.chars().next().is_some_and(char::is_control))
        {
            return CommandPromptInput::Continue;
        }
        match key {
            "Enter" | "C-m" => return self.finish_page(state, hub, context),
            "Escape" | "C-[" => {
                if vi_keys {
                    self.editor.mode = PromptInputMode::ViCommand;
                    self.editor.buffer.cursor = self.editor.buffer.cursor.saturating_sub(1);
                    return CommandPromptInput::Continue;
                }
                return CommandPromptInput::Cancel;
            }
            "C-c" | "C-g" => return CommandPromptInput::Cancel,
            "Left" | "C-b" => {
                self.editor.buffer.cursor = self.editor.buffer.cursor.saturating_sub(1)
            }
            "Right" | "C-f" => {
                self.editor.buffer.cursor =
                    (self.editor.buffer.cursor + 1).min(self.editor.buffer.chars.len())
            }
            "Home" | "C-a" => self.editor.buffer.cursor = 0,
            "End" | "C-e" => self.editor.buffer.cursor = self.editor.buffer.chars.len(),
            "BSpace" | "C-h" => {
                if self.editor.buffer.cursor == 0 && self.request.spec.backspace_exit {
                    return CommandPromptInput::Cancel;
                }
                if self.editor.buffer.cursor > 0 {
                    self.editor.buffer.cursor -= 1;
                    self.editor.buffer.chars.remove(self.editor.buffer.cursor);
                    self.changed('=', state, hub, context);
                }
            }
            "DC" | "C-d" => {
                if self.editor.buffer.cursor < self.editor.buffer.chars.len() {
                    self.editor.buffer.chars.remove(self.editor.buffer.cursor);
                    self.changed('=', state, hub, context);
                }
            }
            "C-u" => {
                self.editor.buffer.chars.clear();
                self.editor.buffer.cursor = 0;
                self.changed('=', state, hub, context);
            }
            "C-k" => {
                self.editor.buffer.chars.truncate(self.editor.buffer.cursor);
                self.changed('=', state, hub, context);
            }
            "C-w" => {
                self.delete_previous_word(&separators);
                self.changed('=', state, hub, context);
            }
            "C-y" => {
                self.paste(state);
                self.changed('=', state, hub, context);
            }
            "C-t" => {
                let end = if self.editor.buffer.cursor < self.editor.buffer.chars.len() {
                    self.editor.buffer.cursor + 1
                } else {
                    self.editor.buffer.cursor
                };
                if end >= 2 {
                    self.editor.buffer.chars.swap(end - 2, end - 1);
                    self.editor.buffer.cursor = end;
                    self.changed('=', state, hub, context);
                }
            }
            "C-v" => self.editor.mode = PromptInputMode::QuoteNext,
            "Tab" => {
                if self.complete_word(state, context) {
                    self.changed('=', state, hub, context);
                }
            }
            "M-b" | "C-Left" => {
                self.move_word_backward(&separators);
                self.changed('=', state, hub, context);
            }
            "M-f" | "C-Right" => {
                self.move_word_forward(false, &separators);
                self.changed('=', state, hub, context);
            }
            "Up" | "C-p" => {
                {
                    let st = state.borrow_mut();
                    let history = st.prompt_history(&self.request.spec.prompt_type);
                    if self.editor.history_index < history.len() {
                        self.editor.history_index += 1;
                        self.editor
                            .buffer
                            .replace(&history[history.len() - self.editor.history_index]);
                    }
                }
                self.changed('=', state, hub, context);
            }
            "Down" | "C-n" => {
                if self.editor.history_index > 0 {
                    self.editor.history_index -= 1;
                    {
                        let st = state.borrow_mut();
                        let history = st.prompt_history(&self.request.spec.prompt_type);
                        let value = if self.editor.history_index == 0 {
                            ""
                        } else {
                            &history[history.len() - self.editor.history_index]
                        };
                        self.editor.buffer.replace(value);
                    }
                    self.changed('=', state, hub, context);
                }
            }
            "C-r" if self.request.spec.incremental => {
                let prefix = if self.editor.buffer.chars.is_empty() {
                    self.editor.buffer.replace(&self.editor.last);
                    '='
                } else {
                    '-'
                };
                self.changed(prefix, state, hub, context);
            }
            "C-s" if self.request.spec.incremental => {
                let prefix = if self.editor.buffer.chars.is_empty() {
                    self.editor.buffer.replace(&self.editor.last);
                    '='
                } else {
                    '+'
                };
                self.changed(prefix, state, hub, context);
            }
            _ => {
                let text = match key {
                    "Space" => Some(" "),
                    _ if key.chars().count() == 1
                        && !key.chars().next().is_some_and(char::is_control) =>
                    {
                        Some(key)
                    }
                    _ => None,
                };
                if let Some(text) = text {
                    let inserted = text.chars().collect::<Vec<_>>();
                    self.editor.buffer.chars.splice(
                        self.editor.buffer.cursor..self.editor.buffer.cursor,
                        inserted.iter().copied(),
                    );
                    self.editor.buffer.cursor += inserted.len();
                    self.changed('=', state, hub, context);
                    if self.request.spec.single {
                        return self.finish_page(state, hub, context);
                    }
                }
            }
        }
        CommandPromptInput::Continue
    }
}

fn run_mode_command(value: &str, item_target: &str) -> command::CommandResult {
    if value.is_empty() {
        return command::CommandResult::ok("");
    }
    let line = command::replace_prompt_template(value, item_target, 1);
    let mut result = command::CommandResult::ok("");
    if !line.trim().is_empty() {
        result
            .deferred_commands
            .push(command::DeferredCommand::Line {
                line,
                tail: Vec::new(),
            });
    }
    result
}

fn run_mode_edit(
    edit: &ModeEdit,
    value: &str,
    target: &str,
    state: &SharedState,
) -> command::CommandResult {
    match edit {
        ModeEdit::Option { name, .. } => {
            let args = vec![
                "set-option".to_string(),
                "-t".to_string(),
                target.to_string(),
                "--".to_string(),
                name.clone(),
                value.to_string(),
            ];
            let mut result = command::CommandResult::ok("");
            result
                .deferred_commands
                .push(command::DeferredCommand::Argv(args));
            result
        }
        ModeEdit::BindingCommand {
            table,
            key,
            note,
            repeat,
            ..
        } => {
            if value.is_empty() {
                return command::CommandResult::ok("");
            }
            let aliases = {
                let state = state.borrow_mut();
                state.command_aliases()
            };
            let compiled = match command::ExecutableCommand::compile(value, &aliases) {
                Ok(compiled) => compiled,
                Err(error) => return command::CommandResult::err(error),
            };
            let Some(key_code) = parse_key_name(key) else {
                return command::CommandResult::err(format!("unknown key: {key}\n"));
            };
            let mut state = {
                let state = state.borrow_mut();
                state
            };
            let commands = compiled.argv();
            let display = command::display_command(&commands);
            state.bind_key(table, key_code, compiled, *repeat, note.clone());
            let _ = state.mode_view_update_binding(
                target,
                ModeBindingUpdate {
                    table: table.clone(),
                    key: key.clone(),
                    command_text: display,
                    command: commands,
                    note: note.clone(),
                    repeat: *repeat,
                },
            );
            command::CommandResult::ok("")
        }
        ModeEdit::BindingNote {
            table,
            key,
            command: commands,
            repeat,
            ..
        } => {
            if value.is_empty() {
                return command::CommandResult::ok("");
            }
            let Some(key_code) = parse_key_name(key) else {
                return command::CommandResult::err(format!("unknown key: {key}\n"));
            };
            let compiled = match command::ExecutableCommand::compile_argv(commands, &[]) {
                Ok(compiled) => compiled,
                Err(error) => return command::CommandResult::err(error),
            };
            let mut state = {
                let state = state.borrow_mut();
                state
            };
            state.bind_key(table, key_code, compiled, *repeat, Some(value.to_string()));
            let display = command::display_command(commands);
            let _ = state.mode_view_update_binding(
                target,
                ModeBindingUpdate {
                    table: table.clone(),
                    key: key.clone(),
                    command_text: display,
                    command: commands.clone(),
                    note: Some(value.to_string()),
                    repeat: *repeat,
                },
            );
            command::CommandResult::ok("")
        }
    }
}

pub(super) fn render_prompt_input(input: &[char]) -> String {
    let mut rendered = String::new();
    for character in input {
        match *character as u32 {
            0x00..=0x1f => {
                rendered.push('^');
                rendered.push(char::from_u32((*character as u32) | 0x40).unwrap_or('?'));
            }
            0x7f => rendered.push_str("^?"),
            0x23 => rendered.push_str("##"),
            _ => rendered.push(*character),
        }
    }
    rendered
}

pub(super) fn prompt_input_width(input: &[char]) -> usize {
    format::display_width(&render_prompt_input(input))
}

pub(super) fn clip_prompt_display(value: &str, offset: usize, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut position = 0;
    let mut output = String::new();
    for (token, token_width) in format::display_tokens(value) {
        if token_width == 0 {
            output.push_str(token);
            continue;
        }
        let end = position + token_width;
        if end > offset && end <= offset + width {
            output.push_str(token);
        } else if end > offset + width {
            break;
        }
        position = end;
    }
    output
}

fn common_prompt_prefix(values: &[String]) -> Option<String> {
    let mut prefix = values.first()?.chars().collect::<Vec<_>>();
    for value in &values[1..] {
        let common = prefix
            .iter()
            .copied()
            .zip(value.chars())
            .take_while(|(left, right)| left == right)
            .count();
        prefix.truncate(common);
    }
    Some(prefix.into_iter().collect())
}

fn prompt_paste_text(data: &[u8]) -> String {
    let mut output = String::new();
    let mut offset = 0;
    while offset < data.len() {
        let first = data[offset];
        if first.is_ascii() {
            if first <= 0x1f || first == 0x7f {
                break;
            }
            output.push(first as char);
            offset += 1;
            continue;
        }
        let width = match first {
            0xc2..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf4 => 4,
            _ => break,
        };
        let Some(bytes) = data.get(offset..offset + width) else {
            break;
        };
        let Ok(value) = std::str::from_utf8(bytes) else {
            break;
        };
        output.push_str(value);
        offset += width;
    }
    output
}

fn prompt_word_range(input: &[char], cursor: usize) -> Option<(usize, usize)> {
    if input.is_empty() {
        return Some((0, 0));
    }
    let index = cursor.saturating_sub(1).min(input.len() - 1);
    let mut first = index;
    while first > 0 && input[first] != ' ' {
        first -= 1;
    }
    while first < input.len() && input[first] == ' ' {
        first += 1;
    }
    let mut last = index;
    while last < input.len() && input[last] != ' ' {
        last += 1;
    }
    while last > 0 && last < input.len() && input[last] == ' ' {
        last -= 1;
    }
    if last < input.len() {
        last += 1;
    }
    (last >= first).then_some((first, last))
}

fn completion_menu(mut values: Vec<String>, prefix: &str) -> PromptCompletion {
    if values.len() > 10 {
        values.drain(..values.len() - 10);
    }
    PromptCompletion::Menu {
        items: values
            .into_iter()
            .map(|value| PromptCompletionItem {
                label: value.clone(),
                replacement: format!("{prefix}{value}"),
            })
            .collect(),
        replace_entire: false,
    }
}

fn command_prompt_completion(
    state: &ServerState,
    context: &command::ClientContext,
    prompt_type: &str,
    word: &str,
    at_start: bool,
) -> PromptCompletion {
    if !matches!(prompt_type, "target" | "window-target")
        && !word.starts_with("-t")
        && !word.starts_with("-s")
    {
        if word.is_empty() {
            return PromptCompletion::None;
        }
        let mut matches = BTreeSet::new();
        for candidate in registry::COMMAND_SPECS
            .iter()
            .flat_map(|spec| std::iter::once(spec.name).chain(spec.alias.iter().copied()))
        {
            if candidate.starts_with(word) {
                matches.insert(candidate.to_string());
            }
        }
        for (alias, _) in state.command_aliases() {
            if alias.starts_with(word) {
                matches.insert(alias);
            }
        }
        if !at_start {
            for candidate in options::option_names().chain(command::LAYOUT_NAMES.iter().copied()) {
                if candidate.starts_with(word) {
                    matches.insert(candidate.to_string());
                }
            }
        }
        let matches = matches.into_iter().collect::<Vec<_>>();
        return match matches.as_slice() {
            [] => PromptCompletion::None,
            [only] => PromptCompletion::Replace(format!("{only} ")),
            _ => match common_prompt_prefix(&matches) {
                Some(prefix) if prefix != word => PromptCompletion::Replace(prefix),
                _ => completion_menu(matches, ""),
            },
        };
    }

    if prompt_type == "window-target" {
        let Some(session_index) = context
            .current_session_id
            .and_then(|id| state.sessions().iter().position(|session| session.id == id))
        else {
            return PromptCompletion::None;
        };
        let session = &state.sessions()[session_index];
        let mut items = session
            .windows
            .iter()
            .enumerate()
            .filter(|(_, link)| link.index.to_string().starts_with(word))
            .map(|(window_index, link)| PromptCompletionItem {
                label: format!(
                    "{} ({})",
                    link.index,
                    state.window(session_index, window_index).name
                ),
                replacement: link.index.to_string(),
            })
            .collect::<Vec<_>>();
        items.truncate(10);
        return match items.len() {
            0 => PromptCompletion::None,
            1 if items[0].replacement == word => PromptCompletion::None,
            1 => PromptCompletion::Replace(items[0].replacement.clone()),
            _ => PromptCompletion::Menu {
                items,
                replace_entire: true,
            },
        };
    }

    let (flag, target) = if let Some(target) = word.strip_prefix("-t") {
        ("-t", target)
    } else if let Some(target) = word.strip_prefix("-s") {
        ("-s", target)
    } else {
        ("", word)
    };
    let Some(colon) = target.find(':') else {
        let mut matches = state
            .sessions()
            .iter()
            .filter_map(|session| {
                if target.starts_with('$') {
                    let candidate = format!("${}:", session.id);
                    candidate.starts_with(target).then_some(candidate)
                } else {
                    let candidate = format!("{}:", session.name);
                    candidate.starts_with(target).then_some(candidate)
                }
            })
            .collect::<Vec<_>>();
        matches.sort();
        matches.dedup();
        return match matches.as_slice() {
            [] => PromptCompletion::None,
            _ => match common_prompt_prefix(&matches) {
                Some(prefix) if format!("{flag}{prefix}") != word => {
                    PromptCompletion::Replace(format!("{flag}{prefix}"))
                }
                _ => completion_menu(matches, flag),
            },
        };
    };
    if target[colon + 1..].contains('.') {
        return PromptCompletion::None;
    }
    let session_target = &target[..colon];
    let session_index = if session_target.is_empty() {
        context
            .current_session_id
            .and_then(|id| state.sessions().iter().position(|session| session.id == id))
    } else if let Some(id) = session_target
        .strip_prefix('$')
        .and_then(|id| id.parse::<u32>().ok())
    {
        state.sessions().iter().position(|session| session.id == id)
    } else {
        state
            .sessions()
            .iter()
            .position(|session| session.name == session_target)
    };
    let Some(session_index) = session_index else {
        return PromptCompletion::None;
    };
    let session = &state.sessions()[session_index];
    let window_prefix = &target[colon + 1..];
    let mut items = session
        .windows
        .iter()
        .enumerate()
        .filter(|(_, link)| link.index.to_string().starts_with(window_prefix))
        .map(|(window_index, link)| PromptCompletionItem {
            label: format!(
                "{}:{} ({})",
                session.name,
                link.index,
                state.window(session_index, window_index).name
            ),
            replacement: format!("{flag}{}:{}", session.name, link.index),
        })
        .collect::<Vec<_>>();
    items.truncate(10);
    match items.len() {
        0 => PromptCompletion::None,
        1 if items[0].replacement == word => PromptCompletion::None,
        1 => PromptCompletion::Replace(items[0].replacement.clone()),
        _ => PromptCompletion::Menu {
            items,
            replace_entire: false,
        },
    }
}

pub(super) fn take_deferred_attach_command(
    result: &mut command::CommandResult,
) -> Option<command::DeferredCommand> {
    result.deferred_commands.pop()
}

pub(super) fn render_prompt_completion(
    prompt: &CommandPrompt,
    state: &ServerState,
    target: &str,
    cols: u16,
    rows: u16,
    status_height: u16,
    terminal: &dyn TerminalCapabilities,
) -> Vec<u8> {
    let Some(completion) = prompt.editor.completion.as_ref() else {
        return Vec::new();
    };
    let label_width = completion
        .items
        .iter()
        .map(|item| format::display_width(&item.label))
        .max()
        .unwrap_or(0);
    let items = completion
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| MenuItem {
            label: format!(
                "{}{}",
                item.label,
                " ".repeat(label_width.saturating_sub(format::display_width(&item.label)))
            ),
            key: char::from(b'0' + index.min(9) as u8).to_string(),
            command: Vec::new(),
        })
        .collect::<Vec<_>>();
    let height = (items.len() + 2).min(rows as usize).max(3) as u16;
    let left = prompt_input_width(&prompt.editor.buffer.chars[..completion.start])
        .saturating_add(format::display_width(prompt.label()))
        .saturating_sub(2)
        .min(usize::from(cols.saturating_sub(1)));
    let top = if status::at_top(state, target) {
        status_height
    } else {
        rows.saturating_sub(height.saturating_add(status_height))
    };
    let overlay = ActiveOverlay::menu(
        MenuRequest {
            title: String::new(),
            items,
            selected: completion.selected,
            x: Some(left.to_string()),
            // A menu's `-y` names the row below its last line, so the top row
            // this menu wants is written as the row past its bottom.
            y: Some(top.saturating_add(height).to_string()),
            pane: None,
            mouse: None,
        },
        completion.selected,
    );
    overlay.render(state, target, cols, rows, terminal)
}
