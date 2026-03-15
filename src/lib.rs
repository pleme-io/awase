//! Awase (合わせ) --- global hotkey abstraction.
//!
//! Provides platform-agnostic types and traits for global hotkey
//! registration, mode systems, key chords, conditional bindings, and
//! key remapping. macOS and Linux backends can be added as separate
//! feature-gated modules.
//!
//! # Quick Start
//!
//! ```rust
//! use awase::{Hotkey, Modifiers, Key, NoopManager, HotkeyManager};
//! use awase::{Action, Binding, Condition};
//!
//! // Parse hotkeys in plus-separated or skhd format
//! let hk = Hotkey::parse("cmd+space").unwrap();
//! let hk2 = Hotkey::parse("cmd - h").unwrap(); // skhd style
//!
//! // Create bindings with actions and conditions
//! let binding = Binding::new(hk, Action::command("launcher_toggle"))
//!     .with_condition(Condition {
//!         app_exclude: Some("com.apple.Terminal".to_string()),
//!         ..Default::default()
//!     });
//!
//! // Use the hotkey manager
//! let mut manager = NoopManager::new();
//! manager.register(1, hk).unwrap();
//! manager.unregister(1).unwrap();
//! ```

pub mod action;
pub mod binding;
pub mod chord;
pub mod condition;
pub mod conflict;
mod error;
mod hotkey;
mod manager;
pub mod mode;
pub mod macos;
pub mod remap;

