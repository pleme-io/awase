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
        write!(f, "{}", parts.join("+"))
    }
}

/// A keyboard key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Space,
    Return,
    Escape,
    Tab,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
}

impl Key {
    /// Parse a single key name (case-insensitive).
    fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
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
            "space" => Some(Self::Space),
            "return" | "enter" => Some(Self::Return),
            "escape" | "esc" => Some(Self::Escape),
            "tab" => Some(Self::Tab),
            "backspace" => Some(Self::Backspace),
            "delete" | "del" => Some(Self::Delete),
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            _ => None,
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::A => "a", Self::B => "b", Self::C => "c", Self::D => "d",
            Self::E => "e", Self::F => "f", Self::G => "g", Self::H => "h",
            Self::I => "i", Self::J => "j", Self::K => "k", Self::L => "l",
            Self::M => "m", Self::N => "n", Self::O => "o", Self::P => "p",
            Self::Q => "q", Self::R => "r", Self::S => "s", Self::T => "t",
            Self::U => "u", Self::V => "v", Self::W => "w", Self::X => "x",
            Self::Y => "y", Self::Z => "z",
            Self::Num0 => "0", Self::Num1 => "1", Self::Num2 => "2",
            Self::Num3 => "3", Self::Num4 => "4", Self::Num5 => "5",
            Self::Num6 => "6", Self::Num7 => "7", Self::Num8 => "8",
            Self::Num9 => "9",
            Self::F1 => "f1", Self::F2 => "f2", Self::F3 => "f3",
            Self::F4 => "f4", Self::F5 => "f5", Self::F6 => "f6",
            Self::F7 => "f7", Self::F8 => "f8", Self::F9 => "f9",
            Self::F10 => "f10", Self::F11 => "f11", Self::F12 => "f12",
            Self::Space => "space",
            Self::Return => "return",
            Self::Escape => "escape",
            Self::Tab => "tab",
            Self::Backspace => "backspace",
            Self::Delete => "delete",
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
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

    /// Parse a hotkey string like `"cmd+space"` or `"ctrl+alt+shift+k"`.
    ///
    /// Parts are separated by `+` and are case-insensitive. Modifier
    /// names: `cmd`, `ctrl`, `alt`, `shift`. The last non-modifier
    /// segment is treated as the key.
    pub fn parse(s: &str) -> Result<Self, AwaseError> {
        let parts: Vec<&str> = s.split('+').map(str::trim).collect();

        if parts.is_empty() {
            return Err(AwaseError::InvalidHotkey(
                "empty hotkey string".to_string(),
            ));
        }

        let mut modifiers = Modifiers::NONE;
        let mut key_part: Option<&str> = None;

        for part in &parts {
            match part.to_ascii_lowercase().as_str() {
                "cmd" | "command" | "super" | "meta" => modifiers |= Modifiers::CMD,
                "ctrl" | "control" => modifiers |= Modifiers::CTRL,
                "alt" | "option" | "opt" => modifiers |= Modifiers::ALT,
                "shift" => modifiers |= Modifiers::SHIFT,
                _ => {
                    if key_part.is_some() {
                        return Err(AwaseError::InvalidHotkey(format!(
                            "multiple keys in hotkey: {s}"
                        )));
                    }
                    key_part = Some(part);
                }
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

#[cfg(test)]
mod tests {
    use super::*;

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
        // "enter" is an alias for Return
        let hk = Hotkey::parse("cmd+enter").unwrap();
        assert_eq!(hk.key, Key::Return);

        // "option" is an alias for Alt
        let hk = Hotkey::parse("option+a").unwrap();
        assert!(hk.modifiers.contains(Modifiers::ALT));

        // "esc" is an alias for Escape
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
}
