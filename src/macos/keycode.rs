//! macOS virtual keycode ↔ awase Key mapping tables.

use crate::Key;

/// Map an awase `Key` to the macOS virtual keycode.
///
/// Returns `None` for keys that don't have a direct macOS keycode
/// (e.g. mouse buttons).
#[must_use]
pub fn key_to_keycode(key: Key) -> Option<u16> {
    // Values from Events.h / Carbon HIToolbox
    match key {
        // Letters (ANSI layout)
        Key::A => Some(0x00),
        Key::B => Some(0x0B),
        Key::C => Some(0x08),
        Key::D => Some(0x02),
        Key::E => Some(0x0E),
        Key::F => Some(0x03),
        Key::G => Some(0x05),
        Key::H => Some(0x04),
        Key::I => Some(0x22),
        Key::J => Some(0x26),
        Key::K => Some(0x28),
        Key::L => Some(0x25),
        Key::M => Some(0x2E),
        Key::N => Some(0x2D),
        Key::O => Some(0x1F),
        Key::P => Some(0x23),
        Key::Q => Some(0x0C),
        Key::R => Some(0x0F),
        Key::S => Some(0x01),
        Key::T => Some(0x11),
        Key::U => Some(0x20),
        Key::V => Some(0x09),
        Key::W => Some(0x0D),
        Key::X => Some(0x07),
        Key::Y => Some(0x10),
        Key::Z => Some(0x06),

        // Numbers (top row)
        Key::Num0 => Some(0x1D),
        Key::Num1 => Some(0x12),
        Key::Num2 => Some(0x13),
        Key::Num3 => Some(0x14),
        Key::Num4 => Some(0x15),
        Key::Num5 => Some(0x17),
        Key::Num6 => Some(0x16),
        Key::Num7 => Some(0x1A),
        Key::Num8 => Some(0x1C),
        Key::Num9 => Some(0x19),

        // Function keys
        Key::F1 => Some(0x7A),
        Key::F2 => Some(0x78),
        Key::F3 => Some(0x63),
        Key::F4 => Some(0x76),
        Key::F5 => Some(0x60),
        Key::F6 => Some(0x61),
        Key::F7 => Some(0x62),
        Key::F8 => Some(0x64),
        Key::F9 => Some(0x65),
        Key::F10 => Some(0x6D),
        Key::F11 => Some(0x67),
        Key::F12 => Some(0x6F),
        Key::F13 => Some(0x69),
        Key::F14 => Some(0x6B),
        Key::F15 => Some(0x71),
        Key::F16 => Some(0x6A),
        Key::F17 => Some(0x40),
        Key::F18 => Some(0x4F),
        Key::F19 => Some(0x50),
        Key::F20 => Some(0x5A),

        // Whitespace / control
        Key::Space => Some(0x31),
        Key::Return => Some(0x24),
        Key::Escape => Some(0x35),
        Key::Tab => Some(0x30),
        Key::Backspace => Some(0x33),
        Key::Delete => Some(0x75), // Forward delete

        // Navigation
        Key::Up => Some(0x7E),
        Key::Down => Some(0x7D),
        Key::Left => Some(0x7B),
        Key::Right => Some(0x7C),
        Key::Home => Some(0x73),
        Key::End => Some(0x77),
        Key::PageUp => Some(0x74),
        Key::PageDown => Some(0x79),

        // Punctuation / symbols (ANSI layout)
        Key::Grave => Some(0x32),
        Key::Minus => Some(0x1B),
        Key::Equal => Some(0x18),
        Key::LeftBracket => Some(0x21),
        Key::RightBracket => Some(0x1E),
        Key::Backslash => Some(0x2A),
        Key::Semicolon => Some(0x29),
        Key::Quote => Some(0x27),
        Key::Comma => Some(0x2B),
        Key::Period => Some(0x2F),
        Key::Slash => Some(0x2C),

        // Numpad
        Key::Numpad0 => Some(0x52),
        Key::Numpad1 => Some(0x53),
        Key::Numpad2 => Some(0x54),
        Key::Numpad3 => Some(0x55),
        Key::Numpad4 => Some(0x56),
        Key::Numpad5 => Some(0x57),
        Key::Numpad6 => Some(0x58),
        Key::Numpad7 => Some(0x59),
        Key::Numpad8 => Some(0x5B),
        Key::Numpad9 => Some(0x5C),
        Key::NumpadAdd => Some(0x45),
        Key::NumpadSubtract => Some(0x4E),
        Key::NumpadMultiply => Some(0x43),
        Key::NumpadDivide => Some(0x4B),
        Key::NumpadDecimal => Some(0x41),
        Key::NumpadEnter => Some(0x4C),

        // Media / special
        Key::VolumeUp => Some(0x48),
        Key::VolumeDown => Some(0x49),
        Key::Mute => Some(0x4A),
        Key::CapsLock => Some(0x39),
        Key::Insert => Some(0x72), // Help/Insert on Mac

        // Keys without standard macOS keycodes
        Key::BrightnessUp
        | Key::BrightnessDown
        | Key::PlayPause
        | Key::NextTrack
        | Key::PreviousTrack
        | Key::PrintScreen
        | Key::Pause
        | Key::NumLock
        | Key::ScrollLock
        | Key::MouseLeft
        | Key::MouseRight
        | Key::MouseMiddle
        | Key::MouseButton4
        | Key::MouseButton5 => None,
    }
}

