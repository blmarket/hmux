//! The paste-buffer commands.

use bytes::Bytes;

use super::*;

#[derive(Clone, Debug)]
pub(in crate::server) enum Command {
    Set(SetBuffer),
    Load(LoadBuffer),
    Show(ShowBuffer),
    Save(SaveBuffer),
    List(ListBuffers),
    Delete(DeleteBuffer),
    Paste(PasteBuffer),
}

impl Command {
    pub(super) async fn execute(self, context: &mut ExecContext<'_>) -> SharedCommandExecution {
        match self {
            // `load-buffer` and `save-buffer` name a path, which may be a FIFO
            // whose peer is another client of this very server.
            Self::Load(command) => command.execute(context).await,
            Self::Save(command) => command.execute(context).await,
            Self::Set(command) => context.sync(|inner| command.run(inner.state, inner.client)),
            Self::Show(command) => context.sync(|inner| command.run(inner.state)),
            Self::List(command) => context.sync(|inner| command.run(inner.state)),
            Self::Delete(command) => context.sync(|inner| command.run(inner.state)),
            Self::Paste(command) => context.sync(|inner| command.run(inner.state)),
        }
    }
}

/// `set-buffer [-aw] [-b buffer-name] [-n new-buffer-name] [-t target-client] [data]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SetBuffer {
    /// `-a`: append to the buffer instead of replacing it.
    append: bool,
    /// `-w`: also send the buffer to the target client's terminal selection.
    write_selection: bool,
    /// `-b`: the buffer to write; the most recent one otherwise.
    buffer: Option<String>,
    /// `-n`: rename the buffer instead of writing data to it.
    new_name: Option<String>,
    /// `-t`: the client whose selection `-w` sets.
    target: Option<String>,
    data: Option<String>,
}

impl SetBuffer {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            append: args.has('a'),
            write_selection: args.has('w'),
            buffer: args.value('b').map(str::to_string),
            new_name: args.value('n').map(str::to_string),
            target: args.value('t').map(str::to_string),
            data: args.positionals().first().cloned(),
        })
    }

    /// Stores or renames a paste buffer.
    fn run(self, st: &mut ServerState, context: &ClientContext) -> CommandResult {
        if let Some(new_name) = self.new_name.as_deref() {
            let name = self
                .buffer
                .clone()
                .or_else(|| st.automatic_buffer_name().map(str::to_string));
            return match name.as_deref() {
                Some(name) if st.rename_buffer(name, new_name) => CommandResult::ok(""),
                Some(name) => CommandResult::err(format!("unknown buffer: {name}\n")),
                None => CommandResult::err("no buffer\n"),
            };
        }
        let name = self.buffer.as_deref();
        match self.data.as_deref() {
            Some("") => CommandResult::ok(""),
            Some(data) => {
                if self.append {
                    if name.is_some() {
                        st.append_buffer(name, data.as_bytes());
                    } else {
                        st.set_buffer(None, data.as_bytes());
                    }
                } else {
                    st.set_buffer(name, data.as_bytes());
                }
                if self.write_selection {
                    if let Some(data) = st.buffer(name).map(<[u8]>::to_vec) {
                        let _ = st.set_client_selection(
                            self.target.as_deref(),
                            context.tty_name.as_deref(),
                            Some(data),
                        );
                    }
                }
                CommandResult::ok("")
            }
            None => CommandResult::err("no data specified\n"),
        }
    }
}

/// `load-buffer [-w] [-b buffer-name] [-t target-client] path`.
#[derive(Clone, Debug)]
pub(in crate::server) struct LoadBuffer {
    /// `-b`: the buffer the file's contents become.
    buffer: Option<String>,
    /// `-t`: the client whose selection `-w` sets.
    target: Option<String>,
    /// `-w`: also send the loaded data to that client's terminal selection.
    write_selection: bool,
    path: String,
}

