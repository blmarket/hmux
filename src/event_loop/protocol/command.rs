//! The command client: one `tmux <command>` invocation, start to exit status.
//!
//! A command line is a sequence of groups, and each group is either run here or
//! handed to the client because the file it names belongs to the client's
//! filesystem — `source-file -` reads the client's stdin, `save-buffer` writes
//! the client's path. Those handshakes and the response that follows used to be
//! phases of a machine the loop stepped; they are statements here, in the order
//! the protocol puts them in.

use std::collections::VecDeque;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::rc::Rc;

use crate::server::command::{self, ClientContext, CommandResult};
use crate::sync::{select, Either};
use crate::tmux::message::{Frame, Message};

use super::wire::Wire;
use super::{ClientRuntime, ProtocolCloseReason, OUTPUT_CHUNK};

/// The stream number the client-file protocol uses, distinct from the stdout
/// and stderr streams a response is written to.
pub(super) const FILE_STREAM: i32 = 3;

const COMMAND_QUEUE_BUDGET: usize = 64;

/// Serve one command client to its end.
pub(super) async fn run(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    args: Vec<String>,
    context: ClientContext,
) -> ProtocolCloseReason {
    match serve(wire, runtime, args, context).await {
        Ok(reason) => reason,
        Err(reason) => reason,
    }
}

async fn serve(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    args: Vec<String>,
    context: ClientContext,
) -> Result<ProtocolCloseReason, ProtocolCloseReason> {
    let result = run_command_line(wire, runtime, args, context).await?;
    respond(wire, result).await?;
    // tmux leaves the socket open here and lets the client disconnect, which is
    // what gives a client still flushing a `save-buffer` of its own the chance
    // to finish; closing first would discard whatever it has buffered.
    Ok(await_client_close(wire).await)
}

/// Run every group of one command line, stopping early where a failure does.
async fn run_command_line(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    args: Vec<String>,
    context: ClientContext,
) -> Result<CommandResult, ProtocolCloseReason> {
    let aliases = runtime.state.borrow_mut().command_aliases();
    let compiled = match command::ExecutableCommand::compile_argv(&args, &aliases) {
        Ok(compiled) => compiled,
        Err(error) => return Ok(command::CommandResult::err(error)),
    };
    // Splitting costs the client a round trip per group, so it is only worth it
    // when a group actually needs the client's filesystem; everything else runs
    // as the one line it was written as.
    let members = compiled.clone().split();
    let mut groups: VecDeque<command::ExecutableCommand> = if members.len() > 1
        && members
            .iter()
            .any(|member| command::uses_client_file_protocol(&member.argv()))
    {
        members.into()
    } else {
        VecDeque::from(vec![compiled])
    };

    let mut output = CommandResult::ok("");
    while let Some(group) = groups.pop_front() {
        let result = run_group(wire, runtime, group, &context).await?;
        if merge(&mut output, &result) {
            groups.clear();
        }
    }
    Ok(output)
}

/// Merge one group's result into the line's, reporting whether the tail stops.
fn merge(output: &mut CommandResult, result: &CommandResult) -> bool {
    let exit = result.exit;
    output.continue_queue |= result.continue_queue;
    output.append_stdout(result);
    output.stderr.push_str(&result.stderr);
    if output.exit == 0 || exit != 0 {
        output.exit = exit;
    }
    exit != 0 && !result.continue_queue
}

async fn run_group(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    group: command::ExecutableCommand,
    context: &ClientContext,
) -> Result<CommandResult, ProtocolCloseReason> {
    // The file protocol is a property of what the group *is*, so it is read off
    // the compiled line rather than by parsing the argv again.
    let args = group.argv();
    let file_write = {
        let mut state = runtime.state.borrow_mut();
        command::save_buffer_client_request(&args, &mut state, context)
    };
    if let Some(request) = file_write {
        return match request {
            Err(result) => Ok(result),
            Ok(request) => write_client_file(wire, runtime, &args, context, request).await,
        };
    }

    let mut context = context.clone();
    if let Some(path) = command::client_input_path(&args, &context) {
        context.input_file = Some(read_client_file(wire, &path).await?);
    }
    run_queue(wire, runtime, group, &context).await
}

