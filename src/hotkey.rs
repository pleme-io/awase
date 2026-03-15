use std::fmt;

use serde::{Deserialize, Serialize};

use crate::AwaseError;

/// Modifier key flags. Uses a bitmask internally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Modifiers(u8);

impl Modifiers {
    pub const NONE: Self = Self(0);
    pub const CMD: Self = Self(1 << 0);
    pub const CTRL: Self = Self(1 << 1);
    pub const ALT: Self = Self(1 << 2);
    pub const SHIFT: Self = Self(1 << 3);
    pub const FN: Self = Self(1 << 4);
    pub const CAPS_LOCK: Self = Self(1 << 5);

    /// Convenience alias: Cmd+Ctrl+Alt+Shift (all four main modifiers).
    pub const HYPER: Self = Self(0b0000_1111);

    /// Returns `true` if `self` contains all flags in `other`.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Returns `true` if no modifier flags are set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Raw bitmask value.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Construct from a raw bitmask.
    #[must_use]
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }
}

impl std::ops::BitOr for Modifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Modifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for Modifiers {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.contains(Self::CMD) {
            parts.push("cmd");
        }
        if self.contains(Self::CTRL) {
            parts.push("ctrl");
        }
        if self.contains(Self::ALT) {
            parts.push("alt");
        }
        if self.contains(Self::SHIFT) {
            parts.push("shift");
        }
        if self.contains(Self::FN) {
            parts.push("fn");
        }
        if self.contains(Self::CAPS_LOCK) {
            parts.push("caps_lock");
        }
        write!(f, "{}", parts.join("+"))
    }
}

/// A keyboard key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Numbers
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,

    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20,

    // Whitespace / control
    Space,
    Return,
    Escape,
    Tab,
    Backspace,
    Delete,

    // Navigation
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,

    // Punctuation / symbols
    Grave,        // ` / ~
    Minus,        // - / _
    Equal,        // = / +
    LeftBracket,  // [ / {
    RightBracket, // ] / }
    Backslash,    // \ / |
    Semicolon,    // ; / :
    Quote,        // ' / "
    Comma,        // , / <
    Period,       // . / >
    Slash,        // / / ?

    // Numpad
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    NumpadDecimal,
    NumpadEnter,

    // Media / special
    VolumeUp,
    VolumeDown,
    Mute,
    BrightnessUp,
    BrightnessDown,
    PlayPause,
    NextTrack,
    PreviousTrack,
    PrintScreen,
    Insert,
    Pause,
    CapsLock,
    NumLock,
    ScrollLock,

    // Mouse buttons (for mouse bindings)
    MouseLeft,
    MouseRight,
    MouseMiddle,
    MouseButton4,
    MouseButton5,
}

impl Key {
    /// Parse a single key name (case-insensitive).
    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            // Letters
            "a" => Some(Self::A),
            "b" => Some(Self::B),
            "c" => Some(Self::C),
            "d" => Some(Self::D),
            "e" => Some(Self::E),
            "f" => Some(Self::F),
            "g" => Some(Self::G),
            "h" => Some(Self::H),
            "i" => Some(Self::I),
            "j" => Some(Self::J),
            "k" => Some(Self::K),
            "l" => Some(Self::L),
            "m" => Some(Self::M),
            "n" => Some(Self::N),
            "o" => Some(Self::O),
            "p" => Some(Self::P),
            "q" => Some(Self::Q),
            "r" => Some(Self::R),
            "s" => Some(Self::S),
            "t" => Some(Self::T),
            "u" => Some(Self::U),
            "v" => Some(Self::V),
            "w" => Some(Self::W),
            "x" => Some(Self::X),
            "y" => Some(Self::Y),
            "z" => Some(Self::Z),

            // Numbers
            "0" => Some(Self::Num0),
            "1" => Some(Self::Num1),
            "2" => Some(Self::Num2),
            "3" => Some(Self::Num3),
            "4" => Some(Self::Num4),
            "5" => Some(Self::Num5),
            "6" => Some(Self::Num6),
            "7" => Some(Self::Num7),
            "8" => Some(Self::Num8),
            "9" => Some(Self::Num9),

            // Function keys
            "f1" => Some(Self::F1),
            "f2" => Some(Self::F2),
            "f3" => Some(Self::F3),
            "f4" => Some(Self::F4),
            "f5" => Some(Self::F5),
            "f6" => Some(Self::F6),
            "f7" => Some(Self::F7),
            "f8" => Some(Self::F8),
            "f9" => Some(Self::F9),
            "f10" => Some(Self::F10),
            "f11" => Some(Self::F11),
            "f12" => Some(Self::F12),
            "f13" => Some(Self::F13),
            "f14" => Some(Self::F14),
            "f15" => Some(Self::F15),
            "f16" => Some(Self::F16),
            "f17" => Some(Self::F17),
            "f18" => Some(Self::F18),
            "f19" => Some(Self::F19),
            "f20" => Some(Self::F20),

            // Whitespace / control
            "space" => Some(Self::Space),
            "return" | "enter" => Some(Self::Return),
            "escape" | "esc" => Some(Self::Escape),
            "tab" => Some(Self::Tab),
            "backspace" | "bs" => Some(Self::Backspace),
            "delete" | "del" => Some(Self::Delete),

            // Navigation
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "home" => Some(Self::Home),
            "end" => Some(Self::End),
            "pageup" | "page_up" | "pgup" => Some(Self::PageUp),
            "pagedown" | "page_down" | "pgdn" => Some(Self::PageDown),

            // Punctuation / symbols
            "grave" | "`" | "backtick" => Some(Self::Grave),
            "minus" | "-" => Some(Self::Minus),
            "equal" | "equals" | "=" => Some(Self::Equal),
            "leftbracket" | "left_bracket" | "[" => Some(Self::LeftBracket),
            "rightbracket" | "right_bracket" | "]" => Some(Self::RightBracket),
            "backslash" | "\\" => Some(Self::Backslash),
            "semicolon" | ";" => Some(Self::Semicolon),
            "quote" | "'" => Some(Self::Quote),
            "comma" | "," => Some(Self::Comma),
            "period" | "." => Some(Self::Period),
            "slash" | "/" => Some(Self::Slash),

