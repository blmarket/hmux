//! Execution boundary for registered commands.

use super::{CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP, CMD_RETURN_WAIT};
use super::cmd;
use crate::cmd::cmdq_item;
use super::{cmd_entry_flag, cmd_retval};
use crate::args::args_parse_t;
use core::ffi::CStr;

/// The effect a command has on command-queue execution.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CommandResult {
    Error,
    Normal,
    Wait,
    Stop,
}

impl CommandResult {
    fn from_raw(result: cmd_retval) -> Self {
        match result {
            CMD_RETURN_ERROR => Self::Error,
            CMD_RETURN_NORMAL => Self::Normal,
            CMD_RETURN_WAIT => Self::Wait,
            CMD_RETURN_STOP => Self::Stop,
            _ => panic!("unknown command result {result}"),
        }
    }

    pub(crate) fn into_raw(self) -> cmd_retval {
        match self {
            Self::Error => CMD_RETURN_ERROR,
            Self::Normal => CMD_RETURN_NORMAL,
            Self::Wait => CMD_RETURN_WAIT,
            Self::Stop => CMD_RETURN_STOP,
        }
    }
}

/// The hmux execution context for one command queue item.
pub struct RustCommandContext<'a> {
    command: &'a cmd,
    item: &'a cmdq_item,
}

impl<'a> RustCommandContext<'a> {
    pub(crate) fn new(command: &'a cmd, item: &'a cmdq_item) -> Self {
        Self { command, item }
    }

    fn parts(&mut self) -> (&cmd, &cmdq_item) {
        (self.command, self.item)
    }
}

/// Behavior shared by every command registered with the hmux dispatcher.
pub trait CommandEntry: Sync {
    /// The command-execution context used by this implementation.
    type Context<'a>
    where
        Self: 'a;

    /// Returns the registered command name.
    fn name(&self) -> &'static CStr;

    /// Returns the exact alias, when the command has one.
    fn alias(&self) -> Option<&'static CStr>;

    /// Returns the command's argument grammar.
    fn argument_parse(&self) -> args_parse_t;

    /// Returns the command's usage suffix.
    fn usage(&self) -> &'static CStr;

    /// Returns the source-target lookup specification.
    fn source(&self) -> cmd_entry_flag;

    /// Returns the target lookup specification.
    fn target(&self) -> cmd_entry_flag;

    /// Returns the dispatcher flags.
    fn flags(&self) -> core::ffi::c_int;

    /// Executes the command represented by `context`.
    ///
    /// # Safety
    ///
    /// The queue item and parsed command must belong to the same live
    /// command queue entry, and the server globals they reference must remain
    /// valid for the call.
    unsafe fn execute(&self, context: &mut Self::Context<'_>) -> CommandResult;
}

/// A command registration executed by the Rust hmux engine.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RustCommandEntry {
    pub(crate) name: &'static CStr,
    pub(crate) alias: Option<&'static CStr>,
    pub(crate) args: args_parse_t,
    pub(crate) usage: &'static CStr,
    pub(crate) source: cmd_entry_flag,
    pub(crate) target: cmd_entry_flag,
    pub(crate) flags: core::ffi::c_int,
    pub(crate) exec: unsafe fn(&cmd, &cmdq_item) -> cmd_retval,
}

impl CommandEntry for RustCommandEntry {
    type Context<'a> = RustCommandContext<'a>;

    fn name(&self) -> &'static CStr {
        self.name
    }

    fn alias(&self) -> Option<&'static CStr> {
        self.alias
    }

    fn argument_parse(&self) -> args_parse_t {
        self.args
    }

    fn usage(&self) -> &'static CStr {
        self.usage
    }

    fn source(&self) -> cmd_entry_flag {
        self.source
    }

    fn target(&self) -> cmd_entry_flag {
        self.target
    }

    fn flags(&self) -> core::ffi::c_int {
        self.flags
    }

    unsafe fn execute(&self, context: &mut Self::Context<'_>) -> CommandResult {
        let (command, item) = context.parts();
        CommandResult::from_raw(unsafe { (self.exec)(command, item) })
    }
}