/// Run one group on a command queue of its own and wait for its result.
async fn run_queue(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    group: command::ExecutableCommand,
    context: &ClientContext,
) -> Result<CommandResult, ProtocolCloseReason> {
    let agents = runtime.hub.snapshot().panes;
    let queue = command::start_compiled_command(group, &agents, context);
    let queued = match command::spawn_queue(
        &runtime.tasks,
        queue,
        Rc::clone(&runtime.state),
        COMMAND_QUEUE_BUDGET,
    ) {
        Ok(queued) => queued,
        Err(error) => return Ok(CommandResult::err(format!("{error}\n"))),
    };
    // The queue takes its first turn as it is spawned, so a command that never
    // has to wait is finished by the time this is awaited. A client that leaves
    // while it runs is still watched for: the queue is the server's work and
    // goes on, but the connection waiting on it is over.
    let mut result = match select(queued, watch_for_close(wire)).await {
        Either::First(result) => {
            result.map_err(|error| ProtocolCloseReason::Error(error.kind()))?
        }
        Either::Second(reason) => return Err(reason),
    };
    for request in result.background_commands.drain(..) {
        runtime.background.start(request);
    }
    // The client's queue is empty, which in tmux's loop is when the global
    // queue gets its turn: the events this line raised become hook bodies now,
    // before the server takes another request.
    runtime.background.pump();
    Ok(result)
}

/// Read the client while something else runs, reporting only its departure.
///
/// Whatever else it says is an answer to a request that is over, or to one
/// never made.
async fn watch_for_close(wire: &mut Wire) -> ProtocolCloseReason {
    loop {
        match wire.recv().await {
            Ok(frame) => match frame.msg {
                Message::Detach(_)
                | Message::DetachKill(_)
                | Message::Exit(..)
                | Message::Shutdown => return ProtocolCloseReason::Completed,
                _ => {}
            },
            Err(reason) => return reason,
        }
    }
}

/// Ask the client for a file only it can open, and take what it sends back.
async fn read_client_file(
    wire: &mut Wire,
    path: &Path,
) -> Result<Result<Vec<u8>, i32>, ProtocolCloseReason> {
    // `-` is the client's own stdin, which it names by descriptor rather than
    // by path.
    let fd = if path.as_os_str() == "-" { 0 } else { -1 };
    let mut wire_path = path.as_os_str().as_bytes().to_vec();
    wire_path.push(0);
    wire.send(Frame::new(Message::ReadOpen {
        stream: FILE_STREAM,
        fd,
        path: wire_path,
    }))
    .await?;

    let mut data = Vec::new();
    loop {
        match wire.recv().await?.msg {
            Message::Read {
                stream: FILE_STREAM,
                data: chunk,
            } => data.extend_from_slice(&chunk),
            Message::ReadDone {
                stream: FILE_STREAM,
                error,
            } => {
                return Ok(if error == 0 { Ok(data) } else { Err(error) });
            }
            Message::Detach(_) | Message::DetachKill(_) | Message::Exit(..) | Message::Shutdown => {
                return Err(ProtocolCloseReason::Completed);
            }
            _ => {}
        }
    }
}