/// Map a macOS virtual keycode to an awase `Key`.
///
/// Returns `None` for keycodes that don't map to any awase key
/// (e.g. modifier-only keycodes like 0x37 for Command).
#[must_use]
pub fn keycode_to_key(keycode: u16) -> Option<Key> {
    match keycode {
        // Letters
        0x00 => Some(Key::A),
        0x0B => Some(Key::B),
        0x08 => Some(Key::C),
        0x02 => Some(Key::D),
        0x0E => Some(Key::E),
        0x03 => Some(Key::F),
        0x05 => Some(Key::G),
        0x04 => Some(Key::H),
        0x22 => Some(Key::I),
        0x26 => Some(Key::J),
        0x28 => Some(Key::K),
        0x25 => Some(Key::L),
        0x2E => Some(Key::M),
        0x2D => Some(Key::N),
        0x1F => Some(Key::O),
        0x23 => Some(Key::P),
        0x0C => Some(Key::Q),
        0x0F => Some(Key::R),
        0x01 => Some(Key::S),
        0x11 => Some(Key::T),
        0x20 => Some(Key::U),
        0x09 => Some(Key::V),
        0x0D => Some(Key::W),
        0x07 => Some(Key::X),
        0x10 => Some(Key::Y),
        0x06 => Some(Key::Z),

        // Numbers
        0x1D => Some(Key::Num0),
        0x12 => Some(Key::Num1),
        0x13 => Some(Key::Num2),
        0x14 => Some(Key::Num3),
        0x15 => Some(Key::Num4),
        0x17 => Some(Key::Num5),
        0x16 => Some(Key::Num6),
        0x1A => Some(Key::Num7),
        0x1C => Some(Key::Num8),
        0x19 => Some(Key::Num9),

        // Function keys
        0x7A => Some(Key::F1),
        0x78 => Some(Key::F2),
        0x63 => Some(Key::F3),
        0x76 => Some(Key::F4),
        0x60 => Some(Key::F5),
        0x61 => Some(Key::F6),
        0x62 => Some(Key::F7),
        0x64 => Some(Key::F8),
        0x65 => Some(Key::F9),
        0x6D => Some(Key::F10),
        0x67 => Some(Key::F11),
        0x6F => Some(Key::F12),
        0x69 => Some(Key::F13),
        0x6B => Some(Key::F14),
        0x71 => Some(Key::F15),
        0x6A => Some(Key::F16),
        0x40 => Some(Key::F17),
        0x4F => Some(Key::F18),
        0x50 => Some(Key::F19),
        0x5A => Some(Key::F20),

        // Whitespace / control
        0x31 => Some(Key::Space),
        0x24 => Some(Key::Return),
        0x35 => Some(Key::Escape),
        0x30 => Some(Key::Tab),
        0x33 => Some(Key::Backspace),
        0x75 => Some(Key::Delete),

        // Navigation
        0x7E => Some(Key::Up),
        0x7D => Some(Key::Down),
        0x7B => Some(Key::Left),
        0x7C => Some(Key::Right),
        0x73 => Some(Key::Home),
        0x77 => Some(Key::End),
        0x74 => Some(Key::PageUp),
        0x79 => Some(Key::PageDown),

        // Punctuation
        0x32 => Some(Key::Grave),
        0x1B => Some(Key::Minus),
        0x18 => Some(Key::Equal),
        0x21 => Some(Key::LeftBracket),
        0x1E => Some(Key::RightBracket),
        0x2A => Some(Key::Backslash),
        0x29 => Some(Key::Semicolon),
        0x27 => Some(Key::Quote),
        0x2B => Some(Key::Comma),
        0x2F => Some(Key::Period),
        0x2C => Some(Key::Slash),

        // Numpad
        0x52 => Some(Key::Numpad0),
        0x53 => Some(Key::Numpad1),
        0x54 => Some(Key::Numpad2),
        0x55 => Some(Key::Numpad3),
        0x56 => Some(Key::Numpad4),
        0x57 => Some(Key::Numpad5),
        0x58 => Some(Key::Numpad6),
        0x59 => Some(Key::Numpad7),
        0x5B => Some(Key::Numpad8),
        0x5C => Some(Key::Numpad9),
        0x45 => Some(Key::NumpadAdd),
        0x4E => Some(Key::NumpadSubtract),
        0x43 => Some(Key::NumpadMultiply),
        0x4B => Some(Key::NumpadDivide),
        0x41 => Some(Key::NumpadDecimal),
        0x4C => Some(Key::NumpadEnter),

        // Media / special
        0x48 => Some(Key::VolumeUp),
        0x49 => Some(Key::VolumeDown),
        0x4A => Some(Key::Mute),
        0x39 => Some(Key::CapsLock),
        0x72 => Some(Key::Insert),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_all_mapped_keys() {
        // Every key that maps to a keycode should roundtrip back
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
            Key::CapsLock, Key::Insert,
        ];

        for key in all_keys {
            let keycode = key_to_keycode(key);
            assert!(keycode.is_some(), "key {key:?} should have a keycode");
            let back = keycode_to_key(keycode.unwrap());
            assert_eq!(back, Some(key), "roundtrip failed for {key:?} (keycode 0x{:02X})", keycode.unwrap());
        }
    }

    #[test]
    fn unmapped_keys_return_none() {
        assert!(key_to_keycode(Key::MouseLeft).is_none());
        assert!(key_to_keycode(Key::MouseRight).is_none());
        assert!(key_to_keycode(Key::BrightnessUp).is_none());
        assert!(key_to_keycode(Key::PlayPause).is_none());
    }

    #[test]
    fn unknown_keycode_returns_none() {
        assert!(keycode_to_key(0xFF).is_none());
        assert!(keycode_to_key(0x37).is_none()); // Command key (modifier only)
    }

    #[test]
    fn specific_keycodes() {
        assert_eq!(key_to_keycode(Key::H), Some(0x04));
        assert_eq!(key_to_keycode(Key::Space), Some(0x31));
        assert_eq!(key_to_keycode(Key::Return), Some(0x24));
        assert_eq!(key_to_keycode(Key::Escape), Some(0x35));
        assert_eq!(keycode_to_key(0x04), Some(Key::H));
    }

    // ── Additional keycode tests ────────────────────────────────────

    #[test]
    fn reverse_roundtrip_all_known_keycodes() {
        // Every keycode that maps to a key should roundtrip back
        for code in 0..=0xFF_u16 {
            if let Some(key) = keycode_to_key(code) {
                let back = key_to_keycode(key);
                assert_eq!(
                    back,
                    Some(code),
                    "reverse roundtrip failed: keycode 0x{code:02X} -> {key:?} -> {back:?}"
                );
            }
        }
    }

    #[test]
    fn all_unmapped_keys() {
        // These keys should NOT have macOS keycodes
        let unmapped = [
            Key::BrightnessUp, Key::BrightnessDown,
            Key::PlayPause, Key::NextTrack, Key::PreviousTrack,
            Key::PrintScreen, Key::Pause,
            Key::NumLock, Key::ScrollLock,
            Key::MouseLeft, Key::MouseRight, Key::MouseMiddle,
            Key::MouseButton4, Key::MouseButton5,
        ];
        for key in unmapped {
            assert!(key_to_keycode(key).is_none(), "{key:?} should not have a keycode");
        }
    }

    #[test]
    fn qwerty_home_row_keycodes() {
        // Verify home row keys have correct ANSI keycodes
        assert_eq!(key_to_keycode(Key::A), Some(0x00));
        assert_eq!(key_to_keycode(Key::S), Some(0x01));
        assert_eq!(key_to_keycode(Key::D), Some(0x02));
        assert_eq!(key_to_keycode(Key::F), Some(0x03));
        assert_eq!(key_to_keycode(Key::J), Some(0x26));
        assert_eq!(key_to_keycode(Key::K), Some(0x28));
        assert_eq!(key_to_keycode(Key::L), Some(0x25));
    }

    #[test]
    fn numpad_keycodes_distinct_from_main() {
        // Numpad keys should have different keycodes than main number keys
        for i in 0..=9 {
            let main_keys = [
                Key::Num0, Key::Num1, Key::Num2, Key::Num3, Key::Num4,
                Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9,
            ];
            let numpad_keys = [
                Key::Numpad0, Key::Numpad1, Key::Numpad2, Key::Numpad3, Key::Numpad4,
                Key::Numpad5, Key::Numpad6, Key::Numpad7, Key::Numpad8, Key::Numpad9,
            ];
            let main_code = key_to_keycode(main_keys[i]).unwrap();
            let numpad_code = key_to_keycode(numpad_keys[i]).unwrap();
            assert_ne!(
                main_code, numpad_code,
                "Num{i} and Numpad{i} should have different keycodes"
            );
        }
    }

    #[test]
    fn modifier_only_keycodes_not_mapped() {
        // macOS modifier-only keycodes should return None
        // Command: 0x37 (left), 0x36 (right)
        assert!(keycode_to_key(0x37).is_none());
        assert!(keycode_to_key(0x36).is_none());
        // Shift: 0x38 (left), 0x3C (right)
        assert!(keycode_to_key(0x38).is_none());
        assert!(keycode_to_key(0x3C).is_none());
        // Option: 0x3A (left), 0x3D (right)
        assert!(keycode_to_key(0x3A).is_none());
        assert!(keycode_to_key(0x3D).is_none());
        // Control: 0x3B (left), 0x3E (right)
        assert!(keycode_to_key(0x3B).is_none());
        assert!(keycode_to_key(0x3E).is_none());
    }

    #[test]
    fn function_key_keycodes() {
        // Verify some well-known function key keycodes
        assert_eq!(key_to_keycode(Key::F1), Some(0x7A));
        assert_eq!(key_to_keycode(Key::F12), Some(0x6F));
        assert_eq!(key_to_keycode(Key::F13), Some(0x69));
    }

    #[test]
    fn navigation_keycodes() {
        assert_eq!(key_to_keycode(Key::Up), Some(0x7E));
        assert_eq!(key_to_keycode(Key::Down), Some(0x7D));
        assert_eq!(key_to_keycode(Key::Left), Some(0x7B));
        assert_eq!(key_to_keycode(Key::Right), Some(0x7C));
        assert_eq!(key_to_keycode(Key::Home), Some(0x73));
        assert_eq!(key_to_keycode(Key::End), Some(0x77));
        assert_eq!(key_to_keycode(Key::PageUp), Some(0x74));
        assert_eq!(key_to_keycode(Key::PageDown), Some(0x79));
    }
}