            // Numpad
            "numpad0" | "kp0" => Some(Self::Numpad0),
            "numpad1" | "kp1" => Some(Self::Numpad1),
            "numpad2" | "kp2" => Some(Self::Numpad2),
            "numpad3" | "kp3" => Some(Self::Numpad3),
            "numpad4" | "kp4" => Some(Self::Numpad4),
            "numpad5" | "kp5" => Some(Self::Numpad5),
            "numpad6" | "kp6" => Some(Self::Numpad6),
            "numpad7" | "kp7" => Some(Self::Numpad7),
            "numpad8" | "kp8" => Some(Self::Numpad8),
            "numpad9" | "kp9" => Some(Self::Numpad9),
            "numpadadd" | "kp_add" | "kp+" => Some(Self::NumpadAdd),
            "numpadsubtract" | "kp_subtract" | "kp-" => Some(Self::NumpadSubtract),
            "numpadmultiply" | "kp_multiply" | "kp*" => Some(Self::NumpadMultiply),
            "numpaddivide" | "kp_divide" | "kp/" => Some(Self::NumpadDivide),
            "numpaddecimal" | "kp_decimal" | "kp." => Some(Self::NumpadDecimal),
            "numpadenter" | "kp_enter" => Some(Self::NumpadEnter),

            // Media / special
            "volumeup" | "volume_up" => Some(Self::VolumeUp),
            "volumedown" | "volume_down" => Some(Self::VolumeDown),
            "mute" => Some(Self::Mute),
            "brightnessup" | "brightness_up" => Some(Self::BrightnessUp),
            "brightnessdown" | "brightness_down" => Some(Self::BrightnessDown),
            "playpause" | "play_pause" | "play" => Some(Self::PlayPause),
            "nexttrack" | "next_track" | "next" => Some(Self::NextTrack),
            "previoustrack" | "previous_track" | "prev" | "previous" => Some(Self::PreviousTrack),
            "printscreen" | "print_screen" | "prtsc" => Some(Self::PrintScreen),
            "insert" | "ins" => Some(Self::Insert),
            "pause" | "break" => Some(Self::Pause),
            "capslock" | "caps_lock" | "caps" => Some(Self::CapsLock),
            "numlock" | "num_lock" => Some(Self::NumLock),
            "scrolllock" | "scroll_lock" => Some(Self::ScrollLock),

            // Mouse buttons
            "mouseleft" | "mouse_left" | "mouse1" => Some(Self::MouseLeft),
            "mouseright" | "mouse_right" | "mouse2" => Some(Self::MouseRight),
            "mousemiddle" | "mouse_middle" | "mouse3" => Some(Self::MouseMiddle),
            "mousebutton4" | "mouse_button4" | "mouse4" => Some(Self::MouseButton4),
            "mousebutton5" | "mouse_button5" | "mouse5" => Some(Self::MouseButton5),

            _ => None,
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            // Letters
            Self::A => "a", Self::B => "b", Self::C => "c", Self::D => "d",
            Self::E => "e", Self::F => "f", Self::G => "g", Self::H => "h",
            Self::I => "i", Self::J => "j", Self::K => "k", Self::L => "l",
            Self::M => "m", Self::N => "n", Self::O => "o", Self::P => "p",
            Self::Q => "q", Self::R => "r", Self::S => "s", Self::T => "t",
            Self::U => "u", Self::V => "v", Self::W => "w", Self::X => "x",
            Self::Y => "y", Self::Z => "z",

            // Numbers
            Self::Num0 => "0", Self::Num1 => "1", Self::Num2 => "2",
            Self::Num3 => "3", Self::Num4 => "4", Self::Num5 => "5",
            Self::Num6 => "6", Self::Num7 => "7", Self::Num8 => "8",
            Self::Num9 => "9",

            // Function keys
            Self::F1 => "f1", Self::F2 => "f2", Self::F3 => "f3",
            Self::F4 => "f4", Self::F5 => "f5", Self::F6 => "f6",
            Self::F7 => "f7", Self::F8 => "f8", Self::F9 => "f9",
            Self::F10 => "f10", Self::F11 => "f11", Self::F12 => "f12",
            Self::F13 => "f13", Self::F14 => "f14", Self::F15 => "f15",
            Self::F16 => "f16", Self::F17 => "f17", Self::F18 => "f18",
            Self::F19 => "f19", Self::F20 => "f20",

            // Whitespace / control
            Self::Space => "space",
            Self::Return => "return",
            Self::Escape => "escape",
            Self::Tab => "tab",
            Self::Backspace => "backspace",
            Self::Delete => "delete",

            // Navigation
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
            Self::Home => "home",
            Self::End => "end",
            Self::PageUp => "pageup",
            Self::PageDown => "pagedown",

            // Punctuation / symbols
            Self::Grave => "grave",
            Self::Minus => "minus",
            Self::Equal => "equal",
            Self::LeftBracket => "leftbracket",
            Self::RightBracket => "rightbracket",
            Self::Backslash => "backslash",
            Self::Semicolon => "semicolon",
            Self::Quote => "quote",
            Self::Comma => "comma",
            Self::Period => "period",
            Self::Slash => "slash",

            // Numpad
            Self::Numpad0 => "numpad0", Self::Numpad1 => "numpad1",
            Self::Numpad2 => "numpad2", Self::Numpad3 => "numpad3",
            Self::Numpad4 => "numpad4", Self::Numpad5 => "numpad5",
            Self::Numpad6 => "numpad6", Self::Numpad7 => "numpad7",
            Self::Numpad8 => "numpad8", Self::Numpad9 => "numpad9",
            Self::NumpadAdd => "numpadadd",
            Self::NumpadSubtract => "numpadsubtract",
            Self::NumpadMultiply => "numpadmultiply",
            Self::NumpadDivide => "numpaddivide",
            Self::NumpadDecimal => "numpaddecimal",
            Self::NumpadEnter => "numpadenter",

            // Media / special
            Self::VolumeUp => "volumeup",
            Self::VolumeDown => "volumedown",
            Self::Mute => "mute",
            Self::BrightnessUp => "brightnessup",
            Self::BrightnessDown => "brightnessdown",
            Self::PlayPause => "playpause",
            Self::NextTrack => "nexttrack",
            Self::PreviousTrack => "previoustrack",
            Self::PrintScreen => "printscreen",
            Self::Insert => "insert",
            Self::Pause => "pause",
            Self::CapsLock => "capslock",
            Self::NumLock => "numlock",
            Self::ScrollLock => "scrolllock",

