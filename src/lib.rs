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

    // ── Additional gap-coverage tests ──────────────────────────────

    /// Verify all AwaseError variants produce non-empty Display output.
    #[test]
    fn error_display_all_variants() {
        let errors = [
            AwaseError::InvalidHotkey("bad combo".to_string()),
            AwaseError::AlreadyRegistered(42),
            AwaseError::ModeNotFound("nonexistent".to_string()),
            AwaseError::DuplicateBinding {
                mode: "default".to_string(),
                hotkey: "cmd+a".to_string(),
            },
            AwaseError::PermissionDenied("accessibility not granted".to_string()),
            AwaseError::Platform("CGEventTap failed".to_string()),
        ];

        for err in &errors {
            let msg = format!("{err}");
            assert!(!msg.is_empty(), "Display for {err:?} should not be empty");
        }

        // Verify specific error messages contain expected substrings.
        assert!(format!("{}", errors[0]).contains("bad combo"));
        assert!(format!("{}", errors[1]).contains("42"));
        assert!(format!("{}", errors[2]).contains("nonexistent"));
        assert!(format!("{}", errors[3]).contains("cmd+a"));
        assert!(format!("{}", errors[3]).contains("default"));
        assert!(format!("{}", errors[4]).contains("accessibility"));
        assert!(format!("{}", errors[5]).contains("CGEventTap"));
    }

    /// Hotkey serde roundtrip (JSON serialization/deserialization).
    #[test]
    fn hotkey_serde_roundtrip() {
        let hotkeys = [
            Hotkey::new(Modifiers::NONE, Key::A),
            Hotkey::new(Modifiers::CMD | Modifiers::SHIFT, Key::F12),
            Hotkey::new(Modifiers::HYPER, Key::Space),
            Hotkey::new(Modifiers::FN | Modifiers::CAPS_LOCK, Key::NumpadEnter),
        ];

        for hk in &hotkeys {
            let json = serde_json::to_string(hk).unwrap();
            let deserialized: Hotkey = serde_json::from_str(&json).unwrap();
            assert_eq!(hk, &deserialized, "serde roundtrip failed for {hk:?}");
        }
    }

    /// Modifiers::BitOrAssign accumulates flags correctly.
    #[test]
    fn modifiers_bitor_assign_accumulates() {
        let mut mods = Modifiers::NONE;
        assert!(mods.is_empty());

        mods |= Modifiers::CMD;
        assert!(mods.contains(Modifiers::CMD));
        assert!(!mods.contains(Modifiers::ALT));

        mods |= Modifiers::ALT;
        assert!(mods.contains(Modifiers::CMD));
        assert!(mods.contains(Modifiers::ALT));

        // Assigning the same flag again is idempotent.
        mods |= Modifiers::CMD;
        assert_eq!(mods, Modifiers::CMD | Modifiers::ALT);
    }

    /// Modifiers::Display with all six flags shows them in canonical order.
    #[test]
    fn modifiers_display_all_six() {
        let all = Modifiers::CMD
            | Modifiers::CTRL
            | Modifiers::ALT
            | Modifiers::SHIFT
            | Modifiers::FN
            | Modifiers::CAPS_LOCK;
        let s = format!("{all}");
        assert_eq!(s, "cmd+ctrl+alt+shift+fn+caps_lock");
    }

    /// BindingMap::mode() immutable accessor returns mode references.
    #[test]
    fn binding_map_mode_immutable_accessor() {
        let mut map = BindingMap::new();

        // Default mode exists.
        let default = map.mode("default");
        assert!(default.is_some());
        assert_eq!(default.unwrap().name, "default");

        // Non-existent mode returns None.
        assert!(map.mode("nonexistent").is_none());

        // After adding a mode, it is accessible.
        map.add_mode(mode::KeyMode::new("resize", false));
        let resize = map.mode("resize");
        assert!(resize.is_some());
        assert!(!resize.unwrap().passthrough);
    }

    /// KeyMode serialization produces valid JSON with expected fields.
    /// Note: KeyMode with bindings cannot roundtrip through JSON because
    /// Hotkey (a struct) is used as a HashMap key, and JSON keys must be strings.
    /// So we test the empty-bindings case which does roundtrip.
    #[test]
    fn key_mode_serde_roundtrip_empty() {
        let mode = mode::KeyMode::new("resize", false);
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: mode::KeyMode = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "resize");
        assert!(!deserialized.passthrough);
        assert!(deserialized.bindings.is_empty());
    }

    /// KeyMode Debug output includes all expected fields.
    /// Note: serde_json cannot serialize HashMap<Hotkey, _> because JSON
    /// keys must be strings and Hotkey is a struct. We verify via Debug instead.
    #[test]
    fn key_mode_debug_includes_fields() {
        let mut mode = mode::KeyMode::new("resize", false);
        mode.add_binding(binding::Binding::new(
            Hotkey::parse("h").unwrap(),
            action::Action::command("shrink"),
        ));

        let debug = format!("{mode:?}");
        assert!(debug.contains("resize"));
        assert!(debug.contains("shrink"));
        assert!(debug.contains("passthrough"));

        // Verify the binding is accessible by key.
        let found = mode.find_binding(
            &Hotkey::parse("h").unwrap(),
            &condition::MatchContext::default(),
        );
        assert!(found.is_some());
        assert_eq!(found.unwrap().action, action::Action::command("shrink"));
    }

    /// MatchResult clone produces equal values.
    #[test]
    fn match_result_clone_and_debug() {
        let variants = [
            mode::MatchResult::NoMatch,
            mode::MatchResult::Matched {
                action: action::Action::command("test"),
                consume: true,
            },
            mode::MatchResult::ChordPending {
                leader: Hotkey::new(Modifiers::CTRL, Key::A),
                timeout_ms: 1000,
            },
            mode::MatchResult::Remapped {
                to: Hotkey::new(Modifiers::NONE, Key::Escape),
            },
        ];

        for variant in &variants {
            let cloned = variant.clone();
            assert_eq!(variant, &cloned);
            // Debug output should not be empty.
            let debug = format!("{variant:?}");
            assert!(!debug.is_empty());
        }
    }

    /// Multiple remaps with mixed conditions: first matching remap wins.
    #[test]
    fn multiple_remaps_with_mixed_conditions() {
        let mut map = BindingMap::new();

        // Remap capslock -> ctrl in Terminal only.
        map.add_remap(remap::KeyRemap::new(
            Hotkey::new(Modifiers::NONE, Key::CapsLock),
            Hotkey::new(Modifiers::NONE, Key::Escape),
        ).with_condition(Condition {
            app: Some("Terminal".to_string()),
            ..Default::default()
        }));

        // Remap capslock -> tab unconditionally (fallback).
        map.add_remap(remap::KeyRemap::new(
            Hotkey::new(Modifiers::NONE, Key::CapsLock),
            Hotkey::new(Modifiers::NONE, Key::Tab),
        ));

        let caps = Hotkey::new(Modifiers::NONE, Key::CapsLock);

        // In Terminal, first (conditional) remap matches.
        let term_ctx = condition::MatchContext {
            focused_app_bundle_id: Some("com.apple.Terminal".to_string()),
            ..Default::default()
        };
        let result = map.match_key(caps, &term_ctx);
        assert_eq!(
            result,
            mode::MatchResult::Remapped {
                to: Hotkey::new(Modifiers::NONE, Key::Escape),
            }
        );

        // In Safari, first remap condition fails, so second (unconditional) wins.
        let safari_ctx = condition::MatchContext {
            focused_app_bundle_id: Some("com.apple.Safari".to_string()),
            ..Default::default()
        };
        let result = map.match_key(caps, &safari_ctx);
        assert_eq!(
            result,
            mode::MatchResult::Remapped {
                to: Hotkey::new(Modifiers::NONE, Key::Tab),
            }
        );
    }

    /// ConflictReport Debug impl works and ConflictEntry can be cloned.
    #[test]
    fn conflict_report_debug_and_entry_clone() {
        let entry = ConflictEntry {
            mode: "default".to_string(),
            hotkey: Hotkey::new(Modifiers::CTRL, Key::A),
            existing: "binding: select_all".to_string(),
            new: "chord leader".to_string(),
        };
        let cloned = entry.clone();
        assert_eq!(entry, cloned);

        let report = ConflictReport {
            conflicts: vec![entry],
        };
        let debug = format!("{report:?}");
        assert!(debug.contains("ConflictReport"));
        assert!(debug.contains("default"));
    }

    /// skhd format requires text around " - "; leading whitespace is trimmed
    /// away, so " - a" becomes "- a" which is not skhd format and fails as
    /// an unknown key in plus-separated format.
    #[test]
    fn skhd_leading_space_dash_not_skhd() {
        // After trim, " - a" becomes "- a" which does not contain " - ".
        // Falls through to parse_plus which fails on "- a" as unknown key.
        let result = Hotkey::parse(" - a");
        assert!(result.is_err());
    }

    /// skhd format with empty modifier segment: explicit "  - a" where trim
    /// gives " - a"? No — after outer trim "  - a" becomes "- a". Verify.
    #[test]
    fn skhd_valid_no_modifier_text() {
        // A valid skhd string with no modifiers is not possible in practice
        // because the outer trim removes leading spaces. But "fn - a" is valid.
        let hk = Hotkey::parse("fn - a").unwrap();
        assert!(hk.modifiers.contains(Modifiers::FN));
        assert_eq!(hk.key, Key::A);
    }

    /// Parsing a string with unicode characters returns an appropriate error.
    #[test]
    fn parse_unicode_returns_error() {
        let result = Hotkey::parse("cmd+\u{00e9}"); // e with acute
        assert!(result.is_err());
        match result.unwrap_err() {
            AwaseError::InvalidHotkey(msg) => assert!(msg.contains("unknown key")),
            other => panic!("expected InvalidHotkey, got {other:?}"),
        }
    }

    /// KeyRemap with complex modifier remapping roundtrips through serde.
    #[test]
    fn remap_complex_modifier_serde() {
        let remap = remap::KeyRemap::new(
            Hotkey::new(Modifiers::HYPER, Key::Space),
            Hotkey::new(Modifiers::CMD | Modifiers::SHIFT, Key::F12),
        ).with_condition(Condition {
            app: Some("Safari|Chrome".to_string()),
            app_exclude: Some("Firefox".to_string()),
            title: Some("Dashboard".to_string()),
            display: Some(2),
        });

        let json = serde_json::to_string_pretty(&remap).unwrap();
        let deserialized: remap::KeyRemap = serde_json::from_str(&json).unwrap();
        assert_eq!(remap, deserialized);
    }

    /// Detect conflicts with multiple chords sharing the same leader.
    #[test]
    fn conflict_detection_same_leader_multiple_chords() {
        let mut mode = mode::KeyMode::new("default", true);
        mode.add_binding(binding::Binding::new(
            Hotkey::new(Modifiers::CTRL, Key::A),
            action::Action::command("select_all"),
        ));

        let chord1 = chord::KeyChord {
            leader: Hotkey::new(Modifiers::CTRL, Key::A),
            follower: Hotkey::new(Modifiers::NONE, Key::C),
            timeout_ms: 1000,
            action: action::Action::command("new_window"),
        };
        let chord2 = chord::KeyChord {
            leader: Hotkey::new(Modifiers::CTRL, Key::A),
            follower: Hotkey::new(Modifiers::NONE, Key::N),
            timeout_ms: 1000,
            action: action::Action::command("next_window"),
        };

        // Both chords conflict with the same binding.
        let report = detect_conflicts(&[&mode], &[chord1, chord2]);
        assert_eq!(report.conflicts.len(), 2);
        assert!(report.conflicts.iter().all(|c| c.hotkey == Hotkey::new(Modifiers::CTRL, Key::A)));
    }

    /// BindingMap::list_bindings respects mode isolation: only shows current mode.
    #[test]
    fn list_bindings_respects_current_mode() {
        let mut map = BindingMap::new();
        map.mode_mut("default").unwrap().add_binding(
            binding::Binding::new(
                Hotkey::parse("cmd+a").unwrap(),
                action::Action::command("default_action"),
            ),
        );

        let mut other = mode::KeyMode::new("other", true);
        other.add_binding(binding::Binding::new(
            Hotkey::parse("cmd+b").unwrap(),
            action::Action::command("other_b"),
        ));
        other.add_binding(binding::Binding::new(
            Hotkey::parse("cmd+c").unwrap(),
            action::Action::command("other_c"),
        ));
        map.add_mode(other);

        // Default mode: 1 binding.
        assert_eq!(map.list_bindings().len(), 1);

        // Switch to "other": 2 bindings.
        map.set_mode("other").unwrap();
        assert_eq!(map.list_bindings().len(), 2);
    }
}
