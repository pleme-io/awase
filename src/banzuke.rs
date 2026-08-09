//! Banzuke (番付) — the ranked declaration chart.
//!
//! A sumo banzuke is a published ranking: every wrestler is on it, in rank
//! order, and you can see who holds each rank *and* who is ranked below them.
//! That is exactly the shape a keymap assembled from several sources needs and
//! does not have.
//!
//! # What it is for
//!
//! [`KeyMode`] is a `HashMap<Hotkey, Binding>`, so binding is destructive:
//! the second declaration for a chord erases the first, and with it the
//! evidence that there ever was a first. [`crate::detect_duplicate_bindings`]
//! exists precisely because a post-hoc checker cannot recover what the insert
//! destroyed — it has to be handed the authored list *before* any of it lands.
//!
//! A `Banzuke` is that list, kept. Nothing is ever overwritten; declarations
//! accumulate, and the effective binding is DERIVED from them by
//! [`Banzuke::holder`]. Three things follow that a map cannot give you:
//!
//! - **Displacement is queryable, not an event to remember.** The losers are
//!   still there ([`Holding::outranked`]); you do not have to have logged the
//!   collision at the moment it happened.
//! - **Provenance is total.** A [`Declaration`] cannot be built without an
//!   [`Origin`], so "a binding of unknown source" has no constructor.
//! - **Precedence is a rule, not an accident of iteration order.** Which
//!   declaration wins stops depending on the order an array happened to list
//!   the plugins in.
//!
//! # The floor
//!
//! Rank alone would have made escriba's bug *correct*: a package outranks the
//! builtin layer, so a bundled plugin taking `<C-h>` from backspace is
//! ordinary layering. [`Declaration::floor`] is the second axis — the lowest
//! rank permitted to take this chord away. A core verb declares a floor of
//! [`Rank::Operator`], which says: the human may rebind this, nothing that
//! ships in the box may.
//!
//! ```
//! use awase::{Banzuke, Binding, Declaration, Hotkey, Origin, Rank, Action};
//!
//! let bs = Hotkey::parse("ctrl+h").unwrap();
//! let mut b = Banzuke::new();
//! b.enter(
//!     Declaration::new(Binding::new(bs, Action::command("backspace")), Origin::builtin())
//!         .protected_below(Rank::Operator),
//! );
//! // A bundled package may not take it.
//! b.enter(Declaration::new(
//!     Binding::new(bs, Action::command("snippet.jump-prev")),
//!     Origin::package("escriba-luasnip"),
//! ));
//! assert_eq!(b.holder(&bs).unwrap().binding.action, Action::command("backspace"));
//!
//! // The human may.
//! b.enter(Declaration::new(
//!     Binding::new(bs, Action::command("my-thing")),
//!     Origin::operator("rc.lisp"),
//! ));
//! assert_eq!(b.holder(&bs).unwrap().binding.action, Action::command("my-thing"));
//! ```

use std::collections::HashSet;

use crate::binding::Binding;
use crate::hotkey::Hotkey;
use crate::mode::KeyMode;
use crate::provenance::{Origin, Rank};

/// One authored binding, plus who authored it and how well defended it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration<A> {
    /// The binding itself. Untouched from [`crate::Binding`] — provenance
    /// lives out here rather than as a field on it, so the 23 consumers that
    /// construct `Binding` today are not disturbed.
    pub binding: Binding<A>,
    /// Who declared it.
    pub origin: Origin,
    /// The lowest rank permitted to displace this declaration.
    ///
    /// `None` is ordinary layering: anything strictly higher-ranked wins.
    /// `Some(r)` additionally demands the challenger be at least `r`, which
    /// is how a load-bearing builtin survives a package that outranks it.
    pub floor: Option<Rank>,
}

impl<A> Declaration<A> {
    /// Declare `binding` on behalf of `origin`, with ordinary layering.
    pub fn new(binding: Binding<A>, origin: Origin) -> Self {
        Self {
            binding,
            origin,
            floor: None,
        }
    }

