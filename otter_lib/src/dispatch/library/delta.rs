use crate::{
    db::keys::ClauseKey,
    structures::{atom::Atom, literal::vbLiteral},
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
    Conflict { from: vbLiteral, via: ClauseKey },
    Instance { via: ClauseKey, to: vbLiteral },
}

#[derive(Clone)]
pub enum ClauseBuider {
    End,
    Index(u32),
    Literal(vbLiteral),
    Start,
}

#[derive(Clone)]
pub enum Resolution {
    Begin,
    End,
    Subsumed(ClauseKey, vbLiteral),
    Used(ClauseKey),
}

#[derive(Clone)]
pub enum ClauseDB {
    Added(ClauseKey),
    BCP(ClauseKey),
    ClauseLiteral(vbLiteral),
    ClauseStart,
    Deletion(ClauseKey),
    Transfer(ClauseKey, ClauseKey),
    Original(ClauseKey),
}

#[derive(Debug, Clone)]
pub enum LiteralDB {}

#[derive(Clone)]
pub enum AtomDB {
    ExternalRepresentation(String),
    Internalised(Atom),
    Unsatisfiable(ClauseKey),
}