pub use action::Action;
pub use binding::Binding;
pub use chord::{ChordState, KeyChord};
pub use condition::{Condition, MatchContext};
pub use conflict::{detect_conflicts, ConflictEntry, ConflictReport};
pub use error::AwaseError;
pub use hotkey::{Hotkey, Key, Modifiers};
pub use manager::{HotkeyManager, NoopManager};
pub use mode::{BindingMap, KeyMode, MatchResult};
pub use remap::KeyRemap;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Simulates a complete window-manager-style configuration:
    /// default mode with bindings + chords, resize mode, remaps, conditions.
    #[test]
    fn full_wm_scenario() {
        let mut map = BindingMap::new();

        // -- Default mode bindings --
        let default = map.mode_mut("default").unwrap();
        default.add_binding(
            Binding::new(
                Hotkey::parse("cmd+h").unwrap(),
                Action::command("focus_west"),
            )
            .with_condition(Condition {
                app_exclude: Some("Terminal|ghostty".to_string()),
                ..Default::default()
            }),
        );
        default.add_binding(Binding::new(
            Hotkey::parse("cmd+j").unwrap(),
            Action::command("focus_south"),
        ));
        default.add_binding(Binding::new(
            Hotkey::parse("ctrl+alt+r").unwrap(),
            Action::mode_switch("resize"),
        ));

        // -- Resize mode --
        let mut resize = KeyMode::new("resize", false);
        resize.add_binding(Binding::new(
            Hotkey::parse("h").unwrap(),
            Action::command("shrink_west"),
        ));
        resize.add_binding(Binding::new(
            Hotkey::parse("l").unwrap(),
            Action::command("grow_east"),
        ));
        resize.add_binding(Binding::new(
            Hotkey::parse("escape").unwrap(),
            Action::mode_switch("default"),
        ));
        map.add_mode(resize);

        // -- Chord: ctrl+a -> c = new window --
        map.add_chord(KeyChord {
            leader: Hotkey::parse("ctrl+a").unwrap(),
            follower: Hotkey::parse("c").unwrap(),
            timeout_ms: 1000,
            action: Action::exec("open -a Terminal"),
        });

        // -- Remap: caps_lock -> escape --
        map.add_remap(KeyRemap::new(
            Hotkey::parse("capslock").unwrap(),
            Hotkey::parse("escape").unwrap(),
        ));

        let safari_ctx = MatchContext {
            focused_app_bundle_id: Some("com.apple.Safari".to_string()),
            ..Default::default()
        };
        let terminal_ctx = MatchContext {
            focused_app_bundle_id: Some("com.apple.Terminal".to_string()),
            ..Default::default()
        };

        // 1. cmd+h in Safari should match (condition passes)
        let result = map.match_key(Hotkey::parse("cmd+h").unwrap(), &safari_ctx);
        assert_eq!(
            result,
            MatchResult::Matched {
                action: Action::command("focus_west"),
                consume: true,
            }
        );

        // 2. cmd+h in Terminal should NOT match (condition excludes Terminal)
        let result = map.match_key(Hotkey::parse("cmd+h").unwrap(), &terminal_ctx);
        assert_eq!(result, MatchResult::NoMatch);

        // 3. CapsLock should remap to Escape
        let result = map.match_key(Hotkey::parse("capslock").unwrap(), &safari_ctx);
        assert_eq!(
            result,
            MatchResult::Remapped {
                to: Hotkey::parse("escape").unwrap(),
            }
        );

        // 4. Enter resize mode
        let result = map.match_key(Hotkey::parse("ctrl+alt+r").unwrap(), &safari_ctx);
        assert_eq!(
            result,
            MatchResult::Matched {
                action: Action::mode_switch("resize"),
                consume: true,
            }
        );
        map.set_mode("resize").unwrap();
        assert_eq!(map.current_mode(), "resize");
        assert!(!map.current_mode_passthrough());

        // 5. In resize mode, 'h' triggers shrink
        let result = map.match_key(Hotkey::parse("h").unwrap(), &safari_ctx);
        assert_eq!(
            result,
            MatchResult::Matched {
                action: Action::command("shrink_west"),
                consume: true,
            }
        );

        // 6. Escape returns to default mode
        let result = map.match_key(Hotkey::parse("escape").unwrap(), &safari_ctx);
        assert_eq!(
            result,
            MatchResult::Matched {
                action: Action::mode_switch("default"),
                consume: true,
            }
        );
        map.set_mode("default").unwrap();
        assert_eq!(map.current_mode(), "default");

        // 7. Chord: ctrl+a then c
        let result = map.match_key(Hotkey::parse("ctrl+a").unwrap(), &safari_ctx);
        assert!(matches!(result, MatchResult::ChordPending { .. }));

        let result = map.match_key(Hotkey::parse("c").unwrap(), &safari_ctx);
        assert_eq!(
            result,
            MatchResult::Matched {
                action: Action::exec("open -a Terminal"),
                consume: true,
            }
        );
    }

    /// Test that conflict detection catches the chord leader / binding overlap
    /// in the scenario above.
    #[test]
    fn conflict_detection_for_wm_scenario() {
        let mut default = KeyMode::new("default", true);
        default.add_binding(Binding::new(
            Hotkey::parse("ctrl+a").unwrap(),
            Action::command("select_all"),
        ));

        let chord = KeyChord {
            leader: Hotkey::parse("ctrl+a").unwrap(),
            follower: Hotkey::parse("c").unwrap(),
            timeout_ms: 1000,
            action: Action::exec("open -a Terminal"),
        };

        let report = detect_conflicts(&[&default], &[chord]);
        assert!(!report.is_clean());
        assert_eq!(report.conflicts.len(), 1);
    }

    /// Test that the NoopManager can be used through the trait interface.
    #[test]
    fn noop_manager_via_trait_object() {
        let mut manager: Box<dyn HotkeyManager> = Box::new(NoopManager::new());
        let hk = Hotkey::parse("cmd+space").unwrap();

        manager.register(1, hk).unwrap();
        assert!(manager.register(1, hk).is_err());
        manager.unregister(1).unwrap();
        manager.register(1, hk).unwrap();
    }

    /// Test that parsed hotkeys from different formats are interchangeable
    /// as HashMap keys.
    #[test]
    fn parsed_hotkeys_as_hashmap_keys() {
        use std::collections::HashMap;

        let mut bindings: HashMap<Hotkey, &str> = HashMap::new();

        let plus_format = Hotkey::parse("cmd+alt+h").unwrap();
        let skhd_format = Hotkey::parse("cmd + alt - h").unwrap();

        bindings.insert(plus_format, "focus_west");

        // skhd format should look up the same entry
        assert_eq!(bindings.get(&skhd_format), Some(&"focus_west"));
    }

    /// Test that Hotkey Display output is always parseable.
    #[test]
    fn display_always_parseable() {
        let hotkeys = [
            Hotkey::new(Modifiers::NONE, Key::Escape),
            Hotkey::new(Modifiers::CMD, Key::Space),
            Hotkey::new(Modifiers::CMD | Modifiers::SHIFT, Key::A),
            Hotkey::new(Modifiers::HYPER, Key::F12),
            Hotkey::new(Modifiers::FN | Modifiers::CAPS_LOCK, Key::H),
            Hotkey::new(Modifiers::NONE, Key::MouseLeft),
            Hotkey::new(Modifiers::NONE, Key::NumpadEnter),
            Hotkey::new(Modifiers::NONE, Key::BrightnessUp),
        ];

        for hk in &hotkeys {
            let displayed = hk.display();
            let reparsed = Hotkey::parse(&displayed).unwrap_or_else(|e| {
                panic!("failed to reparse \"{displayed}\" (from {hk:?}): {e}");
            });
            assert_eq!(hk, &reparsed, "roundtrip failed for {hk:?}");
        }
    }

    /// Test macOS keycode roundtrip through the public API.
    #[test]
    fn macos_keycode_roundtrip_via_public_api() {
        let key = Key::A;
        let code = macos::key_to_keycode(key).unwrap();
        let back = macos::keycode_to_key(code).unwrap();
        assert_eq!(key, back);
    }

    /// Test macOS flags roundtrip through the public API.
    #[test]
    fn macos_flags_roundtrip_via_public_api() {
        let mods = Modifiers::CMD | Modifiers::SHIFT;
        let flags = macos::modifiers_to_cg_flags(mods);
        let back = macos::cg_flags_to_modifiers(flags);
        assert_eq!(mods, back);
    }
}
