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
}