    /// Defend this declaration against everything below `rank`.
    ///
    /// Use for the bindings whose absence is a broken editor rather than a
    /// changed one — the erase verbs, the key that leaves a mode. Naming the
    /// rank rather than a bare `protected` flag is what lets a distribution
    /// defend a chord against packages while still yielding to the operator.
    #[must_use]
    pub fn protected_below(mut self, rank: Rank) -> Self {
        self.floor = Some(rank);
        self
    }

    /// May `challenger` take this chord from this declaration?
    ///
    /// Two conditions, both required: strictly greater authority, and — when
    /// a floor is declared — at least the floor. Equal rank does NOT displace,
    /// which is the deliberate difference from a `HashMap`: two declarations
    /// at the same rank are a genuine authoring conflict, and silently
    /// preferring whichever was entered second is what hid them.
    fn yields_to(&self, challenger: &Origin) -> bool {
        if challenger.rank <= self.origin.rank {
            return false;
        }
        self.floor.is_none_or(|f| challenger.rank >= f)
    }
}

/// Who holds a chord, and who wanted it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holding<'a, A> {
    /// The chord.
    pub hotkey: Hotkey,
    /// The declaration in effect.
    pub holder: &'a Declaration<A>,
    /// Every declaration that lost, in the order they were entered.
    ///
    /// Non-empty means somebody's authored binding is not firing. That is not
    /// automatically wrong — an operator override is exactly this shape — but
    /// it is always worth being able to see.
    pub outranked: Vec<&'a Declaration<A>>,
}

impl<A> Holding<'_, A> {
    /// Is anything losing this chord?
    #[must_use]
    pub fn is_contested(&self) -> bool {
        !self.outranked.is_empty()
    }

    /// Did a declaration lose to something that did NOT outrank it — i.e. an
    /// equal-rank conflict, where the winner is arbitrary?
    ///
    /// This is the case worth failing a build over. An operator overriding a
    /// package is intended; two packages at the same rank fighting over a
    /// chord means one of them silently does nothing.
    #[must_use]
    pub fn has_peer_conflict(&self) -> bool {
        self.outranked
            .iter()
            .any(|d| d.origin.rank == self.holder.origin.rank)
    }
}

/// The ranked chart of every declaration, from every source.
///
/// Append-only by construction: there is no method that removes or replaces a
/// declaration, so a binding cannot be lost — only outranked.
#[derive(Debug, Clone)]
pub struct Banzuke<A> {
    entries: Vec<Declaration<A>>,
}

impl<A> Default for Banzuke<A> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<A> Banzuke<A> {
    /// An empty chart.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enter a declaration onto the chart.
    ///
    /// Always accepted, never displaces anything. Adjudication happens at
    /// read time in [`Self::holder`] — which is what makes the losers
    /// survivable and the result independent of entry order.
    pub fn enter(&mut self, declaration: Declaration<A>) {
        self.entries.push(declaration);
    }

    /// Enter many.
    pub fn extend(&mut self, declarations: impl IntoIterator<Item = Declaration<A>>) {
        self.entries.extend(declarations);
    }

    /// Every declaration ever entered, in entry order.
    pub fn declarations(&self) -> impl Iterator<Item = &Declaration<A>> {
        self.entries.iter()
    }

    /// How many declarations are on the chart (NOT how many chords are bound).
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Is the chart empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The declaration in effect for `hotkey`.
    ///
    /// Walks entry order and keeps the incumbent unless a challenger
    /// [`Declaration::yields_to`] it. Deterministic and order-independent for
    /// distinct ranks; for equal ranks the FIRST entered holds, which is the
    /// deliberate inverse of `HashMap::insert`'s last-write-wins — a later
    /// peer cannot quietly take a chord, it shows up as a peer conflict.
    #[must_use]
    pub fn holder(&self, hotkey: &Hotkey) -> Option<&Declaration<A>> {
        let mut held: Option<&Declaration<A>> = None;
        for d in self.entries.iter().filter(|d| d.binding.hotkey == *hotkey) {
            held = match held {
                None => Some(d),
                Some(cur) if cur.yields_to(&d.origin) => Some(d),
                Some(cur) => Some(cur),
            };
        }
        held
    }

