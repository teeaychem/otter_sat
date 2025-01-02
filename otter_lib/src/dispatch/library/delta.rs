use crate::{
    db::ClauseKey,
    structures::{atom::Atom, literal::abLiteral},
};

#[derive(Clone)]
pub enum Delta {
    BCP(self::BCP),
    ClauseDB(self::ClauseDB),
    LiteralDB(self::LiteralDB),
    Resolution(self::Resolution),
    AtomDB(self::AtomDB),
}

#[derive(Clone)]
pub enum BCP {
    Conflict {
        literal: abLiteral,
        clause: ClauseKey,
    },
    Instance {
        clause: ClauseKey,
        literal: abLiteral,
    },
}

#[derive(Clone)]
pub enum ClauseBuider {
    End,
    Index(u32),
    Literal(abLiteral),
    Start,
}

#[derive(Clone)]
pub enum Resolution {
    Begin,
    End,
    Subsumed(ClauseKey, abLiteral),
    Used(ClauseKey),
}

#[derive(Clone)]
pub enum ClauseDB {
    Added(ClauseKey),
    BCP(ClauseKey),
    ClauseLiteral(abLiteral),
    ClauseStart,
    Deletion(ClauseKey),
    Transfer(ClauseKey, ClauseKey),
    Original(ClauseKey),
}

#[derive(Clone)]
pub enum LiteralDB {}

#[derive(Clone)]
pub enum AtomDB {
    ExternalRepresentation(String),
    Internalised(Atom),
    Unsatisfiable(ClauseKey),
}
