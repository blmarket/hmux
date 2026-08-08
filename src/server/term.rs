//! Resolved capabilities for one attached client terminal.
//!
//! Pane terminal state belongs to libghostty-vt. This module describes the
//! outer terminal driven by the attach compositor: the terminfo values sent by
//! the tmux client, identify-time feature hints, and configured feature and
//! capability overrides.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::CString;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CapabilityType {
    Flag,
    Number,
    String,
}

// Keep this catalog in the same order and spelling as tmux's tty-term.c.
// Identify messages contain only these entries, and overrides for unknown
// names are ignored rather than creating ad-hoc capabilities.
const FLAG_CAPABILITIES: &[&str] = &["am", "AX", "bce", "RGB", "Sxl", "Tc", "XT"];
const NUMBER_CAPABILITIES: &[&str] = &["colors", "U8"];
const STRING_CAPABILITIES: &[&str] = &[
    "acsc", "bel", "Bidi", "blink", "bold", "civis", "clear", "Clmg", "Cmg", "cnorm", "Cr", "csr",
    "Cs", "cub1", "cub", "cud1", "cud", "cuf1", "cuf", "cup", "cuu1", "cuu", "cvvis", "dch1",
    "dch", "dim", "dl1", "dl", "Dseks", "Dsfcs", "Dsbp", "Dsmg", "E3", "ech", "ed", "el1", "el",
    "enacs", "Enbp", "Eneks", "Enfcs", "Enmg", "fsl", "Hls", "home", "hpa", "ich1", "ich", "il1",
    "il", "indn", "invis", "kcbt", "kcub1", "kcud1", "kcuf1", "kcuu1", "kDC", "kDC3", "kDC4",
    "kDC5", "kDC6", "kDC7", "kdch1", "kDN", "kDN3", "kDN4", "kDN5", "kDN6", "kDN7", "kEND",
    "kEND3", "kEND4", "kEND5", "kEND6", "kEND7", "kend", "kf10", "kf11", "kf12", "kf13", "kf14",
    "kf15", "kf16", "kf17", "kf18", "kf19", "kf1", "kf20", "kf21", "kf22", "kf23", "kf24", "kf25",
    "kf26", "kf27", "kf28", "kf29", "kf2", "kf30", "kf31", "kf32", "kf33", "kf34", "kf35", "kf36",
    "kf37", "kf38", "kf39", "kf3", "kf40", "kf41", "kf42", "kf43", "kf44", "kf45", "kf46", "kf47",
    "kf48", "kf49", "kf4", "kf50", "kf51", "kf52", "kf53", "kf54", "kf55", "kf56", "kf57", "kf58",
    "kf59", "kf5", "kf60", "kf61", "kf62", "kf63", "kf6", "kf7", "kf8", "kf9", "kHOM", "kHOM3",
    "kHOM4", "kHOM5", "kHOM6", "kHOM7", "khome", "kIC", "kIC3", "kIC4", "kIC5", "kIC6", "kIC7",
    "kich1", "kind", "kLFT", "kLFT3", "kLFT4", "kLFT5", "kLFT6", "kLFT7", "kmous", "knp", "kNXT",
    "kNXT3", "kNXT4", "kNXT5", "kNXT6", "kNXT7", "kpp", "kPRV", "kPRV3", "kPRV4", "kPRV5", "kPRV6",
    "kPRV7", "kRIT", "kRIT3", "kRIT4", "kRIT5", "kRIT6", "kRIT7", "kri", "kUP", "kUP3", "kUP4",
    "kUP5", "kUP6", "kUP7", "Ms", "Nobr", "ol", "op", "Rect", "rev", "rin", "ri", "rmacs", "rmcup",
    "rmkx", "setab", "setaf", "setal", "setrgbb", "setrgbf", "Setulc", "Setulc1", "Se", "sgr0",
    "sitm", "smacs", "smcup", "smkx", "Smol", "smso", "Smulx", "smul", "smxx", "Spb", "Ss", "Swd",
    "Sync", "tsl", "vpa",
];

const CAPABILITY_NAMES: &[&str] = &[
    "acsc", "am", "AX", "bce", "bel", "Bidi", "blink", "bold", "civis", "clear", "Clmg", "Cmg",
    "cnorm", "colors", "Cr", "Cs", "csr", "cub", "cub1", "cud", "cud1", "cuf", "cuf1", "cup",
    "cuu", "cuu1", "cvvis", "dch", "dch1", "dim", "dl", "dl1", "Dsbp", "Dseks", "Dsfcs", "Dsmg",
    "E3", "ech", "ed", "el", "el1", "enacs", "Enbp", "Eneks", "Enfcs", "Enmg", "fsl", "Hls",
    "home", "hpa", "ich", "ich1", "il", "il1", "indn", "invis", "kcbt", "kcub1", "kcud1", "kcuf1",
    "kcuu1", "kDC", "kDC3", "kDC4", "kDC5", "kDC6", "kDC7", "kdch1", "kDN", "kDN3", "kDN4", "kDN5",
    "kDN6", "kDN7", "kend", "kEND", "kEND3", "kEND4", "kEND5", "kEND6", "kEND7", "kf1", "kf10",
    "kf11", "kf12", "kf13", "kf14", "kf15", "kf16", "kf17", "kf18", "kf19", "kf2", "kf20", "kf21",
    "kf22", "kf23", "kf24", "kf25", "kf26", "kf27", "kf28", "kf29", "kf3", "kf30", "kf31", "kf32",
    "kf33", "kf34", "kf35", "kf36", "kf37", "kf38", "kf39", "kf4", "kf40", "kf41", "kf42", "kf43",
    "kf44", "kf45", "kf46", "kf47", "kf48", "kf49", "kf5", "kf50", "kf51", "kf52", "kf53", "kf54",
    "kf55", "kf56", "kf57", "kf58", "kf59", "kf6", "kf60", "kf61", "kf62", "kf63", "kf7", "kf8",
    "kf9", "kHOM", "kHOM3", "kHOM4", "kHOM5", "kHOM6", "kHOM7", "khome", "kIC", "kIC3", "kIC4",
    "kIC5", "kIC6", "kIC7", "kich1", "kind", "kLFT", "kLFT3", "kLFT4", "kLFT5", "kLFT6", "kLFT7",
    "kmous", "knp", "kNXT", "kNXT3", "kNXT4", "kNXT5", "kNXT6", "kNXT7", "kpp", "kPRV", "kPRV3",
    "kPRV4", "kPRV5", "kPRV6", "kPRV7", "kri", "kRIT", "kRIT3", "kRIT4", "kRIT5", "kRIT6", "kRIT7",
    "kUP", "kUP3", "kUP4", "kUP5", "kUP6", "kUP7", "Ms", "Nobr", "ol", "op", "Rect", "rev", "RGB",
    "ri", "rin", "rmacs", "rmcup", "rmkx", "Se", "setab", "setaf", "setal", "setrgbb", "setrgbf",
    "Setulc", "Setulc1", "sgr0", "sitm", "smacs", "smcup", "smkx", "Smol", "smso", "smul", "Smulx",
    "smxx", "Spb", "Sxl", "Ss", "Swd", "Sync", "Tc", "tsl", "U8", "vpa", "XT",
];

