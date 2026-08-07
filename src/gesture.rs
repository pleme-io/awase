//! `Gesture` — a chord as a VALUE, not a spelling.
//!
//! # Why this exists
//!
//! The fleet carried chords as `&'static str` between components that spell
//! strings differently, and every measured keybinding defect was one class:
//! *a chord was carried as text between two components that spell text
//! differently.* Four of those defects lived in the fleet atlas itself and
//! were invisible until something tried to parse it:
//!
//! | Authored | What happened |
//! |---|---|
//! | `"M-?"` | unparseable — `Hotkey::parse` fails; a resolving consumer panics |
//! | `"M-y"` / `"M-Y"` | collapse to ONE chord under case-folding, so two intents alias |
//! | `"C-x e"` | truncated — the trailing key is dropped |
//! | `"X-c"` | unknown modifier letter silently discarded, leaving bare `c` |
//!
//! None of those is representable here. A `Gesture` is built from typed
//! [`Key`] variants and [`Modifiers`] flags, so an unspellable chord is a
//! compile error rather than a runtime surprise, and the multi-key case has a
//! variant instead of being a parse failure every consumer must handle.
//!
//! # The seam
//!
//! **A string may be parsed *into* a gesture, and a gesture may be rendered
//! *out* to a string. No two components ever exchange one.** Parsing happens
//! at an edge that reads a foreign config; rendering happens at an edge that
//! writes one (a `tmux.conf`, a zsh `bindkey`). Everything between them moves
//! values.
//!
//! # Const-constructible, and why that mattered
//!
//! The atlas records that its fields must be `&'static str` so the whole
//! thing can be a `const`. That objection was already false when it was
//! written: [`Hotkey::new`] is `const fn`, [`Modifiers`] are associated
//! consts on a `u8` newtype, and [`Key`] is a plain C-like enum. Every
//! constructor here is `const`, so an atlas built from gestures is still a
//! `const fn`.

use std::fmt;

use serde::{Serialize, Serializer};

use crate::hotkey::{Hotkey, Key, Modifiers};

/// A chord, or a sequence of chords, as a typed value.
///
/// Three states, because the domain has three — and the first one is the
/// reason this is an enum rather than a bare [`Hotkey`]. A tier-0 `bare()`
/// atlas declares every intent while binding none of them; spelling that as
/// `""` makes "unbound" a *string value* that every consumer must remember to
/// check, and forgetting is silent. [`Gesture::Unbound`] makes it a variant
/// the compiler can see.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gesture {
    /// Declared but deliberately unbound — the tier-0 floor. Replaces `""`.
    Unbound,
    /// One chord: zero or more modifiers plus one key.
    Single(Hotkey),
    /// A key SEQUENCE — press, release, press again (`C-x e`).
    ///
    /// Distinct from [`Self::Single`] with many modifiers, which is one
    /// simultaneous press. Without this variant the atlas's own
    /// `edit_in_editor` intent is unrepresentable, which is exactly why it
    /// was silently truncated to its first chord.
    Sequence(&'static [Hotkey]),
}

impl Gesture {
    /// A bare key with no modifiers — `q`, `escape`, `/`.
    #[must_use]
    pub const fn key(key: Key) -> Self {
        Self::Single(Hotkey::new(Modifiers::NONE, key))
    }

    /// A modified chord — `ctrl+r`, `cmd+shift+g`.
    #[must_use]
    pub const fn chord(modifiers: Modifiers, key: Key) -> Self {
        Self::Single(Hotkey::new(modifiers, key))
    }

    /// A key sequence — `C-x e`.
    ///
    /// The slice is `&'static`, so callers bind it as a `const` first:
    ///
    /// ```
    /// # use awase::{Gesture, Hotkey, Key, Modifiers};
    /// const EDIT_IN_EDITOR: &[Hotkey] = &[
    ///     Hotkey::new(Modifiers::CTRL, Key::X),
    ///     Hotkey::new(Modifiers::NONE, Key::E),
    /// ];
    /// const G: Gesture = Gesture::seq(EDIT_IN_EDITOR);
    /// ```
    ///
    /// An inline `Gesture::seq(&[...])` does **not** compile: rvalue static
    /// promotion deliberately excludes `const fn` calls, so the temporary is
    /// dropped while borrowed. That is a feature here rather than friction —
    /// the atlas is a `const` already, and requiring the named const keeps a
    /// sequence from being conjured in a function body where its lifetime
    /// would be a surprise.
    #[must_use]
    pub const fn seq(chords: &'static [Hotkey]) -> Self {
        Self::Sequence(chords)
    }

