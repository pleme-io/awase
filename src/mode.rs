use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::binding::Binding;
use crate::chord::KeyChord;
use crate::condition::MatchContext;
use crate::hotkey::Hotkey;

/// A named keybinding mode with independent binding sets.
///
/// Inspired by skhd's mode system. Modes allow different sets of
/// keybindings to be active at different times (e.g. "default" vs "resize").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMode {
    /// Mode name (e.g. "default", "resize").
    pub name: String,
    /// Bindings indexed by hotkey for fast lookup.
    pub bindings: HashMap<Hotkey, Binding>,
    /// When `true`, unmatched keys pass through to the focused app.
    /// When `false`, all keys are consumed (modal, like vim normal mode).
    #[serde(default = "default_passthrough")]
    pub passthrough: bool,
}

fn default_passthrough() -> bool {
    true
}

impl KeyMode {
    /// Create a new empty mode.
    #[must_use]
    pub fn new(name: impl Into<String>, passthrough: bool) -> Self {
        Self {
            name: name.into(),
            bindings: HashMap::new(),
            passthrough,
        }
    }

    /// Add a binding to this mode. Returns the previous binding if one
    /// existed for the same hotkey.
    pub fn add_binding(&mut self, binding: Binding) -> Option<Binding> {
        self.bindings.insert(binding.hotkey, binding)
    }

    /// Look up a binding for the given hotkey, filtering by context.
    #[must_use]
    pub fn find_binding(&self, hotkey: &Hotkey, ctx: &MatchContext) -> Option<&Binding> {
        self.bindings
            .get(hotkey)
            .filter(|b| b.matches_context(ctx))
    }
}

/// Result of matching a key event against the binding map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchResult {
    /// A binding matched — perform the action.
    Matched { action: Action, consume: bool },
    /// A chord leader was pressed — waiting for follower.
    ChordPending { leader: Hotkey, timeout_ms: u32 },
    /// A remap was applied — the key should be re-emitted as a different key.
    Remapped { to: Hotkey },
    /// No binding matched.
    NoMatch,
}

/// The main binding map: mode-aware lookup with chord and remap support.
#[derive(Debug)]
pub struct BindingMap {
    modes: HashMap<String, KeyMode>,
    chords: Vec<KeyChord>,
    remaps: Vec<crate::remap::KeyRemap>,
    current_mode: String,
    chord_state: crate::chord::ChordState,
}

impl BindingMap {
    /// Create a new binding map with a default mode.
    #[must_use]
    pub fn new() -> Self {
        let mut modes = HashMap::new();
        modes.insert(
            "default".to_string(),
            KeyMode::new("default", true),
        );
        Self {
            modes,
            chords: Vec::new(),
            remaps: Vec::new(),
            current_mode: "default".to_string(),
            chord_state: crate::chord::ChordState::default(),
        }
    }

    /// Get the current mode name.
    #[must_use]
    pub fn current_mode(&self) -> &str {
        &self.current_mode
    }

    /// Switch to a different mode.
    pub fn set_mode(&mut self, mode: &str) -> Result<(), crate::AwaseError> {
        if self.modes.contains_key(mode) {
            self.current_mode = mode.to_string();
            self.chord_state.reset();
            Ok(())
        } else {
            Err(crate::AwaseError::ModeNotFound(mode.to_string()))
        }
    }

    /// Add or replace a mode.
    pub fn add_mode(&mut self, mode: KeyMode) {
        self.modes.insert(mode.name.clone(), mode);
    }

    /// Get a mutable reference to a mode by name.
    #[must_use]
    pub fn mode_mut(&mut self, name: &str) -> Option<&mut KeyMode> {
        self.modes.get_mut(name)
    }

    /// Get a reference to a mode by name.
    #[must_use]
    pub fn mode(&self, name: &str) -> Option<&KeyMode> {
        self.modes.get(name)
    }

    /// Add a chord definition.
    pub fn add_chord(&mut self, chord: KeyChord) {
        self.chords.push(chord);
    }

    /// Add a key remap.
    pub fn add_remap(&mut self, remap: crate::remap::KeyRemap) {
        self.remaps.push(remap);
    }

