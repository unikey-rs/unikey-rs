//! Text composition rules shared by the GUI, CLI, and Wayland input backend.
//!
//! The composer intentionally works on a raw keystroke stream instead of doing
//! one-shot regex replacement. That makes progressive rendering, backspace, and
//! "restore key" behaviour deterministic and keeps all front-ends in sync.

use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Layout {
    English,
    Telex,
    Vni,
    Viqr,
}

impl Layout {
    pub const ALL: [Self; 4] = [Self::English, Self::Telex, Self::Vni, Self::Viqr];

    pub fn id(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Telex => "telex",
            Self::Vni => "vni",
            Self::Viqr => "viqr",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::English => "English (US)",
            Self::Telex => "Vietnamese Telex",
            Self::Vni => "Vietnamese VNI",
            Self::Viqr => "Vietnamese VIQR",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::English => "Pass-through QWERTY input",
            Self::Telex => "aa=â, aw=ă, ow=ơ, uw=ư, dd=đ, as=á",
            Self::Vni => "a6=â, a8=ă, o7=ơ, u7=ư, d9=đ, a1=á",
            Self::Viqr => "a^=â, a(=ă, o+=ơ, u+=ư, dd=đ, a'=á",
        }
    }
}

#[derive(Debug)]
pub struct ParseLayoutError(String);

impl fmt::Display for ParseLayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown layout `{}`; expected en, telex, vni, or viqr", self.0)
    }
}

impl std::error::Error for ParseLayoutError {}

impl FromStr for Layout {
    type Err = ParseLayoutError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "en" | "eng" | "english" | "us" | "qwerty" => Ok(Self::English),
            "telex" | "vn" | "vietnamese-telex" => Ok(Self::Telex),
            "vni" | "vietnamese-vni" => Ok(Self::Vni),
            "viqr" | "vietnamese-viqr" => Ok(Self::Viqr),
            other => Err(ParseLayoutError(other.to_owned())),
        }
    }
}


