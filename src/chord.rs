use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::hotkey::Hotkey;

/// A two-step key chord: leader key activates chord mode, then a follower
/// key within the timeout triggers an action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyChord {
    /// The leader hotkey that activates chord mode.
    pub leader: Hotkey,
    /// The follower hotkey that completes the chord.
    pub follower: Hotkey,
    /// Timeout in milliseconds after the leader key before the chord is cancelled.
    pub timeout_ms: u32,
    /// The action to perform when the chord is completed.
    pub action: Action,
}

/// Internal state machine for tracking chord input.
#[derive(Debug)]
pub enum ChordState {
    /// No chord in progress.
    Idle,
    /// Leader key was pressed, waiting for a follower key.
    Pending {
        leader: Hotkey,
        started: Instant,
        timeout_ms: u32,
    },
}

impl Default for ChordState {
    fn default() -> Self {
        Self::Idle
    }
}

impl ChordState {
    /// Returns `true` if we're waiting for a follower key.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    /// Returns `true` if a pending chord has timed out.
    #[must_use]
    pub fn is_timed_out(&self) -> bool {
        match self {
            Self::Idle => false,
            Self::Pending {
                started,
                timeout_ms,
                ..
            } => started.elapsed().as_millis() >= u128::from(*timeout_ms),
        }
    }

    /// Enter pending state for a leader key.
    pub fn begin(&mut self, leader: Hotkey, timeout_ms: u32) {
        *self = Self::Pending {
            leader,
            started: Instant::now(),
            timeout_ms,
        };
    }

    /// Reset to idle state.
    pub fn reset(&mut self) {
        *self = Self::Idle;
    }

    /// Returns the pending leader key, if any.
    #[must_use]
    pub fn pending_leader(&self) -> Option<&Hotkey> {
        match self {
            Self::Idle => None,
            Self::Pending { leader, .. } => Some(leader),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hotkey::{Key, Modifiers};

    fn leader() -> Hotkey {
        Hotkey::new(Modifiers::CTRL, Key::A)
    }

    fn follower() -> Hotkey {
        Hotkey::new(Modifiers::NONE, Key::C)
    }

    #[test]
    fn chord_state_default_idle() {
        let state = ChordState::default();
        assert!(!state.is_pending());
        assert!(state.pending_leader().is_none());
    }

    #[test]
    fn chord_state_begin_pending() {
        let mut state = ChordState::default();
        state.begin(leader(), 1000);
        assert!(state.is_pending());
        assert_eq!(state.pending_leader(), Some(&leader()));
    }

    #[test]
    fn chord_state_reset() {
        let mut state = ChordState::default();
        state.begin(leader(), 1000);
        state.reset();
        assert!(!state.is_pending());
    }

    #[test]
    fn chord_state_not_timed_out_initially() {
        let mut state = ChordState::default();
        state.begin(leader(), 5000); // 5 second timeout
        assert!(!state.is_timed_out());
    }

    #[test]
    fn chord_state_times_out() {
        let mut state = ChordState::default();
        state.begin(leader(), 0); // 0ms timeout = immediate
        // A 0ms timeout should be considered timed out immediately
        // (or very nearly so — depends on timing)
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(state.is_timed_out());
    }

    #[test]
    fn idle_never_timed_out() {
        let state = ChordState::default();
        assert!(!state.is_timed_out());
    }

    #[test]
    fn key_chord_serde() {
        let chord = KeyChord {
            leader: leader(),
            follower: follower(),
            timeout_ms: 1000,
            action: Action::command("new_window"),
        };
        let json = serde_json::to_string(&chord).unwrap();
        let deserialized: KeyChord = serde_json::from_str(&json).unwrap();
        assert_eq!(chord, deserialized);
    }
}
