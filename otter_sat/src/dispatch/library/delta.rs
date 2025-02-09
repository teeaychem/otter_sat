//! Details on some change during a procedure or to an interal structure.
use crate::{db::ClauseKey, structures::literal::CLiteral};

/// High level distinction of changes, by 'location' of the change.
#[derive(Clone)]
pub enum Delta {
    /// During [BCP](crate::procedures::bcp).
    BCP(self::BCP),

    /// During resolution
    Resolution(self::Resolution),

    /// Within the [clause database](ClauseDB).
    ClauseDB(self::ClauseDB),

    /// Within the [literal database](LiteralDB).
    LiteralDB(self::LiteralDB),

    /// Within the [atom database](AtomDB).
    AtomDB(self::AtomDB),
}

/// Changes during [BCP](crate::procedures::bcp).
#[derive(Clone)]
pub enum BCP {
    /// A conflict was found with the detailed clause asserting the detailed literal.
    Conflict {
        literal: CLiteral,
        clause: ClauseKey,
    },

    /// An instance of BCP took place, with the detailed literal being asserted by the detailed clause.
    Instance {
        clause: ClauseKey,
        literal: CLiteral,
    },
}

/// Changes when building a clause.
#[derive(Clone)]
pub enum ClauseBuider {
    /// Details of a built clause follow…
    Start,

    /// … details of a built clause have concluded.
    End,

    /// The detailed literal belongs to the clause.
    Literal(CLiteral),
}

/// Changes during resolution.
#[derive(Clone)]
pub enum Resolution {
    /// Details of an instance of resolution follow…
    Begin,

    /// … details of an instance of resolution have concluded.
    End,

    /// The detailed literal was subsumed in the detailed clause (placeholder).
    Subsumed(ClauseKey, CLiteral),

    /// The detailed clause was used.
    Used(ClauseKey),
}

/// Changes within the [clause database](ClauseDB).
#[derive(Clone)]
pub enum ClauseDB {
    /// A unit clause was added via BCP.
    BCP(ClauseKey),

    Deletion(ClauseKey),
    Transfer(ClauseKey, ClauseKey),
    Original(ClauseKey),

    /// Details of an added clause follow (and will be terminated with the key used to access the clause) …
    ClauseStart,

    /// A literal beloning to a clause.
    ClauseLiteral(CLiteral),

    /// A clause with the detailed key was added (and if a clause is being terminated, the clause has concluded and this is the key used to access the clause).
    Added(ClauseKey),

    /// The formula is unsatisfiable, with the key as a witness.
    Unsatisfiable(ClauseKey),
}

/// Changes within the [literal database](LiteralDB).
#[derive(Clone)]
pub enum LiteralDB {}

/// Changes within the [atom database](AtomDB).
#[derive(Clone)]
pub enum AtomDB {}
