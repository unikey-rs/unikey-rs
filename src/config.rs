//! Small dependency-free key/value configuration store.
//!
//! Configuration is intentionally human-editable and forward compatible:
//! unknown keys are preserved by callers that round-trip the file.

use crate::i18n::Language;
use crate::layout::Layout;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub language: Language,
    pub layout: Layout,
    pub restore_keys: bool,
    pub daemon: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::default(),
            layout: Layout::Telex,
            restore_keys: true,
            daemon: false,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        if let Some(path) = std::env::var_os("UNIKEY_RS_CONFIG") {
            return PathBuf::from(path);
        }
        if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(dir).join("unikey-rs/config");
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(".config/unikey-rs/config");
        }
        PathBuf::from("unikey-rs.conf")
    }

    pub fn load() -> io::Result<Self> {
        Self::load_from(Self::config_path())
    }

    pub fn load_from(path: impl AsRef<Path>) -> io::Result<Self> {
        match fs::read_to_string(path.as_ref()) {
            Ok(text) => Ok(Self::parse(&text)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        self.save_to(Self::config_path())
    }

    pub fn save_to(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.render())
    }

    fn parse(text: &str) -> Self {
        let mut config = Self::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "language" => {
                    if let Ok(language) = value.parse() {
                        config.language = language;
                    }
                }
                "layout" => {
                    if let Ok(layout) = value.parse() {
                        config.layout = layout;
                    }
                }
                "restore_keys" => config.restore_keys = parse_bool(value).unwrap_or(true),
                "daemon" => config.daemon = parse_bool(value).unwrap_or(false),
                _ => {}
            }
        }
        config
    }

    pub fn render(&self) -> String {
        format!(
            "# unikey-rs configuration\nlanguage={}\nlayout={}\nrestore_keys={}\ndaemon={}\n",
            self.language.id(),
            self.layout.id(),
            self.restore_keys,
            self.daemon
        )
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_known_and_ignores_unknown_values() {
        let config = Config {
            language: Language::English,
            layout: Layout::Vni,
            restore_keys: false,
            daemon: true,
        };
        let rendered = config.render();
        assert!(rendered.contains("language=en"));
        assert!(rendered.contains("layout=vni"));
        assert_eq!(Config::parse(&rendered), config);

        let with_unknown = format!("{rendered}future_key=hello\n");
        assert_eq!(Config::parse(&with_unknown), config);
    }

    #[test]
    fn defaults_when_file_is_missing() {
        let path = std::env::temp_dir().join(format!("unikey-rs-missing-{}", std::process::id()));
        assert_eq!(Config::load_from(path).unwrap(), Config::default());
    }
}
