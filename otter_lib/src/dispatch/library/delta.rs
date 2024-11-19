use crate::{db::keys::ClauseKey, structures::literal::Literal};

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
    TransferBinary(ClauseKey, ClauseKey, Vec<Literal>),
    Deletion(ClauseKey, Vec<Literal>),
    BinaryOriginal(ClauseKey, Vec<Literal>),
    BinaryResolution(ClauseKey, Vec<Literal>),
    Original(ClauseKey, Vec<Literal>),
    Learned(ClauseKey, Vec<Literal>),
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
    Internalised(String, u32),
    Unsatisfiable(ClauseKey),
}