    /// The full picture for one chord: who holds it and who lost.
    #[must_use]
    pub fn holding(&self, hotkey: &Hotkey) -> Option<Holding<'_, A>> {
        let holder = self.holder(hotkey)?;
        let outranked = self
            .entries
            .iter()
            .filter(|d| d.binding.hotkey == *hotkey && !std::ptr::eq(*d, holder))
            .collect();
        Some(Holding {
            hotkey: *hotkey,
            holder,
            outranked,
        })
    }

    /// Every chord on the chart, each with its holder and its losers.
    ///
    /// In AUTHORED order — first declaration of each chord decides where it
    /// appears — which is both deterministic across runs and the order
    /// [`KeyMode::iter`]'s doc says a legend actually wants. Sorting instead
    /// would need `Ord` on [`Hotkey`], and adding ordering traits to a type
    /// 23 consumers share to get a stable chart here is the tail wagging the
    /// dog; entry order is already stable.
    #[must_use]
    pub fn chart(&self) -> Vec<Holding<'_, A>> {
        let mut seen: HashSet<Hotkey> = HashSet::new();
        let mut order: Vec<Hotkey> = Vec::new();
        for d in &self.entries {
            if seen.insert(d.binding.hotkey) {
                order.push(d.binding.hotkey);
            }
        }
        order.iter().filter_map(|k| self.holding(k)).collect()
    }

    /// Every chord where two declarations of EQUAL rank collide.
    ///
    /// The build-failing set. An operator overriding a package is intended
    /// layering; two packages fighting means one of them silently does
    /// nothing and nobody chose which.
    #[must_use]
    pub fn peer_conflicts(&self) -> Vec<Holding<'_, A>> {
        self.chart()
            .into_iter()
            .filter(Holding::has_peer_conflict)
            .collect()
    }
}