impl LoadBuffer {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        let path = args
            .positionals()
            .first()
            .cloned()
            .ok_or("command load-buffer: too few arguments (need at least 1)\n")?;
        Ok(Self {
            buffer: args.value('b').map(str::to_string),
            target: args.value('t').map(str::to_string),
            write_selection: args.has('w'),
            path,
        })
    }

    async fn execute(self, context: &mut ExecContext<'_>) -> SharedCommandExecution {
        // Without the client's own read already in hand, the file is this
        // command's to read — which is where it waits.
        if context.client().input_file.is_none() {
            let path = if self.path == "-" {
                PathBuf::from(&self.path)
            } else {
                client_file_path(&self.path, context.client())
            };
            let input_file = suspend::load_buffer(context.tasks(), path).await;
            let deferred_error = input_file.is_err();
            let mut client = context.client().clone();
            client.input_file = Some(input_file);
            let mut result = context.run_sync(&client, |inner| self.run(inner.state, inner.client));
            result.continue_queue |= deferred_error && result.exit != 0;
            return SharedCommandExecution::completed(result);
        }
        let deferred_error = context
            .client()
            .input_file
            .as_ref()
            .is_some_and(Result::is_err);
        let mut result = context.sync(|inner| self.run(inner.state, inner.client));
        result.result.continue_queue |= deferred_error && result.result.exit != 0;
        result
    }

    fn run(self, st: &mut ServerState, context: &ClientContext) -> CommandResult {
        // tmux's `file_read` keeps `-` verbatim and otherwise stores the expanded
        // path, which is also what its error message reports.
        let resolved = if self.path == "-" {
            PathBuf::from(&self.path)
        } else {
            client_file_path(&self.path, context)
        };
        let loaded = context
            .input_file
            .clone()
            .map(|result| result.map_err(std::io::Error::from_raw_os_error))
            .unwrap_or_else(|| std::fs::read(&resolved));
        match loaded {
            Ok(data) => {
                if !data.is_empty() {
                    st.set_buffer(self.buffer.as_deref(), &data);
                    // `-w` additionally writes the loaded buffer to the target
                    // client's terminal selection, exactly as `set-buffer -w` does.
                    if self.write_selection {
                        let _ = st.set_client_selection(
                            self.target.as_deref(),
                            context.tty_name.as_deref(),
                            Some(data),
                        );
                    }
                }
                CommandResult::ok("")
            }
            Err(error) => CommandResult::err(format!(
                "{}: {}\n",
                io_error_message(&error),
                resolved.display()
            )),
        }
    }
}

/// `show-buffer [-b buffer-name]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ShowBuffer {
    /// `-b`: the buffer to print; the most recent one otherwise.
    buffer: Option<String>,
}

impl ShowBuffer {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            buffer: args.value('b').map(str::to_string),
        })
    }

    /// Prints the buffer's contents (no trailing newline, matching tmux), or
    /// errors if there's no such buffer.
    fn run(self, st: &ServerState) -> CommandResult {
        let name = self.buffer.as_deref();
        match st.buffer(name) {
            Some(data) => CommandResult::ok_bytes(data.to_vec()),
            None => match name {
                Some(name) => CommandResult::err(format!("no buffer {name}\n")),
                None => CommandResult::err("no buffers\n"),
            },
        }
    }
}

/// `save-buffer [-a] [-b buffer-name] path`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SaveBuffer {
    /// `-a`: append to an existing file instead of truncating it.
    append: bool,
    /// `-b`: the buffer to write out; the most recent one otherwise.
    buffer: Option<String>,
    /// The destination, or `-` for tmux's stdout sink.
    path: String,
}

