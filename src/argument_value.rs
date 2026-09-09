//! Stable access to one parsed argument value.

use crate::types::CmdListRef;
use core::ffi::CStr;
use std::ffi::CString;

/// The payload variant carried by an argument value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgumentValueKind {
    /// No payload.
    None,
    /// A literal string.
    String,
    /// A parsed command list.
    Commands,
}

/// The observable and replaceable state of one parsed argument value.
pub trait ArgumentValue {
    /// Returns the payload variant.
    fn argument_value_kind(&self) -> ArgumentValueKind;

    /// Returns the literal string payload, when this is a string value.
    fn argument_string_value(&self) -> Option<&CStr>;

    /// Returns the parsed commands payload, when this is a commands value.
    fn argument_commands(&self) -> Option<&CmdListRef>;

    /// Returns the value in the string form consumed by command operations.
    fn argument_value_string(&self) -> &CStr;

    /// Clears the payload.
    fn clear_argument_value(&mut self);

    /// Replaces the payload with a literal string.
    fn set_argument_string_value(&mut self, value: CString);

    /// Replaces the payload with parsed commands.
    fn set_argument_commands(&mut self, commands: Option<CmdListRef>);

    /// Builds a literal string value.
    fn from_argument_string(value: CString) -> Self
    where
        Self: Sized + Default,
    {
        let mut argument = Self::default();
        argument.set_argument_string_value(value);
        argument
    }

    /// Builds a parsed commands value.
    fn from_argument_commands(commands: Option<CmdListRef>) -> Self
    where
        Self: Sized + Default,
    {
        let mut argument = Self::default();
        argument.set_argument_commands(commands);
        argument
    }
}

impl ArgumentValue for crate::types::args_value_t {
    fn argument_value_kind(&self) -> ArgumentValueKind {
        match self.value {
            crate::types::ArgsValue::None => ArgumentValueKind::None,
            crate::types::ArgsValue::String(_) => ArgumentValueKind::String,
            crate::types::ArgsValue::Commands { .. } => ArgumentValueKind::Commands,
        }
    }

    fn argument_string_value(&self) -> Option<&CStr> {
        match &self.value {
            crate::types::ArgsValue::String(value) => Some(value),
            crate::types::ArgsValue::None | crate::types::ArgsValue::Commands { .. } => None,
        }
    }

    fn argument_commands(&self) -> Option<&CmdListRef> {
        match &self.value {
            crate::types::ArgsValue::Commands { cmdlist, .. } => cmdlist.as_ref(),
            crate::types::ArgsValue::None | crate::types::ArgsValue::String(_) => None,
        }
    }

    fn argument_value_string(&self) -> &CStr {
        match &self.value {
            crate::types::ArgsValue::None => c"",
            crate::types::ArgsValue::String(value) => value,
            crate::types::ArgsValue::Commands { cmdlist, cached } => cached
                .get_or_init(|| {
                    cmdlist
                        .as_ref()
                        .map_or_else(CString::default, |list| unsafe { list.print(0) })
                })
                .as_c_str(),
        }
    }

    fn clear_argument_value(&mut self) {
        self.value = crate::types::ArgsValue::None;
    }

    fn set_argument_string_value(&mut self, value: CString) {
        self.value = crate::types::ArgsValue::String(value);
    }

    fn set_argument_commands(&mut self, commands: Option<CmdListRef>) {
        self.value = crate::types::ArgsValue::Commands {
            cmdlist: commands,
            cached: std::cell::OnceCell::new(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::args_value_t;

    #[test]
    fn string_values_can_be_replaced_and_cleared() {
        let mut value = args_value_t::from_argument_string(c"one".to_owned());
        assert_eq!(value.argument_value_kind(), ArgumentValueKind::String);
        assert_eq!(value.argument_string_value(), Some(c"one"));
        value.set_argument_string_value(c"two".to_owned());
        assert_eq!(value.argument_value_string(), c"two");
        value.clear_argument_value();
        assert_eq!(value.argument_value_kind(), ArgumentValueKind::None);
        assert_eq!(value.argument_value_string(), c"");
    }
}