    /// Whether this gesture binds anything at all.
    #[must_use]
    pub const fn is_bound(self) -> bool {
        !matches!(self, Self::Unbound)
    }

    /// The chord that BEGINS this gesture — the dispatch entry point.
    ///
    /// For a [`Self::Sequence`] this is the leader, which is what a
    /// dispatcher matches first before entering a pending state. `None` for
    /// [`Self::Unbound`], and for the degenerate empty sequence (which
    /// [`Self::seq`] cannot prevent, since a `const fn` cannot assert).
    #[must_use]
    pub fn first(self) -> Option<Hotkey> {
        match self {
            Self::Unbound => None,
            Self::Single(h) => Some(h),
            Self::Sequence(hs) => hs.first().copied(),
        }
    }

    /// Every chord in this gesture, in press order.
    ///
    /// Allocates (one or two elements) rather than returning a borrowed
    /// slice, and that is deliberate. The zero-copy shape would have to
    /// answer `&[]` for [`Self::Single`], which has no backing slice to
    /// borrow — an empty answer for a *bound* gesture, i.e. precisely the
    /// silently-wrong-answer class this type exists to eliminate. A
    /// two-element `Vec` on a path that runs at authoring time, not per
    /// keystroke, is the right trade.
    #[must_use]
    pub fn chords(self) -> Vec<Hotkey> {
        match self {
            Self::Unbound => Vec::new(),
            Self::Single(h) => vec![h],
            Self::Sequence(hs) => hs.to_vec(),
        }
    }

    /// Render to the canonical long form — `ctrl+r`, `ctrl+x e`, `escape`.
    ///
    /// This is deliberately **the same string the atlas's `normalize_chord`
    /// produces** for the equivalent short form, which is what lets the two
    /// representations be held to each other by a parity test during the
    /// migration instead of being trusted to agree.
    ///
    /// [`Self::Unbound`] renders as the empty string, matching what a tier-0
    /// atlas emits today.
    #[must_use]
    pub fn to_long_form(self) -> String {
        use fmt::Write as _;
        match self {
            Self::Unbound => String::new(),
            Self::Single(h) => h.display(),
            Self::Sequence(hs) => {
                let mut out = String::new();
                for (i, h) in hs.iter().enumerate() {
                    if i > 0 {
                        out.push(' ');
                    }
                    // `write!` into a String is infallible; the Result is
                    // discarded rather than unwrapped so this stays panic-free.
                    let _ = write!(out, "{}", h.display());
                }
                out
            }
        }
    }
}

impl fmt::Display for Gesture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_long_form())
    }
}

/// Serializes to the canonical long-form STRING, not a tagged enum.
///
/// Two reasons it is a string. Diagnostics that emit the atlas today
/// (`frost_status`, `config-show`) print chord strings, and a tagged enum
/// would change every one of those outputs. And the atlas's own
/// catalog-reflection test counts serde object keys to prove no field escapes
/// the intent catalog — a shape that keeps working only while each field
/// serializes to one scalar.
impl Serialize for Gesture {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_long_form())
    }
}

// Deliberately NO `Deserialize`. The atlas is code, not config: its value is
// being the single hand-edited source every fleet app shares, and letting an
// operator override it through YAML would defeat that. A runtime-configurable
// app parses operator input into a `Hotkey` at its own edge — that is the
// parse-in half of the seam, and it belongs to the app, not to the atlas.

#[cfg(test)]
mod tests {
    use super::*;

    /// `C-x e` — the atlas's own multi-key intent, as a named const
    /// because `seq` takes `&'static [Hotkey]` (see its doc).
    const EDIT_IN_EDITOR: &[Hotkey] = &[
        Hotkey::new(Modifiers::CTRL, Key::X),
        Hotkey::new(Modifiers::NONE, Key::E),
    ];

    #[test]
    fn a_bare_key_renders_without_modifiers() {
        assert_eq!(Gesture::key(Key::Q).to_long_form(), "q");
        assert_eq!(Gesture::key(Key::Escape).to_long_form(), "escape");
        assert_eq!(Gesture::key(Key::Return).to_long_form(), "return");
    }

    #[test]
    fn a_modified_chord_renders_long_form() {
        assert_eq!(
            Gesture::chord(Modifiers::CTRL, Key::R).to_long_form(),
            "ctrl+r"
        );
    }