impl SaveBuffer {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        let path = args
            .positionals()
            .first()
            .cloned()
            .ok_or("command save-buffer: too few arguments (need at least 1)\n")?;
        Ok(Self {
            append: args.has('a'),
            buffer: args.value('b').map(str::to_string),
            path,
        })
    }

    /// Writes a paste buffer's contents to `path`; tmux's stdout sink is
    /// `path == "-"`, where the raw bytes are emitted with no trailing newline
    /// (exactly what `show-buffer` prints). Buffer resolution mirrors tmux's
    /// shared `cmd-save-buffer.c`: an unknown named buffer is `no buffer NAME`
    /// and no buffer at all is `no buffers`.
    async fn execute(self, context: &mut ExecContext<'_>) -> SharedCommandExecution {
        // A path the server writes itself may be a FIFO, so the write waits
        // where the command does; `-` is the client's own stdout and does not.
        let request = {
            let state = context.state().borrow_mut();
            self.client_request(&state, context.client())
        };
        match request {
            Some(Ok(request)) => {
                let result = suspend::save_buffer(context.tasks(), request).await;
                SharedCommandExecution::completed(result)
            }
            Some(Err(result)) => SharedCommandExecution::completed(result),
            None => context.sync(|inner| self.run(inner.state, inner.client)),
        }
    }

    /// The write the server performs off the command queue, or `None` when the
    /// destination is the client's own stdout.
    pub(super) fn client_request(
        &self,
        state: &ServerState,
        context: &ClientContext,
    ) -> Option<Result<ClientFileWrite, CommandResult>> {
        let expanded_path = execution::expand_if_cond(&self.path, None, state, &PaneAgents::new());
        if expanded_path == "-" {
            return None;
        }
        let name = self.buffer.as_deref();
        let data = match state.buffer(name) {
            Some(data) => data.to_vec(),
            None => {
                return Some(Err(match name {
                    Some(name) => CommandResult::err(format!("no buffer {name}\n")),
                    None => CommandResult::err("no buffers\n"),
                }));
            }
        };
        let path = client_file_path(&expanded_path, context);
        let flags = libc::O_WRONLY
            | libc::O_CREAT
            | if self.append {
                libc::O_APPEND
            } else {
                libc::O_TRUNC
            };
        let display_path = path.to_string_lossy().into_owned();
        Some(Ok(ClientFileWrite {
            path,
            display_path,
            flags,
            data,
        }))
    }

    fn run(self, st: &ServerState, context: &ClientContext) -> CommandResult {
        let name = self.buffer.as_deref();
        let data = match st.buffer(name) {
            Some(data) => data.to_vec(),
            None => {
                return match name {
                    Some(name) => CommandResult::err(format!("no buffer {name}\n")),
                    None => CommandResult::err("no buffers\n"),
                };
            }
        };
        let expanded_path = execution::expand_if_cond(&self.path, None, st, &PaneAgents::new());
        if expanded_path == "-" {
            // stdout sink: emit the raw bytes with no trailing newline, like tmux.
            return CommandResult::ok_bytes(data);
        }
        // tmux's `file_write` stores the expanded path and reports it on failure.
        let resolved = client_file_path(&expanded_path, context);
        let result = if self.append {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&resolved)
                .and_then(|mut file| std::io::Write::write_all(&mut file, &data))
        } else {
            std::fs::write(&resolved, &data)
        };
        match result {
            Ok(()) => CommandResult::ok(""),
            Err(error) => CommandResult::err(format!(
                "{}: {}\n",
                io_error_message(&error),
                resolved.display()
            )),
        }
    }
}

/// `list-buffers [-r] [-F format] [-f filter] [-O order]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ListBuffers {
    /// `-F`: the line each buffer expands to.
    format: Option<String>,
    /// `-f`: only list buffers this format is true for.
    filter: Option<String>,
    /// `-O`: the sort key, resolved when the command runs.
    order: Option<String>,
    /// `-r`: reverse the sort.
    reversed: bool,
}