fn telex_w_shape(unit: &Unit) -> Shape {
    match unit.base {
        'a' => Shape::Breve,
        'o' | 'u' => Shape::Horn,
        _ => Shape::Horn,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Shape {
    None,
    Circumflex,
    Horn,
    Breve,
    Stroke,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tone {
    None,
    Acute,
    Grave,
    Hook,
    Tilde,
    Dot,
}

#[derive(Clone, Debug)]
struct Unit {
    base: char,
    upper: bool,
    shape: Shape,
    shape_key: Option<char>,
    tone: Tone,
    tone_key: Option<char>,
}

impl Unit {
    fn literal(ch: char) -> Self {
        Self {
            base: ch.to_ascii_lowercase(),
            upper: ch.is_uppercase(),
            shape: Shape::None,
            shape_key: None,
            tone: Tone::None,
            tone_key: None,
        }
    }

    fn accent_vowel(ch: char) -> Option<Self> {
        let upper = ch.is_uppercase();
        let (base, shape, tone) = match ch.to_ascii_lowercase() {
            'a' => ('a', Shape::None, Tone::None),
            'á' | 'à' | 'ả' | 'ã' | 'ạ' => ('a', Shape::None, tone_for_accented(ch)?),
            'â' => ('a', Shape::Circumflex, Tone::None),
            'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' => ('a', Shape::Circumflex, tone_for_accented(ch)?),
            'ă' => ('a', Shape::Breve, Tone::None),
            'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => ('a', Shape::Breve, tone_for_accented(ch)?),
            'e' => ('e', Shape::None, Tone::None),
            'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' => ('e', Shape::None, tone_for_accented(ch)?),
            'ê' => ('e', Shape::Circumflex, Tone::None),
            'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => ('e', Shape::Circumflex, tone_for_accented(ch)?),
            'i' => ('i', Shape::None, Tone::None),
            'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => ('i', Shape::None, tone_for_accented(ch)?),
            'o' => ('o', Shape::None, Tone::None),
            'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' => ('o', Shape::None, tone_for_accented(ch)?),
            'ô' => ('o', Shape::Circumflex, Tone::None),
            'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' => ('o', Shape::Circumflex, tone_for_accented(ch)?),
            'ơ' => ('o', Shape::Horn, Tone::None),
            'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => ('o', Shape::Horn, tone_for_accented(ch)?),
            'u' => ('u', Shape::None, Tone::None),
            'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' => ('u', Shape::None, tone_for_accented(ch)?),
            'ư' => ('u', Shape::Horn, Tone::None),
            'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => ('u', Shape::Horn, tone_for_accented(ch)?),
            'y' => ('y', Shape::None, Tone::None),
            'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => ('y', Shape::None, tone_for_accented(ch)?),
            'đ' => ('d', Shape::Stroke, Tone::None),
            _ => return None,
        };
        Some(Self {
            base,
            upper,
            shape,
            shape_key: if shape == Shape::Stroke { Some('d') } else { None },
            tone,
            tone_key: None,
        })
    }

    fn render(&self) -> char {
        let rendered = apply_tone(apply_shape(self.base, self.shape), self.tone);
        if self.upper {
            to_upper_vietnamese(rendered)
        } else {
            rendered
        }
    }

    fn supports_tone(&self) -> bool {
        matches!(self.base, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
    }

    fn supports_shape(&self, shape: Shape) -> bool {
        matches!(
            (self.base, shape),
            ('a', Shape::Circumflex | Shape::Breve)
                | ('e', Shape::Circumflex)
                | ('o', Shape::Circumflex | Shape::Horn)
                | ('u', Shape::Horn)
                | ('d', Shape::Stroke)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComposerOptions {
    /// When a modifier is typed twice, restore the previous letter and emit the
    /// second key literally (`ass` -> `as`). This is standard Telex behaviour.
    pub restore_keys: bool,
}

impl Default for ComposerOptions {
    fn default() -> Self {
        Self { restore_keys: true }
    }
}

#[derive(Clone, Debug)]
pub struct Composer {
    layout: Layout,
    typed: String,
    options: ComposerOptions,
}

impl Composer {
    pub fn new(layout: Layout) -> Self {
        Self::with_options(layout, ComposerOptions::default())
    }

    pub fn with_options(layout: Layout, options: ComposerOptions) -> Self {
        Self {
            layout,
            typed: String::new(),
            options,
        }
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }

    pub fn set_layout(&mut self, layout: Layout) {
        self.layout = layout;
    }

    pub fn typed(&self) -> &str {
        &self.typed
    }

    pub fn output(&self) -> String {
        compose_with_options(self.layout, &self.typed, self.options)
    }

    pub fn push_str(&mut self, text: &str) {
        self.typed.push_str(text);
    }

    pub fn push_char(&mut self, ch: char) {
        self.typed.push(ch);
    }

    pub fn backspace(&mut self) -> bool {
        self.typed.pop().is_some()
    }

    pub fn clear(&mut self) {
        self.typed.clear();
    }
}

pub fn compose_text(layout: Layout, typed: &str) -> String {
    compose_with_options(layout, typed, ComposerOptions::default())
}

pub fn compose_with_options(layout: Layout, typed: &str, options: ComposerOptions) -> String {
    if layout == Layout::English {
        return typed.to_owned();
    }

    let mut output = String::with_capacity(typed.len());
    let mut word: Vec<Unit> = Vec::new();

    for ch in typed.chars() {
        if is_word_boundary(ch) {
            flush(&mut output, &mut word);
            output.push(ch);
            continue;
        }

        if !push_key(layout, &mut word, ch, options.restore_keys) {
            word.push(Unit::literal(ch));
        }
    }

    flush(&mut output, &mut word);
    output
}

fn flush(output: &mut String, word: &mut Vec<Unit>) {
    output.extend(word.drain(..).map(|unit| unit.render()));
}

fn is_word_boundary(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, ',' | '.' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '/')
}

fn push_key(layout: Layout, word: &mut Vec<Unit>, key: char, restore_keys: bool) -> bool {
    let lower = key.to_ascii_lowercase();
    let tone = tone_key(layout, key);
    if layout == Layout::Telex && key.eq_ignore_ascii_case(&'w') && apply_telex_w(word, key, restore_keys) {
        return true;
    }

    let shape = shape_key(layout, key);

    if let Some(tone) = tone {
        return apply_or_restore_tone(word, key, tone, restore_keys);
    }

    if let Some(shape) = shape {
        return apply_or_restore_shape(word, key, shape, restore_keys);
    }

    if matches!(layout, Layout::Telex | Layout::Viqr | Layout::Vni) {
        if let Some(last) = word.last_mut() {
            let doubled = match layout {
                Layout::Telex | Layout::Viqr => matches!(lower, 'a' | 'e' | 'o' | 'd'),
                Layout::Vni => lower == 'd',
                Layout::English => false,
            };
            if doubled && last.base == lower && last.tone_key.is_none() {
                if last.shape == Shape::None && last.supports_shape(Shape::Circumflex) {
                    last.shape = Shape::Circumflex;
                    last.shape_key = Some(lower);
                    return true;
                }
                if last.shape == Shape::None && lower == 'd' {
                    last.shape = Shape::Stroke;
                    last.shape_key = Some('d');
                    return true;
                }
                if restore_keys && last.shape_key == Some(lower) {
                    last.shape = Shape::None;
                    last.shape_key = None;
                    return false;
                }
                return false;
            }
        }
    }

    if Unit::accent_vowel(key).is_some() {
        word.push(Unit::accent_vowel(key).expect("accent vowel"));
    } else {
        word.push(Unit::literal(key));
    }
    true
}

fn apply_or_restore_tone(word: &mut [Unit], key: char, tone: Tone, restore_keys: bool) -> bool {
    let Some(target) = find_tone_target(word) else {
        return false;
    };
    let last = &mut word[target];
    if !last.supports_tone() {
        return false;
    }
    if last.tone == Tone::None {
        last.tone = tone;
        last.tone_key = Some(key);
        true
    } else if restore_keys && last.tone_key == Some(key) {
        last.tone = Tone::None;
        last.tone_key = None;
        false
    } else {
        last.tone = tone;
        last.tone_key = Some(key);
        true
    }
}

fn apply_or_restore_shape(word: &mut [Unit], key: char, shape: Shape, restore_keys: bool) -> bool {
    let Some(target) = find_shape_target(word, shape) else {
        return false;
    };
    let last = &mut word[target];
    if !last.supports_shape(shape) {
        return false;
    }
    if last.shape == Shape::None {
        last.shape = shape;
        last.shape_key = Some(key);
        true
    } else if restore_keys && last.shape_key == Some(key) {
        last.shape = Shape::None;
        last.shape_key = None;
        false
    } else {
        false
    }
}

fn apply_telex_w(word: &mut [Unit], key: char, restore_keys: bool) -> bool {
    if word.len() >= 2 {
        let n = word.len();
        let pair = (word[n - 2].base, word[n - 1].base);
        if pair == ('u', 'o') {
            if word[n - 2].shape == Shape::None && word[n - 1].shape == Shape::None {
                word[n - 2].shape = Shape::Horn;
                word[n - 2].shape_key = Some(key);
                word[n - 1].shape = Shape::Horn;
                word[n - 1].shape_key = Some(key);
                return true;
            }
            if restore_keys
                && word[n - 2].shape_key == Some(key)
                && word[n - 1].shape_key == Some(key)
            {
                word[n - 2].shape = Shape::None;
                word[n - 2].shape_key = None;
                word[n - 1].shape = Shape::None;
                word[n - 1].shape_key = None;
                return false;
            }
            return true;
        }
    }

    let Some(target) = word.iter().rposition(|unit| matches!(unit.base, 'a' | 'o' | 'u')) else {
        return false;
    };
    let unit = &mut word[target];
    let shape = telex_w_shape(unit);
    if !unit.supports_shape(shape) {
        return false;
    }
    if unit.shape == Shape::None {
        unit.shape = shape;
        unit.shape_key = Some(key);
        true
    } else if restore_keys && unit.shape_key == Some(key) {
        unit.shape = Shape::None;
        unit.shape_key = None;
        false
    } else {
        false
    }
}

fn find_shape_target(word: &[Unit], shape: Shape) -> Option<usize> {
    word.iter().rposition(|unit| unit.supports_shape(shape))
}

fn find_tone_target(word: &[Unit]) -> Option<usize> {
    let vowels: Vec<usize> = word
        .iter()
        .enumerate()
        .filter_map(|(index, unit)| unit.supports_tone().then_some(index))
        .collect();
    match vowels.len() {
        0 => None,
        1 => Some(vowels[0]),
        _ => {
            if let Some(shaped) = vowels.iter().copied().rfind(|index| word[*index].shape != Shape::None) {
                return Some(shaped);
            }
            let last = *vowels.last().expect("vowel");
            let previous = vowels[vowels.len() - 2];
            let pair = (word[previous].base, word[last].base);
            let first_is_anchor = matches!(
                pair,
                ('a', 'o') | ('a', 'i') | ('a', 'u') | ('a', 'y') | ('e', 'o') | ('e', 'u') | ('e', 'y') | ('i', 'a') | ('i', 'u') | ('o', 'i') | ('o', 'o') | ('u', 'a') | ('u', 'i') | ('u', 'o') | ('y', 'e')
            );
            Some(if first_is_anchor { previous } else { last })
        }
    }
}

fn tone_key(layout: Layout, key: char) -> Option<Tone> {
    let lower = key.to_ascii_lowercase();
    let tone = match layout {
        Layout::Telex => match lower {
            's' => Tone::Acute,
            'f' => Tone::Grave,
            'r' => Tone::Hook,
            'x' => Tone::Tilde,
            'j' => Tone::Dot,
            _ => return None,
        },
        Layout::Vni => match key {
            '1' => Tone::Acute,
            '2' => Tone::Grave,
            '3' => Tone::Hook,
            '4' => Tone::Tilde,
            '5' => Tone::Dot,
            _ => return None,
        },
        Layout::Viqr => match key {
            '\'' => Tone::Acute,
            '`' => Tone::Grave,
            '?' => Tone::Hook,
            '~' => Tone::Tilde,
            '.' => Tone::Dot,
            _ => return None,
        },
        Layout::English => return None,
    };
    Some(tone)
}

fn shape_key(layout: Layout, key: char) -> Option<Shape> {
    match layout {
        Layout::Telex => None,

        Layout::Vni => match key {
            '6' => Some(Shape::Circumflex),
            '7' => Some(Shape::Horn),
            '8' => Some(Shape::Breve),
            '9' => Some(Shape::Stroke),
            _ => None,
        },
        Layout::Viqr => match key {
            '^' => Some(Shape::Circumflex),
            '+' => Some(Shape::Horn),
            '(' => Some(Shape::Breve),
            _ => None,
        },
        Layout::English => None,
    }
}

fn tone_for_accented(ch: char) -> Option<Tone> {
    let tone = match ch.to_ascii_lowercase() {
        'á' | 'ấ' | 'ắ' | 'é' | 'ế' | 'í' | 'ó' | 'ố' | 'ớ' | 'ú' | 'ứ' | 'ý' => Tone::Acute,
        'à' | 'ầ' | 'ằ' | 'è' | 'ề' | 'ì' | 'ò' | 'ồ' | 'ờ' | 'ù' | 'ừ' | 'ỳ' => Tone::Grave,
        'ả' | 'ẩ' | 'ẳ' | 'ẻ' | 'ể' | 'ỉ' | 'ỏ' | 'ổ' | 'ở' | 'ủ' | 'ử' | 'ỷ' => Tone::Hook,
        'ã' | 'ẫ' | 'ẵ' | 'ẽ' | 'ễ' | 'ĩ' | 'õ' | 'ỗ' | 'ỡ' | 'ũ' | 'ữ' | 'ỹ' => Tone::Tilde,
        'ạ' | 'ậ' | 'ặ' | 'ẹ' | 'ệ' | 'ị' | 'ọ' | 'ộ' | 'ợ' | 'ụ' | 'ự' | 'ỵ' => Tone::Dot,
        _ => return None,
    };
    Some(tone)
}

fn apply_shape(ch: char, shape: Shape) -> char {
    match (ch, shape) {
        ('a', Shape::Circumflex) => 'â',
        ('a', Shape::Breve) => 'ă',
        ('e', Shape::Circumflex) => 'ê',
        ('o', Shape::Circumflex) => 'ô',
        ('o', Shape::Horn) => 'ơ',
        ('u', Shape::Horn) => 'ư',
        ('d', Shape::Stroke) => 'đ',
        _ => ch,
    }
}

fn apply_tone(ch: char, tone: Tone) -> char {
    let index = match tone {
        Tone::None => return ch,
        Tone::Acute => 0,
        Tone::Grave => 1,
        Tone::Hook => 2,
        Tone::Tilde => 3,
        Tone::Dot => 4,
    };

    let table: &[char] = match ch {
        'a' => &['á', 'à', 'ả', 'ã', 'ạ'],
        'â' => &['ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'],
        'ă' => &['ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'],
        'e' => &['é', 'è', 'ẻ', 'ẽ', 'ẹ'],
        'ê' => &['ế', 'ề', 'ể', 'ễ', 'ệ'],
        'i' => &['í', 'ì', 'ỉ', 'ĩ', 'ị'],
        'o' => &['ó', 'ò', 'ỏ', 'õ', 'ọ'],
        'ô' => &['ố', 'ồ', 'ổ', 'ỗ', 'ộ'],
        'ơ' => &['ớ', 'ờ', 'ở', 'ỡ', 'ợ'],
        'u' => &['ú', 'ù', 'ủ', 'ũ', 'ụ'],
        'ư' => &['ứ', 'ừ', 'ử', 'ữ', 'ự'],
        'y' => &['ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'],
        _ => return ch,
    };
    table[index]
}

fn to_upper_vietnamese(ch: char) -> char {
    match ch {
        'â' => 'Â',
        'ă' => 'Ă',
        'ê' => 'Ê',
        'ô' => 'Ô',
        'ơ' => 'Ơ',
        'ư' => 'Ư',
        'đ' => 'Đ',
        'á' => 'Á',
        'à' => 'À',
        'ả' => 'Ả',
        'ã' => 'Ã',
        'ạ' => 'Ạ',
        'ấ' => 'Ấ',
        'ầ' => 'Ầ',
        'ẩ' => 'Ẩ',
        'ẫ' => 'Ẫ',
        'ậ' => 'Ậ',
        'ắ' => 'Ắ',
        'ằ' => 'Ằ',
        'ẳ' => 'Ẳ',
        'ẵ' => 'Ẵ',
        'ặ' => 'Ặ',
        'é' => 'É',
        'è' => 'È',
        'ẻ' => 'Ẻ',
        'ẽ' => 'Ẽ',
        'ẹ' => 'Ẹ',
        'ế' => 'Ế',
        'ề' => 'Ề',
        'ể' => 'Ể',
        'ễ' => 'Ễ',
        'ệ' => 'Ệ',
        'í' => 'Í',
        'ì' => 'Ì',
        'ỉ' => 'Ỉ',
        'ĩ' => 'Ĩ',
        'ị' => 'Ị',
        'ó' => 'Ó',
        'ò' => 'Ò',
        'ỏ' => 'Ỏ',
        'õ' => 'Õ',
        'ọ' => 'Ọ',
        'ố' => 'Ố',
        'ồ' => 'Ồ',
        'ổ' => 'Ổ',
        'ỗ' => 'Ỗ',
        'ộ' => 'Ộ',
        'ớ' => 'Ớ',
        'ờ' => 'Ờ',
        'ở' => 'Ở',
        'ỡ' => 'Ỡ',
        'ợ' => 'Ợ',
        'ú' => 'Ú',
        'ù' => 'Ù',
        'ủ' => 'Ủ',
        'ũ' => 'Ũ',
        'ụ' => 'Ụ',
        'ứ' => 'Ứ',
        'ừ' => 'Ừ',
        'ử' => 'Ử',
        'ữ' => 'Ữ',
        'ự' => 'Ự',
        'ý' => 'Ý',
        'ỳ' => 'Ỳ',
        'ỷ' => 'Ỷ',
        'ỹ' => 'Ỹ',
        'ỵ' => 'Ỵ',
        _ => ch.to_ascii_uppercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn telex(input: &str) -> String {
        compose_text(Layout::Telex, input)
    }

    #[test]
    fn parses_layout_aliases() {
        assert_eq!("telex".parse::<Layout>().unwrap(), Layout::Telex);
        assert_eq!("VN".parse::<Layout>().unwrap(), Layout::Telex);
        assert_eq!("us".parse::<Layout>().unwrap(), Layout::English);
        assert!("unknown".parse::<Layout>().is_err());
    }

    #[test]
    fn telex_shapes_and_tones() {
        assert_eq!(telex("aa"), "â");
        assert_eq!(telex("aw"), "ă");
        assert_eq!(telex("ow"), "ơ");
        assert_eq!(telex("uw"), "ư");
        assert_eq!(telex("dd"), "đ");
        assert_eq!(telex("as"), "á");
        assert_eq!(telex("awf"), "ằ");
        assert_eq!(telex("owr"), "ở");
    }

    #[test]
    fn vni_shapes_and_tones() {
        assert_eq!(compose_text(Layout::Vni, "a6"), "â");
        assert_eq!(compose_text(Layout::Vni, "a8"), "ă");
        assert_eq!(compose_text(Layout::Vni, "o7"), "ơ");
        assert_eq!(compose_text(Layout::Vni, "u7"), "ư");
        assert_eq!(compose_text(Layout::Vni, "d9"), "đ");
        assert_eq!(compose_text(Layout::Vni, "a1"), "á");
        assert_eq!(compose_text(Layout::Vni, "a61"), "ấ");
    }

    #[test]
    fn viqr_shapes_and_tones() {
        assert_eq!(compose_text(Layout::Viqr, "a^"), "â");
        assert_eq!(compose_text(Layout::Viqr, "a("), "ă");
        assert_eq!(compose_text(Layout::Viqr, "o+"), "ơ");
        assert_eq!(compose_text(Layout::Viqr, "u+"), "ư");
        assert_eq!(compose_text(Layout::Viqr, "dd"), "đ");
        assert_eq!(compose_text(Layout::Viqr, "a'"), "á");
    }

    #[test]
    fn progressive_words_and_boundaries() {
        let mut composer = Composer::new(Layout::Telex);
        composer.push_str("Xin chaof the gioi");
        assert_eq!(composer.output(), "Xin chào the gioi");
        composer.push_str(", ong ay nois gi?");
        assert_eq!(composer.output(), "Xin chào the gioi, ong ay nói gi?");
    }

    #[test]
    fn telex_composes_common_rhymes() {
        assert_eq!(telex("uowfi"), "ười");
        assert_eq!(telex("chaof"), "chào");
    }

    #[test]
    fn restore_key_emits_literal_after_toggling() {
        assert_eq!(telex("ass"), "as");
        assert_eq!(telex("aaas"), "aá");
        assert_eq!(telex("ddj"), "đj");
    }

    #[test]
    fn preserves_uppercase() {
        assert_eq!(telex("Aa Aw Ow Uw Dd As"), "Â Ă Ơ Ư Đ Á");
        assert_eq!(compose_text(Layout::Vni, "A6 A8 O7 U7 D9 A1"), "Â Ă Ơ Ư Đ Á");
    }

    #[test]
    fn english_is_passthrough() {
        assert_eq!(compose_text(Layout::English, "aa as d9"), "aa as d9");
    }
}