const DEFAULT_TERMINAL_FEATURES: &[&str] = &[
    "xterm*:clipboard:ccolour:cstyle:focus:title",
    "screen*:title",
    "rxvt*:ignorefkeys",
];
const DEFAULT_TERMINAL_OVERRIDES: &[&str] = &["linux*:AX@"];

const TERM_256_COLOURS: u8 = 0x01;
const TERM_NO_AM: u8 = 0x02;
const TERM_DECSLRM: u8 = 0x04;
const TERM_DECFRA: u8 = 0x08;
const TERM_RGB_COLOURS: u8 = 0x10;
const TERM_VT100_LIKE: u8 = 0x20;
const TERM_SIXEL: u8 = 0x40;

#[derive(Clone, Copy)]
struct Feature {
    name: &'static str,
    capabilities: &'static [&'static str],
    flags: u8,
}

const FEATURE_256: &[&str] = &[
    "AX",
    "setab=\x1b[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m",
    "setaf=\x1b[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m",
];
const FEATURE_BPASTE: &[&str] = &["Enbp=\x1b[?2004h", "Dsbp=\x1b[?2004l"];
const FEATURE_CCOLOUR: &[&str] = &["Cs=\x1b]12;%p1%s\x07", "Cr=\x1b]112\x07"];
const FEATURE_CLIPBOARD: &[&str] = &["Ms=\x1b]52;%p1%s;%p2%s\x07"];
const FEATURE_HYPERLINKS: &[&str] = &["Hls=\x1b]8;%?%p1%l%tid=%p1%s%;;%p2%s\x1b\\"];
const FEATURE_CSTYLE: &[&str] = &["Ss=\x1b[%p1%d q", "Se=\x1b[2 q"];
const FEATURE_EXTKEYS: &[&str] = &["Eneks=\x1b[>4;2m", "Dseks=\x1b[>4m"];
const FEATURE_FOCUS: &[&str] = &["Enfcs=\x1b[?1004h", "Dsfcs=\x1b[?1004l"];
const FEATURE_IGNORE_FKEYS: &[&str] = &[
    "kf0@", "kf1@", "kf2@", "kf3@", "kf4@", "kf5@", "kf6@", "kf7@", "kf8@", "kf9@", "kf10@",
    "kf11@", "kf12@", "kf13@", "kf14@", "kf15@", "kf16@", "kf17@", "kf18@", "kf19@", "kf20@",
    "kf21@", "kf22@", "kf23@", "kf24@", "kf25@", "kf26@", "kf27@", "kf28@", "kf29@", "kf30@",
    "kf31@", "kf32@", "kf33@", "kf34@", "kf35@", "kf36@", "kf37@", "kf38@", "kf39@", "kf40@",
    "kf41@", "kf42@", "kf43@", "kf44@", "kf45@", "kf46@", "kf47@", "kf48@", "kf49@", "kf50@",
    "kf51@", "kf52@", "kf53@", "kf54@", "kf55@", "kf56@", "kf57@", "kf58@", "kf59@", "kf60@",
    "kf61@", "kf62@", "kf63@",
];
const FEATURE_MARGINS: &[&str] = &[
    "Enmg=\x1b[?69h",
    "Dsmg=\x1b[?69l",
    "Clmg=\x1b[s",
    "Cmg=\x1b[%i%p1%d;%p2%ds",
];
const FEATURE_MOUSE: &[&str] = &["kmous=\x1b[M"];
const FEATURE_OSC7: &[&str] = &["Swd=\x1b]7;", "fsl=\x07"];
const FEATURE_OVERLINE: &[&str] = &["Smol=\x1b[53m"];
const FEATURE_PROGRESSBAR: &[&str] = &["Spb=\x1b]9;4;%p1%d;%p2%d\x1b\\"];
const FEATURE_RECTFILL: &[&str] = &["Rect"];
const FEATURE_RGB: &[&str] = &[
    "AX",
    "setrgbf=\x1b[38;2;%p1%d;%p2%d;%p3%dm",
    "setrgbb=\x1b[48;2;%p1%d;%p2%d;%p3%dm",
    "setab=\x1b[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m",
    "setaf=\x1b[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m",
];
const FEATURE_SIXEL: &[&str] = &["Sxl"];
const FEATURE_STRIKETHROUGH: &[&str] = &["smxx=\x1b[9m"];
const FEATURE_SYNC: &[&str] = &["Sync=\x1b[?2026%?%p1%{1}%-%tl%eh%;"];
const FEATURE_TITLE: &[&str] = &["tsl=\x1b]0;", "fsl=\x07"];
const FEATURE_USSTYLE: &[&str] = &[
    "Smulx=\x1b[4::%p1%dm",
    "Setulc=\x1b[58::2::%p1%{65536}%/%d::%p1%{256}%/%{255}%&%d::%p1%{255}%&%d%;m",
    "Setulc1=\x1b[58::5::%p1%dm",
    "ol=\x1b[59m",
];