            // Mouse buttons
            Self::MouseLeft => "mouseleft",
            Self::MouseRight => "mouseright",
            Self::MouseMiddle => "mousemiddle",
            Self::MouseButton4 => "mousebutton4",
            Self::MouseButton5 => "mousebutton5",
        };
        write!(f, "{name}")
    }
}

/// A hotkey: a combination of modifier keys and a single key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hotkey {
    pub modifiers: Modifiers,
    pub key: Key,
}

impl Hotkey {
    /// Create a new hotkey from modifiers and a key.
    #[must_use]
    pub const fn new(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }

    /// Parse a hotkey string.
    ///
    /// Supports two formats:
    /// - Plus-separated: `"cmd+space"`, `"ctrl+alt+shift+k"`, `"f5"`
    /// - skhd-style dash-separated: `"cmd - h"`, `"ctrl + alt - space"`
    ///
    /// Parts are case-insensitive. Modifier names: `cmd`, `ctrl`, `alt`,
    /// `shift`, `fn`, `hyper`, `caps_lock`. The last non-modifier segment
    /// is treated as the key.
    pub fn parse(s: &str) -> Result<Self, AwaseError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(AwaseError::InvalidHotkey(
                "empty hotkey string".to_string(),
            ));
        }

        // Detect skhd format: contains " - " (space-dash-space)
        if trimmed.contains(" - ") {
            return Self::parse_skhd(trimmed);
        }

        Self::parse_plus(trimmed)
    }

    /// Parse plus-separated format: `"cmd+space"`, `"ctrl+alt+shift+k"`.
    fn parse_plus(s: &str) -> Result<Self, AwaseError> {
        let parts: Vec<&str> = s.split('+').map(str::trim).collect();

        if parts.is_empty() {
            return Err(AwaseError::InvalidHotkey(
                "empty hotkey string".to_string(),
            ));
        }

        let mut modifiers = Modifiers::NONE;
        let mut key_part: Option<&str> = None;

        for part in &parts {
            if let Some(m) = parse_modifier(part) {
                modifiers |= m;
            } else if key_part.is_some() {
                return Err(AwaseError::InvalidHotkey(format!(
                    "multiple keys in hotkey: {s}"
                )));
            } else {
                key_part = Some(part);
            }
        }

        // If a single token matched as modifier but is also a valid key
        // (e.g. "capslock"), treat it as a key.
        if key_part.is_none() && parts.len() == 1 {
            if Key::parse(parts[0]).is_some() {
                key_part = Some(parts[0]);
                modifiers = Modifiers::NONE;
            }
        }

        let Some(key_str) = key_part else {
            return Err(AwaseError::InvalidHotkey(format!(
                "no key found in hotkey: {s}"
            )));
        };

        let key = Key::parse(key_str).ok_or_else(|| {
            AwaseError::InvalidHotkey(format!("unknown key: {key_str}"))
        })?;

        Ok(Self { modifiers, key })
    }

    /// Parse skhd-style format: `"cmd - h"`, `"ctrl + alt - space"`.
    ///
    /// In skhd format, modifiers are separated by `+` on the left side of
    /// ` - `, and the key is on the right side.
    fn parse_skhd(s: &str) -> Result<Self, AwaseError> {
        let parts: Vec<&str> = s.splitn(2, " - ").collect();
        if parts.len() != 2 {
            return Err(AwaseError::InvalidHotkey(format!(
                "invalid skhd format: {s}"
            )));
        }

        let modifier_str = parts[0].trim();
        let key_str = parts[1].trim();

        if key_str.is_empty() {
            return Err(AwaseError::InvalidHotkey(format!(
                "no key after ' - ' in: {s}"
            )));
        }

        // Parse modifiers (separated by + or whitespace)
        let mut modifiers = Modifiers::NONE;
        if !modifier_str.is_empty() {
            for part in modifier_str.split('+').flat_map(|p| p.split_whitespace()) {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                if let Some(m) = parse_modifier(part) {
                    modifiers |= m;
                } else {
                    return Err(AwaseError::InvalidHotkey(format!(
                        "unknown modifier '{part}' in skhd format: {s}"
                    )));
                }
            }
        }

        let key = Key::parse(key_str).ok_or_else(|| {
            AwaseError::InvalidHotkey(format!("unknown key: {key_str}"))
        })?;

        Ok(Self { modifiers, key })
    }

    /// Format the hotkey as a human-readable string (e.g. `"cmd+space"`).
    ///
    /// The output is compatible with [`parse`](Self::parse).
    #[must_use]
    pub fn display(&self) -> String {
        if self.modifiers.is_empty() {
            self.key.to_string()
        } else {
            format!("{}+{}", self.modifiers, self.key)
        }
    }
}

