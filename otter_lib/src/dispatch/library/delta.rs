use crate::{
    db::keys::ClauseKey,
    structures::{literal::Literal, variable::Variable},
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
    Conflict {
        from: Literal,
        via: ClauseKey,
    },
}

#[derive(Clone)]
pub enum ClauseBuider {
    Start,
    Index(u32),
    Literal(Literal),
    End,
}

#[derive(Clone)]
pub enum Resolution {
    Begin,
    End,
    Used(ClauseKey),
    Subsumed(ClauseKey, Literal),
}

#[derive(Clone)]
pub enum ClauseDB {
    ClauseStart,
    ClauseLiteral(Literal),
    TransferBinary(ClauseKey, ClauseKey),
    Deletion(ClauseKey),
    BinaryOriginal(ClauseKey),
    BinaryResolution(ClauseKey),
    Original(ClauseKey),
    Resolution(ClauseKey),
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
pub enum VariableDB {
    ExternalRepresentation(String),
    Internalised(Variable),
    Unsatisfiable(ClauseKey),
}
