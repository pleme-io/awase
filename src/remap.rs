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
}
