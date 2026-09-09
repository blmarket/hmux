//! Stable access to one immutable option-table definition.

use core::ffi::CStr;

/// The metadata that defines one built-in option.
pub trait OptionTableEntry {
    /// Builds an option-table entry from its complete portable state.
    #[allow(clippy::too_many_arguments)]
    fn from_option_table_entry(
        name: &'static CStr,
        alternative_name: Option<&'static CStr>,
        option_type: u32,
        scope: i32,
        flags: i32,
        minimum: u32,
        maximum: u32,
        choices: Option<&'static [&'static CStr]>,
        default_string: Option<&'static CStr>,
        default_number: i64,
        default_array: Option<&'static [&'static CStr]>,
        separator: Option<&'static CStr>,
        pattern: Option<&'static CStr>,
        text: Option<&'static CStr>,
        unit: Option<&'static CStr>,
    ) -> Self;

    /// Returns the canonical option name.
    fn option_table_name(&self) -> &CStr;

    /// Returns the accepted alternative name, if any.
    fn option_table_alternative_name(&self) -> Option<&CStr>;

    /// Returns the option value kind.
    fn option_table_type(&self) -> u32;

    /// Returns the scopes in which the option is available.
    fn option_table_scope(&self) -> i32;

    /// Returns the option definition flags.
    fn option_table_flags(&self) -> i32;

    /// Returns the minimum numeric value.
    fn option_table_minimum(&self) -> u32;

    /// Returns the maximum numeric value.
    fn option_table_maximum(&self) -> u32;

    /// Returns the number of permitted choice strings.
    fn option_table_choices_len(&self) -> usize;

    /// Reports whether a choice list is present.
    fn option_table_has_choices(&self) -> bool;

    /// Returns one permitted choice string.
    fn option_table_choice(&self, index: usize) -> Option<&CStr>;

    /// Returns the default string value, if any.
    fn option_table_default_string(&self) -> Option<&CStr>;

    /// Returns the default numeric value.
    fn option_table_default_number(&self) -> i64;

    /// Returns the number of strings in the default array.
    fn option_table_default_array_len(&self) -> usize;

    /// Reports whether a default array is present.
    fn option_table_has_default_array(&self) -> bool;

    /// Returns one string from the default array.
    fn option_table_default_array_item(&self, index: usize) -> Option<&CStr>;

    /// Returns the array separator, if any.
    fn option_table_separator(&self) -> Option<&CStr>;

    /// Returns the validation pattern, if any.
    fn option_table_pattern(&self) -> Option<&CStr>;

    /// Returns the help text, if any.
    fn option_table_text(&self) -> Option<&CStr>;

    /// Returns the numeric unit label, if any.
    fn option_table_unit(&self) -> Option<&CStr>;
}

impl OptionTableEntry for crate::types::options_table_entry_t {
    fn from_option_table_entry(
        name: &'static CStr,
        alternative_name: Option<&'static CStr>,
        option_type: u32,
        scope: i32,
        flags: i32,
        minimum: u32,
        maximum: u32,
        choices: Option<&'static [&'static CStr]>,
        default_string: Option<&'static CStr>,
        default_number: i64,
        default_array: Option<&'static [&'static CStr]>,
        separator: Option<&'static CStr>,
        pattern: Option<&'static CStr>,
        text: Option<&'static CStr>,
        unit: Option<&'static CStr>,
    ) -> Self {
        Self {
            name,
            alternative_name,
            type_0: option_type,
            scope,
            flags,
            minimum,
            maximum,
            choices,
            default_str: default_string,
            default_num: default_number,
            default_arr: default_array,
            separator,
            pattern,
            text,
            unit,
        }
    }

    fn option_table_name(&self) -> &CStr {
        self.name
    }
    fn option_table_alternative_name(&self) -> Option<&CStr> {
        self.alternative_name
    }
    fn option_table_type(&self) -> u32 {
        self.type_0
    }
    fn option_table_scope(&self) -> i32 {
        self.scope
    }
    fn option_table_flags(&self) -> i32 {
        self.flags
    }
    fn option_table_minimum(&self) -> u32 {
        self.minimum
    }
    fn option_table_maximum(&self) -> u32 {
        self.maximum
    }
    fn option_table_choices_len(&self) -> usize {
        self.choices.map_or(0, <[_]>::len)
    }
    fn option_table_has_choices(&self) -> bool {
        self.choices.is_some()
    }
    fn option_table_choice(&self, index: usize) -> Option<&CStr> {
        self.choices.and_then(|choices| choices.get(index).copied())
    }
    fn option_table_default_string(&self) -> Option<&CStr> {
        self.default_str
    }
    fn option_table_default_number(&self) -> i64 {
        self.default_num
    }
    fn option_table_default_array_len(&self) -> usize {
        self.default_arr.map_or(0, <[_]>::len)
    }
    fn option_table_has_default_array(&self) -> bool {
        self.default_arr.is_some()
    }
    fn option_table_default_array_item(&self, index: usize) -> Option<&CStr> {
        self.default_arr.and_then(|items| items.get(index).copied())
    }
    fn option_table_separator(&self) -> Option<&CStr> {
        self.separator
    }
    fn option_table_pattern(&self) -> Option<&CStr> {
        self.pattern
    }
    fn option_table_text(&self) -> Option<&CStr> {
        self.text
    }
    fn option_table_unit(&self) -> Option<&CStr> {
        self.unit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::options_table_entry_t;

    static CHOICES: [&CStr; 2] = [c"first", c"second"];
    static DEFAULT_ARRAY: [&CStr; 2] = [c"left", c"right"];

    #[test]
    fn complete_option_definition_round_trips() {
        let value = options_table_entry_t::from_option_table_entry(
            c"sample",
            Some(c"old-sample"),
            5,
            3,
            7,
            2,
            90,
            Some(&CHOICES),
            Some(c"default"),
            42,
            Some(&DEFAULT_ARRAY),
            Some(c","),
            Some(c"*"),
            Some(c"help"),
            Some(c"cells"),
        );

        assert_eq!(value.option_table_name(), c"sample");
        assert_eq!(value.option_table_alternative_name(), Some(c"old-sample"));
        assert_eq!(value.option_table_type(), 5);
        assert_eq!(value.option_table_scope(), 3);
        assert_eq!(value.option_table_flags(), 7);
        assert_eq!(value.option_table_minimum(), 2);
        assert_eq!(value.option_table_maximum(), 90);
        assert_eq!(value.option_table_choices_len(), 2);
        assert!(value.option_table_has_choices());
        assert_eq!(value.option_table_choice(1), Some(c"second"));
        assert_eq!(value.option_table_choice(2), None);
        assert_eq!(value.option_table_default_string(), Some(c"default"));
        assert_eq!(value.option_table_default_number(), 42);
        assert_eq!(value.option_table_default_array_len(), 2);
        assert!(value.option_table_has_default_array());
        assert_eq!(value.option_table_default_array_item(0), Some(c"left"));
        assert_eq!(value.option_table_default_array_item(2), None);
        assert_eq!(value.option_table_separator(), Some(c","));
        assert_eq!(value.option_table_pattern(), Some(c"*"));
        assert_eq!(value.option_table_text(), Some(c"help"));
        assert_eq!(value.option_table_unit(), Some(c"cells"));
    }
}