impl fmt::Display for Hotkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Parse a modifier name (case-insensitive). Returns `None` if not a modifier.
fn parse_modifier(s: &str) -> Option<Modifiers> {
    match s.to_ascii_lowercase().as_str() {
        "cmd" | "command" | "super" | "meta" | "lcmd" | "rcmd" => Some(Modifiers::CMD),
        "ctrl" | "control" | "lctrl" | "rctrl" => Some(Modifiers::CTRL),
        "alt" | "option" | "opt" | "lalt" | "ralt" => Some(Modifiers::ALT),
        "shift" | "lshift" | "rshift" => Some(Modifiers::SHIFT),
        "fn" => Some(Modifiers::FN),
        "hyper" => Some(Modifiers::HYPER),
        "caps_lock" | "capslock" => Some(Modifiers::CAPS_LOCK),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Original tests ──────────────────────────────────────────────

    #[test]
    fn parse_cmd_space() {
        let hk = Hotkey::parse("cmd+space").unwrap();
        assert_eq!(hk.modifiers, Modifiers::CMD);
        assert_eq!(hk.key, Key::Space);
    }

    #[test]
    fn parse_ctrl_alt_shift_k() {
        let hk = Hotkey::parse("ctrl+alt+shift+k").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CTRL));
        assert!(hk.modifiers.contains(Modifiers::ALT));
        assert!(hk.modifiers.contains(Modifiers::SHIFT));
        assert!(!hk.modifiers.contains(Modifiers::CMD));
        assert_eq!(hk.key, Key::K);
    }

    #[test]
    fn parse_case_insensitive() {
        let hk = Hotkey::parse("CMD+SPACE").unwrap();
        assert_eq!(hk.modifiers, Modifiers::CMD);
        assert_eq!(hk.key, Key::Space);
    }

    #[test]
    fn parse_invalid_returns_error() {
        let result = Hotkey::parse("invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AwaseError::InvalidHotkey(_)));
    }

    #[test]
    fn parse_no_key_returns_error() {
        let result = Hotkey::parse("cmd+ctrl");
        assert!(result.is_err());
    }

    #[test]
    fn parse_multiple_keys_returns_error() {
        let result = Hotkey::parse("cmd+a+b");
        assert!(result.is_err());
    }

    #[test]
    fn modifiers_bitor() {
        let mods = Modifiers::CMD | Modifiers::SHIFT;
        assert!(mods.contains(Modifiers::CMD));
        assert!(mods.contains(Modifiers::SHIFT));
        assert!(!mods.contains(Modifiers::ALT));
        assert!(!mods.contains(Modifiers::CTRL));
    }

    #[test]
    fn modifiers_contains_none() {
        let mods = Modifiers::NONE;
        assert!(mods.is_empty());
        assert!(!mods.contains(Modifiers::CMD));
    }

    #[test]
    fn display_roundtrip() {
        let original = Hotkey::parse("cmd+space").unwrap();
        let displayed = original.display();
        let reparsed = Hotkey::parse(&displayed).unwrap();
        assert_eq!(original, reparsed);
    }

    #[test]
    fn display_roundtrip_multi_modifier() {
        let original = Hotkey::parse("ctrl+alt+shift+f5").unwrap();
        let displayed = original.display();
        let reparsed = Hotkey::parse(&displayed).unwrap();
        assert_eq!(original, reparsed);
    }

    #[test]
    fn display_key_only() {
        let hk = Hotkey::new(Modifiers::NONE, Key::Escape);
        assert_eq!(hk.display(), "escape");
    }

    #[test]
    fn parse_aliases() {
        let hk = Hotkey::parse("cmd+enter").unwrap();
        assert_eq!(hk.key, Key::Return);

        let hk = Hotkey::parse("option+a").unwrap();
        assert!(hk.modifiers.contains(Modifiers::ALT));

        let hk = Hotkey::parse("esc").unwrap();
        assert_eq!(hk.key, Key::Escape);
    }

    #[test]
    fn parse_function_keys() {
        for i in 1..=12 {
            let s = format!("f{i}");
            let hk = Hotkey::parse(&s).unwrap();
            assert_eq!(hk.display(), s);
        }
    }

    // ── New modifier tests ──────────────────────────────────────────

    #[test]
    fn parse_fn_modifier() {
        let hk = Hotkey::parse("fn+h").unwrap();
        assert!(hk.modifiers.contains(Modifiers::FN));
        assert_eq!(hk.key, Key::H);
    }

    #[test]
    fn parse_hyper_modifier() {
        let hk = Hotkey::parse("hyper+space").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CMD));
        assert!(hk.modifiers.contains(Modifiers::CTRL));
        assert!(hk.modifiers.contains(Modifiers::ALT));
        assert!(hk.modifiers.contains(Modifiers::SHIFT));
        assert_eq!(hk.key, Key::Space);
    }

    #[test]
    fn parse_caps_lock_modifier() {
        let hk = Hotkey::parse("caps_lock+a").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CAPS_LOCK));
        assert_eq!(hk.key, Key::A);

        let hk2 = Hotkey::parse("capslock+a").unwrap();
        assert_eq!(hk, hk2);
    }

    #[test]
    fn hyper_equals_all_four() {
        assert_eq!(
            Modifiers::HYPER,
            Modifiers::CMD | Modifiers::CTRL | Modifiers::ALT | Modifiers::SHIFT,
        );
    }

    #[test]
    fn fn_display_roundtrip() {
        let hk = Hotkey::parse("fn+f5").unwrap();
        let displayed = hk.display();
        let reparsed = Hotkey::parse(&displayed).unwrap();
        assert_eq!(hk, reparsed);
    }

    // ── New key tests ───────────────────────────────────────────────

    #[test]
    fn parse_navigation_keys() {
        assert_eq!(Hotkey::parse("home").unwrap().key, Key::Home);
        assert_eq!(Hotkey::parse("end").unwrap().key, Key::End);
        assert_eq!(Hotkey::parse("pageup").unwrap().key, Key::PageUp);
        assert_eq!(Hotkey::parse("pgdn").unwrap().key, Key::PageDown);
    }

    #[test]
    fn parse_punctuation_keys() {
        assert_eq!(Hotkey::parse("cmd+grave").unwrap().key, Key::Grave);
        assert_eq!(Hotkey::parse("cmd+minus").unwrap().key, Key::Minus);
        assert_eq!(Hotkey::parse("cmd+equal").unwrap().key, Key::Equal);
        assert_eq!(Hotkey::parse("cmd+leftbracket").unwrap().key, Key::LeftBracket);
        assert_eq!(Hotkey::parse("cmd+rightbracket").unwrap().key, Key::RightBracket);
        assert_eq!(Hotkey::parse("cmd+backslash").unwrap().key, Key::Backslash);
        assert_eq!(Hotkey::parse("cmd+semicolon").unwrap().key, Key::Semicolon);
        assert_eq!(Hotkey::parse("cmd+quote").unwrap().key, Key::Quote);
        assert_eq!(Hotkey::parse("cmd+comma").unwrap().key, Key::Comma);
        assert_eq!(Hotkey::parse("cmd+period").unwrap().key, Key::Period);
        assert_eq!(Hotkey::parse("cmd+slash").unwrap().key, Key::Slash);
    }

    #[test]
    fn parse_numpad_keys() {
        for i in 0..=9 {
            let s = format!("numpad{i}");
            assert_eq!(Hotkey::parse(&s).unwrap().key.to_string(), s);
        }
        assert_eq!(Hotkey::parse("numpadadd").unwrap().key, Key::NumpadAdd);
        assert_eq!(Hotkey::parse("kp_subtract").unwrap().key, Key::NumpadSubtract);
        assert_eq!(Hotkey::parse("kp_enter").unwrap().key, Key::NumpadEnter);
    }

    #[test]
    fn parse_media_keys() {
        assert_eq!(Hotkey::parse("volumeup").unwrap().key, Key::VolumeUp);
        assert_eq!(Hotkey::parse("volume_down").unwrap().key, Key::VolumeDown);
        assert_eq!(Hotkey::parse("mute").unwrap().key, Key::Mute);
        assert_eq!(Hotkey::parse("playpause").unwrap().key, Key::PlayPause);
        assert_eq!(Hotkey::parse("next_track").unwrap().key, Key::NextTrack);
        assert_eq!(Hotkey::parse("previous").unwrap().key, Key::PreviousTrack);
    }

    #[test]
    fn parse_mouse_buttons() {
        assert_eq!(Hotkey::parse("cmd+mouse1").unwrap().key, Key::MouseLeft);
        assert_eq!(Hotkey::parse("cmd+mouse_right").unwrap().key, Key::MouseRight);
        assert_eq!(Hotkey::parse("mouse3").unwrap().key, Key::MouseMiddle);
        assert_eq!(Hotkey::parse("mouse4").unwrap().key, Key::MouseButton4);
        assert_eq!(Hotkey::parse("mouse5").unwrap().key, Key::MouseButton5);
    }

    #[test]
    fn parse_extended_function_keys() {
        for i in 13..=20 {
            let s = format!("f{i}");
            let hk = Hotkey::parse(&s).unwrap();
            assert_eq!(hk.display(), s);
        }
    }

    #[test]
    fn parse_lock_keys() {
        assert_eq!(Hotkey::parse("capslock").unwrap().key, Key::CapsLock);
        assert_eq!(Hotkey::parse("numlock").unwrap().key, Key::NumLock);
        assert_eq!(Hotkey::parse("scrolllock").unwrap().key, Key::ScrollLock);
    }

    #[test]
    fn parse_special_keys() {
        assert_eq!(Hotkey::parse("insert").unwrap().key, Key::Insert);
        assert_eq!(Hotkey::parse("printscreen").unwrap().key, Key::PrintScreen);
        assert_eq!(Hotkey::parse("pause").unwrap().key, Key::Pause);
    }

    // ── skhd format tests ───────────────────────────────────────────

    #[test]
    fn parse_skhd_cmd_h() {
        let hk = Hotkey::parse("cmd - h").unwrap();
        assert_eq!(hk.modifiers, Modifiers::CMD);
        assert_eq!(hk.key, Key::H);
    }

    #[test]
    fn parse_skhd_multi_modifier() {
        let hk = Hotkey::parse("ctrl + alt - space").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CTRL));
        assert!(hk.modifiers.contains(Modifiers::ALT));
        assert_eq!(hk.key, Key::Space);
    }

    #[test]
    fn parse_key_only_escape() {
        let hk = Hotkey::parse("escape").unwrap();
        assert!(hk.modifiers.is_empty());
        assert_eq!(hk.key, Key::Escape);
    }

    #[test]
    fn parse_skhd_hyper() {
        let hk = Hotkey::parse("hyper - j").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CMD));
        assert!(hk.modifiers.contains(Modifiers::CTRL));
        assert!(hk.modifiers.contains(Modifiers::ALT));
        assert!(hk.modifiers.contains(Modifiers::SHIFT));
        assert_eq!(hk.key, Key::J);
    }

    #[test]
    fn parse_skhd_fn() {
        let hk = Hotkey::parse("fn - h").unwrap();
        assert!(hk.modifiers.contains(Modifiers::FN));
        assert_eq!(hk.key, Key::H);
    }

    #[test]
    fn skhd_and_plus_equivalent() {
        let skhd = Hotkey::parse("cmd + alt - h").unwrap();
        let plus = Hotkey::parse("cmd+alt+h").unwrap();
        assert_eq!(skhd, plus);
    }

    #[test]
    fn parse_skhd_invalid_modifier() {
        let result = Hotkey::parse("bogus - h");
        assert!(result.is_err());
    }

    #[test]
    fn parse_skhd_no_key() {
        let result = Hotkey::parse("cmd - ");
        assert!(result.is_err());
    }

    // ── Modifier left/right alias tests ─────────────────────────────

    #[test]
    fn parse_left_right_modifier_aliases() {
        let hk = Hotkey::parse("lcmd+a").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CMD));

        let hk = Hotkey::parse("ralt+b").unwrap();
        assert!(hk.modifiers.contains(Modifiers::ALT));

        let hk = Hotkey::parse("lshift+c").unwrap();
        assert!(hk.modifiers.contains(Modifiers::SHIFT));

        let hk = Hotkey::parse("rctrl+d").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CTRL));
    }

    // ── BitAnd test ─────────────────────────────────────────────────

    #[test]
    fn modifiers_bitand() {
        let mods = Modifiers::CMD | Modifiers::SHIFT;
        let masked = mods & Modifiers::CMD;
        assert_eq!(masked, Modifiers::CMD);

        let empty = mods & Modifiers::ALT;
        assert!(empty.is_empty());
    }

    // ── from_bits test ──────────────────────────────────────────────

    #[test]
    fn modifiers_from_bits() {
        let mods = Modifiers::from_bits(0b0000_0101); // CMD | ALT
        assert!(mods.contains(Modifiers::CMD));
        assert!(mods.contains(Modifiers::ALT));
        assert!(!mods.contains(Modifiers::SHIFT));
    }

    // ── Additional hotkey parsing edge cases ────────────────────────

    #[test]
    fn parse_empty_string_returns_error() {
        let result = Hotkey::parse("");
        assert!(result.is_err());
        match result.unwrap_err() {
            AwaseError::InvalidHotkey(msg) => assert!(msg.contains("empty")),
            other => panic!("expected InvalidHotkey, got {other:?}"),
        }
    }

    #[test]
    fn parse_whitespace_only_returns_error() {
        let result = Hotkey::parse("   ");
        assert!(result.is_err());
    }

    #[test]
    fn parse_trims_whitespace() {
        let hk = Hotkey::parse("  cmd+space  ").unwrap();
        assert_eq!(hk.modifiers, Modifiers::CMD);
        assert_eq!(hk.key, Key::Space);
    }

    #[test]
    fn parse_whitespace_around_plus() {
        let hk = Hotkey::parse("cmd + space").unwrap();
        assert_eq!(hk.modifiers, Modifiers::CMD);
        assert_eq!(hk.key, Key::Space);
    }

    #[test]
    fn parse_single_letter_key_only() {
        let hk = Hotkey::parse("a").unwrap();
        assert!(hk.modifiers.is_empty());
        assert_eq!(hk.key, Key::A);
    }

    #[test]
    fn parse_single_digit_key_only() {
        let hk = Hotkey::parse("5").unwrap();
        assert!(hk.modifiers.is_empty());
        assert_eq!(hk.key, Key::Num5);
    }

    #[test]
    fn parse_mixed_case_modifiers() {
        let hk = Hotkey::parse("Cmd+Shift+A").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CMD));
        assert!(hk.modifiers.contains(Modifiers::SHIFT));
        assert_eq!(hk.key, Key::A);
    }

    #[test]
    fn parse_modifier_aliases() {
        // command == cmd
        let hk = Hotkey::parse("command+a").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CMD));

        // super == cmd
        let hk = Hotkey::parse("super+a").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CMD));

        // meta == cmd
        let hk = Hotkey::parse("meta+a").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CMD));

        // control == ctrl
        let hk = Hotkey::parse("control+a").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CTRL));

        // option == alt
        let hk = Hotkey::parse("option+a").unwrap();
        assert!(hk.modifiers.contains(Modifiers::ALT));

        // opt == alt
        let hk = Hotkey::parse("opt+a").unwrap();
        assert!(hk.modifiers.contains(Modifiers::ALT));
    }

    #[test]
    fn parse_duplicate_modifier_is_idempotent() {
        // Specifying the same modifier twice should be the same as once
        let hk = Hotkey::parse("cmd+cmd+a").unwrap();
        assert_eq!(hk.modifiers, Modifiers::CMD);
        assert_eq!(hk.key, Key::A);
    }

    #[test]
    fn parse_hyper_is_all_four_modifiers() {
        let hk = Hotkey::parse("hyper+a").unwrap();
        let manual = Hotkey::parse("cmd+ctrl+alt+shift+a").unwrap();
        assert_eq!(hk, manual);
    }

    #[test]
    fn parse_unknown_key_error_message_contains_name() {
        let result = Hotkey::parse("cmd+nonexistent");
        assert!(result.is_err());
        match result.unwrap_err() {
            AwaseError::InvalidHotkey(msg) => assert!(msg.contains("nonexistent")),
            other => panic!("expected InvalidHotkey, got {other:?}"),
        }
    }

    #[test]
    fn parse_backspace_aliases() {
        let hk = Hotkey::parse("backspace").unwrap();
        assert_eq!(hk.key, Key::Backspace);
        let hk = Hotkey::parse("bs").unwrap();
        assert_eq!(hk.key, Key::Backspace);
    }

    #[test]
    fn parse_delete_aliases() {
        let hk = Hotkey::parse("delete").unwrap();
        assert_eq!(hk.key, Key::Delete);
        let hk = Hotkey::parse("del").unwrap();
        assert_eq!(hk.key, Key::Delete);
    }

    #[test]
    fn parse_page_up_aliases() {
        let a = Hotkey::parse("pageup").unwrap();
        let b = Hotkey::parse("page_up").unwrap();
        let c = Hotkey::parse("pgup").unwrap();
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn parse_page_down_aliases() {
        let a = Hotkey::parse("pagedown").unwrap();
        let b = Hotkey::parse("page_down").unwrap();
        let c = Hotkey::parse("pgdn").unwrap();
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn parse_numpad_kp_aliases() {
        assert_eq!(Hotkey::parse("kp0").unwrap().key, Key::Numpad0);
        assert_eq!(Hotkey::parse("kp9").unwrap().key, Key::Numpad9);
        // "kp+" cannot be parsed in plus-separated format since + is the delimiter.
        // Use the full name or kp_add alias instead.
        assert_eq!(Hotkey::parse("kp_add").unwrap().key, Key::NumpadAdd);
        assert_eq!(Hotkey::parse("kp_subtract").unwrap().key, Key::NumpadSubtract);
        assert_eq!(Hotkey::parse("kp_multiply").unwrap().key, Key::NumpadMultiply);
        assert_eq!(Hotkey::parse("kp_divide").unwrap().key, Key::NumpadDivide);
        assert_eq!(Hotkey::parse("kp_decimal").unwrap().key, Key::NumpadDecimal);
        assert_eq!(Hotkey::parse("kp_enter").unwrap().key, Key::NumpadEnter);
    }

    #[test]
    fn parse_symbol_literals() {
        // Test parsing keys by their symbol character
        assert_eq!(Hotkey::parse("cmd+`").unwrap().key, Key::Grave);
        assert_eq!(Hotkey::parse("cmd+[").unwrap().key, Key::LeftBracket);
        assert_eq!(Hotkey::parse("cmd+]").unwrap().key, Key::RightBracket);
        assert_eq!(Hotkey::parse("cmd+;").unwrap().key, Key::Semicolon);
        assert_eq!(Hotkey::parse("cmd+'").unwrap().key, Key::Quote);
        assert_eq!(Hotkey::parse("cmd+,").unwrap().key, Key::Comma);
        assert_eq!(Hotkey::parse("cmd+.").unwrap().key, Key::Period);
        assert_eq!(Hotkey::parse("cmd+/").unwrap().key, Key::Slash);
    }

    #[test]
    fn parse_print_screen_aliases() {
        let a = Hotkey::parse("printscreen").unwrap();
        let b = Hotkey::parse("print_screen").unwrap();
        let c = Hotkey::parse("prtsc").unwrap();
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn parse_pause_break_alias() {
        let a = Hotkey::parse("pause").unwrap();
        let b = Hotkey::parse("break").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn parse_brightness_keys() {
        assert_eq!(Hotkey::parse("brightnessup").unwrap().key, Key::BrightnessUp);
        assert_eq!(Hotkey::parse("brightness_up").unwrap().key, Key::BrightnessUp);
        assert_eq!(Hotkey::parse("brightnessdown").unwrap().key, Key::BrightnessDown);
        assert_eq!(Hotkey::parse("brightness_down").unwrap().key, Key::BrightnessDown);
    }

    #[test]
    fn parse_insert_alias() {
        let a = Hotkey::parse("insert").unwrap();
        let b = Hotkey::parse("ins").unwrap();
        assert_eq!(a, b);
    }

    // ── skhd format additional edge cases ───────────────────────────

    #[test]
    fn parse_skhd_no_modifier() {
        // skhd format with empty modifier side: " - escape"
        // This won't be detected as skhd because " - " requires text on the left.
        // Instead we test with whitespace on the left
        let hk = Hotkey::parse("shift - escape").unwrap();
        assert!(hk.modifiers.contains(Modifiers::SHIFT));
        assert_eq!(hk.key, Key::Escape);
    }

    #[test]
    fn parse_skhd_unknown_key_error() {
        let result = Hotkey::parse("cmd - nonexistent");
        assert!(result.is_err());
        match result.unwrap_err() {
            AwaseError::InvalidHotkey(msg) => assert!(msg.contains("nonexistent")),
            other => panic!("expected InvalidHotkey, got {other:?}"),
        }
    }

    #[test]
    fn parse_skhd_three_modifiers() {
        let hk = Hotkey::parse("ctrl + alt + shift - k").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CTRL));
        assert!(hk.modifiers.contains(Modifiers::ALT));
        assert!(hk.modifiers.contains(Modifiers::SHIFT));
        assert_eq!(hk.key, Key::K);
    }

    #[test]
    fn parse_skhd_fn_modifier() {
        let hk = Hotkey::parse("fn - left").unwrap();
        assert!(hk.modifiers.contains(Modifiers::FN));
        assert_eq!(hk.key, Key::Left);
    }

    // ── Display and formatting tests ────────────────────────────────

    #[test]
    fn display_no_modifiers_key_only() {
        let hk = Hotkey::new(Modifiers::NONE, Key::F5);
        assert_eq!(format!("{hk}"), "f5");
    }

    #[test]
    fn display_single_modifier() {
        let hk = Hotkey::new(Modifiers::CTRL, Key::C);
        assert_eq!(format!("{hk}"), "ctrl+c");
    }

    #[test]
    fn display_multiple_modifiers_deterministic_order() {
        // Modifiers should always display in cmd, ctrl, alt, shift, fn, caps_lock order
        let hk = Hotkey::new(Modifiers::SHIFT | Modifiers::CMD | Modifiers::ALT, Key::A);
        assert_eq!(format!("{hk}"), "cmd+alt+shift+a");
    }

    #[test]
    fn display_hyper_shows_all_four() {
        let hk = Hotkey::new(Modifiers::HYPER, Key::Space);
        assert_eq!(format!("{hk}"), "cmd+ctrl+alt+shift+space");
    }

    #[test]
    fn display_fn_modifier() {
        let hk = Hotkey::new(Modifiers::FN, Key::F5);
        assert_eq!(format!("{hk}"), "fn+f5");
    }

    #[test]
    fn display_caps_lock_modifier() {
        let hk = Hotkey::new(Modifiers::CAPS_LOCK, Key::A);
        assert_eq!(format!("{hk}"), "caps_lock+a");
    }

    #[test]
    fn modifiers_display_empty() {
        let mods = Modifiers::NONE;
        assert_eq!(format!("{mods}"), "");
    }

    #[test]
    fn display_roundtrip_all_modifiers_combined() {
        let mods = Modifiers::CMD | Modifiers::CTRL | Modifiers::ALT | Modifiers::SHIFT | Modifiers::FN;
        let hk = Hotkey::new(mods, Key::A);
        let displayed = hk.display();
        let reparsed = Hotkey::parse(&displayed).unwrap();
        assert_eq!(hk, reparsed);
    }

    #[test]
    fn display_roundtrip_caps_lock_modifier() {
        let hk = Hotkey::new(Modifiers::CAPS_LOCK, Key::A);
        let displayed = hk.display();
        let reparsed = Hotkey::parse(&displayed).unwrap();
        assert_eq!(hk, reparsed);
    }

    // ── Key Display roundtrip ───────────────────────────────────────

    #[test]
    fn key_display_roundtrip_all_keys() {
        // Every Key variant's Display should parse back to the same key
        let all_keys = [
            Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H,
            Key::I, Key::J, Key::K, Key::L, Key::M, Key::N, Key::O, Key::P,
            Key::Q, Key::R, Key::S, Key::T, Key::U, Key::V, Key::W, Key::X,
            Key::Y, Key::Z,
            Key::Num0, Key::Num1, Key::Num2, Key::Num3, Key::Num4,
            Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9,
            Key::F1, Key::F2, Key::F3, Key::F4, Key::F5, Key::F6,
            Key::F7, Key::F8, Key::F9, Key::F10, Key::F11, Key::F12,
            Key::F13, Key::F14, Key::F15, Key::F16, Key::F17, Key::F18,
            Key::F19, Key::F20,
            Key::Space, Key::Return, Key::Escape, Key::Tab, Key::Backspace,
            Key::Delete,
            Key::Up, Key::Down, Key::Left, Key::Right,
            Key::Home, Key::End, Key::PageUp, Key::PageDown,
            Key::Grave, Key::Minus, Key::Equal, Key::LeftBracket,
            Key::RightBracket, Key::Backslash, Key::Semicolon, Key::Quote,
            Key::Comma, Key::Period, Key::Slash,
            Key::Numpad0, Key::Numpad1, Key::Numpad2, Key::Numpad3,
            Key::Numpad4, Key::Numpad5, Key::Numpad6, Key::Numpad7,
            Key::Numpad8, Key::Numpad9,
            Key::NumpadAdd, Key::NumpadSubtract, Key::NumpadMultiply,
            Key::NumpadDivide, Key::NumpadDecimal, Key::NumpadEnter,
            Key::VolumeUp, Key::VolumeDown, Key::Mute,
            Key::BrightnessUp, Key::BrightnessDown,
            Key::PlayPause, Key::NextTrack, Key::PreviousTrack,
            Key::PrintScreen, Key::Insert, Key::Pause,
            Key::CapsLock, Key::NumLock, Key::ScrollLock,
            Key::MouseLeft, Key::MouseRight, Key::MouseMiddle,
            Key::MouseButton4, Key::MouseButton5,
        ];

        for key in all_keys {
            let displayed = key.to_string();
            let parsed = Key::parse(&displayed);
            assert_eq!(
                parsed,
                Some(key),
                "Display roundtrip failed for {key:?}: displayed as \"{displayed}\", parsed as {parsed:?}"
            );
        }
    }

    // ── Modifiers bitwise operations ────────────────────────────────

    #[test]
    fn modifiers_bitor_assign() {
        let mut mods = Modifiers::CMD;
        mods |= Modifiers::SHIFT;
        assert!(mods.contains(Modifiers::CMD));
        assert!(mods.contains(Modifiers::SHIFT));
        assert!(!mods.contains(Modifiers::ALT));
    }

    #[test]
    fn modifiers_bits_roundtrip() {
        let original = Modifiers::CMD | Modifiers::ALT | Modifiers::FN;
        let bits = original.bits();
        let restored = Modifiers::from_bits(bits);
        assert_eq!(original, restored);
    }

    #[test]
    fn modifiers_none_contains_nothing() {
        assert!(!Modifiers::NONE.contains(Modifiers::CMD));
        assert!(!Modifiers::NONE.contains(Modifiers::CTRL));
        assert!(!Modifiers::NONE.contains(Modifiers::ALT));
        assert!(!Modifiers::NONE.contains(Modifiers::SHIFT));
        assert!(!Modifiers::NONE.contains(Modifiers::FN));
        assert!(!Modifiers::NONE.contains(Modifiers::CAPS_LOCK));
    }

    #[test]
    fn modifiers_any_contains_none() {
        // NONE (0) is always "contained" because (x & 0) == 0
        assert!(Modifiers::CMD.contains(Modifiers::NONE));
        assert!(Modifiers::NONE.contains(Modifiers::NONE));
    }

    #[test]
    fn modifiers_hyper_contains_all_four() {
        assert!(Modifiers::HYPER.contains(Modifiers::CMD));
        assert!(Modifiers::HYPER.contains(Modifiers::CTRL));
        assert!(Modifiers::HYPER.contains(Modifiers::ALT));
        assert!(Modifiers::HYPER.contains(Modifiers::SHIFT));
        // HYPER does not include FN or CAPS_LOCK
        assert!(!Modifiers::HYPER.contains(Modifiers::FN));
        assert!(!Modifiers::HYPER.contains(Modifiers::CAPS_LOCK));
    }

    // ── Hotkey equality and hashing ─────────────────────────────────

    #[test]
    fn hotkey_equality_different_parse_same_result() {
        let a = Hotkey::parse("cmd+space").unwrap();
        let b = Hotkey::parse("CMD+SPACE").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn hotkey_inequality_different_modifiers() {
        let a = Hotkey::parse("cmd+a").unwrap();
        let b = Hotkey::parse("ctrl+a").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn hotkey_inequality_different_keys() {
        let a = Hotkey::parse("cmd+a").unwrap();
        let b = Hotkey::parse("cmd+b").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn hotkey_hash_consistent() {
        use std::collections::HashSet;
        let a = Hotkey::parse("cmd+space").unwrap();
        let b = Hotkey::parse("cmd+space").unwrap();
        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }

    #[test]
    fn hotkey_hash_different_hotkeys() {
        use std::collections::HashSet;
        let a = Hotkey::parse("cmd+a").unwrap();
        let b = Hotkey::parse("cmd+b").unwrap();
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        assert_eq!(set.len(), 2);
    }

    // ── Serde roundtrip for Hotkey ──────────────────────────────────

    #[test]
    fn hotkey_serde_roundtrip() {
        let hk = Hotkey::parse("cmd+shift+a").unwrap();
        let json = serde_json::to_string(&hk).unwrap();
        let deserialized: Hotkey = serde_json::from_str(&json).unwrap();
        assert_eq!(hk, deserialized);
    }

    #[test]
    fn hotkey_serde_no_modifiers() {
        let hk = Hotkey::new(Modifiers::NONE, Key::Escape);
        let json = serde_json::to_string(&hk).unwrap();
        let deserialized: Hotkey = serde_json::from_str(&json).unwrap();
        assert_eq!(hk, deserialized);
    }

    #[test]
    fn key_serde_roundtrip_all_variants() {
        let keys = [
            Key::A, Key::Z, Key::Num0, Key::Num9, Key::F1, Key::F20,
            Key::Space, Key::Return, Key::Escape, Key::Tab,
            Key::Up, Key::Home, Key::PageDown,
            Key::Grave, Key::Semicolon, Key::Slash,
            Key::Numpad0, Key::NumpadEnter,
            Key::VolumeUp, Key::Mute, Key::PlayPause,
            Key::MouseLeft, Key::MouseButton5,
        ];
        for key in keys {
            let json = serde_json::to_string(&key).unwrap();
            let deserialized: Key = serde_json::from_str(&json).unwrap();
            assert_eq!(key, deserialized, "serde roundtrip failed for {key:?}");
        }
    }

    #[test]
    fn modifiers_serde_roundtrip() {
        let cases = [
            Modifiers::NONE,
            Modifiers::CMD,
            Modifiers::HYPER,
            Modifiers::CMD | Modifiers::FN | Modifiers::CAPS_LOCK,
        ];
        for mods in cases {
            let json = serde_json::to_string(&mods).unwrap();
            let deserialized: Modifiers = serde_json::from_str(&json).unwrap();
            assert_eq!(mods, deserialized, "serde roundtrip failed for {mods:?}");
        }
    }

    // ── Capslock ambiguity: modifier vs key ─────────────────────────

    #[test]
    fn capslock_alone_is_key() {
        // "capslock" alone should parse as a key, not a modifier-only combo
        let hk = Hotkey::parse("capslock").unwrap();
        assert!(hk.modifiers.is_empty());
        assert_eq!(hk.key, Key::CapsLock);
    }

    #[test]
    fn caps_lock_as_modifier_with_key() {
        // "caps_lock+a" should treat caps_lock as modifier
        let hk = Hotkey::parse("caps_lock+a").unwrap();
        assert!(hk.modifiers.contains(Modifiers::CAPS_LOCK));
        assert_eq!(hk.key, Key::A);
    }
}