// The index is part of the identify wire protocol (MSG_IDENTIFY_FEATURES).
const FEATURES: &[Feature] = &[
    Feature {
        name: "256",
        capabilities: FEATURE_256,
        flags: TERM_256_COLOURS,
    },
    Feature {
        name: "bpaste",
        capabilities: FEATURE_BPASTE,
        flags: 0,
    },
    Feature {
        name: "ccolour",
        capabilities: FEATURE_CCOLOUR,
        flags: 0,
    },
    Feature {
        name: "clipboard",
        capabilities: FEATURE_CLIPBOARD,
        flags: 0,
    },
    Feature {
        name: "hyperlinks",
        capabilities: FEATURE_HYPERLINKS,
        flags: 0,
    },
    Feature {
        name: "cstyle",
        capabilities: FEATURE_CSTYLE,
        flags: 0,
    },
    Feature {
        name: "extkeys",
        capabilities: FEATURE_EXTKEYS,
        flags: 0,
    },
    Feature {
        name: "focus",
        capabilities: FEATURE_FOCUS,
        flags: 0,
    },
    Feature {
        name: "ignorefkeys",
        capabilities: FEATURE_IGNORE_FKEYS,
        flags: 0,
    },
    Feature {
        name: "margins",
        capabilities: FEATURE_MARGINS,
        flags: TERM_DECSLRM,
    },
    Feature {
        name: "mouse",
        capabilities: FEATURE_MOUSE,
        flags: 0,
    },
    Feature {
        name: "osc7",
        capabilities: FEATURE_OSC7,
        flags: 0,
    },
    Feature {
        name: "overline",
        capabilities: FEATURE_OVERLINE,
        flags: 0,
    },
    Feature {
        name: "progressbar",
        capabilities: FEATURE_PROGRESSBAR,
        flags: 0,
    },
    Feature {
        name: "rectfill",
        capabilities: FEATURE_RECTFILL,
        flags: TERM_DECFRA,
    },
    Feature {
        name: "RGB",
        capabilities: FEATURE_RGB,
        flags: TERM_256_COLOURS | TERM_RGB_COLOURS,
    },
    Feature {
        name: "sixel",
        capabilities: FEATURE_SIXEL,
        flags: TERM_SIXEL,
    },
    Feature {
        name: "strikethrough",
        capabilities: FEATURE_STRIKETHROUGH,
        flags: 0,
    },
    Feature {
        name: "sync",
        capabilities: FEATURE_SYNC,
        flags: 0,
    },
    Feature {
        name: "title",
        capabilities: FEATURE_TITLE,
        flags: 0,
    },
    Feature {
        name: "usstyle",
        capabilities: FEATURE_USSTYLE,
        flags: 0,
    },
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CapabilityValue {
    Flag(bool),
    Number(i32),
    String(Vec<u8>),
}

/// Read-only capability boundary used by terminal input and output consumers.
/// There is one production implementation, [`ResolvedTerm`].
pub(crate) trait TerminalCapabilities {
    fn name(&self) -> &str;

    fn capability(&self, name: &str) -> Option<&CapabilityValue>;

    #[allow(dead_code)] // consumed as terminal-feature-driven output migrates here
    fn has_feature(&self, name: &str) -> bool;

    fn generation(&self) -> u64 {
        0
    }

    fn flag(&self, name: &str) -> bool {
        matches!(self.capability(name), Some(CapabilityValue::Flag(true)))
    }

    fn auto_margin(&self) -> bool {
        self.flag("am")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)] // string parameters are used as clipboard/title output migrates here
pub(crate) enum CapabilityParameter<'a> {
    Number(i32),
    String(&'a str),
}

/// tmux's sixel test in `tty_cmd_sixelimage`: the `sixel` terminal feature —
/// which is what sets `TERM_SIXEL` — or a terminfo entry that declares `Sxl` on
/// its own. Either is enough; a client with neither gets the text placeholder.
pub(crate) fn supports_sixel(terminal: &dyn TerminalCapabilities) -> bool {
    terminal.has_feature("sixel") || terminal.capability("Sxl").is_some()
}

pub(crate) fn string_capability<'a>(
    terminal: &'a dyn TerminalCapabilities,
    name: &str,
) -> Option<&'a [u8]> {
    match terminal.capability(name) {
        Some(CapabilityValue::String(value)) => Some(value),
        _ => None,
    }
}

#[allow(dead_code)] // consumed as color selection migrates to resolved capabilities
pub(crate) fn number_capability(terminal: &dyn TerminalCapabilities, name: &str) -> Option<i32> {
    match terminal.capability(name) {
        Some(CapabilityValue::Number(value)) => Some(*value),
        _ => None,
    }
}

/// Expand a terminfo string with the same numeric and string parameter forms
/// exposed by tmux's tty_term_string_* family.
///
/// The expansion is performed by the pure-Rust `terminfo::expand` engine rather
/// than the C `tparm`; capability strings arrive from the tmux client identify
/// message, so only the parameter interpreter is needed, not the terminfo
/// database.
///
/// This engine is stricter than ncurses `tparm` about malformed format strings:
/// ncurses silently ignores a `%;` that closes no open `%?`, whereas the crate
/// rejects the whole string. Normalize those unmatched terminators before
/// expansion so the terminal adapter has tmux's leniency without weakening the
/// parser for balanced conditionals.
pub(crate) fn expand_capability(
    terminal: &dyn TerminalCapabilities,
    name: &str,
    parameters: &[CapabilityParameter<'_>],
) -> Option<Vec<u8>> {
    use terminfo::expand::{Context, Expand, Parameter};

    let value = string_capability(terminal, name)?;
    let arguments = parameters
        .iter()
        .map(|parameter| match parameter {
            CapabilityParameter::Number(value) => Parameter::Number(*value),
            CapabilityParameter::String(value) => Parameter::String(value.as_bytes().to_vec()),
        })
        .collect::<Vec<_>>();

    let mut context = Context::default();
    let normalized = strip_unmatched_conditional_ends(value);
    let mut expanded = Vec::new();
    normalized
        .as_slice()
        .expand(&mut expanded, &arguments, &mut context)
        .ok()?;
    Some(expanded)
}

/// Remove only conditional terminators that have no matching `%?` opener.
///
/// ncurses treats those stray `%;` sequences as no-ops. Keeping balanced
/// conditionals intact lets the stricter Rust expander continue to validate
/// the rest of the capability string.
fn strip_unmatched_conditional_ends(value: &[u8]) -> Vec<u8> {
    let mut normalized = Vec::with_capacity(value.len());
    let mut conditional_depth = 0usize;
    let mut index = 0;
    while index < value.len() {
        if value[index] == b'%' {
            if let Some(operator) = value.get(index + 1).copied() {
                match operator {
                    b'%' => {
                        normalized.extend_from_slice(&value[index..=index + 1]);
                        index += 2;
                        continue;
                    }
                    b'?' => {
                        conditional_depth += 1;
                        normalized.extend_from_slice(&value[index..=index + 1]);
                        index += 2;
                        continue;
                    }
                    b';' if conditional_depth == 0 => {
                        index += 2;
                        continue;
                    }
                    b';' => {
                        conditional_depth -= 1;
                        normalized.extend_from_slice(&value[index..=index + 1]);
                        index += 2;
                        continue;
                    }
                    _ => {}
                }
            }
        }
        normalized.push(value[index]);
        index += 1;
    }
    normalized
}

/// Immutable identify data retained so configured features and overrides can
/// be re-resolved when a command client changes them after attach.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TerminalIdentity {
    name: String,
    capabilities: Vec<String>,
    feature_bits: u32,
    colorterm: Option<String>,
    utf8: bool,
}

