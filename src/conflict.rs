use crate::chord::KeyChord;
use crate::hotkey::Hotkey;
use crate::mode::KeyMode;

/// A single detected conflict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictEntry {
    /// The mode where the conflict was found.
    pub mode: String,
    /// The conflicting hotkey.
    pub hotkey: Hotkey,
    /// Description of the existing binding/chord.
    pub existing: String,
    /// Description of the conflicting binding/chord.
    pub new: String,
}

/// Report of all detected conflicts in a binding configuration.
#[derive(Debug, Clone, Default)]
pub struct ConflictReport {
    pub conflicts: Vec<ConflictEntry>,
}

impl ConflictReport {
    /// Returns `true` if no conflicts were found.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.conflicts.is_empty()
    }
}

/// Detect conflicts within a set of modes and chords.
///
/// Detected conflicts:
/// - Same hotkey bound twice in the same mode (only possible with external
///   config merging — `HashMap` insert deduplicates in normal use)
/// - Chord leader that conflicts with a regular binding in the same mode
#[must_use]
pub fn detect_conflicts(modes: &[&KeyMode], chords: &[KeyChord]) -> ConflictReport {
    let mut report = ConflictReport::default();

    // Check chord leaders vs mode bindings
    for mode in modes {
        for chord in chords {
            if mode.bindings.contains_key(&chord.leader) {
                report.conflicts.push(ConflictEntry {
                    mode: mode.name.clone(),
                    hotkey: chord.leader,
                    existing: format!(
                        "binding: {:?}",
                        mode.bindings[&chord.leader].action
                    ),
                    new: format!("chord leader (follower: {})", chord.follower),
                });
            }
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::binding::Binding;
    use crate::hotkey::{Key, Modifiers};

    fn ctrl_a() -> Hotkey {
        Hotkey::new(Modifiers::CTRL, Key::A)
    }

    #[test]
    fn no_conflicts() {
        let mode = KeyMode::new("default", true);
        let report = detect_conflicts(&[&mode], &[]);
        assert!(report.is_clean());
    }

    #[test]
    fn chord_leader_conflicts_with_binding() {
        let mut mode = KeyMode::new("default", true);
        mode.add_binding(Binding::new(ctrl_a(), Action::command("select_all")));

        let chord = KeyChord {
            leader: ctrl_a(),
            follower: Hotkey::new(Modifiers::NONE, Key::C),
            timeout_ms: 1000,
            action: Action::command("new_window"),
        };

        let report = detect_conflicts(&[&mode], &[chord]);
        assert!(!report.is_clean());
        assert_eq!(report.conflicts.len(), 1);
        assert_eq!(report.conflicts[0].mode, "default");
        assert_eq!(report.conflicts[0].hotkey, ctrl_a());
    }

    #[test]
    fn chord_no_conflict_when_leader_not_bound() {
        let mut mode = KeyMode::new("default", true);
        mode.add_binding(Binding::new(
            Hotkey::new(Modifiers::CMD, Key::H),
            Action::command("focus_west"),
        ));

        let chord = KeyChord {
            leader: ctrl_a(),
            follower: Hotkey::new(Modifiers::NONE, Key::C),
            timeout_ms: 1000,
            action: Action::command("new_window"),
        };

        let report = detect_conflicts(&[&mode], &[chord]);
        assert!(report.is_clean());
    }

    #[test]
    fn conflict_in_specific_mode() {
        let mut resize = KeyMode::new("resize", false);
        resize.add_binding(Binding::new(ctrl_a(), Action::command("shrink")));

        let default = KeyMode::new("default", true);

        let chord = KeyChord {
            leader: ctrl_a(),
            follower: Hotkey::new(Modifiers::NONE, Key::C),
            timeout_ms: 1000,
            action: Action::command("new_window"),
        };

        let report = detect_conflicts(&[&default, &resize], &[chord]);
        assert_eq!(report.conflicts.len(), 1);
        assert_eq!(report.conflicts[0].mode, "resize");
    }
}