impl ListBuffers {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            format: args.value('F').map(str::to_string),
            filter: args.value('f').map(str::to_string),
            order: args.value('O').map(str::to_string),
            reversed: args.has('r'),
        })
    }

    /// Lists paste buffers, newest first.
    fn run(self, st: &ServerState) -> CommandResult {
        let sort_order = match list_sort_order(self.order.as_deref()) {
            Ok(order) => order,
            Err(error) => return error,
        };
        let mut buffers: Vec<(usize, &(String, Bytes))> =
            st.buffers().iter().enumerate().collect();
        apply_list_sort(
            &mut buffers,
            sort_order,
            self.reversed,
            |key, (pos_a, (_, data_a)), (pos_b, (_, data_b))| match key {
                // The stack is newest first, which is tmux's descending
                // `pb->order` comparison for the creation key.
                ListSortOrder::Creation => pos_a.cmp(pos_b),
                ListSortOrder::Size => data_a.len().cmp(&data_b.len()),
                _ => std::cmp::Ordering::Equal,
            },
            |(_, (name, _))| name.clone(),
        );
        let default_line = "#{buffer_name}: #{buffer_size} bytes: \"#{buffer_sample}\"";
        let mut out = String::new();
        for (_, (name, data)) in buffers {
            let vars = buffer_vars(st, name, data);
            if let Some(filter) = self.filter.as_deref() {
                if !format::is_true(&expand_command_format(st, filter, &vars, None)) {
                    continue;
                }
            }
            out.push_str(&expand_command_format(
                st,
                self.format.as_deref().unwrap_or(default_line),
                &vars,
                None,
            ));
            out.push('\n');
        }
        CommandResult::ok(out)
    }
}

/// `delete-buffer [-b buffer-name]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct DeleteBuffer {
    /// `-b`: the buffer to remove; the most recent one otherwise.
    buffer: Option<String>,
}

impl DeleteBuffer {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            buffer: args.value('b').map(str::to_string),
        })
    }

    fn run(self, st: &mut ServerState) -> CommandResult {
        let name = self
            .buffer
            .or_else(|| st.automatic_buffer_name().map(str::to_string));
        match name {
            Some(name) if st.delete_buffer(&name) => CommandResult::ok(""),
            Some(name) => CommandResult::err(format!("unknown buffer: {name}\n")),
            None if st.buffers().is_empty() => CommandResult::err("no buffers\n"),
            None => CommandResult::err("no buffer\n"),
        }
    }
}

/// `paste-buffer [-dpr] [-s separator] [-b buffer-name] [-t target-pane]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct PasteBuffer {
    /// `-d`: delete the buffer once it has been pasted.
    delete: bool,
    /// `-b`: the buffer to paste; the most recent one otherwise.
    buffer: Option<String>,
    /// `-p`: wrap the paste in bracketed-paste markers when the pane asked for
    /// them.
    bracketed: bool,
    /// `-r`: leave newlines alone instead of turning them into carriage returns.
    raw_newlines: bool,
    /// `-s`: what a newline in the buffer is replaced with.
    separator: Option<String>,
    /// `-S`: send the buffer's bytes as they stand instead of escaping the
    /// unsafe ones.
    unescaped: bool,
    /// `-t`: the pane the buffer is pasted into.
    target: Option<String>,
}

