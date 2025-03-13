/*!
Consequences of the context in some state.

Each consequence is recorded as a [CLiteral] and [ConsequenceSource] pair, with the literal representing an atom-value bind which must hold and the source noting the direct ancestor of the consequence.

If any assumptions or decisions have been made, the consequence is established only relative to those assumptions or decisions.
However, it does not follow the consequence *requires* those assumptions or decisions.

*/
use std::borrow::Borrow;

use crate::{
    db::ClauseKey,
    structures::{
        atom::Atom,
        literal::{CLiteral, Literal},
    },
};

/// The source of a bind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum AssignmentSource {
    /// A decision was made where the alternative the alternative would make no difference to satisfiability.
    PureLiteral,

    /// A consequence of boolean constraint propagation.
    BCP(ClauseKey),

    /// A decision.
    Decision,

    /// An assumption.
    Assumption,

    /// An original (unit) clause.
    Original,

    /// An addition (unit) clause.
    Addition,
}

#[derive(Clone, Debug)]
/// A consequence of the context in some state.
pub struct Assignment {
    /// The atom-value bind which must hold, represented as a literal.
    pub literal: CLiteral,

    /// The immediate reason why the atom-value pair must be.
    pub source: AssignmentSource,
}

impl Assignment {
    /// Creates a consequence from a bind represented as a literal and a source.
    pub fn from(literal: impl Borrow<CLiteral>, source: AssignmentSource) -> Self {
        Assignment {
            literal: literal.borrow().canonical(),
            source,
        }
    }

    /// Creates a consequence of the given atom bound to the given value due to the given source.
    pub fn from_bind(atom: Atom, value: bool, source: AssignmentSource) -> Self {
        Assignment {
            literal: CLiteral::new(atom, value),
            source,
        }
    }

    /// The bound atom.
    pub fn atom(&self) -> Atom {
        self.literal.atom()
    }

    /// The value the atom is bound to.
    pub fn value(&self) -> bool {
        self.literal.polarity()
    }

    /// The atom-value bind, represented as a literal.
    pub fn literal(&self) -> &CLiteral {
        &self.literal
    }

    /// The (immediate) reason why the atom-value bind must hold.
    pub fn source(&self) -> &AssignmentSource {
        &self.source
    }
}
