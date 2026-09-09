//! Stable access to one prompt in a command-prompt sequence.

use core::ffi::CStr;

/// The text and initial input for one command prompt.
pub trait CommandPromptItem {
    /// Builds a command prompt, copying the supplied strings.
    fn from_command_prompt_item(prompt: Option<&CStr>, input: Option<&CStr>) -> Self
    where
        Self: Sized;

    /// Returns the text displayed for the prompt.
    fn command_prompt_text(&self) -> Option<&CStr>;

    /// Returns the prompt's initial input.
    fn command_prompt_initial_input(&self) -> Option<&CStr>;
}

impl CommandPromptItem for crate::types::cmd_command_prompt_prompt {
    fn from_command_prompt_item(prompt: Option<&CStr>, input: Option<&CStr>) -> Self {
        Self {
            input: input.map(CStr::to_owned),
            prompt: prompt.map(CStr::to_owned),
        }
    }

    fn command_prompt_text(&self) -> Option<&CStr> {
        self.prompt.as_deref()
    }

    fn command_prompt_initial_input(&self) -> Option<&CStr> {
        self.input.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::cmd_command_prompt_prompt;

    #[test]
    fn prompt_and_initial_input_round_trip() {
        let value =
            cmd_command_prompt_prompt::from_command_prompt_item(Some(c"Name: "), Some(c"initial"));
        assert_eq!(value.command_prompt_text(), Some(c"Name: "));
        assert_eq!(value.command_prompt_initial_input(), Some(c"initial"));
        let empty = cmd_command_prompt_prompt::from_command_prompt_item(None, None);
        assert_eq!(empty.command_prompt_text(), None);
        assert_eq!(empty.command_prompt_initial_input(), None);
    }
}