impl TerminalIdentity {
    pub(crate) fn new(
        name: impl Into<String>,
        capabilities: Vec<String>,
        feature_bits: u32,
        colorterm: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            capabilities,
            feature_bits,
            colorterm,
            utf8: false,
        }
    }

    pub(crate) fn with_utf8(mut self, utf8: bool) -> Self {
        self.utf8 = utf8;
        self
    }
}

/// One attached client's effective terminal profile.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ResolvedTerm {
    identity: TerminalIdentity,
    capabilities: BTreeMap<String, CapabilityValue>,
    features: BTreeSet<String>,
    terminal_flags: u8,
    acs: BTreeMap<u8, u8>,
    generation: u64,
}

impl ResolvedTerm {
    pub(crate) fn resolve<'a>(
        identity: TerminalIdentity,
        options: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Self {
        let mut resolved = Self {
            identity,
            ..Self::default()
        };
        resolved.refresh(options);
        resolved
    }

    pub(crate) fn refresh<'a>(&mut self, options: impl IntoIterator<Item = (&'a str, &'a str)>) {
        self.generation = self.generation.wrapping_add(1);
        self.capabilities.clear();
        for entry in &self.identity.capabilities {
            let Some((name, value)) = entry.split_once('=') else {
                continue;
            };
            if let Some(value) = identify_value(name, value) {
                self.capabilities.insert(name.to_string(), value);
            }
        }

        self.features.clear();
        for (index, feature) in FEATURES.iter().enumerate() {
            if self.identity.feature_bits & (1_u32 << index) != 0 {
                self.features.insert(feature.name.to_string());
            }
        }
        match self.identity.colorterm.as_deref() {
            Some(value)
                if value.eq_ignore_ascii_case("truecolor")
                    || value.eq_ignore_ascii_case("24bit") =>
            {
                self.features.insert("RGB".to_string());
            }
            Some(value) if value.contains("256") => {
                self.features.insert("256".to_string());
            }
            _ => {}
        }

        let options = terminal_option_values(options);
        for value in options
            .iter()
            .filter(|value| value.name == "terminal-features")
            .flat_map(|value| split_array_entries(value.value))
        {
            let mut fields = split_override_fields(value).into_iter();
            let Some(pattern) = fields.next() else {
                continue;
            };
            if !terminal_pattern_matches(&pattern, &self.identity.name) {
                continue;
            }
            for feature in fields {
                let Some(canonical) = FEATURES
                    .iter()
                    .find(|known| known.name.eq_ignore_ascii_case(&feature))
                else {
                    // tty_add_features stops at the first unknown name.
                    break;
                };
                self.features.insert(canonical.name.to_string());
            }
        }

        // tmux applies overrides once before inference, then reapplies them
        // after feature capabilities so user configuration always wins.
        self.apply_overrides(&options);
        self.recompute_derived_flags(0);

        let vt100_like = self.flag_value("XT")
            || self
                .string_value("clear")
                .is_some_and(|clear| clear.starts_with(b"\x1b["));
        if vt100_like {
            self.add_feature_names(["bpaste", "focus", "title"]);
        }

        if (self.flag_value("Tc") || self.has_capability("RGB"))
            && (!self.has_capability("setrgbf") || !self.has_capability("setrgbb"))
        {
            self.add_feature_names(["RGB"]);
        }

        let mut feature_flags = 0;
        for feature in FEATURES {
            if !self.features.contains(feature.name) {
                continue;
            }
            for capabilities in feature.capabilities {
                self.apply_capabilities(capabilities);
            }
            feature_flags |= feature.flags;
        }

        // tmux's nested screen-family clients use ECMA-48 overline even when
        // the base terminfo entry predates the extended Smol name. Resolve it
        // into the profile before the final override pass so `Smol@` can still
        // explicitly remove it.
        if (self.identity.name.starts_with("screen") || self.identity.name.starts_with("tmux"))
            && !self.has_capability("Smol")
        {
            self.apply_capabilities(FEATURE_OVERLINE[0]);
            self.features.insert("overline".into());
        }

