use crate::binding::Binding;
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

/// Detect conflicts within a set of **built** modes and chords.
///
/// Detected here:
/// - A chord leader that collides with a regular binding in the same mode.
/// - The same chord (leader *and* follower) declared twice.
///
/// # What this function structurally CANNOT see
///
/// **The same hotkey bound twice in one mode.** This is not an omission that
/// could be fixed here — it is unreachable from the signature. A [`KeyMode`]
/// holds `bindings: HashMap<Hotkey, Binding>`, so the second `add_binding`
/// has already *replaced* the first by the time any checker runs. The
/// duplicate is gone, and with it the evidence.
///
/// This function's doc used to claim it caught that class, qualified with
/// "only possible with external config merging". That qualifier was
/// backwards: `HashMap` deduplication is not what makes the case rare, it is
/// what makes the case **invisible** — and silently losing a binding is the
/// common authoring mistake, not an exotic one.
///
/// To catch it, check the bindings **before** they are inserted, with
/// [`detect_duplicate_bindings`]. banken discovered this the hard way and
/// hand-rolled that check over `add_binding`'s returned `Some(prev)`; this is
/// that mechanism, lifted so nobody else has to rediscover it.
#[must_use]
pub fn detect_conflicts(modes: &[&KeyMode], chords: &[KeyChord]) -> ConflictReport {
    let mut report = ConflictReport::default();

    // Chord leaders vs mode bindings.
    for mode in modes {
        for chord in chords {
            if let Some(existing) = mode.bindings.get(&chord.leader) {
                report.conflicts.push(ConflictEntry {
                    mode: mode.name.clone(),
                    hotkey: chord.leader,
                    existing: format!("binding: {:?}", existing.action),
                    new: format!("chord leader (follower: {})", chord.follower),
                });
            }
        }
    }

    // A chord declared twice. Visible here — unlike the binding case — because
    // `chords` arrives as a slice, not a map, so both copies are still present.
    for (i, chord) in chords.iter().enumerate() {
        for prior in &chords[..i] {
            if prior.leader == chord.leader && prior.follower == chord.follower {
                report.conflicts.push(ConflictEntry {
                    mode: "*".to_owned(),
                    hotkey: chord.leader,
                    existing: format!("chord: {:?}", prior.action),
                    new: format!("duplicate chord: {:?}", chord.action),
                });
            }
        }
    }

    report
}