    /// Match a key event against current mode bindings, chords, and remaps.
    ///
    /// Processing order:
    /// 1. Check remaps (transform the key before matching)
    /// 2. If chord is pending, check for follower match or timeout
    /// 3. Check if the key is a chord leader
    /// 4. Check current mode bindings
    /// 5. Return NoMatch (passthrough depends on mode setting)
    pub fn match_key(
        &mut self,
        hotkey: Hotkey,
        ctx: &MatchContext,
    ) -> MatchResult {
        // 1. Check remaps
        for remap in &self.remaps {
            if remap.from == hotkey {
                if let Some(ref condition) = remap.condition {
                    if !condition.matches(ctx) {
                        continue;
                    }
                }
                return MatchResult::Remapped { to: remap.to };
            }
        }

        // 2. If chord is pending, check for follower or timeout
        if self.chord_state.is_pending() {
            if self.chord_state.is_timed_out() {
                self.chord_state.reset();
                // Fall through to normal matching
            } else if let Some(leader) = self.chord_state.pending_leader().copied() {
                // Look for a matching chord follower
                for chord in &self.chords {
                    if chord.leader == leader && chord.follower == hotkey {
                        self.chord_state.reset();
                        return MatchResult::Matched {
                            action: chord.action.clone(),
                            consume: true,
                        };
                    }
                }
                // No follower matched — cancel chord, fall through
                self.chord_state.reset();
            }
        }

        // 3. Check if this key is a chord leader
        for chord in &self.chords {
            if chord.leader == hotkey {
                self.chord_state.begin(hotkey, chord.timeout_ms);
                return MatchResult::ChordPending {
                    leader: hotkey,
                    timeout_ms: chord.timeout_ms,
                };
            }
        }

        // 4. Check current mode bindings
        if let Some(mode) = self.modes.get(&self.current_mode) {
            if let Some(binding) = mode.find_binding(&hotkey, ctx) {
                return MatchResult::Matched {
                    action: binding.action.clone(),
                    consume: binding.consume,
                };
            }
        }

        // 5. No match
        MatchResult::NoMatch
    }

    /// Returns whether unmatched keys in the current mode should pass through.
    #[must_use]
    pub fn current_mode_passthrough(&self) -> bool {
        self.modes
            .get(&self.current_mode)
            .is_some_and(|m| m.passthrough)
    }