        self.apply_overrides(&options);
        if vt100_like {
            feature_flags |= TERM_VT100_LIKE;
        }
        self.recompute_derived_flags(feature_flags);
        self.rebuild_acs();
        if self.identity.utf8 {
            self.capabilities
                .insert("__hmux_utf8".into(), CapabilityValue::Flag(true));
        }
    }

    pub(crate) fn validation_error(&self) -> Option<&'static str> {
        if !self.has_capability("clear") {
            Some("terminal does not support clear")
        } else if !self.has_capability("cup") {
            Some("terminal does not support cup")
        } else {
            None
        }
    }

    pub(crate) fn flags(&self) -> u8 {
        self.terminal_flags
    }

    /// Whether the terminal answers the VT100-family private sequences tmux
    /// sends unconditionally (theme subscription, extended keys).
    pub(crate) fn is_vt100_like(&self) -> bool {
        self.terminal_flags & TERM_VT100_LIKE != 0
    }

    /// The resolved features in tmux's feature-bit order, as
    /// `#{client_termfeatures}` reports them (`tty_get_features`).
    pub(crate) fn feature_list(&self) -> String {
        FEATURES
            .iter()
            .filter(|feature| self.features.contains(feature.name))
            .map(|feature| feature.name)
            .collect::<Vec<_>>()
            .join(",")
    }

    pub(crate) fn descriptions(&self) -> Vec<String> {
        CAPABILITY_NAMES
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let description = match self.capabilities.get(*name) {
                    None => "[missing]".to_string(),
                    Some(CapabilityValue::Flag(value)) => format!("(flag) {value}"),
                    Some(CapabilityValue::Number(value)) => format!("(number) {value}"),
                    Some(CapabilityValue::String(value)) => {
                        format!("(string) {}", visible_capability(value))
                    }
                };
                format!("{index:4}: {name}: {description}")
            })
            .collect()
    }

    fn add_feature_names<const N: usize>(&mut self, names: [&str; N]) {
        for name in names {
            self.features.insert(name.to_string());
        }
    }

    fn apply_overrides(&mut self, options: &[TerminalOptionValue<'_>]) {
        for value in options
            .iter()
            .filter(|value| value.name == "terminal-overrides")
            .flat_map(|value| split_array_entries(value.value))
        {
            self.apply_override(value);
        }
    }

    fn apply_override(&mut self, value: &str) {
        let mut fields = split_override_fields(value).into_iter();
        let Some(pattern) = fields.next() else {
            return;
        };
        if !terminal_pattern_matches(&pattern, &self.identity.name) {
            return;
        }
        for field in fields {
            self.apply_capability_field(&field);
        }
    }

    fn apply_capabilities(&mut self, capabilities: &str) {
        for field in split_override_fields(capabilities) {
            self.apply_capability_field(&field);
        }
    }

    fn apply_capability_field(&mut self, field: &str) {
        if field.is_empty() {
            return;
        }
        if let Some(name) = field.strip_suffix('@').filter(|_| !field.contains('=')) {
            if capability_type(name).is_some() {
                self.capabilities.remove(name);
            }
            return;
        }

        let (name, encoded) = field.split_once('=').unwrap_or((field, ""));
        let Some(kind) = capability_type(name) else {
            return;
        };
        let decoded = decode_vis(encoded).unwrap_or_else(|| encoded.as_bytes().to_vec());
        let value = match kind {
            // The presence of a flag in an override enables it. tmux ignores
            // the assigned value even for spellings such as `am=0`.
            CapabilityType::Flag => CapabilityValue::Flag(true),
            CapabilityType::Number => {
                let Ok(value) = std::str::from_utf8(&decoded).unwrap_or("").parse::<i32>() else {
                    return;
                };
                if value < 0 {
                    return;
                }
                CapabilityValue::Number(value)
            }
            CapabilityType::String => CapabilityValue::String(decoded),
        };
        self.capabilities.insert(name.to_string(), value);
    }

    fn has_capability(&self, name: &str) -> bool {
        self.capabilities.contains_key(name)
    }

    fn flag_value(&self, name: &str) -> bool {
        matches!(
            self.capabilities.get(name),
            Some(CapabilityValue::Flag(true))
        )
    }

    fn string_value(&self, name: &str) -> Option<&[u8]> {
        match self.capabilities.get(name) {
            Some(CapabilityValue::String(value)) => Some(value),
            _ => None,
        }
    }

    fn recompute_derived_flags(&mut self, base: u8) {
        self.terminal_flags = base;
        if self.has_capability("setrgbf") && self.has_capability("setrgbb") {
            self.terminal_flags |= TERM_RGB_COLOURS;
        } else {
            self.terminal_flags &= !TERM_RGB_COLOURS;
        }
        if self.has_capability("Cmg") && self.has_capability("Clmg") {
            self.terminal_flags |= TERM_DECSLRM;
        } else {
            self.terminal_flags &= !TERM_DECSLRM;
        }
        if self.has_capability("Rect") {
            self.terminal_flags |= TERM_DECFRA;
        } else {
            self.terminal_flags &= !TERM_DECFRA;
        }
        if !self.flag_value("am") {
            self.terminal_flags |= TERM_NO_AM;
        } else {
            self.terminal_flags &= !TERM_NO_AM;
        }
    }

    fn rebuild_acs(&mut self) {
        const ASCII_ACS: &str = "a#j+k+l+m+n+o-p-q-r-s-t+u+v+w+x|y<z>~.";
        self.acs.clear();
        let acs = self
            .string_value("acsc")
            .unwrap_or(ASCII_ACS.as_bytes())
            .to_vec();
        for pair in acs.chunks_exact(2) {
            self.acs.insert(pair[0], pair[1]);
        }
    }

    #[cfg(test)]
    fn acs(&self, input: u8) -> Option<u8> {
        self.acs.get(&input).copied()
    }
}

fn visible_capability(value: &[u8]) -> String {
    let mut output = String::new();
    for byte in value {
        let visible = match byte {
            b'\\' => "\\\\".to_string(),
            b'\n' => "\\n".to_string(),
            b'\r' => "\\r".to_string(),
            b'\t' => "\\t".to_string(),
            0x07 => "\\a".to_string(),
            0x08 => "\\b".to_string(),
            0x0b => "\\v".to_string(),
            0x0c => "\\f".to_string(),
            0x1b => "\\033".to_string(),
            0x20..=0x7e => char::from(*byte).to_string(),
            byte => format!("\\{byte:03o}"),
        };
        if output.len() + visible.len() > 127 {
            break;
        }
        output.push_str(&visible);
    }
    output
}

impl TerminalCapabilities for ResolvedTerm {
    fn name(&self) -> &str {
        &self.identity.name
    }

    fn capability(&self, name: &str) -> Option<&CapabilityValue> {
        self.capabilities.get(name)
    }

    fn has_feature(&self, name: &str) -> bool {
        if self.features.contains(name) {
            return true;
        }
        let Some(feature) = FEATURES.iter().find(|feature| feature.name == name) else {
            return false;
        };
        if name == "ignorefkeys" || self.terminal_flags & feature.flags != feature.flags {
            return false;
        }
        feature.capabilities.iter().all(|capability| {
            let name = capability
                .split_once('=')
                .map_or(*capability, |(name, _)| name);
            let name = name.strip_suffix('@').unwrap_or(name);
            self.has_capability(name)
        })
    }

    fn generation(&self) -> u64 {
        self.generation
    }
}

pub(crate) fn terminal_utf8(terminal: &dyn TerminalCapabilities) -> bool {
    terminal.flag("__hmux_utf8")
}

pub(crate) fn terminal_acs(terminal: &dyn TerminalCapabilities, input: u8) -> Option<u8> {
    const ASCII_ACS: &[u8] = b"a#j+k+l+m+n+o-p-q-r-s-t+u+v+w+x|y<z>~.";
    let acs = match terminal.capability("acsc") {
        Some(CapabilityValue::String(value)) => value.as_slice(),
        _ => ASCII_ACS,
    };
    acs.chunks_exact(2)
        .find(|pair| pair[0] == input)
        .map(|pair| pair[1])
}

/// Width that may be written at `row` without triggering the no-auto-margin
/// bottom-right workaround. Logical rows remain full width.
pub(crate) fn writable_width(
    terminal: &dyn TerminalCapabilities,
    row: u16,
    columns: u16,
    rows: u16,
) -> usize {
    let width = if !terminal.auto_margin() && row == rows {
        columns.saturating_sub(1)
    } else {
        columns
    };
    usize::from(width)
}

