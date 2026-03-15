use serde::{Deserialize, Serialize};

/// An action to perform when a hotkey is triggered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Named command string (consumer interprets).
    Command(String),
    /// Switch to a different keybinding mode.
    ModeSwitch(String),
    /// Execute shell command.
    Exec(String),
    /// Rhai script evaluation.
    Script(String),
    /// Multiple actions in sequence.
    Chain(Vec<Action>),
}

impl Action {
    /// Create a command action.
    #[must_use]
    pub fn command(name: impl Into<String>) -> Self {
        Self::Command(name.into())
    }

    /// Create a mode switch action.
    #[must_use]
    pub fn mode_switch(mode: impl Into<String>) -> Self {
        Self::ModeSwitch(mode.into())
    }

    /// Create a shell exec action.
    #[must_use]
    pub fn exec(cmd: impl Into<String>) -> Self {
        Self::Exec(cmd.into())
    }

    /// Create a script action.
    #[must_use]
    pub fn script(code: impl Into<String>) -> Self {
        Self::Script(code.into())
    }

    /// Create a chain of actions.
    #[must_use]
    pub fn chain(actions: Vec<Action>) -> Self {
        Self::Chain(actions)
    }

    /// Returns `true` if this is a mode switch action.
    #[must_use]
    pub fn is_mode_switch(&self) -> bool {
        matches!(self, Self::ModeSwitch(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_action() {
        let a = Action::command("window_focus_west");
        assert_eq!(a, Action::Command("window_focus_west".to_string()));
    }

    #[test]
    fn mode_switch_action() {
        let a = Action::mode_switch("resize");
        assert!(a.is_mode_switch());
        assert_eq!(a, Action::ModeSwitch("resize".to_string()));
    }

    #[test]
    fn exec_action() {
        let a = Action::exec("open -a Terminal");
        assert_eq!(a, Action::Exec("open -a Terminal".to_string()));
    }

    #[test]
    fn script_action() {
        let a = Action::script("focus_window(\"west\")");
        assert!(!a.is_mode_switch());
    }

    #[test]
    fn chain_action() {
        let a = Action::chain(vec![
            Action::command("window_focus_west"),
            Action::mode_switch("default"),
        ]);
        match &a {
            Action::Chain(actions) => assert_eq!(actions.len(), 2),
            _ => panic!("expected Chain"),
        }
    }

    #[test]
    fn serde_roundtrip() {
        let actions = vec![
            Action::command("test"),
            Action::mode_switch("resize"),
            Action::exec("echo hello"),
            Action::script("1 + 2"),
            Action::chain(vec![Action::command("a"), Action::command("b")]),
        ];

        for action in &actions {
            let json = serde_json::to_string(action).unwrap();
            let deserialized: Action = serde_json::from_str(&json).unwrap();
            assert_eq!(action, &deserialized);
        }
    }

    // ── Additional action tests ─────────────────────────────────────

    #[test]
    fn is_mode_switch_false_for_other_variants() {
        assert!(!Action::command("test").is_mode_switch());
        assert!(!Action::exec("echo").is_mode_switch());
        assert!(!Action::script("1+2").is_mode_switch());
        assert!(!Action::chain(vec![]).is_mode_switch());
    }

    #[test]
    fn chain_with_nested_chain() {
        let inner = Action::chain(vec![Action::command("a"), Action::command("b")]);
        let outer = Action::chain(vec![inner.clone(), Action::exec("echo done")]);
        match &outer {
            Action::Chain(actions) => {
                assert_eq!(actions.len(), 2);
                assert_eq!(actions[0], inner);
            }
            _ => panic!("expected Chain"),
        }
    }

    #[test]
    fn chain_empty() {
        let a = Action::chain(vec![]);
        match &a {
            Action::Chain(actions) => assert!(actions.is_empty()),
            _ => panic!("expected Chain"),
        }
    }

    #[test]
    fn chain_single_element() {
        let a = Action::chain(vec![Action::command("only")]);
        match &a {
            Action::Chain(actions) => assert_eq!(actions.len(), 1),
            _ => panic!("expected Chain"),
        }
    }

    #[test]
    fn action_equality() {
        assert_eq!(Action::command("a"), Action::command("a"));
        assert_ne!(Action::command("a"), Action::command("b"));
        assert_ne!(Action::command("a"), Action::exec("a"));
    }

    #[test]
    fn action_clone() {
        let original = Action::chain(vec![
            Action::command("focus"),
            Action::mode_switch("default"),
        ]);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn action_debug_format() {
        let a = Action::command("test");
        let debug = format!("{a:?}");
        assert!(debug.contains("Command"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn serde_command_json_structure() {
        let a = Action::command("test");
        let json = serde_json::to_string(&a).unwrap();
        // Should serialize as a tagged enum
        assert!(json.contains("Command"));
    }

    #[test]
    fn serde_mode_switch_json_structure() {
        let a = Action::mode_switch("resize");
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("ModeSwitch"));
    }

    #[test]
    fn command_accepts_string() {
        let a = Action::command(String::from("test"));
        assert_eq!(a, Action::Command("test".to_string()));
    }

    #[test]
    fn exec_accepts_string() {
        let a = Action::exec(String::from("echo hello"));
        assert_eq!(a, Action::Exec("echo hello".to_string()));
    }
}