/// Hand the client a file to write, and run the originating command's hooks
/// once it has taken all of it.
async fn write_client_file(
    wire: &mut Wire,
    runtime: &ClientRuntime,
    args: &[String],
    context: &ClientContext,
    request: command::ClientFileWrite,
) -> Result<CommandResult, ProtocolCloseReason> {
    let mut path = request.path.as_os_str().as_bytes().to_vec();
    path.push(0);
    wire.send(Frame::new(Message::WriteOpen {
        stream: FILE_STREAM,
        fd: -1,
        flags: request.flags,
        path,
    }))
    .await?;

    let error = await_write_ready(wire, FILE_STREAM).await?;
    if error != 0 {
        // The queue goes on: a path the client could not open is this group's
        // failure, not the line's.
        let mut result = CommandResult::err(format!(
            "{}: {}\n",
            command::io_error_message(&io::Error::from_raw_os_error(error)),
            request.display_path
        ));
        result.continue_queue = true;
        return Ok(result);
    }

    for chunk in request.data.chunks(OUTPUT_CHUNK) {
        wire.send(Frame::new(Message::Write {
            stream: FILE_STREAM,
            data: chunk.to_vec(),
        }))
        .await?;
    }
    wire.send(Frame::new(Message::WriteClose {
        stream: FILE_STREAM,
    }))
    .await?;

    // The write is the command; its `after-*` hook belongs to the completed
    // handshake, not to the queue step that never ran it.
    let hooks = {
        let mut state = runtime.state.borrow_mut();
        command::client_file_after_hooks(args, &mut state, context)
    };
    for request in hooks {
        runtime.background.start(request);
    }
    runtime.background.pump();
    Ok(CommandResult::ok(""))
}

/// Answer a client with one result it is already waiting for, then let it go.
///
/// An attach that cannot start is the caller: it never becomes an attach
/// client, so what reaches the peer is the ordinary command response.
pub(super) async fn report(wire: &mut Wire, result: CommandResult) -> ProtocolCloseReason {
    match respond(wire, result).await {
        Ok(()) => await_client_close(wire).await,
        Err(reason) => reason,
    }
}

/// Write one command's output back over the client's own stdout and stderr,
/// then its exit status.
async fn respond(wire: &mut Wire, result: CommandResult) -> Result<(), ProtocolCloseReason> {
    write_stream(wire, 1, result.stdout_data()).await?;
    write_stream(wire, 2, result.stderr.as_bytes()).await?;
    wire.send(Frame::new(Message::Exit(Some(result.exit), None)))
        .await?;
    wire.flush()
}

/// Open one of the client's standard streams, fill it and close it.
///
/// A stream with nothing to say is never opened, and one the client fails to
/// open is closed without being written — which is what tells the client the
/// exchange for that stream is over either way.
async fn write_stream(
    wire: &mut Wire,
    stream: i32,
    data: &[u8],
) -> Result<(), ProtocolCloseReason> {
    if data.is_empty() {
        return Ok(());
    }
    wire.send(Frame::new(Message::WriteOpen {
        stream,
        fd: stream,
        flags: 0,
        path: Vec::new(),
    }))
    .await?;
    if await_write_ready(wire, stream).await? == 0 {
        for chunk in data.chunks(OUTPUT_CHUNK) {
            wire.send(Frame::new(Message::Write {
                stream,
                data: chunk.to_vec(),
            }))
            .await?;
        }
    }
    wire.send(Frame::new(Message::WriteClose { stream })).await
}

/// Wait for the client's answer to one `WriteOpen`, reporting its errno.
async fn await_write_ready(wire: &mut Wire, stream: i32) -> Result<i32, ProtocolCloseReason> {
    loop {
        match wire.recv().await?.msg {
            Message::WriteReady {
                stream: ready,
                error,
            } if ready == stream => return Ok(error),
            Message::Detach(_) | Message::DetachKill(_) | Message::Exit(..) | Message::Shutdown => {
                return Err(ProtocolCloseReason::Completed);
            }
            _ => {}
        }
    }
}

/// Keep the socket open until the client drops it, flushing what is left.
///
/// Whatever it says now is moot; the read is only here to see the close.
pub(super) async fn await_client_close(wire: &mut Wire) -> ProtocolCloseReason {
    loop {
        if let Err(reason) = wire.recv().await {
            return reason;
        }
    }
}
