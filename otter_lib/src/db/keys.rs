use crate::{
    misc::log::targets::{self},
    structures::literal::{Literal, LiteralT},
    types::err::{self},
};

pub type ChoiceIndex = u32;
pub type FormulaIndex = u32;
pub type FormulaToken = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ClauseKey {
    Unit(Literal),
    Original(FormulaIndex),
    Binary(FormulaIndex),
    Addition(FormulaIndex, FormulaToken),
}

impl ClauseKey {
    pub fn index(&self) -> usize {
        match self {
            Self::Unit(l) => l.var() as usize,
            Self::Original(i) => *i as usize,
            Self::Binary(i) => *i as usize,
            Self::Addition(i, _) => *i as usize,
        }
    }

    pub fn retoken(&self) -> Result<Self, err::ClauseDB> {
        match self {
            Self::Unit(_) => {
                log::error!(target: targets::CLAUSE_DB, "Unit keys have a unique token");
                Err(err::ClauseDB::InvalidKeyToken)
            }
            Self::Original(_) => {
                log::error!(target: targets::CLAUSE_DB, "Formula keys have a unique token");
                Err(err::ClauseDB::InvalidKeyToken)
            }
            Self::Binary(_) => {
                log::error!(target: targets::CLAUSE_DB, "Binary keys have a unique token");
                Err(err::ClauseDB::InvalidKeyToken)
            }
            Self::Addition(index, token) => {
                if *token == FormulaToken::MAX {
                    return Err(err::ClauseDB::StorageExhausted);
                }
                Ok(ClauseKey::Addition(*index, token + 1))
            }
        }
    }
}

impl std::fmt::Display for ClauseKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit(key) => write!(f, "Unit({key})"),
            Self::Original(key) => write!(f, "Formula({key})"),
            Self::Addition(key, token) => write!(f, "Learned({key}, {token})"),
            Self::Binary(key) => write!(f, "Binary({key})"),
        }
    }
}