    /// List all bindings in the current mode.
    #[must_use]
    pub fn list_bindings(&self) -> Vec<(&Hotkey, &Action)> {
        self.modes
            .get(&self.current_mode)
            .map(|m| {
                m.bindings
                    .iter()
                    .map(|(hk, b)| (hk, &b.action))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// List all mode names.
    #[must_use]
    pub fn mode_names(&self) -> Vec<&str> {
        self.modes.keys().map(String::as_str).collect()
    }
}

impl Default for BindingMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hotkey::{Key, Modifiers};

    fn cmd_h() -> Hotkey {
        Hotkey::new(Modifiers::CMD, Key::H)
    }

    fn cmd_j() -> Hotkey {
        Hotkey::new(Modifiers::CMD, Key::J)
    }

    fn escape() -> Hotkey {
        Hotkey::new(Modifiers::NONE, Key::Escape)
    }

    fn ctx() -> MatchContext {
        MatchContext::default()
    }

    // ── KeyMode tests ───────────────────────────────────────────────

    #[test]
    fn mode_add_and_find_binding() {
        let mut mode = KeyMode::new("default", true);
        mode.add_binding(Binding::new(cmd_h(), Action::command("focus_west")));

        let found = mode.find_binding(&cmd_h(), &ctx());
        assert!(found.is_some());
        assert_eq!(found.unwrap().action, Action::command("focus_west"));
    }

    #[test]
    fn mode_find_unbound_key() {
        let mode = KeyMode::new("default", true);
        assert!(mode.find_binding(&cmd_h(), &ctx()).is_none());
    }

    #[test]
    fn mode_conditional_binding() {
        let mut mode = KeyMode::new("default", true);
        mode.add_binding(
            Binding::new(cmd_h(), Action::command("focus_west"))
                .with_condition(crate::Condition {
                    app_exclude: Some("Terminal".to_string()),
                    ..Default::default()
                }),
        );

        // Should match when not in Terminal
        let safari_ctx = MatchContext {
            focused_app_bundle_id: Some("com.apple.Safari".to_string()),
            ..Default::default()
        };
        assert!(mode.find_binding(&cmd_h(), &safari_ctx).is_some());

        // Should NOT match when in Terminal
        let term_ctx = MatchContext {
            focused_app_bundle_id: Some("com.apple.Terminal".to_string()),
            ..Default::default()
        };
        assert!(mode.find_binding(&cmd_h(), &term_ctx).is_none());
    }

    #[test]
    fn mode_replace_binding() {
        let mut mode = KeyMode::new("default", true);
        let old = mode.add_binding(Binding::new(cmd_h(), Action::command("a")));
        assert!(old.is_none());

        let old = mode.add_binding(Binding::new(cmd_h(), Action::command("b")));
        assert!(old.is_some());
        assert_eq!(old.unwrap().action, Action::command("a"));
    }

    // ── BindingMap tests ────────────────────────────────────────────

    #[test]
    fn binding_map_default_mode() {
        let map = BindingMap::new();
        assert_eq!(map.current_mode(), "default");
        assert!(map.current_mode_passthrough());
    }

    #[test]
    fn binding_map_match_in_default_mode() {
        let mut map = BindingMap::new();
        map.mode_mut("default")
            .unwrap()
            .add_binding(Binding::new(cmd_h(), Action::command("focus_west")));

        let result = map.match_key(cmd_h(), &ctx());
        assert_eq!(
            result,
            MatchResult::Matched {
                action: Action::command("focus_west"),
                consume: true,
            }
        );
    }

    #[test]
    fn binding_map_no_match() {
        let mut map = BindingMap::default();
        let result = map.match_key(cmd_h(), &ctx()).clone();
        assert_eq!(result, MatchResult::NoMatch);
    }

    #[test]
    fn binding_map_mode_switch() {
        let mut map = BindingMap::new();

        // Add a resize mode
        let mut resize = KeyMode::new("resize", false);
        resize.add_binding(Binding::new(
            Hotkey::new(Modifiers::NONE, Key::H),
            Action::command("shrink_west"),
        ));
        resize.add_binding(Binding::new(escape(), Action::mode_switch("default")));
        map.add_mode(resize);

        // Switch to resize mode
        map.set_mode("resize").unwrap();
        assert_eq!(map.current_mode(), "resize");
        assert!(!map.current_mode_passthrough());

        // Key H (no modifier) should match in resize mode
        let result = map.match_key(Hotkey::new(Modifiers::NONE, Key::H), &ctx());
        assert_eq!(
            result,
            MatchResult::Matched {
                action: Action::command("shrink_west"),
                consume: true,
            }
        );
    }

    #[test]
    fn binding_map_invalid_mode_switch() {
        let mut map = BindingMap::new();
        let result = map.set_mode("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn binding_map_chord() {
        let mut map = BindingMap::new();

        let ctrl_a = Hotkey::new(Modifiers::CTRL, Key::A);
        let plain_c = Hotkey::new(Modifiers::NONE, Key::C);

        map.add_chord(crate::chord::KeyChord {
            leader: ctrl_a,
            follower: plain_c,
            timeout_ms: 1000,
            action: Action::command("new_window"),
        });

        // Press leader
        let result = map.match_key(ctrl_a, &ctx());
        assert_eq!(
            result,
            MatchResult::ChordPending {
                leader: ctrl_a,
                timeout_ms: 1000,
            }
        );

        // Press follower
        let result = map.match_key(plain_c, &ctx());
        assert_eq!(
            result,
            MatchResult::Matched {
                action: Action::command("new_window"),
                consume: true,
            }
        );
    }

    #[test]
    fn binding_map_chord_wrong_follower() {
        let mut map = BindingMap::new();

        let ctrl_a = Hotkey::new(Modifiers::CTRL, Key::A);
        let plain_c = Hotkey::new(Modifiers::NONE, Key::C);

        map.add_chord(crate::chord::KeyChord {
            leader: ctrl_a,
            follower: plain_c,
            timeout_ms: 1000,
            action: Action::command("new_window"),
        });

        // Press leader
        map.match_key(ctrl_a, &ctx());

        // Press wrong follower
        let result = map.match_key(Hotkey::new(Modifiers::NONE, Key::X), &ctx());
        // Chord cancelled, falls through to normal matching
        assert_eq!(result, MatchResult::NoMatch);
    }

    #[test]
    fn binding_map_chord_timeout() {
        let mut map = BindingMap::new();

        let ctrl_a = Hotkey::new(Modifiers::CTRL, Key::A);
        let plain_c = Hotkey::new(Modifiers::NONE, Key::C);

        map.add_chord(crate::chord::KeyChord {
            leader: ctrl_a,
            follower: plain_c,
            timeout_ms: 0, // immediate timeout
            action: Action::command("new_window"),
        });

        // Press leader
        map.match_key(ctrl_a, &ctx());
        std::thread::sleep(std::time::Duration::from_millis(1));

        // Chord should have timed out — follower should not match chord
        let result = map.match_key(plain_c, &ctx());
        assert_eq!(result, MatchResult::NoMatch);
    }

    #[test]
    fn binding_map_remap() {
        let mut map = BindingMap::new();

        map.add_remap(crate::remap::KeyRemap {
            from: Hotkey::new(Modifiers::NONE, Key::CapsLock),
            to: Hotkey::new(Modifiers::NONE, Key::Escape),
            condition: None,
        });

        let result = map.match_key(
            Hotkey::new(Modifiers::NONE, Key::CapsLock),
            &ctx(),
        );
        assert_eq!(
            result,
            MatchResult::Remapped {
                to: Hotkey::new(Modifiers::NONE, Key::Escape),
            }
        );
    }

    #[test]
    fn binding_map_conditional_remap() {
        let mut map = BindingMap::new();

        map.add_remap(crate::remap::KeyRemap {
            from: Hotkey::new(Modifiers::FN, Key::H),
            to: Hotkey::new(Modifiers::NONE, Key::Left),
            condition: Some(crate::Condition {
                app: Some("Terminal".to_string()),
                ..Default::default()
            }),
        });

        // Should remap in Terminal
        let term_ctx = MatchContext {
            focused_app_bundle_id: Some("com.apple.Terminal".to_string()),
            ..Default::default()
        };
        let result = map.match_key(Hotkey::new(Modifiers::FN, Key::H), &term_ctx);
        assert!(matches!(result, MatchResult::Remapped { .. }));

        // Should NOT remap in Safari
        let safari_ctx = MatchContext {
            focused_app_bundle_id: Some("com.apple.Safari".to_string()),
            ..Default::default()
        };
        let result = map.match_key(Hotkey::new(Modifiers::FN, Key::H), &safari_ctx);
        assert_eq!(result, MatchResult::NoMatch);
    }

    #[test]
    fn binding_map_list_bindings() {
        let mut map = BindingMap::new();
        map.mode_mut("default").unwrap().add_binding(
            Binding::new(cmd_h(), Action::command("a")),
        );
        map.mode_mut("default").unwrap().add_binding(
            Binding::new(cmd_j(), Action::command("b")),
        );

        let bindings = map.list_bindings();
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn binding_map_mode_names() {
        let mut map = BindingMap::new();
        map.add_mode(KeyMode::new("resize", false));
        map.add_mode(KeyMode::new("launch", true));

        let mut names: Vec<&str> = map.mode_names();
        names.sort();
        assert_eq!(names, vec!["default", "launch", "resize"]);
    }

    #[test]
    fn remap_takes_priority_over_binding() {
        let mut map = BindingMap::new();

        let caps = Hotkey::new(Modifiers::NONE, Key::CapsLock);
        let esc = Hotkey::new(Modifiers::NONE, Key::Escape);

        // Add both a remap and a binding for CapsLock
        map.add_remap(crate::remap::KeyRemap {
            from: caps,
            to: esc,
            condition: None,
        });
        map.mode_mut("default")
            .unwrap()
            .add_binding(Binding::new(caps, Action::command("should_not_match")));

        // Remap should take priority
        let result = map.match_key(caps, &ctx());
        assert!(matches!(result, MatchResult::Remapped { .. }));
    }

    // ── Additional BindingMap tests ─────────────────────────────────

    #[test]
    fn binding_map_default_has_default_mode() {
        let map = BindingMap::default();
        assert!(map.mode("default").is_some());
        assert_eq!(map.mode_names().len(), 1);
    }

    #[test]
    fn binding_map_mode_switch_resets_chord_state() {
        let mut map = BindingMap::new();
        map.add_mode(KeyMode::new("other", true));

        let ctrl_a = Hotkey::new(Modifiers::CTRL, Key::A);
        let plain_c = Hotkey::new(Modifiers::NONE, Key::C);

        map.add_chord(crate::chord::KeyChord {
            leader: ctrl_a,
            follower: plain_c,
            timeout_ms: 5000,
            action: Action::command("chord_action"),
        });

        // Start a chord
        let result = map.match_key(ctrl_a, &ctx());
        assert!(matches!(result, MatchResult::ChordPending { .. }));

        // Switch mode -- should reset chord state
        map.set_mode("other").unwrap();

        // Now pressing the follower should not complete the chord
        let result = map.match_key(plain_c, &ctx());
        assert_eq!(result, MatchResult::NoMatch);
    }

    #[test]
    fn binding_map_bindings_isolated_across_modes() {
        let mut map = BindingMap::new();

        // Add binding only in default mode
        map.mode_mut("default")
            .unwrap()
            .add_binding(Binding::new(cmd_h(), Action::command("default_action")));

        // Add a second mode without that binding
        map.add_mode(KeyMode::new("other", true));

        // Should match in default mode
        let result = map.match_key(cmd_h(), &ctx());
        assert!(matches!(result, MatchResult::Matched { .. }));

        // Switch to other mode -- should not match
        map.set_mode("other").unwrap();
        let result = map.match_key(cmd_h(), &ctx());
        assert_eq!(result, MatchResult::NoMatch);
    }

    #[test]
    fn binding_map_non_consuming_binding() {
        let mut map = BindingMap::new();
        map.mode_mut("default")
            .unwrap()
            .add_binding(Binding::new(cmd_h(), Action::command("passthrough")).with_consume(false));

        let result = map.match_key(cmd_h(), &ctx());
        assert_eq!(
            result,
            MatchResult::Matched {
                action: Action::command("passthrough"),
                consume: false,
            }
        );
    }

    #[test]
    fn binding_map_conditional_binding_in_mode() {
        let mut map = BindingMap::new();
        map.mode_mut("default").unwrap().add_binding(
            Binding::new(cmd_h(), Action::command("focus_west"))
                .with_condition(crate::Condition {
                    app: Some("Safari".to_string()),
                    ..Default::default()
                }),
        );

        // Match in Safari
        let safari_ctx = MatchContext {
            focused_app_bundle_id: Some("com.apple.Safari".to_string()),
            ..Default::default()
        };
        let result = map.match_key(cmd_h(), &safari_ctx);
        assert!(matches!(result, MatchResult::Matched { .. }));

        // No match in other apps
        let terminal_ctx = MatchContext {
            focused_app_bundle_id: Some("com.apple.Terminal".to_string()),
            ..Default::default()
        };
        let result = map.match_key(cmd_h(), &terminal_ctx);
        assert_eq!(result, MatchResult::NoMatch);
    }

    #[test]
    fn binding_map_multiple_chords_different_leaders() {
        let mut map = BindingMap::new();

        let ctrl_a = Hotkey::new(Modifiers::CTRL, Key::A);
        let ctrl_b = Hotkey::new(Modifiers::CTRL, Key::B);
        let plain_c = Hotkey::new(Modifiers::NONE, Key::C);

        map.add_chord(crate::chord::KeyChord {
            leader: ctrl_a,
            follower: plain_c,
            timeout_ms: 1000,
            action: Action::command("chord_a_c"),
        });
        map.add_chord(crate::chord::KeyChord {
            leader: ctrl_b,
            follower: plain_c,
            timeout_ms: 1000,
            action: Action::command("chord_b_c"),
        });

        // Activate chord with ctrl+a, complete with c
        map.match_key(ctrl_a, &ctx());
        let result = map.match_key(plain_c, &ctx());
        assert_eq!(
            result,
            MatchResult::Matched {
                action: Action::command("chord_a_c"),
                consume: true,
            }
        );

        // Activate chord with ctrl+b, complete with c
        map.match_key(ctrl_b, &ctx());
        let result = map.match_key(plain_c, &ctx());
        assert_eq!(
            result,
            MatchResult::Matched {
                action: Action::command("chord_b_c"),
                consume: true,
            }
        );
    }

    #[test]
    fn binding_map_multiple_chords_same_leader_different_followers() {
        let mut map = BindingMap::new();

        let ctrl_a = Hotkey::new(Modifiers::CTRL, Key::A);
        let plain_c = Hotkey::new(Modifiers::NONE, Key::C);
        let plain_n = Hotkey::new(Modifiers::NONE, Key::N);

        map.add_chord(crate::chord::KeyChord {
            leader: ctrl_a,
            follower: plain_c,
            timeout_ms: 1000,
            action: Action::command("new_window"),
        });
        map.add_chord(crate::chord::KeyChord {
            leader: ctrl_a,
            follower: plain_n,
            timeout_ms: 1000,
            action: Action::command("next_window"),
        });

        // ctrl+a then n
        map.match_key(ctrl_a, &ctx());
        let result = map.match_key(plain_n, &ctx());
        assert_eq!(
            result,
            MatchResult::Matched {
                action: Action::command("next_window"),
                consume: true,
            }
        );

        // ctrl+a then c
        map.match_key(ctrl_a, &ctx());
        let result = map.match_key(plain_c, &ctx());
        assert_eq!(
            result,
            MatchResult::Matched {
                action: Action::command("new_window"),
                consume: true,
            }
        );
    }

    #[test]
    fn binding_map_remap_does_not_affect_different_key() {
        let mut map = BindingMap::new();

        map.add_remap(crate::remap::KeyRemap {
            from: Hotkey::new(Modifiers::NONE, Key::CapsLock),
            to: Hotkey::new(Modifiers::NONE, Key::Escape),
            condition: None,
        });

        // A different key should not be remapped
        let result = map.match_key(
            Hotkey::new(Modifiers::NONE, Key::A),
            &ctx(),
        );
        assert_eq!(result, MatchResult::NoMatch);
    }

    #[test]
    fn binding_map_multiple_remaps_first_match_wins() {
        let mut map = BindingMap::new();

        let caps = Hotkey::new(Modifiers::NONE, Key::CapsLock);

        map.add_remap(crate::remap::KeyRemap {
            from: caps,
            to: Hotkey::new(Modifiers::NONE, Key::Escape),
            condition: None,
        });
        map.add_remap(crate::remap::KeyRemap {
            from: caps,
            to: Hotkey::new(Modifiers::NONE, Key::Tab),
            condition: None,
        });

        // First remap should win
        let result = map.match_key(caps, &ctx());
        assert_eq!(
            result,
            MatchResult::Remapped {
                to: Hotkey::new(Modifiers::NONE, Key::Escape),
            }
        );
    }

    #[test]
    fn binding_map_remap_takes_priority_over_chord() {
        let mut map = BindingMap::new();

        let caps = Hotkey::new(Modifiers::NONE, Key::CapsLock);

        map.add_remap(crate::remap::KeyRemap {
            from: caps,
            to: Hotkey::new(Modifiers::NONE, Key::Escape),
            condition: None,
        });

        map.add_chord(crate::chord::KeyChord {
            leader: caps,
            follower: Hotkey::new(Modifiers::NONE, Key::C),
            timeout_ms: 1000,
            action: Action::command("should_not_match"),
        });

        // Remap should take priority over chord leader
        let result = map.match_key(caps, &ctx());
        assert!(matches!(result, MatchResult::Remapped { .. }));
    }

    #[test]
    fn binding_map_chord_takes_priority_over_mode_binding() {
        let mut map = BindingMap::new();

        let ctrl_a = Hotkey::new(Modifiers::CTRL, Key::A);
        let plain_c = Hotkey::new(Modifiers::NONE, Key::C);

        // Add a binding for ctrl+a in default mode
        map.mode_mut("default")
            .unwrap()
            .add_binding(Binding::new(ctrl_a, Action::command("select_all")));

        // Also add a chord with ctrl+a as leader
        map.add_chord(crate::chord::KeyChord {
            leader: ctrl_a,
            follower: plain_c,
            timeout_ms: 1000,
            action: Action::command("chord_action"),
        });

        // Chord leader should take priority -- returns ChordPending
        let result = map.match_key(ctrl_a, &ctx());
        assert!(matches!(result, MatchResult::ChordPending { .. }));
    }

    #[test]
    fn binding_map_mode_passthrough_false() {
        let mut map = BindingMap::new();
        map.add_mode(KeyMode::new("modal", false));
        map.set_mode("modal").unwrap();
        assert!(!map.current_mode_passthrough());
    }

    #[test]
    fn binding_map_list_bindings_empty_mode() {
        let map = BindingMap::new();
        assert!(map.list_bindings().is_empty());
    }

    #[test]
    fn binding_map_add_mode_replaces_existing() {
        let mut map = BindingMap::new();

        let mut mode1 = KeyMode::new("default", true);
        mode1.add_binding(Binding::new(cmd_h(), Action::command("a")));
        map.add_mode(mode1);

        // Replace default mode with empty one
        let mode2 = KeyMode::new("default", false);
        map.add_mode(mode2);

        // Binding should be gone, passthrough should be false
        assert!(!map.current_mode_passthrough());
        assert!(map.list_bindings().is_empty());
    }
}
