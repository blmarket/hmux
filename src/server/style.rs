//! Shared visual cell style, parsing, decoding, and serialization.
//!
//! Layout and cursor operations deliberately stay with their renderers. This
//! module owns only cell presentation and transitions between presentations.

use super::term::{
    expand_capability, number_capability, string_capability, Capability, CapabilityParameter,
    TerminalCapabilities,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Colour {
    #[default]
    Default,
    /// One of the sixteen colours in its own SGR spelling — 30–37 and 90–97,
    /// or their background counterparts.
    Palette(u8),
    /// A palette index that arrived through the 256-colour form, tmux's
    /// `COLOUR_FLAG_256`. It is kept apart from [`Colour::Palette`] because a
    /// capture reproduces the spelling a colour arrived in rather than picking
    /// one: `38;5;1` comes back as `38;5;1`, not as `31`.
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Underline {
    #[default]
    None,
    Single,
    Double,
    Curly,
    Dotted,
    Dashed,
}

impl Underline {
    fn from_sgr(value: u16) -> Option<Self> {
        Some(match value {
            0 => Self::None,
            1 => Self::Single,
            2 => Self::Double,
            3 => Self::Curly,
            4 => Self::Dotted,
            5 => Self::Dashed,
            _ => return None,
        })
    }

    fn sgr(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Single => 1,
            Self::Double => 2,
            Self::Curly => 3,
            Self::Dotted => 4,
            Self::Dashed => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Attributes(u16);

impl Attributes {
    pub(crate) const BOLD: u16 = 1 << 0;
    pub(crate) const DIM: u16 = 1 << 1;
    pub(crate) const ITALICS: u16 = 1 << 2;
    pub(crate) const BLINK: u16 = 1 << 3;
    pub(crate) const REVERSE: u16 = 1 << 4;
    pub(crate) const HIDDEN: u16 = 1 << 5;
    pub(crate) const STRIKETHROUGH: u16 = 1 << 6;
    pub(crate) const OVERLINE: u16 = 1 << 7;

    pub(crate) fn has(self, flag: u16) -> bool {
        self.0 & flag != 0
    }

    pub(crate) fn set(&mut self, flag: u16, enabled: bool) {
        if enabled {
            self.0 |= flag;
        } else {
            self.0 &= !flag;
        }
    }

    fn removed_from(self, newer: Self) -> bool {
        self.0 & !newer.0 != 0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CellStyle {
    pub(crate) fg: Colour,
    pub(crate) bg: Colour,
    pub(crate) underline_colour: Colour,
    pub(crate) attributes: Attributes,
    pub(crate) underline: Underline,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Hyperlink {
    pub(crate) id: String,
    pub(crate) uri: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CellPresentation {
    pub(crate) style: CellStyle,
    pub(crate) hyperlink: Option<Hyperlink>,
    /// Which OSC 8 in the stream opened this link, counted from the last close.
    ///
    /// Two anonymous OSC 8s naming the same URI are two links, not one — tmux
    /// gives each its own inner id, so a capture re-announces the URI between
    /// them. The URI and the `id=` cannot tell them apart, so the sequence they
    /// arrived in does. Zero whenever no link is open, which keeps this out of
    /// the comparison everywhere a link is not involved.
    pub(crate) hyperlink_epoch: u64,
    pub(crate) acs: bool,
}

/// Fields mentioned by a visual style directive. Attribute changes use a mask
/// and value so `bold` and `nobold` remain distinct from inheritance.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct StylePatch {
    fg: Option<Colour>,
    bg: Option<Colour>,
    underline_colour: Option<Colour>,
    attribute_mask: u16,
    attribute_values: u16,
    underline: Option<Underline>,
    reset_attributes: bool,
    reset_to_default: bool,
}

impl StylePatch {
    pub(crate) fn apply(self, base: &CellStyle, current_default: &CellStyle) -> CellStyle {
        let mut style = if self.reset_to_default {
            *current_default
        } else {
            *base
        };
        if self.reset_attributes {
            style.attributes = Attributes::default();
            style.underline = Underline::None;
        }
        style.attributes.0 = (style.attributes.0 & !self.attribute_mask)
            | (self.attribute_values & self.attribute_mask);
        if let Some(value) = self.fg {
            style.fg = value;
        }
        if let Some(value) = self.bg {
            style.bg = value;
        }
        if let Some(value) = self.underline_colour {
            style.underline_colour = value;
        }
        if let Some(value) = self.underline {
            style.underline = value;
        }
        style
    }

    fn set_attribute(&mut self, flag: u16, enabled: bool) {
        self.attribute_mask |= flag;
        if enabled {
            self.attribute_values |= flag;
        } else {
            self.attribute_values &= !flag;
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VisualToken {
    Applied,
    NotVisual,
}

/// Apply one visual token. Invalid values return an error; status-specific
/// callers can distinguish nonvisual metadata and retain atomic rollback.
pub(crate) fn apply_visual_token(
    token: &str,
    style: &mut CellStyle,
    current_default: &CellStyle,
) -> Result<VisualToken, ()> {
    let mut patch = StylePatch::default();
    match token {
        "default" => patch.reset_to_default = true,
        "noattr" | "none" => patch.reset_attributes = true,
        "bold" | "bright" => patch.set_attribute(Attributes::BOLD, true),
        "nobold" | "nobright" => patch.set_attribute(Attributes::BOLD, false),
        "dim" => patch.set_attribute(Attributes::DIM, true),
        "nodim" => patch.set_attribute(Attributes::DIM, false),
        "italics" => patch.set_attribute(Attributes::ITALICS, true),
        "noitalics" => patch.set_attribute(Attributes::ITALICS, false),
        "blink" => patch.set_attribute(Attributes::BLINK, true),
        "noblink" => patch.set_attribute(Attributes::BLINK, false),
        "reverse" => patch.set_attribute(Attributes::REVERSE, true),
        "noreverse" => patch.set_attribute(Attributes::REVERSE, false),
        "hidden" => patch.set_attribute(Attributes::HIDDEN, true),
        "nohidden" => patch.set_attribute(Attributes::HIDDEN, false),
        "strikethrough" => patch.set_attribute(Attributes::STRIKETHROUGH, true),
        "nostrikethrough" => patch.set_attribute(Attributes::STRIKETHROUGH, false),
        "overline" => patch.set_attribute(Attributes::OVERLINE, true),
        "nooverline" => patch.set_attribute(Attributes::OVERLINE, false),
        "underscore" => patch.underline = Some(Underline::Single),
        "double-underscore" => patch.underline = Some(Underline::Double),
        "curly-underscore" => patch.underline = Some(Underline::Curly),
        "dotted-underscore" => patch.underline = Some(Underline::Dotted),
        "dashed-underscore" => patch.underline = Some(Underline::Dashed),
        "nounderscore" if matches!(style.underline, Underline::None | Underline::Single) => {
            patch.underline = Some(Underline::None)
        }
        "nodouble-underscore" if style.underline == Underline::Double => {
            patch.underline = Some(Underline::None)
        }
        "nocurly-underscore" if style.underline == Underline::Curly => {
            patch.underline = Some(Underline::None)
        }
        "nodotted-underscore" if style.underline == Underline::Dotted => {
            patch.underline = Some(Underline::None)
        }
        "nodashed-underscore" if style.underline == Underline::Dashed => {
            patch.underline = Some(Underline::None)
        }
        "nounderscore"
        | "nodouble-underscore"
        | "nocurly-underscore"
        | "nodotted-underscore"
        | "nodashed-underscore" => {}
        _ => {
            if let Some(value) = token.strip_prefix("fg=") {
                patch.fg = Some(parse_colour(value).ok_or(())?);
            } else if let Some(value) = token.strip_prefix("bg=") {
                patch.bg = Some(parse_colour(value).ok_or(())?);
            } else if let Some(value) = token.strip_prefix("us=") {
                patch.underline_colour = Some(parse_underline_colour(value).ok_or(())?);
            } else {
                return Ok(VisualToken::NotVisual);
            }
        }
    }
    *style = patch.apply(style, current_default);
    Ok(VisualToken::Applied)
}

pub(crate) fn split_style_parts(value: &str) -> impl Iterator<Item = &str> {
    value
        .split(|character: char| character == ',' || character.is_ascii_whitespace())
        .filter(|part| !part.is_empty())
}

pub(crate) fn invalid_underline_colour(value: &str) -> Option<&str> {
    split_style_parts(value).find(|token| {
        token
            .strip_prefix("us=")
            .is_some_and(|value| parse_underline_colour(value).is_none())
    })
}

/// tmux's `colour_totheme`: whether a colour reads as a dark or light
/// background. `None` is `THEME_UNKNOWN` — the default colour, which says
/// nothing about the terminal's theme.
pub(crate) fn colour_theme(colour: Colour) -> Option<&'static str> {
    let (red, green, blue) = match colour {
        Colour::Default => return None,
        Colour::Rgb(red, green, blue) => (red, green, blue),
        Colour::Palette(index) | Colour::Indexed(index) => palette_rgb(index),
    };
    let brightness = u32::from(red) + u32::from(green) + u32::from(blue);
    Some(if brightness > 382 { "light" } else { "dark" })
}

/// A colour *option*'s value as the packed `0xrrggbb` an OSC 4 reply carries —
/// tmux's `colour_force_rgb` over what `colour_fromstring` read. A palette
/// index resolves through the stock xterm palette rather than through the X11
/// colour names, which name different colours for the same words.
pub(crate) fn packed_option_colour(value: &str) -> Option<u32> {
    let (red, green, blue) = match parse_colour(value)? {
        Colour::Default => return None,
        Colour::Rgb(red, green, blue) => (red, green, blue),
        Colour::Palette(index) | Colour::Indexed(index) => palette_rgb(index),
    };
    Some((u32::from(red) << 16) | (u32::from(green) << 8) | u32::from(blue))
}

/// The RGB value of an xterm 256-colour palette entry.
fn palette_rgb(index: u8) -> (u8, u8, u8) {
    const BASE: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (0, 0, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];
    const STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    match index {
        0..=15 => BASE[index as usize],
        16..=231 => {
            let cube = index - 16;
            (
                STEPS[(cube / 36) as usize],
                STEPS[((cube / 6) % 6) as usize],
                STEPS[(cube % 6) as usize],
            )
        }
        _ => {
            let level = 8 + 10 * (index - 232);
            (level, level, level)
        }
    }
}

/// The colour names tmux's `colour_fromstring` recognises, paired with the
/// number it maps them to; `colour_tostring` spells the same number back.
const OPTION_COLOUR_NAMES: [(&str, u16); 18] = [
    ("black", 0),
    ("red", 1),
    ("green", 2),
    ("yellow", 3),
    ("blue", 4),
    ("magenta", 5),
    ("cyan", 6),
    ("white", 7),
    ("default", 8),
    ("terminal", 9),
    ("brightblack", 90),
    ("brightred", 91),
    ("brightgreen", 92),
    ("brightyellow", 93),
    ("brightblue", 94),
    ("brightmagenta", 95),
    ("brightcyan", 96),
    ("brightwhite", 97),
];

/// The canonical spelling of a colour-valued option: tmux's
/// `colour_fromstring` followed by `colour_tostring`. `None` is a value tmux
/// rejects outright with `bad colour`.
///
/// This is deliberately not [`parse_colour`], which reads the colours a *style*
/// or an SGR sequence names and folds the bright names onto palette entries
/// 8-15. An option keeps tmux's distinction between `colour9` and `brightred`.
pub(crate) fn canonical_option_colour(value: &str) -> Option<String> {
    if value.len() == 7 && value.starts_with('#') {
        let hex = &value[1..];
        if hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Some(format!("#{}", hex.to_ascii_lowercase()));
        }
    }
    for prefix in ["colour", "color"] {
        if value.len() >= prefix.len() && value[..prefix.len()].eq_ignore_ascii_case(prefix) {
            let index = value[prefix.len()..].parse::<u16>().ok().filter(|n| *n < 256)?;
            return Some(format!("colour{index}"));
        }
    }
    let named = OPTION_COLOUR_NAMES
        .iter()
        .find(|(name, number)| {
            value.eq_ignore_ascii_case(name)
                // `default` and `terminal` have no numeric spelling; the
                // sixteen ANSI names do, and tmux compares those exactly rather
                // than case-insensitively.
                || (!matches!(number, 8 | 9) && value == number.to_string())
        });
    if let Some((name, _)) = named {
        return Some((*name).to_string());
    }
    hmux_vt::colour_by_name(value).map(|rgb| format!("#{rgb:06x}"))
}

pub(crate) fn parse_colour(value: &str) -> Option<Colour> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("default")
        || value.eq_ignore_ascii_case("none")
        || value.eq_ignore_ascii_case("terminal")
    {
        return Some(Colour::Default);
    }
    let named = match value.to_ascii_lowercase().as_str() {
        "black" => Some(0),
        "red" => Some(1),
        "green" => Some(2),
        "yellow" => Some(3),
        "blue" => Some(4),
        "magenta" => Some(5),
        "cyan" => Some(6),
        "white" => Some(7),
        "brightblack" | "bright-black" | "grey" | "gray" => Some(8),
        "brightred" | "bright-red" => Some(9),
        "brightgreen" | "bright-green" => Some(10),
        "brightyellow" | "bright-yellow" => Some(11),
        "brightblue" | "bright-blue" => Some(12),
        "brightmagenta" | "bright-magenta" => Some(13),
        "brightcyan" | "bright-cyan" => Some(14),
        "brightwhite" | "bright-white" => Some(15),
        _ => None,
    };
    if let Some(index) = named {
        return Some(Colour::Palette(index));
    }
    if let Some(number) = value
        .strip_prefix("colour")
        .or_else(|| value.strip_prefix("color"))
    {
        return number.parse::<u8>().ok().map(Colour::Palette);
    }
    if let Some(hex) = value.strip_prefix('#').filter(|hex| hex.len() == 6) {
        return Some(Colour::Rgb(
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
        ));
    }
    if let Some(rgb) = value.strip_prefix("rgb:") {
        let mut parts = rgb.split('/');
        let colour = Colour::Rgb(
            u8::from_str_radix(parts.next()?, 16).ok()?,
            u8::from_str_radix(parts.next()?, 16).ok()?,
            u8::from_str_radix(parts.next()?, 16).ok()?,
        );
        return parts.next().is_none().then_some(colour);
    }
    value.parse::<u8>().ok().map(Colour::Palette)
}

fn parse_underline_colour(value: &str) -> Option<Colour> {
    let value = value.trim();
    if value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    parse_colour(value)
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SgrDecoder {
    style: CellStyle,
}

impl SgrDecoder {
    pub(crate) fn style(&self) -> CellStyle {
        self.style
    }

    pub(crate) fn apply(&mut self, parameters: &[u8]) {
        let text = String::from_utf8_lossy(parameters);
        let fields = if text.is_empty() {
            vec![""]
        } else {
            text.split(';').collect::<Vec<_>>()
        };
        let mut index = 0;
        while index < fields.len() {
            let field = fields[index];
            if field.contains(':') {
                self.apply_colon(field);
                index += 1;
                continue;
            }
            let code = field.parse::<u16>().unwrap_or(0);
            index += 1;
            match code {
                38 | 48 | 58 => {
                    if let Some((colour, used)) = parse_semicolon_colour(&fields[index..]) {
                        self.set_colour(code, colour);
                        index += used;
                    }
                }
                _ => self.apply_code(code),
            }
        }
    }

    fn apply_colon(&mut self, field: &str) {
        let values = field
            .split(':')
            .map(|value| {
                (!value.is_empty())
                    .then(|| value.parse::<u16>().ok())
                    .flatten()
            })
            .collect::<Vec<_>>();
        let Some(code) = values.first().copied().flatten() else {
            return;
        };
        if code == 4 {
            if let Some(value) = values
                .get(1)
                .copied()
                .flatten()
                .and_then(Underline::from_sgr)
            {
                self.style.underline = value;
            }
            return;
        }
        if matches!(code, 38 | 48 | 58) {
            let mode = values.get(1).copied().flatten();
            let components = &values[2..];
            let colour = match mode {
                Some(5) => components
                    .iter()
                    .flatten()
                    .next()
                    .map(|value| Colour::Indexed(*value as u8)),
                Some(2) => {
                    let rgb = components
                        .iter()
                        .filter_map(|value| *value)
                        .collect::<Vec<_>>();
                    (rgb.len() >= 3).then(|| {
                        Colour::Rgb(
                            rgb[rgb.len() - 3] as u8,
                            rgb[rgb.len() - 2] as u8,
                            rgb[rgb.len() - 1] as u8,
                        )
                    })
                }
                _ => None,
            };
            if let Some(colour) = colour {
                self.set_colour(code, colour);
            }
            return;
        }
        self.apply_code(code);
    }

    fn set_colour(&mut self, code: u16, colour: Colour) {
        match code {
            38 => self.style.fg = colour,
            48 => self.style.bg = colour,
            58 => self.style.underline_colour = colour,
            _ => {}
        }
    }

    fn apply_code(&mut self, code: u16) {
        match code {
            0 => self.style = CellStyle::default(),
            1 => self.style.attributes.set(Attributes::BOLD, true),
            2 => self.style.attributes.set(Attributes::DIM, true),
            3 => self.style.attributes.set(Attributes::ITALICS, true),
            4 => self.style.underline = Underline::Single,
            5 | 6 => self.style.attributes.set(Attributes::BLINK, true),
            7 => self.style.attributes.set(Attributes::REVERSE, true),
            8 => self.style.attributes.set(Attributes::HIDDEN, true),
            9 => self.style.attributes.set(Attributes::STRIKETHROUGH, true),
            21 => self.style.underline = Underline::Double,
            22 => {
                self.style.attributes.set(Attributes::BOLD, false);
                self.style.attributes.set(Attributes::DIM, false);
            }
            23 => self.style.attributes.set(Attributes::ITALICS, false),
            24 => self.style.underline = Underline::None,
            25 => self.style.attributes.set(Attributes::BLINK, false),
            27 => self.style.attributes.set(Attributes::REVERSE, false),
            28 => self.style.attributes.set(Attributes::HIDDEN, false),
            29 => self.style.attributes.set(Attributes::STRIKETHROUGH, false),
            30..=37 => self.style.fg = Colour::Palette((code - 30) as u8),
            39 => self.style.fg = Colour::Default,
            40..=47 => self.style.bg = Colour::Palette((code - 40) as u8),
            49 => self.style.bg = Colour::Default,
            53 => self.style.attributes.set(Attributes::OVERLINE, true),
            55 => self.style.attributes.set(Attributes::OVERLINE, false),
            59 => self.style.underline_colour = Colour::Default,
            90..=97 => self.style.fg = Colour::Palette((code - 90 + 8) as u8),
            100..=107 => self.style.bg = Colour::Palette((code - 100 + 8) as u8),
            _ => {}
        }
    }
}

fn parse_semicolon_colour(fields: &[&str]) -> Option<(Colour, usize)> {
    let mode = fields.first()?.parse::<u8>().ok()?;
    match mode {
        5 => Some((Colour::Indexed(fields.get(1)?.parse().ok()?), 2)),
        2 => {
            let skip = usize::from(fields.first().is_some_and(|_| fields.get(1) == Some(&"")));
            let start = 1 + skip;
            Some((
                Colour::Rgb(
                    fields.get(start)?.parse().ok()?,
                    fields.get(start + 1)?.parse().ok()?,
                    fields.get(start + 2)?.parse().ok()?,
                ),
                start + 3,
            ))
        }
        _ => None,
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CaptureStyleWriter {
    current: CellPresentation,
    /// tmux's `has_link`: whether *this row* has opened a hyperlink in the
    /// output. It is the only per-row state `grid_string_cells` keeps; the cell
    /// it compares against is carried across rows with everything else.
    has_link: bool,
    /// The bytes the last [`Self::transition`] wrote — tmux's `code`, the
    /// buffer `grid_string_cells_code` fills for one cell. It outlives the cell
    /// there, and the OSC 8 that closes a row's open link is appended to it
    /// rather than to a fresh buffer, which is why that close can arrive
    /// carrying the last cell's sequences in front of it.
    last_code: Vec<u8>,
}

impl CaptureStyleWriter {
    pub(crate) fn transition(&mut self, out: &mut Vec<u8>, next: &CellPresentation) {
        let start = out.len();
        write_capture_style(out, &self.current.style, &next.style);
        // tmux writes the shift in/out between the style codes and the
        // hyperlink, so a cell changing both is preceded by SO then OSC 8.
        if self.current.acs != next.acs {
            out.push(if next.acs { 0x0e } else { 0x0f });
        }
        if self.current.hyperlink != next.hyperlink
            || self.current.hyperlink_epoch != next.hyperlink_epoch
        {
            if next.hyperlink.is_some() {
                write_capture_hyperlink(out, next.hyperlink.as_ref());
                self.has_link = true;
            } else if self.has_link {
                // Only a link this row opened is closed here. One carried in
                // from the row above is left alone, which is what leaves a
                // continuing link unmentioned on the rows that follow it.
                write_capture_hyperlink(out, None);
                self.has_link = false;
            }
        }
        self.last_code.clear();
        self.last_code.extend_from_slice(&out[start..]);
        self.current = next.clone();
    }

    /// End one row of a capture.
    ///
    /// The style, the line-drawing set and the cell's hyperlink are
    /// deliberately left open. tmux carries `grid_string_cells`'s last cell
    /// across the rows of a single `capture-pane`, so a row that ends mid-style
    /// is followed either by a row that opens with the transition or, if it is
    /// the last one, by nothing at all — closing here would put a reset at the
    /// end of every capture that tmux does not write. Only the row's own open
    /// hyperlink is closed, because that is the one piece of state tmux keeps
    /// per row.
    ///
    /// `repeat_last_code` reproduces the one place `grid_string_cells` shows
    /// its buffer: the close is appended to the sequences the last cell needed
    /// rather than to a fresh buffer, so a row whose *last* cell is the one
    /// that opened the link ends with that cell's sequences written a second
    /// time and then closed. The caller decides, because only it can see
    /// whether the last cell is where the last transition happened.
    pub(crate) fn finish_row(&mut self, out: &mut Vec<u8>, repeat_last_code: bool) {
        if self.has_link {
            if repeat_last_code {
                let code = std::mem::take(&mut self.last_code);
                out.extend_from_slice(&code);
                self.last_code = code;
            }
            write_capture_hyperlink(out, None);
            self.has_link = false;
        }
    }
}

fn write_capture_style(out: &mut Vec<u8>, old: &CellStyle, new: &CellStyle) {
    if old == new {
        return;
    }
    let reset = old.attributes.removed_from(new.attributes)
        || (old.underline != Underline::None && new.underline != old.underline)
        || (old.underline_colour != Colour::Default && new.underline_colour == Colour::Default);
    let baseline = if reset { CellStyle::default() } else { *old };
    // `grid_string_cells_code` collects the reset and every attribute that has
    // to be set again into one parameter list, so a capture that has to reset
    // before it can set writes `ESC [ 0 ; 7 m` rather than two sequences.
    let mut attrs = Vec::new();
    if reset {
        attrs.push("0".to_string());
    }
    for (flag, code) in [
        (Attributes::BOLD, "1"),
        (Attributes::DIM, "2"),
        (Attributes::ITALICS, "3"),
    ] {
        if new.attributes.has(flag) && !baseline.attributes.has(flag) {
            attrs.push(code.to_string());
        }
    }
    if new.underline == Underline::Single && new.underline != baseline.underline {
        attrs.push("4".into());
    }
    for (flag, code) in [
        (Attributes::BLINK, "5"),
        (Attributes::REVERSE, "7"),
        (Attributes::HIDDEN, "8"),
        (Attributes::STRIKETHROUGH, "9"),
    ] {
        if new.attributes.has(flag) && !baseline.attributes.has(flag) {
            attrs.push(code.to_string());
        }
    }
    if !matches!(new.underline, Underline::None | Underline::Single)
        && new.underline != baseline.underline
    {
        attrs.push(format!("4:{}", new.underline.sgr()));
    }
    if new.attributes.has(Attributes::OVERLINE) && !baseline.attributes.has(Attributes::OVERLINE) {
        attrs.push("5:3".into());
    }
    push_sgr(out, &attrs);
    if new.fg != baseline.fg {
        push_sgr(out, &colour_codes(new.fg, 38));
    }
    if new.bg != baseline.bg {
        push_sgr(out, &colour_codes(new.bg, 48));
    }
    if new.underline_colour != baseline.underline_colour {
        push_sgr(out, &colour_codes(new.underline_colour, 58));
    }
}

fn colour_codes(colour: Colour, prefix: u8) -> Vec<String> {
    match colour {
        Colour::Default => vec![match prefix {
            38 => "39",
            48 => "49",
            _ => "59",
        }
        .into()],
        Colour::Palette(index) if prefix != 58 && index < 8 => {
            vec![((if prefix == 48 { 40 } else { 30 }) + index).to_string()]
        }
        Colour::Palette(index) if prefix != 58 && index < 16 => {
            vec![((if prefix == 48 { 100 } else { 90 }) + index - 8).to_string()]
        }
        Colour::Palette(index) | Colour::Indexed(index) => {
            vec![prefix.to_string(), "5".into(), index.to_string()]
        }
        Colour::Rgb(red, green, blue) => vec![
            prefix.to_string(),
            "2".into(),
            red.to_string(),
            green.to_string(),
            blue.to_string(),
        ],
    }
}

fn push_sgr(out: &mut Vec<u8>, codes: &[String]) {
    if codes.is_empty() {
        return;
    }
    out.extend_from_slice(b"\x1b[");
    out.extend_from_slice(codes.join(";").as_bytes());
    out.push(b'm');
}

/// The OSC 8 that opens `link`, or closes whatever is open when it is `None`.
///
/// This is unconditional: the caller has already decided the link changed, and
/// two anonymous links naming one URI are a change the sequences cannot show.
pub(crate) fn write_capture_hyperlink(out: &mut Vec<u8>, new: Option<&Hyperlink>) {
    out.extend_from_slice(b"\x1b]8;");
    if let Some(link) = new {
        if !link.id.is_empty() {
            out.extend_from_slice(b"id=");
            out.extend_from_slice(link.id.as_bytes());
        }
        out.push(b';');
        out.extend_from_slice(link.uri.as_bytes());
    } else {
        out.push(b';');
    }
    out.extend_from_slice(b"\x1b\\");
}

pub(crate) struct TerminalStyleWriter<'a> {
    terminal: &'a dyn TerminalCapabilities,
    current: Option<CellPresentation>,
}

impl<'a> TerminalStyleWriter<'a> {
    pub(crate) fn new(terminal: &'a dyn TerminalCapabilities) -> Self {
        Self {
            terminal,
            current: Some(CellPresentation::default()),
        }
    }

    #[allow(dead_code)] // used at raw-output boundaries as more surfaces migrate
    pub(crate) fn invalidate(&mut self) {
        self.current = None;
    }

    pub(crate) fn reset(&mut self, out: &mut Vec<u8>) {
        if let Some(value) = string_capability(self.terminal, Capability::sgr0) {
            out.extend_from_slice(value);
            self.current = Some(CellPresentation::default());
        } else {
            self.current = None;
        }
    }

    pub(crate) fn transition(&mut self, out: &mut Vec<u8>, requested: &CellPresentation) {
        let next = self.supported(requested);
        let mut old = self.current.clone().unwrap_or_default();
        if self.current.is_none()
            || old.style.attributes.removed_from(next.style.attributes)
            || (old.style.underline != Underline::None
                && old.style.underline != next.style.underline)
            || (old.style.underline_colour != Colour::Default
                && next.style.underline_colour == Colour::Default)
            || (!self.terminal.flag(Capability::AX)
                && ((old.style.fg != Colour::Default && next.style.fg == Colour::Default)
                    || (old.style.bg != Colour::Default && next.style.bg == Colour::Default)))
        {
            self.reset(out);
            old = CellPresentation::default();
        }
        self.write_colours(out, &old.style, &next.style);
        self.write_attributes(out, &old.style, &next.style);
        if old.acs != next.acs {
            append_capability(
                out,
                self.terminal,
                if next.acs {
                    Capability::smacs
                } else {
                    Capability::rmacs
                },
                &[],
            );
        }
        self.write_hyperlink(out, old.hyperlink.as_ref(), next.hyperlink.as_ref());
        self.current = Some(next);
    }

    fn supported(&self, requested: &CellPresentation) -> CellPresentation {
        let mut value = requested.clone();
        value.style.fg = terminal_colour(self.terminal, value.style.fg);
        value.style.bg = terminal_colour(self.terminal, value.style.bg);
        if value.style.underline != Underline::None {
            if value.style.underline != Underline::Single
                && string_capability(self.terminal, Capability::Smulx).is_none()
            {
                value.style.underline = Underline::Single;
            }
            if value.style.underline == Underline::Single
                && string_capability(self.terminal, Capability::smul).is_none()
            {
                value.style.underline = Underline::None;
            }
        }
        if value.style.underline_colour != Colour::Default
            && string_capability(self.terminal, Capability::Setulc).is_none()
            && string_capability(self.terminal, Capability::Setulc1).is_none()
        {
            value.style.underline_colour = Colour::Default;
        }
        for (flag, capability) in [
            (Attributes::BOLD, Capability::bold),
            (Attributes::DIM, Capability::dim),
            (Attributes::ITALICS, Capability::sitm),
            (Attributes::BLINK, Capability::blink),
            (Attributes::HIDDEN, Capability::invis),
            (Attributes::STRIKETHROUGH, Capability::smxx),
            (Attributes::OVERLINE, Capability::Smol),
        ] {
            if string_capability(self.terminal, capability).is_none() {
                value.style.attributes.set(flag, false);
            }
        }
        if string_capability(self.terminal, Capability::Hls).is_none() {
            value.hyperlink = None;
        }
        if string_capability(self.terminal, Capability::rev).is_none()
            && string_capability(self.terminal, Capability::smso).is_none()
        {
            value.style.attributes.set(Attributes::REVERSE, false);
        }
        if self.terminal.utf8()
            || string_capability(self.terminal, Capability::smacs).is_none()
            || string_capability(self.terminal, Capability::rmacs).is_none()
        {
            value.acs = false;
        }
        value
    }

    fn write_colours(&self, out: &mut Vec<u8>, old: &CellStyle, new: &CellStyle) {
        if old.fg != new.fg {
            self.write_colour(out, new.fg, false);
        }
        if old.bg != new.bg {
            self.write_colour(out, new.bg, true);
        }
        if old.underline_colour != new.underline_colour {
            match new.underline_colour {
                Colour::Default => append_capability(out, self.terminal, Capability::ol, &[]),
                Colour::Palette(index) | Colour::Indexed(index) => append_capability(
                    out,
                    self.terminal,
                    Capability::Setulc1,
                    &[CapabilityParameter::Number(index.into())],
                ),
                Colour::Rgb(red, green, blue) => append_capability(
                    out,
                    self.terminal,
                    Capability::Setulc,
                    &[CapabilityParameter::Number(
                        (i32::from(red) << 16) | (i32::from(green) << 8) | i32::from(blue),
                    )],
                ),
            }
        }
    }

    fn write_colour(&self, out: &mut Vec<u8>, colour: Colour, background: bool) {
        match colour {
            Colour::Default if self.terminal.flag(Capability::AX) => {
                out.extend_from_slice(if background { b"\x1b[49m" } else { b"\x1b[39m" });
            }
            Colour::Default => {}
            Colour::Palette(index) | Colour::Indexed(index) => append_capability(
                out,
                self.terminal,
                if background {
                    Capability::setab
                } else {
                    Capability::setaf
                },
                &[CapabilityParameter::Number(index.into())],
            ),
            Colour::Rgb(red, green, blue) => append_capability(
                out,
                self.terminal,
                if background {
                    Capability::setrgbb
                } else {
                    Capability::setrgbf
                },
                &[
                    CapabilityParameter::Number(red.into()),
                    CapabilityParameter::Number(green.into()),
                    CapabilityParameter::Number(blue.into()),
                ],
            ),
        }
    }

    fn write_attributes(&self, out: &mut Vec<u8>, old: &CellStyle, new: &CellStyle) {
        for (flag, capability) in [
            (Attributes::BOLD, Capability::bold),
            (Attributes::DIM, Capability::dim),
            (Attributes::ITALICS, Capability::sitm),
            (Attributes::BLINK, Capability::blink),
            (Attributes::HIDDEN, Capability::invis),
            (Attributes::STRIKETHROUGH, Capability::smxx),
            (Attributes::OVERLINE, Capability::Smol),
        ] {
            if new.attributes.has(flag) && !old.attributes.has(flag) {
                append_capability(out, self.terminal, capability, &[]);
            }
        }
        if new.attributes.has(Attributes::REVERSE) && !old.attributes.has(Attributes::REVERSE) {
            append_capability(
                out,
                self.terminal,
                if string_capability(self.terminal, Capability::rev).is_some() {
                    Capability::rev
                } else {
                    Capability::smso
                },
                &[],
            );
        }
        if new.underline != Underline::None && new.underline != old.underline {
            if new.underline == Underline::Single {
                append_capability(out, self.terminal, Capability::smul, &[]);
            } else {
                append_capability(
                    out,
                    self.terminal,
                    Capability::Smulx,
                    &[CapabilityParameter::Number(new.underline.sgr().into())],
                );
            }
        }
    }

    fn write_hyperlink(&self, out: &mut Vec<u8>, old: Option<&Hyperlink>, new: Option<&Hyperlink>) {
        if old == new {
            return;
        }
        let (id, uri) = new.map_or(("", ""), |link| (link.id.as_str(), link.uri.as_str()));
        append_capability(
            out,
            self.terminal,
            Capability::Hls,
            &[
                CapabilityParameter::String(id),
                CapabilityParameter::String(uri),
            ],
        );
    }
}

fn append_capability(
    out: &mut Vec<u8>,
    terminal: &dyn TerminalCapabilities,
    capability: Capability,
    parameters: &[CapabilityParameter<'_>],
) {
    if parameters.is_empty() {
        if let Some(value) = string_capability(terminal, capability) {
            out.extend_from_slice(value);
        }
    } else if let Some(value) = expand_capability(terminal, capability, parameters) {
        out.extend_from_slice(&value);
    }
}

fn terminal_colour(terminal: &dyn TerminalCapabilities, colour: Colour) -> Colour {
    let colours = number_capability(terminal, Capability::colors)
        .unwrap_or(8)
        .max(0) as u16;
    match colour {
        Colour::Rgb(red, green, blue)
            if string_capability(terminal, Capability::setrgbf).is_some()
                && string_capability(terminal, Capability::setrgbb).is_some() =>
        {
            Colour::Rgb(red, green, blue)
        }
        Colour::Rgb(red, green, blue) => {
            terminal_colour(terminal, Colour::Palette(rgb_to_256(red, green, blue)))
        }
        Colour::Palette(index) | Colour::Indexed(index) if u16::from(index) >= colours => {
            Colour::Palette(colour_256_to_16(index, colours >= 16))
        }
        value => value,
    }
}

fn rgb_to_256(red: u8, green: u8, blue: u8) -> u8 {
    const LEVELS: [i32; 6] = [0, 95, 135, 175, 215, 255];
    let cube = |value: u8| {
        if value < 48 {
            0
        } else if value < 114 {
            1
        } else {
            (value - 35) / 40
        }
    };
    let (qr, qg, qb) = (cube(red), cube(green), cube(blue));
    let (cr, cg, cb) = (
        LEVELS[usize::from(qr)],
        LEVELS[usize::from(qg)],
        LEVELS[usize::from(qb)],
    );
    if (cr, cg, cb) == (i32::from(red), i32::from(green), i32::from(blue)) {
        return 16 + 36 * qr + 6 * qg + qb;
    }
    let average = (i32::from(red) + i32::from(green) + i32::from(blue)) / 3;
    let grey_index = if average > 238 {
        23
    } else {
        ((average - 3) / 10).max(0) as u8
    };
    let grey = 8 + 10 * i32::from(grey_index);
    let distance = |r: i32, g: i32, b: i32| {
        (r - i32::from(red)).pow(2) + (g - i32::from(green)).pow(2) + (b - i32::from(blue)).pow(2)
    };
    if distance(grey, grey, grey) < distance(cr, cg, cb) {
        232 + grey_index
    } else {
        16 + 36 * qr + 6 * qg + qb
    }
}

fn colour_256_to_16(index: u8, bright: bool) -> u8 {
    #[rustfmt::skip]
    const MAP: [u8; 256] = [
         0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11,12,13,14,15,
         0, 4, 4, 4,12,12, 2, 6, 4, 4,12,12, 2, 2, 6, 4,
        12,12, 2, 2, 2, 6,12,12,10,10,10,10,14,12,10,10,
        10,10,10,14, 1, 5, 4, 4,12,12, 3, 8, 4, 4,12,12,
         2, 2, 6, 4,12,12, 2, 2, 2, 6,12,12,10,10,10,10,
        14,12,10,10,10,10,10,14, 1, 1, 5, 4,12,12, 1, 1,
         5, 4,12,12, 3, 3, 8, 4,12,12, 2, 2, 2, 6,12,12,
        10,10,10,10,14,12,10,10,10,10,10,14, 1, 1, 1, 5,
        12,12, 1, 1, 1, 5,12,12, 1, 1, 1, 5,12,12, 3, 3,
         3, 7,12,12,10,10,10,10,14,12,10,10,10,10,10,14,
         9, 9, 9, 9,13,12, 9, 9, 9, 9,13,12, 9, 9, 9, 9,
        13,12, 9, 9, 9, 9,13,12,11,11,11,11, 7,12,10,10,
        10,10,10,14, 9, 9, 9, 9, 9,13, 9, 9, 9, 9, 9,13,
         9, 9, 9, 9, 9,13, 9, 9, 9, 9, 9,13, 9, 9, 9, 9,
         9,13,11,11,11,11,11,15, 0, 0, 0, 0, 0, 0, 8, 8,
         8, 8, 8, 8, 7, 7, 7, 7, 7, 7,15,15,15,15,15,15,
    ];
    let mapped = MAP[usize::from(index)];
    if bright {
        mapped
    } else {
        mapped & 7
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::term::{ResolvedTerm, TerminalIdentity};

    fn terminal(capabilities: &[&str]) -> ResolvedTerm {
        ResolvedTerm::resolve(
            TerminalIdentity::new(
                "style-test",
                capabilities
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect(),
                0,
                None,
            ),
            [],
        )
    }

    #[test]
    fn decoder_handles_colon_and_semicolon_colours() {
        let mut decoder = SgrDecoder::default();
        decoder.apply(b"1;4:3;38;2;1;2;3;48:5:17;58:2::4:5:6");
        assert!(decoder.style.attributes.has(Attributes::BOLD));
        assert_eq!(decoder.style.underline, Underline::Curly);
        assert_eq!(decoder.style.fg, Colour::Rgb(1, 2, 3));
        assert_eq!(
            decoder.style.bg,
            Colour::Indexed(17),
            "an index that arrived through the 256-colour form stays one"
        );
        assert_eq!(decoder.style.underline_colour, Colour::Rgb(4, 5, 6));
    }

    #[test]
    fn capture_writer_uses_tmux_attribute_and_colour_order() {
        let mut writer = CaptureStyleWriter::default();
        let mut out = Vec::new();
        let mut presentation = CellPresentation::default();
        presentation.style.attributes.set(Attributes::BOLD, true);
        presentation.style.fg = Colour::Palette(1);
        presentation.style.bg = Colour::Palette(2);
        writer.transition(&mut out, &presentation);
        assert_eq!(out, b"\x1b[1m\x1b[31m\x1b[42m");
        presentation.style.attributes.set(Attributes::BOLD, false);
        writer.transition(&mut out, &presentation);
        assert!(out.ends_with(b"\x1b[0m\x1b[31m\x1b[42m"));
    }

    #[test]
    fn terminal_writer_uses_resolved_capabilities_and_gates_removed_features() {
        let rich = terminal(&[
            "sgr0=\x1b[RESET",
            "colors=256",
            "AX=1",
            "setaf=\x1b[FG%p1%d",
            "setab=\x1b[BG%p1%d",
            "bold=\x1b[BOLD",
            "smxx=\x1b[STRIKE",
            "Smol=\x1b[OVERLINE",
            "smul=\x1b[UNDERLINE",
            "Hls=\x1b]8;%p1%s;%p2%s\x1b\\",
        ]);
        let mut presentation = CellPresentation::default();
        presentation.style.fg = Colour::Palette(12);
        presentation.style.attributes.set(Attributes::BOLD, true);
        presentation
            .style
            .attributes
            .set(Attributes::STRIKETHROUGH, true);
        presentation
            .style
            .attributes
            .set(Attributes::OVERLINE, true);
        presentation.style.underline = Underline::Single;
        presentation.hyperlink = Some(Hyperlink {
            id: "id".into(),
            uri: "https://example.test".into(),
        });

        let mut out = Vec::new();
        TerminalStyleWriter::new(&rich).transition(&mut out, &presentation);
        assert!(out.windows(4).any(|window| window == b"FG12"));
        assert!(out.windows(4).any(|window| window == b"BOLD"));
        assert!(out.windows(6).any(|window| window == b"STRIKE"));
        assert!(out.windows(8).any(|window| window == b"OVERLINE"));
        assert!(out.windows(9).any(|window| window == b"UNDERLINE"));
        assert!(out
            .windows(20)
            .any(|window| window == b"https://example.test"));

        let reduced = terminal(&["sgr0=\x1b[RESET", "colors=8", "setaf=\x1b[FG%p1%d"]);
        let mut reduced_out = Vec::new();
        TerminalStyleWriter::new(&reduced).transition(&mut reduced_out, &presentation);
        assert!(!reduced_out.windows(4).any(|window| window == b"BOLD"));
        assert!(!reduced_out.windows(6).any(|window| window == b"STRIKE"));
        assert!(!reduced_out
            .windows(20)
            .any(|window| window == b"https://example.test"));
    }
}