fn capability_type(name: &str) -> Option<CapabilityType> {
    if FLAG_CAPABILITIES.contains(&name) {
        Some(CapabilityType::Flag)
    } else if NUMBER_CAPABILITIES.contains(&name) {
        Some(CapabilityType::Number)
    } else if STRING_CAPABILITIES.contains(&name) {
        Some(CapabilityType::String)
    } else {
        None
    }
}

fn identify_value(name: &str, value: &str) -> Option<CapabilityValue> {
    match capability_type(name)? {
        CapabilityType::Flag => Some(CapabilityValue::Flag(value == "1")),
        CapabilityType::Number => value
            .parse::<i32>()
            .ok()
            .filter(|value| *value >= 0)
            .map(CapabilityValue::Number),
        CapabilityType::String => Some(CapabilityValue::String(strip_padding(value))),
    }
}

/// Remove terminfo delay markers. tmux emits terminal output without honoring
/// padding, so `$<5>` and `$<5/>` never reach the outer terminal.
fn strip_padding(value: &str) -> Vec<u8> {
    if !value.contains('$') {
        return value.as_bytes().to_vec();
    }
    let bytes = value.as_bytes();
    let mut stripped = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'$' && bytes.get(index + 1) == Some(&b'<') {
            let Some(end) = bytes[index + 2..].iter().position(|byte| *byte == b'>') else {
                break;
            };
            index += end + 3;
            continue;
        }
        stripped.push(bytes[index]);
        index += 1;
    }
    stripped
}

/// Decode the vis(3) spellings accepted by tmux terminal overrides. Returning
/// `None` preserves the original value, matching tmux's strunvis fallback.
fn decode_vis(value: &str) -> Option<Vec<u8>> {
    let bytes = value
        .as_bytes()
        .split(|byte| *byte == 0)
        .next()
        .unwrap_or_default();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'\\' {
            decoded.push(bytes[index]);
            index += 1;
            continue;
        }
        index += 1;
        let escaped = *bytes.get(index)?;
        index += 1;
        match escaped {
            b'\\' => decoded.push(b'\\'),
            b'E' => decoded.push(0x1b),
            b'n' => decoded.push(b'\n'),
            b'r' => decoded.push(b'\r'),
            b'b' => decoded.push(0x08),
            b'a' => decoded.push(0x07),
            b'v' => decoded.push(0x0b),
            b't' => decoded.push(b'\t'),
            b'f' => decoded.push(0x0c),
            b's' => decoded.push(b' '),
            b'\n' | b'$' => {}
            b'M' => {
                let kind = *bytes.get(index)?;
                index += 1;
                let value = *bytes.get(index)?;
                index += 1;
                decoded.push(match kind {
                    b'-' => 0x80 | value,
                    b'^' if value == b'?' => 0xff,
                    b'^' => 0x80 | (value & 0x1f),
                    _ => return None,
                });
            }
            b'^' => {
                let control = *bytes.get(index)?;
                index += 1;
                decoded.push(if control == b'?' {
                    0x7f
                } else {
                    control & 0x1f
                });
            }
            b'0'..=b'7' => {
                let mut byte = escaped - b'0';
                let mut digits = 1;
                while digits < 3 && index < bytes.len() && matches!(bytes[index], b'0'..=b'7') {
                    byte = byte.wrapping_mul(8).wrapping_add(bytes[index] - b'0');
                    index += 1;
                    digits += 1;
                }
                decoded.push(byte);
            }
            _ => return None,
        }
    }
    if let Some(nul) = decoded.iter().position(|byte| *byte == 0) {
        decoded.truncate(nul);
    }
    Some(decoded)
}

struct TerminalOptionValue<'a> {
    name: &'a str,
    index: Option<u32>,
    value: &'a str,
}

fn terminal_option_values<'a>(
    options: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Vec<TerminalOptionValue<'a>> {
    let mut values = options
        .into_iter()
        .filter_map(|(key, value)| {
            ["terminal-features", "terminal-overrides"]
                .into_iter()
                .find_map(|name| {
                    if key == name {
                        return Some(TerminalOptionValue {
                            name,
                            index: None,
                            value,
                        });
                    }
                    let index = key
                        .strip_prefix(name)?
                        .strip_prefix('[')?
                        .strip_suffix(']')?
                        .parse::<u32>()
                        .ok()?;
                    Some(TerminalOptionValue {
                        name,
                        index: Some(index),
                        value,
                    })
                })
        })
        .collect::<Vec<_>>();

    for (name, defaults) in [
        ("terminal-features", DEFAULT_TERMINAL_FEATURES),
        ("terminal-overrides", DEFAULT_TERMINAL_OVERRIDES),
    ] {
        if values
            .iter()
            .any(|value| value.name == name && value.index.is_none())
        {
            continue;
        }
        for (index, default) in defaults.iter().enumerate() {
            let index = index as u32;
            if values
                .iter()
                .any(|value| value.name == name && value.index == Some(index))
            {
                continue;
            }
            values.push(TerminalOptionValue {
                name,
                index: Some(index),
                value: default,
            });
        }
    }
    values.sort_by_key(|value| (value.name, value.index));
    values
}

fn split_array_entries(value: &str) -> impl Iterator<Item = &str> {
    value.split(',').filter(|entry| !entry.is_empty())
}

/// Split tmux override fields. A doubled colon represents one literal colon.
fn split_override_fields(value: &str) -> Vec<String> {
    let mut fields = vec![String::new()];
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == ':' {
            if chars.peek() == Some(&':') {
                chars.next();
                fields.last_mut().expect("one field").push(':');
            } else {
                fields.push(String::new());
            }
        } else {
            fields.last_mut().expect("one field").push(ch);
        }
    }
    fields
}

