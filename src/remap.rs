use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::hotkey::Hotkey;

/// A key remapping: transforms one key event into another before binding
/// matching. Applied at the CGEventTap/input level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyRemap {
    /// The source key event to remap from.
    pub from: Hotkey,
    /// The target key event to remap to.
    pub to: Hotkey,
    /// Optional conditions for when this remap is active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,
}

impl KeyRemap {
    /// Create a simple unconditional remap.
    #[must_use]
    pub fn new(from: Hotkey, to: Hotkey) -> Self {
        Self {
            from,
            to,
            condition: None,
        }
    }

    /// Builder: set condition.
    #[must_use]
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition = Some(condition);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hotkey::{Key, Modifiers};

    #[test]
    fn new_remap() {
        let remap = KeyRemap::new(
            Hotkey::new(Modifiers::NONE, Key::CapsLock),
            Hotkey::new(Modifiers::NONE, Key::Escape),
        );
        assert!(remap.condition.is_none());
    }

    #[test]
    fn remap_with_condition() {
        let remap = KeyRemap::new(
            Hotkey::new(Modifiers::FN, Key::H),
            Hotkey::new(Modifiers::NONE, Key::Left),
        )
        .with_condition(Condition {
            app: Some("Terminal".to_string()),
            ..Default::default()
        });
        assert!(remap.condition.is_some());
    }

    #[test]
    fn serde_roundtrip() {
        let remap = KeyRemap::new(
            Hotkey::parse("caps_lock").unwrap(),
            Hotkey::parse("escape").unwrap(),
        );
        let json = serde_json::to_string(&remap).unwrap();
        let deserialized: KeyRemap = serde_json::from_str(&json).unwrap();
        assert_eq!(remap, deserialized);
    }

    // ── Additional remap tests ──────────────────────────────────────

    #[test]
    fn remap_equality() {
        let a = KeyRemap::new(
            Hotkey::new(Modifiers::NONE, Key::CapsLock),
            Hotkey::new(Modifiers::NONE, Key::Escape),
        );
        let b = KeyRemap::new(
            Hotkey::new(Modifiers::NONE, Key::CapsLock),
            Hotkey::new(Modifiers::NONE, Key::Escape),
        );
        assert_eq!(a, b);
    }

    #[test]
    fn remap_inequality_different_from() {
        let a = KeyRemap::new(
            Hotkey::new(Modifiers::NONE, Key::CapsLock),
            Hotkey::new(Modifiers::NONE, Key::Escape),
        );
        let b = KeyRemap::new(
            Hotkey::new(Modifiers::NONE, Key::Tab),
            Hotkey::new(Modifiers::NONE, Key::Escape),
        );
        assert_ne!(a, b);
    }

    #[test]
    fn remap_inequality_different_to() {
        let a = KeyRemap::new(
            Hotkey::new(Modifiers::NONE, Key::CapsLock),
            Hotkey::new(Modifiers::NONE, Key::Escape),
        );
        let b = KeyRemap::new(
            Hotkey::new(Modifiers::NONE, Key::CapsLock),
            Hotkey::new(Modifiers::NONE, Key::Tab),
        );
        assert_ne!(a, b);
    }

    #[test]
    fn remap_clone() {
        let original = KeyRemap::new(
            Hotkey::new(Modifiers::FN, Key::H),
            Hotkey::new(Modifiers::NONE, Key::Left),
        )
        .with_condition(Condition {
            app: Some("Terminal".to_string()),
            ..Default::default()
        });
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn serde_roundtrip_with_condition() {
        let remap = KeyRemap::new(
            Hotkey::new(Modifiers::FN, Key::H),
            Hotkey::new(Modifiers::NONE, Key::Left),
        )
        .with_condition(Condition {
            app: Some("Terminal".to_string()),
            app_exclude: Some("iTerm".to_string()),
            title: None,
            display: Some(0),
        });
        let json = serde_json::to_string(&remap).unwrap();
        let deserialized: KeyRemap = serde_json::from_str(&json).unwrap();
        assert_eq!(remap, deserialized);
    }

    #[test]
    fn serde_condition_none_skipped() {
        let remap = KeyRemap::new(
            Hotkey::new(Modifiers::NONE, Key::CapsLock),
            Hotkey::new(Modifiers::NONE, Key::Escape),
        );
        let json = serde_json::to_string(&remap).unwrap();
        assert!(!json.contains("condition"));
    }

    #[test]
    fn remap_with_modifiers() {
        let remap = KeyRemap::new(
            Hotkey::parse("fn+h").unwrap(),
            Hotkey::parse("left").unwrap(),
        );
        assert_eq!(remap.from.modifiers, Modifiers::FN);
        assert_eq!(remap.from.key, Key::H);
        assert!(remap.to.modifiers.is_empty());
        assert_eq!(remap.to.key, Key::Left);
    }

    #[test]
    fn remap_arrow_keys_via_fn() {
        // Common macOS remap: fn+hjkl -> arrow keys
        let remaps = vec![
            KeyRemap::new(Hotkey::parse("fn+h").unwrap(), Hotkey::parse("left").unwrap()),
            KeyRemap::new(Hotkey::parse("fn+j").unwrap(), Hotkey::parse("down").unwrap()),
            KeyRemap::new(Hotkey::parse("fn+k").unwrap(), Hotkey::parse("up").unwrap()),
            KeyRemap::new(Hotkey::parse("fn+l").unwrap(), Hotkey::parse("right").unwrap()),
        ];

        assert_eq!(remaps[0].to.key, Key::Left);
        assert_eq!(remaps[1].to.key, Key::Down);
        assert_eq!(remaps[2].to.key, Key::Up);
        assert_eq!(remaps[3].to.key, Key::Right);
    }
}