/// Detect duplicate bindings in a list that has **not yet been inserted** into
/// a [`KeyMode`].
///
/// This is the peer [`detect_conflicts`] cannot be: it runs on the authored
/// list, where both copies still exist, rather than on the map that has
/// already collapsed them.
///
/// Two bindings conflict when they share a hotkey **and** carry the same
/// condition — then one of them can never fire, and nothing at runtime will
/// say so. Bindings on one hotkey with *different* conditions are left alone
/// deliberately: conditional binding is the feature, and flagging it would
/// make the checker cry wolf on correct code.
///
/// ```
/// # use awase::{conflict::detect_duplicate_bindings, Action, Binding, Hotkey, Key, Modifiers};
/// let hk = Hotkey::new(Modifiers::CTRL, Key::A);
/// let report = detect_duplicate_bindings("default", &[
///     Binding::new(hk, Action::command("select_all")),
///     Binding::new(hk, Action::command("select_none")),
/// ]);
/// assert!(!report.is_clean(), "the second binding would be silently lost");
/// ```
#[must_use]
pub fn detect_duplicate_bindings(mode: &str, bindings: &[Binding]) -> ConflictReport {
    let mut report = ConflictReport::default();
    for (i, binding) in bindings.iter().enumerate() {
        for prior in &bindings[..i] {
            if prior.hotkey == binding.hotkey && prior.condition == binding.condition {
                report.conflicts.push(ConflictEntry {
                    mode: mode.to_owned(),
                    hotkey: binding.hotkey,
                    existing: format!("binding: {:?}", prior.action),
                    new: format!("shadowed by: {:?}", binding.action),
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

    /// The class the old doc claimed and the signature forbids. Pinned as a
    /// *characterization* test: it asserts the limitation is real, so nobody
    /// "fixes" `detect_conflicts` by adding a check that cannot work.
    #[test]
    fn a_duplicate_binding_is_invisible_once_inserted_into_a_mode() {
        let mut mode = KeyMode::new("default", true);
        let first = mode.add_binding(Binding::new(ctrl_a(), Action::command("select_all")));
        let displaced = mode.add_binding(Binding::new(ctrl_a(), Action::command("select_none")));

        assert!(first.is_none(), "nothing was there to displace");
        assert!(
            displaced.is_some(),
            "the returned Some(prev) IS the duplicate — it is the only evidence that survives"
        );
        assert_eq!(mode.bindings.len(), 1, "the HashMap collapsed them");

        // …and so the built-mode checker cannot see it. Not a bug here: the
        // evidence was destroyed before this function was called.
        assert!(detect_conflicts(&[&mode], &[]).is_clean());
    }

    /// …and the peer that CAN see it, because it runs before insertion.
    #[test]
    fn a_duplicate_binding_is_caught_before_insertion() {
        let report = detect_duplicate_bindings(
            "default",
            &[
                Binding::new(ctrl_a(), Action::command("select_all")),
                Binding::new(ctrl_a(), Action::command("select_none")),
            ],
        );
        assert_eq!(report.conflicts.len(), 1);
        assert_eq!(report.conflicts[0].hotkey, ctrl_a());
        assert_eq!(report.conflicts[0].mode, "default");
    }

    #[test]
    fn distinct_hotkeys_are_not_duplicates() {
        // Non-vacuity: the checker must not flag everything.
        let report = detect_duplicate_bindings(
            "default",
            &[
                Binding::new(ctrl_a(), Action::command("select_all")),
                Binding::new(
                    Hotkey::new(Modifiers::CTRL, Key::B),
                    Action::command("other"),
                ),
            ],
        );
        assert!(report.is_clean());
    }

    #[test]
    fn one_hotkey_under_two_different_conditions_is_not_a_conflict() {
        // Conditional binding is the feature. Flagging this would make the
        // checker cry wolf on correct code, so it is deliberately allowed.
        use crate::condition::Condition;
        let scoped_to = |app: &str| Condition {
            app: Some(app.to_owned()),
            ..Condition::default()
        };
        let a = Binding::new(ctrl_a(), Action::command("in_editor"))
            .with_condition(scoped_to("com.editor"));
        let b = Binding::new(ctrl_a(), Action::command("in_browser"))
            .with_condition(scoped_to("com.browser"));
        assert!(detect_duplicate_bindings("default", &[a, b]).is_clean());
    }

    #[test]
    fn the_same_chord_declared_twice_is_caught() {
        let dup = || KeyChord {
            leader: ctrl_a(),
            follower: Hotkey::new(Modifiers::NONE, Key::C),
            timeout_ms: 1000,
            action: Action::command("new_window"),
        };
        let mode = KeyMode::new("default", true);
        let report = detect_conflicts(&[&mode], &[dup(), dup()]);
        assert_eq!(report.conflicts.len(), 1, "a chord declared twice");
    }

    #[test]
    fn two_chords_sharing_a_leader_are_not_a_conflict() {
        // Non-vacuity for the chord check: sharing a leader with a DIFFERENT
        // follower is what a chord tree IS (`C-x e`, `C-x s`), not a clash.
        let mode = KeyMode::new("default", true);
        let mk = |k| KeyChord {
            leader: ctrl_a(),
            follower: Hotkey::new(Modifiers::NONE, k),
            timeout_ms: 1000,
            action: Action::command("x"),
        };
        assert!(detect_conflicts(&[&mode], &[mk(Key::C), mk(Key::D)]).is_clean());
    }

    #[test]
    fn chord_leader_conflicts_with_binding() {
        let mut mode = KeyMode::new("default", true);
        drop(mode.add_binding(Binding::new(ctrl_a(), Action::command("select_all"))));

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
        drop(mode.add_binding(Binding::new(
            Hotkey::new(Modifiers::CMD, Key::H),
            Action::command("focus_west"),
        )));

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
        drop(resize.add_binding(Binding::new(ctrl_a(), Action::command("shrink"))));

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

    // ── Additional conflict detection tests ─────────────────────────

    #[test]
    fn no_modes_no_conflicts() {
        let report = detect_conflicts(&[], &[]);
        assert!(report.is_clean());
    }

    #[test]
    fn chords_only_no_modes_no_conflicts() {
        let chord = KeyChord {
            leader: ctrl_a(),
            follower: Hotkey::new(Modifiers::NONE, Key::C),
            timeout_ms: 1000,
            action: Action::command("test"),
        };
        let report = detect_conflicts(&[], &[chord]);
        assert!(report.is_clean());
    }

    #[test]
    fn modes_only_no_chords_no_conflicts() {
        let mut mode = KeyMode::new("default", true);
        drop(mode.add_binding(Binding::new(ctrl_a(), Action::command("test"))));
        let report = detect_conflicts(&[&mode], &[]);
        assert!(report.is_clean());
    }

    #[test]
    fn same_chord_conflicts_in_multiple_modes() {
        let mut default = KeyMode::new("default", true);
        drop(default.add_binding(Binding::new(ctrl_a(), Action::command("select_all"))));

        let mut resize = KeyMode::new("resize", false);
        drop(resize.add_binding(Binding::new(ctrl_a(), Action::command("shrink"))));

        let chord = KeyChord {
            leader: ctrl_a(),
            follower: Hotkey::new(Modifiers::NONE, Key::C),
            timeout_ms: 1000,
            action: Action::command("new_window"),
        };

        let report = detect_conflicts(&[&default, &resize], &[chord]);
        assert_eq!(report.conflicts.len(), 2);
        let modes: Vec<&str> = report.conflicts.iter().map(|c| c.mode.as_str()).collect();
        assert!(modes.contains(&"default"));
        assert!(modes.contains(&"resize"));
    }

    #[test]
    fn multiple_chords_multiple_conflicts() {
        let mut mode = KeyMode::new("default", true);
        drop(mode.add_binding(Binding::new(ctrl_a(), Action::command("a"))));
        drop(mode.add_binding(Binding::new(
            Hotkey::new(Modifiers::CTRL, Key::B),
            Action::command("b"),
        )));

        let chord1 = KeyChord {
            leader: ctrl_a(),
            follower: Hotkey::new(Modifiers::NONE, Key::C),
            timeout_ms: 1000,
            action: Action::command("chord_a"),
        };
        let chord2 = KeyChord {
            leader: Hotkey::new(Modifiers::CTRL, Key::B),
            follower: Hotkey::new(Modifiers::NONE, Key::C),
            timeout_ms: 1000,
            action: Action::command("chord_b"),
        };

        let report = detect_conflicts(&[&mode], &[chord1, chord2]);
        assert_eq!(report.conflicts.len(), 2);
    }

    #[test]
    fn conflict_entry_contains_hotkey() {
        let mut mode = KeyMode::new("default", true);
        drop(mode.add_binding(Binding::new(ctrl_a(), Action::command("select_all"))));

        let chord = KeyChord {
            leader: ctrl_a(),
            follower: Hotkey::new(Modifiers::NONE, Key::C),
            timeout_ms: 1000,
            action: Action::command("new_window"),
        };

        let report = detect_conflicts(&[&mode], &[chord]);
        assert_eq!(report.conflicts[0].hotkey, ctrl_a());
        assert!(report.conflicts[0].existing.contains("select_all"));
        assert!(report.conflicts[0].new.contains("chord leader"));
    }

    #[test]
    fn conflict_report_is_clean_true_when_empty() {
        let report = ConflictReport::default();
        assert!(report.is_clean());
    }

    #[test]
    fn conflict_report_is_clean_false_with_entries() {
        let report = ConflictReport {
            conflicts: vec![ConflictEntry {
                mode: "default".to_string(),
                hotkey: ctrl_a(),
                existing: "a".to_string(),
                new: "b".to_string(),
            }],
        };
        assert!(!report.is_clean());
    }
}
