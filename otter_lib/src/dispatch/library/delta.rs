use crate::{
    db::keys::ClauseKey,
    structures::{clause::Clause, literal::Literal, variable::Variable},
};

#[derive(Clone)]
pub enum Delta {
    ClauseDB(self::ClauseDB),
    LiteralDB(self::LiteralDB),
    Resolution(self::Resolution),
    VariableDB(self::VariableDB),
    BCP(self::BCP),
}

#[derive(Clone)]
pub enum BCP {
    Instance {
        from: Literal,
        via: ClauseKey,
        to: Literal,
    },
    Conflict(Literal, ClauseKey), // Literal + ClauseKey -> falsum
}

#[derive(Clone)]
pub enum ClauseBuider {
    Start,
    Index(u32),
    Literal(Literal),
    End,
}

#[derive(Clone)]
pub enum ClauseDB {
    TransferBinary(ClauseKey, ClauseKey, Clause),
    Deletion(ClauseKey, Clause),
    BinaryOriginal(ClauseKey, Clause),
    BinaryResolution(ClauseKey, Clause),
    Original(ClauseKey, Clause),
    Resolution(ClauseKey, Clause),
}

#[derive(Debug, Clone)]
pub enum LiteralDB {
    Assumption(Literal),
    ResolutionProof(Literal),
    Proof(Literal),
    Forced(ClauseKey, Literal),
    Pure(Literal),
}

#[derive(Clone)]
pub enum Resolution {
    Begin,
    End,
    Used(ClauseKey),
    Subsumed(ClauseKey, Literal),
}

#[derive(Clone)]
pub enum VariableDB {
    Internalised(Variable, String),
    Unsatisfiable(ClauseKey),
}