impl<A: Clone> Banzuke<A> {
    /// Resolve to the flat dispatch table.
    ///
    /// The hot path stays exactly what it was — a `HashMap` lookup in a
    /// [`KeyMode`] — so adopting the chart costs nothing at keypress time.
    /// The chart is consulted when ASSEMBLING a keymap and when explaining
    /// one, never when pressing a key.
    #[must_use]
    pub fn resolve(&self, name: impl Into<String>, passthrough: bool) -> KeyMode<A> {
        let mut mode = KeyMode::typed(name, passthrough);
        for h in self.chart() {
            drop(mode.add_binding(h.holder.binding.clone()));
        }
        mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::hotkey::{Key, Modifiers};

    fn hk(k: Key) -> Hotkey {
        Hotkey::new(Modifiers::CTRL, k)
    }

    fn decl(k: Key, action: &str, origin: Origin) -> Declaration<Action> {
        Declaration::new(Binding::new(hk(k), Action::command(action)), origin)
    }

    fn held(b: &Banzuke<Action>, k: Key) -> Action {
        b.holder(&hk(k)).unwrap().binding.action.clone()
    }

    #[test]
    fn a_higher_rank_takes_the_chord() {
        let mut b = Banzuke::new();
        b.enter(decl(Key::A, "builtin-thing", Origin::builtin()));
        b.enter(decl(Key::A, "user-thing", Origin::operator("rc")));
        assert_eq!(held(&b, Key::A), Action::command("user-thing"));
    }

    #[test]
    fn entry_order_does_not_decide_it() {
        // The whole point. A HashMap gives last-write-wins, so the answer
        // depends on the order the plugin list happened to be written in.
        let mut forward = Banzuke::new();
        forward.enter(decl(Key::A, "builtin", Origin::builtin()));
        forward.enter(decl(Key::A, "user", Origin::operator("rc")));

        let mut backward = Banzuke::new();
        backward.enter(decl(Key::A, "user", Origin::operator("rc")));
        backward.enter(decl(Key::A, "builtin", Origin::builtin()));

        assert_eq!(held(&forward, Key::A), held(&backward, Key::A));
        assert_eq!(held(&forward, Key::A), Action::command("user"));
    }

    #[test]
    fn a_package_cannot_take_a_protected_builtin() {
        // escriba's bug, as a test. A bundled plugin outranks the builtin
        // layer under ordinary layering, so ONLY the floor stops it.
        let mut b = Banzuke::new();
        b.enter(decl(Key::H, "backspace", Origin::builtin()).protected_below(Rank::Operator));
        b.enter(decl(Key::H, "snippet.jump-prev", Origin::package("luasnip")));
        assert_eq!(held(&b, Key::H), Action::command("backspace"));
    }

    #[test]
    fn the_operator_still_may() {
        // A floor defends against what ships in the box, never against the
        // human. An editor that refuses to let you rebind your own backspace
        // has replaced one bad failure with another.
        let mut b = Banzuke::new();
        b.enter(decl(Key::H, "backspace", Origin::builtin()).protected_below(Rank::Operator));
        b.enter(decl(Key::H, "my-thing", Origin::operator("rc")));
        assert_eq!(held(&b, Key::H), Action::command("my-thing"));
    }

    #[test]
    fn the_loser_is_still_on_the_chart() {
        let mut b = Banzuke::new();
        b.enter(decl(Key::H, "backspace", Origin::builtin()).protected_below(Rank::Operator));
        b.enter(decl(Key::H, "snippet.jump-prev", Origin::package("luasnip")));

        let h = b.holding(&hk(Key::H)).unwrap();
        assert!(h.is_contested());
        assert_eq!(h.outranked.len(), 1);
        // And it can name the file to go edit — the thing `Collision` could
        // never do, because a displaced Binding carries no source.
        assert_eq!(h.outranked[0].origin.report(), "package luasnip");
    }

    #[test]
    fn equal_ranks_are_a_peer_conflict_and_the_first_holds() {
        let mut b = Banzuke::new();
        b.enter(decl(Key::A, "from-one", Origin::package("one")));
        b.enter(decl(Key::A, "from-two", Origin::package("two")));

        assert_eq!(held(&b, Key::A), Action::command("from-one"));
        let conflicts = b.peer_conflicts();
        assert_eq!(conflicts.len(), 1, "two packages on one chord");
        assert!(conflicts[0].has_peer_conflict());
    }

    #[test]
    fn an_operator_override_is_not_a_peer_conflict() {
        // Intended layering must not fail a build, or the gate gets muted.
        let mut b = Banzuke::new();
        b.enter(decl(Key::A, "shipped", Origin::distribution("defaults.lisp")));
        b.enter(decl(Key::A, "mine", Origin::operator("rc.lisp")));
        assert!(b.peer_conflicts().is_empty());
        assert!(b.holding(&hk(Key::A)).unwrap().is_contested());
    }

    #[test]
    fn resolve_produces_the_ordinary_dispatch_table() {
        let mut b = Banzuke::new();
        b.enter(decl(Key::H, "backspace", Origin::builtin()).protected_below(Rank::Operator));
        b.enter(decl(Key::H, "snippet.jump-prev", Origin::package("luasnip")));
        b.enter(decl(Key::A, "other", Origin::builtin()));

        let mode = b.resolve("default", false);
        assert_eq!(mode.len(), 2, "two chords, not three declarations");
        assert_eq!(
            mode.bindings.get(&hk(Key::H)).unwrap().action,
            Action::command("backspace"),
        );
    }

    #[test]
    fn nothing_is_ever_removed() {
        // Append-only is a property of the API surface, not of a convention:
        // there is no remove/replace method to call. This asserts the count
        // so a future `fn remove` has to break a test to exist.
        let mut b = Banzuke::new();
        for i in 0..5 {
            let _ = i;
            b.enter(decl(Key::A, "same-chord", Origin::builtin()));
        }
        assert_eq!(b.len(), 5);
        assert_eq!(b.chart().len(), 1, "five declarations, one chord");
    }
}
