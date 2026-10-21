//! Compact user-interface localization.
//!
//! Vietnamese is the default beta language. English remains available through
//! `config-set language en`; the product identity `UniKey RS` is not translated.

use crate::layout::Layout;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Language {
    #[default]
    Vietnamese,
    English,
}

impl Language {
    pub fn id(self) -> &'static str {
        match self {
            Self::Vietnamese => "vi",
            Self::English => "en",
        }
    }

    pub fn strings(self) -> UiStrings {
        match self {
            Self::Vietnamese => UiStrings {
                layouts_heading: "Kiểu gõ",
                layout_name: "Tên kiểu gõ",
                layout_description: "Cách nhập",
                composer_layout: "Bố cục bộ gõ",
                type_here: "Nhập tại đây",
                raw: "GỐC",
                output: "KẾT QUẢ",
                click_help: "Chọn kiểu gõ hoặc nhấn 1-4. Backspace sửa bản xem trước.",
                quit: "THOÁT",
                wayland_native: "WAYLAND NGUYÊN BẢN",
                doctor_title: "UniKey RS",
                os: "Hệ điều hành",
                session_type: "Phiên làm việc",
                desktop: "Môi trường màn hình",
                wayland_display: "Wayland display",
                runtime_dir: "Thư mục runtime",
                platform_note: "Hỗ trợ distro bất biến qua AppImage",
                wayland_socket: "Wayland socket",
                wayland_globals: "Đối tượng Wayland",
                input_method_v2: "Input Method v2",
                input_method_v1: "Input Method v1",
                available: "khả dụng",
                unavailable: "không được quảng bá",
                saved: "đã lưu",
            },
            Self::English => UiStrings {
                layouts_heading: "Input layouts",
                layout_name: "Layout",
                layout_description: "Composition",
                composer_layout: "Composer layout",
                type_here: "Type here",
                raw: "RAW",
                output: "OUT",
                click_help: "Click a layout or press 1-4. Backspace edits the preview.",
                quit: "QUIT",
                wayland_native: "WAYLAND NATIVE",
                doctor_title: "UniKey RS",
                os: "OS",
                session_type: "Session type",
                desktop: "Desktop",
                wayland_display: "Wayland display",
                runtime_dir: "XDG_RUNTIME_DIR",
                platform_note: "Immutable distributions supported through AppImage",
                wayland_socket: "Wayland socket",
                wayland_globals: "Wayland globals",
                input_method_v2: "Input Method v2",
                input_method_v1: "Input Method v1",
                available: "available",
                unavailable: "not advertised",
                saved: "saved",
            },
        }
    }

    pub fn layout_short(self, layout: Layout) -> &'static str {
        match (self, layout) {
            (Self::Vietnamese, Layout::English) => "ANH",
            (Self::Vietnamese, Layout::Telex) => "TELEX",
            (Self::Vietnamese, Layout::Vni) => "VNI",
            (Self::Vietnamese, Layout::Viqr) => "VIQR",
            (Self::English, Layout::English) => "ENGLISH",
            (Self::English, Layout::Telex) => "TELEX",
            (Self::English, Layout::Vni) => "VNI",
            (Self::English, Layout::Viqr) => "VIQR",
        }
    }

    pub fn layout_name(self, layout: Layout) -> &'static str {
        match (self, layout) {
            (Self::Vietnamese, Layout::English) => "Tiếng Anh (US)",
            (Self::Vietnamese, Layout::Telex) => "Tiếng Việt Telex",
            (Self::Vietnamese, Layout::Vni) => "Tiếng Việt VNI",
            (Self::Vietnamese, Layout::Viqr) => "Tiếng Việt VIQR",
            (Self::English, _) => layout.display_name(),
        }
    }

    pub fn layout_description(self, layout: Layout) -> &'static str {
        match self {
            Self::Vietnamese => match layout {
                Layout::English => "Nhập QWERTY không biến đổi",
                Layout::Telex => "aa=â, aw=ă, ow=ơ, uw=ư, dd=đ, as=á",
                Layout::Vni => "a6=â, a8=ă, o7=ơ, u7=ư, d9=đ, a1=á",
                Layout::Viqr => "a^=â, a(=ă, o+=ơ, u+=ư, dd=đ, a'=á",
            },
            Self::English => layout.description(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UiStrings {
    pub layouts_heading: &'static str,
    pub layout_name: &'static str,
    pub layout_description: &'static str,
    pub composer_layout: &'static str,
    pub type_here: &'static str,
    pub raw: &'static str,
    pub output: &'static str,
    pub click_help: &'static str,
    pub quit: &'static str,
    pub wayland_native: &'static str,
    pub doctor_title: &'static str,
    pub os: &'static str,
    pub session_type: &'static str,
    pub desktop: &'static str,
    pub wayland_display: &'static str,
    pub runtime_dir: &'static str,
    pub platform_note: &'static str,
    pub wayland_socket: &'static str,
    pub wayland_globals: &'static str,
    pub input_method_v2: &'static str,
    pub input_method_v1: &'static str,
    pub available: &'static str,
    pub unavailable: &'static str,
    pub saved: &'static str,
}

#[derive(Debug)]
pub struct ParseLanguageError(String);

impl fmt::Display for ParseLanguageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown language `{}`; expected vi or en", self.0)
    }
}

impl std::error::Error for ParseLanguageError {}

impl FromStr for Language {
    type Err = ParseLanguageError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "vi" | "vi-vn" | "vietnamese" | "tieng-viet" => Ok(Self::Vietnamese),
            "en" | "en-us" | "english" => Ok(Self::English),
            other => Err(ParseLanguageError(other.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vietnamese_is_default_and_parses_aliases() {
        assert_eq!(Language::default(), Language::Vietnamese);
        assert_eq!("vi-VN".parse::<Language>().unwrap(), Language::Vietnamese);
        assert_eq!("english".parse::<Language>().unwrap(), Language::English);
        assert!("fr".parse::<Language>().is_err());
    }

    #[test]
    fn translates_layout_metadata() {
        assert_eq!(Language::Vietnamese.layout_name(Layout::Telex), "Tiếng Việt Telex");
        assert_eq!(Language::English.layout_name(Layout::Telex), "Vietnamese Telex");
    }
}
