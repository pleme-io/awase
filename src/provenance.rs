//! WHO declared a binding, and with what authority.
//!
//! # The gap this closes
//!
//! [`crate::detect_duplicate_bindings`] and [`crate::KeyMode::try_bind`] can
//! see that two declarations want the same chord. Neither can say which one
//! *should* have it, because a [`Binding`](crate::Binding) does not know where
//! it came from — [`BindRefusal::AlreadyBound`](crate::BindRefusal) hands back
//! the incumbent binding and nothing else. So awase could REPORT a collision
//! and never ADJUDICATE one, and every consumer that merges declarations from
//! more than one source had to invent its own answer.
//!
//! escriba paid for that: a bundled plugin bound `<C-h>` to a snippet verb
//! that no subsystem implements, displacing the core backspace binding. Last
//! writer won, because the only rule available was insertion order — and
//! insertion order was an accident of the array the plugins were listed in.
//!
//! # The two axes, and why one is not enough
//!
//! [`Rank`] is *authority to override*: an operator's own config beats the
//! distribution's defaults, which beat a package's, which beat what is
//! compiled in. That is ordinary config layering and it is what most
//! consumers want most of the time.
//!
//! Rank alone gets escriba's case exactly backwards. A plugin outranks the
//! builtin layer, so under pure layering a plugin taking backspace is
//! *correct* — which is how the bug was legal. What was actually missing is
//! that some builtin declarations are load-bearing: the operator may rebind
//! backspace, but a plugin that ships in the box may not take it away. That
//! is [`Declaration::floor`](crate::banzuke::Declaration), a separate axis,
//! and it is the one that makes the class impossible rather than unlikely.

use serde::{Deserialize, Serialize};

/// The authority a declaration carries.
///
/// Ordered lowest-to-highest, and the ordering is the whole point:
/// `Builtin < Package < Distribution < Operator`. `derive(PartialOrd, Ord)` on
/// a fieldless enum orders by declaration position, so moving a variant here
/// changes fleet-wide precedence — the variants are in the order an operator
/// would defend, not alphabetical.
///
/// Four rungs, because four is what the consumers actually have: escriba has
/// compiled defaults / bundled caixas / its shipped rc / the user's rc; frost
/// has builtins / frostmourne's rc / the user's rc; mado has defaults /
/// config / user. A consumer that needs fewer simply never constructs the
/// rungs it lacks.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum Rank {
    /// Compiled into the application. The floor — what the app guarantees
    /// works before any configuration is read.
    #[default]
    Builtin,
    /// A plugin, extension, or caixa the application ships or loads.
    ///
    /// Above `Builtin` because a package exists to change behaviour, and
    /// below `Distribution` because the distribution chose which packages to
    /// ship in the first place.
    Package,
    /// The curated default configuration the application distributes.
    Distribution,
    /// The human's own configuration. Nothing outranks the operator.
    Operator,
}

impl Rank {
    /// Every rank, lowest authority first. For rendering a legend or walking
    /// the ladder in tests without hand-listing it.
    #[must_use]
    pub const fn ladder() -> [Self; 4] {
        [Self::Builtin, Self::Package, Self::Distribution, Self::Operator]
    }

    /// A short word for an operator-facing report.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Builtin => "builtin",
            Self::Package => "package",
            Self::Distribution => "distribution",
            Self::Operator => "operator",
        }
    }
}

/// WHERE a declaration came from, in terms an operator can act on.
///
/// The string is for a human to read in a report and then go edit, so it
/// carries whatever identifies the thing: a package name, a config path. It
/// is deliberately not a `PathBuf` — a bundled package has a name and no
/// path, and forcing one into the other loses the identity that matters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Source {
    /// The application's own code.
    Builtin,
    /// A named unit — a plugin, extension, or caixa.
    Named(String),
    /// A configuration file, identified by the path the operator would open.
    File(String),
}

impl Source {
    /// What to print when telling an operator where a binding came from.
    #[must_use]
    pub fn describe(&self) -> &str {
        match self {
            Self::Builtin => "builtin",
            Self::Named(n) | Self::File(n) => n,
        }
    }
}

/// A declaration's full provenance: how much authority, and from where.
///
/// Both halves are required. A rank with no source cannot be reported ("some
/// package took your backspace" is not actionable), and a source with no rank
/// cannot be adjudicated. Making it one struct with two required fields is
/// what stops either half from being omitted at a call site.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Origin {
    /// How much authority this declaration carries.
    pub rank: Rank,
    /// Where it came from.
    pub source: Source,
}

impl Origin {
    /// Compiled into the application.
    #[must_use]
    pub const fn builtin() -> Self {
        Self {
            rank: Rank::Builtin,
            source: Source::Builtin,
        }
    }

    /// From a named package / plugin / caixa.
    #[must_use]
    pub fn package(name: impl Into<String>) -> Self {
        Self {
            rank: Rank::Package,
            source: Source::Named(name.into()),
        }
    }

    /// From the distribution's shipped configuration file.
    #[must_use]
    pub fn distribution(file: impl Into<String>) -> Self {
        Self {
            rank: Rank::Distribution,
            source: Source::File(file.into()),
        }
    }

    /// From the operator's own configuration file.
    #[must_use]
    pub fn operator(file: impl Into<String>) -> Self {
        Self {
            rank: Rank::Operator,
            source: Source::File(file.into()),
        }
    }

    /// One line an operator can act on: `package escriba-luasnip`.
    #[must_use]
    pub fn report(&self) -> String {
        let mut s = String::with_capacity(32);
        s.push_str(self.rank.label());
        s.push(' ');
        s.push_str(self.source.describe());
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ladder_is_ordered_lowest_authority_first() {
        // Asserted rather than assumed: `derive(Ord)` orders by declaration
        // position, so this test is what stops a well-meaning alphabetical
        // re-sort from silently inverting fleet-wide precedence.
        assert!(Rank::Builtin < Rank::Package);
        assert!(Rank::Package < Rank::Distribution);
        assert!(Rank::Distribution < Rank::Operator);
        assert_eq!(Rank::ladder()[0], Rank::Builtin);
        assert_eq!(Rank::ladder()[3], Rank::Operator);
    }

    #[test]
    fn default_rank_is_the_floor_not_the_ceiling() {
        // A forgotten rank must be the LEAST authoritative, so an omission
        // fails closed — it loses an argument it should have lost anyway,
        // rather than silently outranking the operator.
        assert_eq!(Rank::default(), Rank::Builtin);
    }

    #[test]
    fn an_origin_reports_both_halves() {
        assert_eq!(
            Origin::package("escriba-luasnip").report(),
            "package escriba-luasnip"
        );
        assert_eq!(Origin::builtin().report(), "builtin builtin");
        assert_eq!(
            Origin::operator("~/.config/escriba/rc.lisp").report(),
            "operator ~/.config/escriba/rc.lisp"
        );
    }
}