    /// The four atlas defects this type exists to make unrepresentable.
    /// Each row is a chord the string atlas got wrong; here each is a
    /// distinct, correctly-rendered value.
    #[test]
    fn the_four_broken_atlas_chords_are_now_representable() {
        // "M-?" — unparseable as a string; here it is just a value.
        let help = Gesture::chord(Modifiers::ALT.with(Modifiers::SHIFT), Key::Slash);
        assert!(help.is_bound());

        // "M-y" / "M-Y" — aliased under case-folding. Distinct here, and the
        // assertion is on the VALUES, because that is what a conflict check
        // compares.
        let copy = Gesture::chord(Modifiers::ALT, Key::Y);
        let paste = Gesture::chord(Modifiers::ALT.with(Modifiers::SHIFT), Key::Y);
        assert_ne!(copy, paste, "shift must distinguish these two intents");

        // "C-x e" — truncated to its leader. Both chords survive.
        let edit = Gesture::seq(EDIT_IN_EDITOR);
        assert_eq!(edit.to_long_form(), "ctrl+x e");
        assert_eq!(edit.chords().len(), 2);

        // "X-c" — an unknown modifier letter cannot be written at all: there
        // is no `Modifiers::X`, so this class is a compile error, not a test.
    }

    #[test]
    fn a_sequence_leader_is_the_dispatch_entry() {
        let g = Gesture::seq(EDIT_IN_EDITOR);
        assert_eq!(g.first(), Some(Hotkey::new(Modifiers::CTRL, Key::X)));
    }

    #[test]
    fn unbound_is_not_bound_and_renders_empty() {
        assert!(!Gesture::Unbound.is_bound());
        assert_eq!(Gesture::Unbound.to_long_form(), "");
        assert_eq!(Gesture::Unbound.first(), None);
        assert!(Gesture::Unbound.chords().is_empty());
        // …and every real gesture IS bound, so the predicate is not vacuous.
        assert!(Gesture::key(Key::Q).is_bound());
    }

    #[test]
    fn chords_is_uniform_across_both_bound_shapes() {
        assert_eq!(Gesture::key(Key::Q).chords().len(), 1);
        assert_eq!(
            Gesture::seq(EDIT_IN_EDITOR)
            .chords()
            .len(),
            2
        );
    }

    #[test]
    fn serializes_to_a_bare_string_not_a_tagged_enum() {
        // Diagnostics print these; a tagged enum would change every one.
        let j = serde_json::to_string(&Gesture::chord(Modifiers::CTRL, Key::R)).expect("serializes");
        assert_eq!(j, "\"ctrl+r\"");
        let u = serde_json::to_string(&Gesture::Unbound).expect("serializes");
        assert_eq!(u, "\"\"");
    }

    /// The migration-safety property: a gesture's long form is exactly what
    /// the atlas's own normalizer produces from the equivalent short form.
    /// This is what the ishou↔awase parity gate rests on.
    #[test]
    fn long_form_matches_the_atlas_normalizer_output_shape() {
        // ishou's `normalize_chord("C-r")` == "ctrl+r"
        assert_eq!(
            Gesture::chord(Modifiers::CTRL, Key::R).to_long_form(),
            "ctrl+r"
        );
        // …and `normalize_chord("C-x e")` == "ctrl+x e"
        assert_eq!(
            Gesture::seq(EDIT_IN_EDITOR)
            .to_long_form(),
            "ctrl+x e"
        );
    }

    #[test]
    fn every_constructor_is_usable_in_a_const() {
        // The whole point of the type: an atlas stays a `const fn`.
        const BARE: Gesture = Gesture::key(Key::Q);
        const CHORD: Gesture = Gesture::chord(Modifiers::CTRL, Key::R);
        const SEQ: Gesture = Gesture::seq(EDIT_IN_EDITOR);
        const UNBOUND: Gesture = Gesture::Unbound;
        assert!(BARE.is_bound() && CHORD.is_bound() && SEQ.is_bound());
        assert!(!UNBOUND.is_bound());
    }

    #[test]
    fn a_gesture_round_trips_through_the_hotkey_parser() {
        // The parse-in half of the seam: rendering out and parsing back must
        // land on the same value, or the two edges disagree.
        for g in [
            Gesture::key(Key::Q),
            Gesture::key(Key::Escape),
            Gesture::chord(Modifiers::CTRL, Key::R),
            Gesture::chord(Modifiers::CMD.with(Modifiers::SHIFT), Key::G),
        ] {
            let rendered = g.to_long_form();
            let parsed = Hotkey::parse(&rendered)
                .unwrap_or_else(|e| panic!("`{rendered}` did not parse back: {e:?}"));
            assert_eq!(Gesture::Single(parsed), g, "round trip changed `{rendered}`");
        }
    }
}
