//! Small XKB keymap reader for Wayland `wl_keyboard.keymap` events.
//!
//! It intentionally handles the subset needed by UniKey RS: keycodes and the
//! first two keysyms per key. This avoids bundling libxkbcommon in the AppImage
//! while still respecting the compositor-provided physical keyboard map.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::OwnedFd;

#[derive(Clone, Debug, Default)]
pub struct Keymap {
    symbols: HashMap<u32, Vec<Option<char>>>,
}

impl Keymap {
    pub fn from_fd(fd: OwnedFd, size: usize) -> io::Result<Self> {
        let mut file = File::from(fd);
        let mut text = String::new();
        file.by_ref().take(size as u64).read_to_string(&mut text)?;
        Self::parse(&text)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let mut names = HashMap::new();
        for line in text.lines() {
            let line = line.trim();
            if !line.starts_with('<') {
                continue;
            }
            let Some(name_end) = line.find('>') else { continue };
            let Some(equal) = line[name_end..].find('=') else { continue };
            let Some(semicolon) = line[name_end + equal..].find(';') else {
                continue;
            };
            let name = line[1..name_end].trim();
            let value = line[name_end + equal + 1..name_end + equal + semicolon].trim();
            let Ok(code) = value.parse::<u32>() else {
                continue;
            };
            names.insert(name.to_owned(), code);
        }

        let mut raw_symbols = Vec::new();
        let mut cursor = 0;
        while let Some(relative) = text[cursor..].find("key <") {
            let start = cursor + relative + "key <".len();
            let Some(name_end) = text[start..].find('>') else {
                break;
            };
            let name = text[start..start + name_end].trim().to_owned();
            let after_name = start + name_end + 1;
            let Some(open) = text[after_name..].find('{') else {
                cursor = after_name;
                continue;
            };
            let body_start = after_name + open + 1;
            let Some(close) = text[body_start..].find('}') else {
                break;
            };
            let body = &text[body_start..body_start + close];
            if let Some(symbols) = parse_symbols(body) {
                raw_symbols.push((name, symbols));
            }
            cursor = body_start + close + 1;
        }

        let mut symbols = HashMap::new();
        for (name, values) in raw_symbols {
            if let Some(code) = names.get(&name) {
                symbols.insert(*code, values);
            }
        }
        Ok(Self { symbols })
    }

    pub fn key_to_char(&self, evdev_key: u32, shift: bool) -> Option<char> {
        let code = evdev_key.checked_add(8)?;
        let symbols = self.symbols.get(&code)?;
        let index = usize::from(shift && symbols.len() > 1);
        symbols.get(index).copied().flatten().or_else(|| symbols.first().copied().flatten())
    }
}

fn parse_symbols(body: &str) -> Option<Vec<Option<char>>> {
    let start = body.find('[')?;
    let end = body[start..].find(']')? + start;
    let values = body[start + 1..end]
        .split(',')
        .map(|value| symbol_to_char(value.trim()))
        .collect::<Vec<_>>();
    (!values.is_empty()).then_some(values)
}

fn symbol_to_char(symbol: &str) -> Option<char> {
    if symbol.is_empty() || symbol == "NoSymbol" {
        return None;
    }
    if symbol.len() >= 2 && symbol.starts_with('"') && symbol.ends_with('"') {
        return symbol[1..symbol.len() - 1].chars().next();
    }
    if symbol.chars().count() == 1 {
        return symbol.chars().next();
    }
    if let Some(hex) = symbol.strip_prefix('U') {
        return u32::from_str_radix(hex, 16).ok().and_then(char::from_u32);
    }
    if let Some(hex) = symbol.strip_prefix("0x") {
        return keysym_to_char(u32::from_str_radix(hex, 16).ok()?);
    }

    let code = match symbol {
        "space" => 0x20,
        "exclam" => 0x21,
        "quotedbl" => 0x22,
        "numbersign" => 0x23,
        "dollar" => 0x24,
        "percent" => 0x25,
        "ampersand" => 0x26,
        "apostrophe" => 0x27,
        "parenleft" => 0x28,
        "parenright" => 0x29,
        "asterisk" => 0x2a,
        "plus" => 0x2b,
        "comma" => 0x2c,
        "minus" => 0x2d,
        "period" => 0x2e,
        "slash" => 0x2f,
        "colon" => 0x3a,
        "semicolon" => 0x3b,
        "less" => 0x3c,
        "equal" => 0x3d,
        "greater" => 0x3e,
        "question" => 0x3f,
        "at" => 0x40,
        "bracketleft" => 0x5b,
        "backslash" => 0x5c,
        "bracketright" => 0x5d,
        "asciicircum" => 0x5e,
        "underscore" => 0x5f,
        "grave" => 0x60,
        "braceleft" => 0x7b,
        "bar" => 0x7c,
        "braceright" => 0x7d,
        "asciitilde" => 0x7e,
        "nobreakspace" => 0xa0,
        _ => return None,
    };
    char::from_u32(code)
}

fn keysym_to_char(keysym: u32) -> Option<char> {
    match keysym {
        0x20..=0x7e | 0xa0..=0xff => char::from_u32(keysym),
        0x0100_0000..=0x0110_ffff => char::from_u32(keysym & 0x00ff_ffff),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEYMAP: &str = r#"
        xkb_keymap {
            xkb_keycodes {
                minimum = 8;
                maximum = 256;
                <AE01> = 10;
                <AD01> = 24;
                <AC01> = 38;
            };
            xkb_symbols {
                key <AE01> { [ 1, exclam ] };
                key <AD01> { [ q, Q ] };
                key <AC01> { [ a, A ] };
            };
        };
    "#;

    #[test]
    fn parses_keycodes_and_levels() {
        let keymap = Keymap::parse(KEYMAP).unwrap();
        assert_eq!(keymap.key_to_char(2, false), Some('1'));
        assert_eq!(keymap.key_to_char(2, true), Some('!'));
        assert_eq!(keymap.key_to_char(16, false), Some('q'));
        assert_eq!(keymap.key_to_char(16, true), Some('Q'));
        assert_eq!(keymap.key_to_char(30, true), Some('A'));
    }

    #[test]
    fn recognizes_unicode_and_named_keysyms() {
        assert_eq!(symbol_to_char("U1EC7"), Some('ệ'));
        assert_eq!(symbol_to_char("quotedbl"), Some('"'));
        assert_eq!(symbol_to_char("NoSymbol"), None);
    }
}