impl PasteBuffer {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            delete: args.has('d'),
            buffer: args.value('b').map(str::to_string),
            bracketed: args.has('p'),
            raw_newlines: args.has('r'),
            separator: args.value('s').map(str::to_string),
            unescaped: args.has('S'),
            target: args.value('t').map(str::to_string),
        })
    }

    /// Transforms buffer newlines and enqueues the result on the target pane's
    /// nonblocking PTY input path.
    fn run(self, st: &mut ServerState) -> CommandResult {
        let target = self.target.clone().or_else(|| current_target(st));
        let Some(target) = target else {
            return CommandResult::err("can't establish current session\n");
        };
        if st.resolve(&target).is_none() {
            return CommandResult::err(format!("{}\n", st.pane_target_error(&target)));
        }
        let requested = self.buffer.as_deref();
        let selected = match requested {
            Some(name) => st
                .buffers()
                .iter()
                .find(|(buffer_name, _)| buffer_name == name)
                .cloned(),
            None => st.buffers().first().cloned(),
        };
        let Some((name, data)) = selected else {
            return match requested {
                Some(name) => CommandResult::err(format!("no buffer {name}\n")),
                None => CommandResult::ok(""),
            };
        };
        let separator =
            self.separator
                .as_deref()
                .unwrap_or(if self.raw_newlines { "\n" } else { "\r" });
        // Decided before the body is built so the wrapper's opening bytes are
        // written at the front rather than spliced in ahead of the whole paste.
        let bracketed = self.bracketed && st.pane_bracketed_paste(&target).unwrap_or(false);
        const PASTE_START: &[u8] = b"\x1b[200~";
        const PASTE_END: &[u8] = b"\x1b[201~";
        let mut bytes = Vec::with_capacity(
            data.len() + if bracketed { PASTE_START.len() + PASTE_END.len() } else { 0 },
        );
        if bracketed {
            bytes.extend_from_slice(PASTE_START);
        }
        // The runs between newlines are copied whole; only a newline is
        // rewritten, and a paste is mostly not newlines. tmux escapes each run
        // on its own and writes the separator as it stands.
        let copy = |run: &[u8], bytes: &mut Vec<u8>| {
            if self.unescaped {
                bytes.extend_from_slice(run);
            } else {
                vis_safe(run, bytes);
            }
        };
        for run in data.split_inclusive(|byte| *byte == b'\n') {
            match run.split_last() {
                Some((b'\n', head)) => {
                    copy(head, &mut bytes);
                    bytes.extend_from_slice(separator.as_bytes());
                }
                _ => copy(run, &mut bytes),
            }
        }
        if bracketed {
            bytes.extend_from_slice(PASTE_END);
        }
        if let Err(error) = st.input_to_pane(&target, &bytes) {
            return CommandResult::err(format!("{error}\n"));
        }
        if self.delete {
            st.delete_buffer(&name);
        }
        CommandResult::ok("")
    }
}

/// tmux's `utf8_stravisx(..., VIS_SAFE|VIS_NOSLASH)`: whole UTF-8 characters
/// travel untouched and so do the bytes a terminal can be trusted with, while
/// everything else is rendered as the `^X`/`M-X` text `vis(3)` gives it. It is
/// what keeps a pasted escape sequence out of the pane's parser.
fn vis_safe(data: &[u8], out: &mut Vec<u8>) {
    let mut index = 0;
    while index < data.len() {
        if let Some(width) = utf8_character(&data[index..]) {
            out.extend_from_slice(&data[index..index + width]);
            index += width;
            continue;
        }
        vis_safe_byte(data[index], out);
        index += 1;
    }
}

/// The length of the complete, valid multi-byte UTF-8 character `data` starts
/// with. An ASCII byte is not one: `vis(3)` is what decides its fate.
fn utf8_character(data: &[u8]) -> Option<usize> {
    let width = match data[0] {
        0x00..=0x7f => return None,
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return None,
    };
    let candidate = data.get(..width)?;
    std::str::from_utf8(candidate).ok().map(|_| width)
}

/// One byte through `vis(3)` under `VIS_SAFE|VIS_NOSLASH`.
fn vis_safe_byte(byte: u8, out: &mut Vec<u8>) {
    let safe = matches!(byte, 0x07 | 0x08 | b'\t' | b'\n' | b'\r' | b' ');
    if byte.is_ascii_graphic() || safe {
        out.push(byte);
        return;
    }
    // `\240` and nothing else: the octal escape is the one form that keeps its
    // backslash under `VIS_NOSLASH`.
    if byte & 0o177 == b' ' {
        out.push(b'\\');
        out.extend_from_slice(&[
            (byte >> 6 & 0o7) + b'0',
            (byte >> 3 & 0o7) + b'0',
            (byte & 0o7) + b'0',
        ]);
        return;
    }
    let byte = if byte & 0o200 != 0 {
        out.push(b'M');
        byte & 0o177
    } else {
        byte
    };
    if byte.is_ascii_control() {
        out.push(b'^');
        out.push(if byte == 0o177 { b'?' } else { byte + b'@' });
    } else {
        out.push(b'-');
        out.push(byte);
    }
}
