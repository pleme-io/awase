//! Chords the WORLD owns — the ones no application may bind.
//!
//! ## Why this is not `conflict`
//!
//! [`crate::conflict`] answers "do two of MY bindings collide". This answers
//! a different question: "is this chord mine to bind at all". They are not
//! the same relation and collapsing them loses the distinction that matters.
//!
//! `Ctrl+Space` is not *in conflict* with anything an app declared — it is
//! **unavailable**, because the operating system took it first. Binding it
//! produces no error, no warning, and no key press: the app waits for an
//! event the OS already consumed. That is the worst failure shape available,
//! because it looks exactly like a bug in the feature the key was bound to.
//!
//! ## Why it lives here rather than in each app
//!
//! Because it is a fact about the machine, not about any application, and a
//! fact copied into four apps is a fact that will be right in two of them.
//! pleme-io currently carries four keymap implementations; this is the class
//! of knowledge that has to sit under all of them.
//!
//! ## What "reserved" does NOT mean
//!
//! It does not mean "someone else uses it somewhere". Two apps that are never
//! focused together may share a chord freely — that is [`Scope`]'s job. A
//! chord is reserved when something OUTSIDE the app layer consumes the event
//! before any app sees it.

use std::collections::BTreeMap;

use crate::hotkey::{Hotkey, Key, Modifiers};

/// Who owns a reserved chord, and therefore why it cannot be bound.
///
/// Carried rather than dropped because the remedy differs: an operator can
/// rebind their window manager, and cannot rebind the OS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Owner {
    /// The operating system itself. Not reconfigurable in practice.
    OperatingSystem,
    /// The window manager — aerospace on darwin. An operator CAN change this,
    /// which is why the reason names it.
    WindowManager,
    /// The terminal emulator between the app and the keyboard.
    Terminal,
}

impl Owner {
    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Self::OperatingSystem => "the operating system",
            Self::WindowManager => "the window manager",
            Self::Terminal => "the terminal emulator",
        }
    }
}

/// Why one chord is unavailable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub owner: Owner,
    /// What the owner does with it — shown to an operator who asks why their
    /// binding does nothing.
    pub purpose: &'static str,
}

/// The chords something outside the app layer consumes.
#[derive(Debug, Clone, Default)]
pub struct Reserved {
    claims: BTreeMap<String, Claim>,
}

impl Reserved {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reserve `hotkey`. Keyed on the canonical `Display` form, so the same
    /// chord written two ways cannot occupy two entries.
    #[must_use]
    pub fn claim(mut self, hotkey: Hotkey, owner: Owner, purpose: &'static str) -> Self {
        self.claims
            .insert(hotkey.display(), Claim { owner, purpose });
        self
    }

    /// Who owns `hotkey`, if anyone.
    #[must_use]
    pub fn claim_on(&self, hotkey: &Hotkey) -> Option<&Claim> {
        self.claims.get(&hotkey.display())
    }

    /// Is this chord available to an application?
    #[must_use]
    pub fn is_available(&self, hotkey: &Hotkey) -> bool {
        self.claim_on(hotkey).is_none()
    }

