use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::condition::{Condition, MatchContext};
use crate::hotkey::Hotkey;

/// A complete keybinding: hotkey + action + consume flag + optional conditions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    /// The hotkey that triggers this binding.
    pub hotkey: Hotkey,
    /// The action to perform.
    pub action: Action,
    /// Whether to consume the key event (not pass to the focused app).
    /// Default: `true`.
    #[serde(default = "default_consume")]
    pub consume: bool,
    /// Optional conditions for when this binding is active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,
}

fn default_consume() -> bool {
    true
}

impl Binding {
    /// Create a simple binding with default consume=true and no conditions.
    #[must_use]
    pub fn new(hotkey: Hotkey, action: Action) -> Self {
        Self {
            hotkey,
            action,
            consume: true,
            condition: None,
        }
    }

    /// Builder: set consume flag.
    #[must_use]
    pub fn with_consume(mut self, consume: bool) -> Self {
        self.consume = consume;
        self
    }

    /// Builder: set condition.
    #[must_use]
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Returns `true` if this binding's conditions match the given context.
    ///
    /// A binding with no condition always matches.
    #[must_use]
    pub fn matches_context(&self, ctx: &MatchContext) -> bool {
        match &self.condition {
            Some(c) => c.matches(ctx),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hotkey::{Key, Modifiers};

    fn test_hotkey() -> Hotkey {
        Hotkey::new(Modifiers::CMD, Key::H)
    }

    #[test]
    fn new_binding_defaults() {
        let b = Binding::new(test_hotkey(), Action::command("focus_west"));
        assert!(b.consume);
        assert!(b.condition.is_none());
    }

    #[test]
    fn builder_consume() {
        let b = Binding::new(test_hotkey(), Action::command("focus_west"))
            .with_consume(false);
        assert!(!b.consume);
    }

    #[test]
    fn builder_condition() {
        let c = Condition {
            app_exclude: Some("com.apple.Terminal".to_string()),
            ..Default::default()
        };
        let b = Binding::new(test_hotkey(), Action::command("focus_west"))
            .with_condition(c.clone());
        assert_eq!(b.condition, Some(c));
    }

    #[test]
    fn matches_context_no_condition() {
        let b = Binding::new(test_hotkey(), Action::command("test"));
        let ctx = MatchContext::default();
        assert!(b.matches_context(&ctx));
    }

    #[test]
    fn matches_context_with_condition() {
        let b = Binding::new(test_hotkey(), Action::command("test"))
            .with_condition(Condition {
                app: Some("Safari".to_string()),
                ..Default::default()
            });

        let ctx_match = MatchContext {
            focused_app_bundle_id: Some("com.apple.Safari".to_string()),
            ..Default::default()
        };
        assert!(b.matches_context(&ctx_match));

        let ctx_no_match = MatchContext {
            focused_app_bundle_id: Some("com.mitchellh.ghostty".to_string()),
            ..Default::default()
        };
        assert!(!b.matches_context(&ctx_no_match));
    }

    #[test]
    fn serde_roundtrip() {
        let b = Binding::new(
            Hotkey::parse("cmd+shift+h").unwrap(),
            Action::chain(vec![
                Action::command("focus_west"),
                Action::mode_switch("default"),
            ]),
        )
        .with_consume(false)
        .with_condition(Condition {
            app_exclude: Some("Terminal|ghostty".to_string()),
            display: Some(0),
            ..Default::default()
        });

        let json = serde_json::to_string_pretty(&b).unwrap();
        let deserialized: Binding = serde_json::from_str(&json).unwrap();
        assert_eq!(b, deserialized);
    }

    #[test]
    fn serde_minimal() {
        let json = r#"{
            "hotkey": { "modifiers": 1, "key": "Space" },
            "action": { "Command": "test" }
        }"#;
        let b: Binding = serde_json::from_str(json).unwrap();
        assert!(b.consume); // default true
        assert!(b.condition.is_none());
    }
}