fn terminal_pattern_matches(pattern: &str, terminal: &str) -> bool {
    let (Ok(pattern), Ok(terminal)) = (CString::new(pattern), CString::new(terminal)) else {
        return false;
    };
    // SAFETY: both arguments are live NUL-terminated strings for the call.
    unsafe { libc::fnmatch(pattern.as_ptr(), terminal.as_ptr(), 0) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(name: &str, capabilities: &[&str]) -> TerminalIdentity {
        TerminalIdentity::new(
            name,
            capabilities
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            0,
            None,
        )
    }

    #[test]
    fn identified_capabilities_are_typed() {
        let term = ResolvedTerm::resolve(
            identity("screen", &["am=1", "colors=256", "clear=\x1b[H\x1b[2J"]),
            [],
        );
        assert!(term.auto_margin());
        assert_eq!(
            term.capability("colors"),
            Some(&CapabilityValue::Number(256))
        );
        assert_eq!(
            term.capability("clear"),
            Some(&CapabilityValue::String(b"\x1b[H\x1b[2J".to_vec()))
        );
    }

    #[test]
    fn complete_catalog_types_identify_values_and_ignores_unknown_entries() {
        let term = ResolvedTerm::resolve(
            identity(
                "dumb",
                &[
                    "Sxl=0",
                    "U8=1",
                    "kf63=last-key",
                    "bel=beep$<5/>done",
                    "colors=-1",
                    "not-a-tmux-capability=value",
                ],
            ),
            [],
        );
        assert_eq!(term.capability("Sxl"), Some(&CapabilityValue::Flag(false)));
        assert_eq!(term.capability("U8"), Some(&CapabilityValue::Number(1)));
        assert_eq!(
            term.capability("kf63"),
            Some(&CapabilityValue::String(b"last-key".to_vec()))
        );
        assert_eq!(
            term.capability("bel"),
            Some(&CapabilityValue::String(b"beepdone".to_vec()))
        );
        assert_eq!(term.capability("colors"), None);
        assert_eq!(term.capability("not-a-tmux-capability"), None);
    }

    #[test]
    fn overrides_match_term_and_apply_in_order() {
        let options = [("terminal-overrides", "xterm*:am@,screen*:am@,screen*:am")];
        let screen = ResolvedTerm::resolve(identity("screen-256color", &["am=1"]), options);
        let xterm = ResolvedTerm::resolve(identity("xterm-256color", &["am=1"]), options);
        assert!(screen.auto_margin(), "later matching entry restores am");
        assert!(!xterm.auto_margin(), "matching removal clears am");
    }

    #[test]
    fn screen_family_overline_inference_respects_explicit_removal() {
        let inferred = ResolvedTerm::resolve(identity("screen-256color", &[]), []);
        assert!(inferred.capability("Smol").is_some());

        let removed = ResolvedTerm::resolve(
            identity("screen-256color", &[]),
            [("terminal-overrides", "screen*:Smol@")],
        );
        assert!(removed.capability("Smol").is_none());
    }

    #[test]
    fn indexed_overrides_use_numeric_order() {
        let options = [
            ("terminal-overrides[10]", "screen*:am"),
            ("terminal-overrides[2]", "screen*:am@"),
        ];
        let term = ResolvedTerm::resolve(identity("screen", &["am=1"]), options);
        assert!(term.auto_margin());
    }

    #[test]
    fn configured_and_identified_features_are_client_scoped() {
        let mut identified = identity("xterm-256color", &[]);
        identified.feature_bits = 1 << 1; // bpaste
        identified.colorterm = Some("truecolor".into());
        let term = ResolvedTerm::resolve(
            identified,
            [("terminal-features", "screen*:title,xterm*:clipboard")],
        );
        assert!(term.has_feature("bpaste"));
        assert!(term.has_feature("RGB"));
        assert!(term.has_feature("clipboard"));
        assert!(!term.has_feature("title"));
    }

    #[test]
    fn tmux_default_feature_and_override_arrays_are_resolved() {
        let xterm = ResolvedTerm::resolve(identity("xterm-256color", &[]), []);
        for feature in ["clipboard", "ccolour", "cstyle", "focus", "title"] {
            assert!(
                xterm.has_feature(feature),
                "missing default feature {feature}"
            );
        }
        assert!(xterm.capability("Ms").is_some());
        assert!(xterm.capability("Ss").is_some());

        let linux = ResolvedTerm::resolve(identity("linux", &["AX=1"]), []);
        assert_eq!(linux.capability("AX"), None);
    }

    #[test]
    fn indexed_option_entries_replace_the_same_default_index() {
        let term = ResolvedTerm::resolve(
            identity("xterm", &[]),
            [("terminal-features[0]", "other:clipboard")],
        );
        assert!(!term.has_feature("clipboard"));
        assert!(!term.has_feature("ccolour"));
    }

    #[test]
    fn feature_presets_install_capabilities_before_final_overrides() {
        let term = ResolvedTerm::resolve(
            identity("xterm", &[]),
            [
                ("terminal-features", "xterm:RGB:sync:usstyle"),
                (
                    "terminal-overrides",
                    r"xterm:setrgbf=custom:setrgbb@:Sync=\E[custom",
                ),
            ],
        );
        assert!(term.has_feature("RGB"));
        assert!(term.has_feature("sync"));
        assert_eq!(
            term.capability("setrgbf"),
            Some(&CapabilityValue::String(b"custom".to_vec()))
        );
        assert_eq!(term.capability("setrgbb"), None);
        assert_eq!(
            term.capability("Sync"),
            Some(&CapabilityValue::String(b"\x1b[custom".to_vec()))
        );
        assert_eq!(
            term.capability("Smulx"),
            Some(&CapabilityValue::String(b"\x1b[4:%p1%dm".to_vec()))
        );
    }

    #[test]
    fn vt100_and_rgb_capabilities_infer_tmux_features() {
        let term = ResolvedTerm::resolve(identity("xterm", &["clear=\x1b[H\x1b[2J", "Tc=1"]), []);
        for feature in ["bpaste", "focus", "title", "RGB"] {
            assert!(
                term.has_feature(feature),
                "missing inferred feature {feature}"
            );
        }
        assert_eq!(
            term.capability("Enbp"),
            Some(&CapabilityValue::String(b"\x1b[?2004h".to_vec()))
        );
        assert!(term.capability("setrgbf").is_some());
        assert!(term.terminal_flags & TERM_VT100_LIKE != 0);
        assert!(term.terminal_flags & TERM_RGB_COLOURS != 0);
    }

    #[test]
    fn unknown_feature_stops_the_current_feature_list() {
        let term = ResolvedTerm::resolve(
            identity("screen", &[]),
            [("terminal-features", "screen:mouse:unknown:title")],
        );
        assert!(term.has_feature("mouse"));
        assert!(!term.has_feature("title"));
        assert!(term.capability("kmous").is_some());
        assert!(term.capability("tsl").is_none());
    }

    #[test]
    fn ignorefkeys_removes_the_catalogued_function_keys() {
        let mut identified = identity("rxvt", &["kf1=one", "kf12=twelve", "kf63=last"]);
        identified.feature_bits = 1 << 8;
        let term = ResolvedTerm::resolve(identified, []);
        assert!(term.has_feature("ignorefkeys"));
        assert_eq!(term.capability("kf1"), None);
        assert_eq!(term.capability("kf12"), None);
        assert_eq!(term.capability("kf63"), None);
    }

    #[test]
    fn features_can_be_detected_from_existing_capabilities() {
        let term = ResolvedTerm::resolve(
            identity("dumb", &["Enbp=on", "Dsbp=off", "kmous=mouse"]),
            [],
        );
        assert!(term.has_feature("bpaste"));
        assert!(term.has_feature("mouse"));
        assert!(!term.has_feature("ignorefkeys"));
    }

    #[test]
    fn acs_uses_terminfo_pairs_or_tmux_ascii_fallback() {
        let identified = ResolvedTerm::resolve(identity("screen", &["acsc=q-x|"]), []);
        assert_eq!(identified.acs(b'q'), Some(b'-'));
        assert_eq!(identified.acs(b'x'), Some(b'|'));

        let fallback = ResolvedTerm::resolve(identity("dumb", &[]), []);
        assert_eq!(fallback.acs(b'j'), Some(b'+'));
        assert_eq!(fallback.acs(b'x'), Some(b'|'));
    }

    #[test]
    fn override_values_decode_vis_sequences_and_escaped_colons() {
        let term = ResolvedTerm::resolve(
            identity("screen", &[]),
            [("terminal-overrides", r"screen:bel=\E]1::ready\007")],
        );
        assert_eq!(
            term.capability("bel"),
            Some(&CapabilityValue::String(b"\x1b]1:ready\x07".to_vec()))
        );
    }

    #[test]
    fn vis_decoding_matches_tmux_meta_hidden_and_error_rules() {
        assert_eq!(decode_vis(r"\M-A"), Some(vec![0xc1]));
        assert_eq!(
            decode_vis("left\\\nright\\$done"),
            Some(b"leftrightdone".to_vec())
        );
        assert_eq!(decode_vis(r"left\000right"), Some(b"left".to_vec()));
        assert_eq!(decode_vis(r"\e"), None);
        assert_eq!(decode_vis(r"\x1b"), None);
    }

    #[test]
    fn parameterized_strings_expand_numeric_and_string_arguments() {
        let term = ResolvedTerm::resolve(
            identity(
                "screen",
                &[
                    "colors=256",
                    "cup=\x1b[%i%p1%d;%p2%dH",
                    "Ms=\x1b]52;%p1%s;%p2%s\x07",
                ],
            ),
            [],
        );
        assert_eq!(number_capability(&term, "colors"), Some(256));
        assert_eq!(
            expand_capability(
                &term,
                "cup",
                &[
                    CapabilityParameter::Number(0),
                    CapabilityParameter::Number(4),
                ],
            ),
            Some(b"\x1b[1;5H".to_vec())
        );
        assert_eq!(
            expand_capability(
                &term,
                "Ms",
                &[
                    CapabilityParameter::String("c"),
                    CapabilityParameter::String("aGVsbG8="),
                ],
            ),
            Some(b"\x1b]52;c;aGVsbG8=\x07".to_vec())
        );
        assert_eq!(expand_capability(&term, "colors", &[]), None);
    }

    #[test]
    fn feature_strings_expand_tmux_conditionals_and_arithmetic() {
        let term = ResolvedTerm::resolve(
            identity("modern", &[]),
            [("terminal-features", "modern:RGB:hyperlinks:sync:usstyle")],
        );
        for (colour, expected) in [
            (1, b"\x1b[31m".as_slice()),
            (8, b"\x1b[90m".as_slice()),
            (16, b"\x1b[38;5;16m".as_slice()),
        ] {
            assert_eq!(
                expand_capability(&term, "setaf", &[CapabilityParameter::Number(colour)]),
                Some(expected.to_vec())
            );
        }
        assert_eq!(
            expand_capability(&term, "Sync", &[CapabilityParameter::Number(1)]),
            Some(b"\x1b[?2026h".to_vec())
        );
        assert_eq!(
            expand_capability(&term, "Sync", &[CapabilityParameter::Number(2)]),
            Some(b"\x1b[?2026l".to_vec())
        );
        assert_eq!(
            expand_capability(
                &term,
                "Hls",
                &[
                    CapabilityParameter::String("link"),
                    CapabilityParameter::String("https://example.test"),
                ],
            ),
            Some(b"\x1b]8;id=link;https://example.test\x1b\\".to_vec())
        );
    }

    #[test]
    fn setulc_stray_conditional_matches_tmux() {
        let term = ResolvedTerm::resolve(
            identity("modern", &[]),
            [("terminal-features", "modern:RGB:hyperlinks:sync:usstyle")],
        );
        assert_eq!(
            expand_capability(&term, "Setulc", &[CapabilityParameter::Number(0x11_22_33)]),
            Some(b"\x1b[58:2:17:34:51m".to_vec())
        );
    }

    #[test]
    fn conditional_normalization_preserves_escaped_percent_sequences() {
        assert_eq!(
            strip_unmatched_conditional_ends(b"%%?literal%%;%;"),
            b"%%?literal%%;"
        );
        assert_eq!(
            strip_unmatched_conditional_ends(b"%?%p1%ttrue%;%;"),
            b"%?%p1%ttrue%;"
        );
    }

    #[test]
    fn clear_and_cup_are_required_for_an_attached_terminal() {
        let missing_both = ResolvedTerm::resolve(identity("dumb", &[]), []);
        assert_eq!(
            missing_both.validation_error(),
            Some("terminal does not support clear")
        );

        let missing_cup = ResolvedTerm::resolve(identity("dumb", &["clear=clear"]), []);
        assert_eq!(
            missing_cup.validation_error(),
            Some("terminal does not support cup")
        );

        let complete = ResolvedTerm::resolve(identity("dumb", &["clear=clear", "cup=move"]), []);
        assert_eq!(complete.validation_error(), None);
    }

    #[test]
    fn only_the_physical_bottom_row_loses_a_cell_without_am() {
        let term = ResolvedTerm::resolve(identity("screen", &["am=0"]), []);
        assert_eq!(writable_width(&term, 1, 20, 2), 20);
        assert_eq!(writable_width(&term, 2, 20, 2), 19);
    }
}
