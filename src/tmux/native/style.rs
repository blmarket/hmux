//! Shared visual cell style, parsing, decoding, and serialization.
//!
//! Layout and cursor operations deliberately stay with their renderers. This
//! module owns only cell presentation and transitions between presentations.

use super::term::{
    expand_capability, number_capability, string_capability, terminal_utf8, CapabilityParameter,
    TerminalCapabilities,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Colour {
    #[default]
    Default,
    Palette(u8),
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
                patch.underline_colour = Some(parse_colour(value).ok_or(())?);
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
                    .map(|value| Colour::Palette(*value as u8)),
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
        5 => Some((Colour::Palette(fields.get(1)?.parse().ok()?), 2)),
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
}

impl CaptureStyleWriter {
    pub(crate) fn transition(&mut self, out: &mut Vec<u8>, next: &CellPresentation) {
        write_capture_style(out, &self.current.style, &next.style);
        write_capture_hyperlink(
            out,
            self.current.hyperlink.as_ref(),
            next.hyperlink.as_ref(),
        );
        if self.current.acs != next.acs {
            out.push(if next.acs { 0x0e } else { 0x0f });
        }
        self.current = next.clone();
    }

    pub(crate) fn finish_row(&mut self, out: &mut Vec<u8>) {
        self.transition(out, &CellPresentation::default());
    }
}

fn write_capture_style(out: &mut Vec<u8>, old: &CellStyle, new: &CellStyle) {
    if old == new {
        return;
    }
    let reset = old.attributes.removed_from(new.attributes)
        || (old.underline != Underline::None && new.underline != old.underline)
        || (old.underline_colour != Colour::Default && new.underline_colour == Colour::Default);
    let baseline = if reset {
        push_sgr(out, &["0".into()]);
        CellStyle::default()
    } else {
        *old
    };
    let mut attrs = Vec::new();
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
        Colour::Palette(index) => vec![prefix.to_string(), "5".into(), index.to_string()],
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

pub(crate) fn write_capture_hyperlink(
    out: &mut Vec<u8>,
    old: Option<&Hyperlink>,
    new: Option<&Hyperlink>,
) {
    if old == new {
        return;
    }
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
        if let Some(value) = string_capability(self.terminal, "sgr0") {
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
            || (!self.terminal.flag("AX")
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
                if next.acs { "smacs" } else { "rmacs" },
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
                && string_capability(self.terminal, "Smulx").is_none()
            {
                value.style.underline = Underline::Single;
            }
            if value.style.underline == Underline::Single
                && string_capability(self.terminal, "smul").is_none()
            {
                value.style.underline = Underline::None;
            }
        }
        if value.style.underline_colour != Colour::Default
            && string_capability(self.terminal, "Setulc").is_none()
            && string_capability(self.terminal, "Setulc1").is_none()
        {
            value.style.underline_colour = Colour::Default;
        }
        for (flag, capability) in [
            (Attributes::BOLD, "bold"),
            (Attributes::DIM, "dim"),
            (Attributes::ITALICS, "sitm"),
            (Attributes::BLINK, "blink"),
            (Attributes::HIDDEN, "invis"),
            (Attributes::STRIKETHROUGH, "smxx"),
            (Attributes::OVERLINE, "Smol"),
        ] {
            if string_capability(self.terminal, capability).is_none() {
                value.style.attributes.set(flag, false);
            }
        }
        if string_capability(self.terminal, "Hls").is_none() {
            value.hyperlink = None;
        }
        if string_capability(self.terminal, "rev").is_none()
            && string_capability(self.terminal, "smso").is_none()
        {
            value.style.attributes.set(Attributes::REVERSE, false);
        }
        if terminal_utf8(self.terminal)
            || string_capability(self.terminal, "smacs").is_none()
            || string_capability(self.terminal, "rmacs").is_none()
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
                Colour::Default => append_capability(out, self.terminal, "ol", &[]),
                Colour::Palette(index) => append_capability(
                    out,
                    self.terminal,
                    "Setulc1",
                    &[CapabilityParameter::Number(index.into())],
                ),
                Colour::Rgb(red, green, blue) => append_capability(
                    out,
                    self.terminal,
                    "Setulc",
                    &[CapabilityParameter::Number(
                        (i32::from(red) << 16) | (i32::from(green) << 8) | i32::from(blue),
                    )],
                ),
            }
        }
    }

    fn write_colour(&self, out: &mut Vec<u8>, colour: Colour, background: bool) {
        match colour {
            Colour::Default if self.terminal.flag("AX") => {
                out.extend_from_slice(if background { b"\x1b[49m" } else { b"\x1b[39m" });
            }
            Colour::Default => {}
            Colour::Palette(index) => append_capability(
                out,
                self.terminal,
                if background { "setab" } else { "setaf" },
                &[CapabilityParameter::Number(index.into())],
            ),
            Colour::Rgb(red, green, blue) => append_capability(
                out,
                self.terminal,
                if background { "setrgbb" } else { "setrgbf" },
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
            (Attributes::BOLD, "bold"),
            (Attributes::DIM, "dim"),
            (Attributes::ITALICS, "sitm"),
            (Attributes::BLINK, "blink"),
            (Attributes::HIDDEN, "invis"),
            (Attributes::STRIKETHROUGH, "smxx"),
            (Attributes::OVERLINE, "Smol"),
        ] {
            if new.attributes.has(flag) && !old.attributes.has(flag) {
                append_capability(out, self.terminal, capability, &[]);
            }
        }
        if new.attributes.has(Attributes::REVERSE) && !old.attributes.has(Attributes::REVERSE) {
            append_capability(
                out,
                self.terminal,
                if string_capability(self.terminal, "rev").is_some() {
                    "rev"
                } else {
                    "smso"
                },
                &[],
            );
        }
        if new.underline != Underline::None && new.underline != old.underline {
            if new.underline == Underline::Single {
                append_capability(out, self.terminal, "smul", &[]);
            } else {
                append_capability(
                    out,
                    self.terminal,
                    "Smulx",
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
            "Hls",
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
    name: &str,
    parameters: &[CapabilityParameter<'_>],
) {
    if parameters.is_empty() {
        if let Some(value) = string_capability(terminal, name) {
            out.extend_from_slice(value);
        }
    } else if let Some(value) = expand_capability(terminal, name, parameters) {
        out.extend_from_slice(&value);
    }
}

fn terminal_colour(terminal: &dyn TerminalCapabilities, colour: Colour) -> Colour {
    let colours = number_capability(terminal, "colors").unwrap_or(8).max(0) as u16;
    match colour {
        Colour::Rgb(red, green, blue)
            if string_capability(terminal, "setrgbf").is_some()
                && string_capability(terminal, "setrgbb").is_some() =>
        {
            Colour::Rgb(red, green, blue)
        }
        Colour::Rgb(red, green, blue) => {
            terminal_colour(terminal, Colour::Palette(rgb_to_256(red, green, blue)))
        }
        Colour::Palette(index) if u16::from(index) >= colours => {
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
    use crate::tmux::native::term::{ResolvedTerm, TerminalIdentity};

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
        assert_eq!(decoder.style.bg, Colour::Palette(17));
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
