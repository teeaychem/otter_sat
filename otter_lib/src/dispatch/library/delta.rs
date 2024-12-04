use crate::{
    db::keys::ClauseKey,
    structures::{literal::Literal, variable::Variable},
};

#[derive(Clone)]
pub enum Delta {
    BCP(self::BCP),
    ClauseDB(self::ClauseDB),
    LiteralDB(self::LiteralDB),
    Resolution(self::Resolution),
    VariableDB(self::VariableDB),
}

#[derive(Clone)]
pub enum BCP {
    Conflict { from: Literal, via: ClauseKey },
    Instance { via: ClauseKey, to: Literal },
}

#[derive(Clone)]
pub enum ClauseBuider {
    End,
    Index(u32),
    Literal(Literal),
    Start,
}

#[derive(Clone)]
pub enum Resolution {
    Begin,
    End,
    Subsumed(ClauseKey, Literal),
    Used(ClauseKey),
}

#[derive(Clone)]
pub enum ClauseDB {
    Added(ClauseKey),
    BCP(ClauseKey),
    ClauseLiteral(Literal),
    ClauseStart,
    Deletion(ClauseKey),
    Transfer(ClauseKey, ClauseKey),
    Original(ClauseKey),
}

#[derive(Debug, Clone)]
pub enum LiteralDB {}

#[derive(Clone)]
pub enum VariableDB {
    ExternalRepresentation(String),
    Internalised(Variable),
    Unsatisfiable(ClauseKey),
}