    /// Every reserved chord, canonical spelling first.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Claim)> {
        self.claims.iter().map(|(k, v)| (k.as_str(), v))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.claims.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.claims.is_empty()
    }

    /// Explain a refusal, for an operator who asks why their binding is dead.
    ///
    /// Returns `None` when the chord is theirs to bind — so a caller writes
    /// `if let Some(why) = reserved.refuse(&hk)` and cannot forget the happy
    /// path.
    #[must_use]
    pub fn refuse(&self, hotkey: &Hotkey) -> Option<String> {
        let claim = self.claim_on(hotkey)?;
        let mut s = String::with_capacity(96);
        s.push_str(&hotkey.display());
        s.push_str(" is taken by ");
        s.push_str(claim.owner.describe());
        s.push_str(" (");
        s.push_str(claim.purpose);
        s.push_str(") — an application binding it would never fire");
        Some(s)
    }

    /// The pleme-io fleet's reserved set on darwin.
    ///
    /// **Sourced from what the fleet actually declares**, not from a guess
    /// about what macOS might take: `alt-hjkl` and `alt-shift-hjkl` are
    /// aerospace's focus and move bindings in
    /// `nix/profiles/darwin-developer/home/window-management.nix`, which is
    /// the file that puts them beyond an app's reach.
    ///
    /// **The movement grammar this protects:** `hjkl` is DIRECTION everywhere
    /// in the fleet, and the PREFIX is the scope — `alt-` moves between OS
    /// windows, `<C-w>` between editor panes. Three places already agree on
    /// that by convention. Reserving the OS half is what keeps an app from
    /// quietly claiming a direction key that belongs to the layer above it.
    #[must_use]
    pub fn fleet_darwin() -> Self {
        let mut r = Self::new();
        for (key, what) in [
            (Key::H, "focus the window to the left"),
            (Key::J, "focus the window below"),
            (Key::K, "focus the window above"),
            (Key::L, "focus the window to the right"),
        ] {
            r = r.claim(Hotkey::new(Modifiers::ALT, key), Owner::WindowManager, what);
            r = r.claim(
                Hotkey::new(Modifiers::ALT | Modifiers::SHIFT, key),
                Owner::WindowManager,
                "move the window in that direction",
            );
        }
        // The operator's own example, and the reason this module exists.
        r = r.claim(
            Hotkey::new(Modifiers::CTRL, Key::Space),
            Owner::OperatingSystem,
            "application/input-source switching",
        );
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reserved_chord_is_not_available() {
        let r = Reserved::fleet_darwin();
        let alt_h = Hotkey::new(Modifiers::ALT, Key::H);
        assert!(!r.is_available(&alt_h));
        assert!(r.claim_on(&alt_h).is_some());
    }

    #[test]
    fn an_unreserved_chord_is_available() {
        // `<C-w>` is the editor-pane prefix and must stay bindable — the
        // whole point of scoping by prefix rather than forbidding hjkl.
        let r = Reserved::fleet_darwin();
        assert!(r.is_available(&Hotkey::new(Modifiers::CTRL, Key::W)));
        assert!(r.is_available(&Hotkey::new(Modifiers::NONE, Key::H)));
    }

    #[test]
    fn the_whole_movement_row_is_claimed_in_both_forms() {
        // Focus AND move. Reserving only the focus half would let an app
        // claim `alt-shift-j`, which is just as dead.
        let r = Reserved::fleet_darwin();
        for key in [Key::H, Key::J, Key::K, Key::L] {
            assert!(
                !r.is_available(&Hotkey::new(Modifiers::ALT, key)),
                "alt-{key:?}"
            );
            assert!(
                !r.is_available(&Hotkey::new(Modifiers::ALT | Modifiers::SHIFT, key)),
                "alt-shift-{key:?}",
            );
        }
    }

    #[test]
    fn ctrl_space_is_reserved_to_the_os() {
        // The operator's own example: taken by application switching, so an
        // app binding it waits for an event that never arrives.
        let r = Reserved::fleet_darwin();
        let hk = Hotkey::new(Modifiers::CTRL, Key::Space);
        assert_eq!(
            r.claim_on(&hk).map(|c| c.owner),
            Some(Owner::OperatingSystem)
        );
    }

    #[test]
    fn a_refusal_says_who_took_it_and_what_for() {
        // The message an operator reads when their binding does nothing. A
        // bare "reserved" would leave them exactly as stuck.
        let r = Reserved::fleet_darwin();
        let why = r
            .refuse(&Hotkey::new(Modifiers::ALT, Key::J))
            .expect("alt-j is reserved");
        assert!(why.contains("window manager"), "{why}");
        assert!(why.contains("below"), "names what it does: {why}");
        assert!(why.contains("never fire"), "says what happens: {why}");
    }

    #[test]
    fn refusing_an_available_chord_returns_none() {
        let r = Reserved::fleet_darwin();
        assert!(r.refuse(&Hotkey::new(Modifiers::CTRL, Key::W)).is_none());
    }

    #[test]
    fn the_same_chord_written_two_ways_is_one_entry() {
        // Keyed on the canonical Display form. Two spellings occupying two
        // entries is the same class of defect `KeyCombo` had, where a chord
        // could be present and unfindable at once.
        let a = Hotkey::parse("cmd+p").expect("parse");
        let b = Hotkey::parse("super + p").expect("parse");
        let r = Reserved::new().claim(a, Owner::OperatingSystem, "x");
        assert!(!r.is_available(&b), "one chord, one entry");
        assert_eq!(r.len(), 1);
    }
}

#[cfg(test)]
mod binding_refusal {
    use super::*;
    use crate::{Action, BindRefusal, Binding, KeyMode};

    fn mode() -> KeyMode<Action> {
        KeyMode::new("normal", false)
    }

    #[test]
    fn binding_a_reserved_chord_is_refused_with_a_reason() {
        // The load-bearing one. Without this, an author binds `alt-j`, the
        // app reports success, and the key does nothing forever — because
        // the window manager consumed the event before the app saw it.
        let r = Reserved::fleet_darwin();
        let mut m = mode();
        let b = Binding::new(
            Hotkey::new(Modifiers::ALT, Key::J),
            Action::Command("noop".into()),
        );
        match m.bind_available(b, &r) {
            Err(BindRefusal::Reserved(why)) => {
                assert!(why.contains("window manager"), "{why}");
            }
            other => panic!("a reserved chord must be refused, got {other:?}"),
        }
        assert_eq!(m.len(), 0, "and nothing is inserted");
    }

    #[test]
    #[allow(non_snake_case)]
    fn an_in_app_collision_is_a_DIFFERENT_refusal() {
        // Distinct variants, because the remedies differ: this one the author
        // can arbitrate; a reserved chord they cannot.
        let r = Reserved::fleet_darwin();
        let mut m = mode();
        let hk = Hotkey::new(Modifiers::CTRL, Key::W);
        m.bind_available(Binding::new(hk, Action::Command("noop".into())), &r)
            .expect("first bind is fine");
        match m.bind_available(Binding::new(hk, Action::Command("noop".into())), &r) {
            Err(BindRefusal::AlreadyBound(_)) => {}
            other => panic!("a collision must be AlreadyBound, got {other:?}"),
        }
    }

    #[test]
    fn an_available_chord_binds() {
        // The editor-pane prefix must still work — scoping by prefix is the
        // whole reason `hjkl` itself is not forbidden.
        let r = Reserved::fleet_darwin();
        let mut m = mode();
        assert!(
            m.bind_available(
                Binding::new(
                    Hotkey::new(Modifiers::CTRL, Key::W),
                    Action::Command("noop".into())
                ),
                &r,
            )
            .is_ok()
        );
        assert_eq!(m.len(), 1);
    }
}
