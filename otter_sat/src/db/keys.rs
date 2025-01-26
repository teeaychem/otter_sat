use crate::{
    misc::log::targets::{self},
    structures::literal::{cLiteral, Literal},
    types::err::{self},
};

/// The index to a formula.
pub type FormulaIndex = u32;

/// The token of a formula index, used to distinguish re-use of the same [FormulaIndex].
pub type FormulaToken = u16;

/// A key to access a clause stored in the clause database.
///
/// Within the clause database clauses are stored in some indexed structure (e.g. a vector) an keys contain the index to the clause together with a token to distinguish reuse of the same index, where relevant.
///
/// The only exception to this is unit clauses.
/// Here, the index is to the atom database.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ClauseKey {
    /// The key to a unit clause contains the (unit) clause.
    // Note, the size of an abLiteral is smaller than the key for an addition clause.
    Unit(cLiteral),
    /// The key to a binary clause.
    Binary(FormulaIndex),
    /// The key to an original clause.
    Original(FormulaIndex),
    /// The key to an addition.
    Addition(FormulaIndex, FormulaToken),
}

impl ClauseKey {
    /// Extracts the index from a key.
    pub fn index(&self) -> usize {
        match self {
            Self::Unit(l) => l.atom() as usize,
            Self::Original(i) => *i as usize,
            Self::Binary(i) => *i as usize,
            Self::Addition(i, _) => *i as usize,
        }
    }

    /// Retokens an addition key to distnguish multiple uses of the same index.
    ///
    /// Returns an error if used on any other key, or if the token limit has been reached.
    pub fn retoken(&self) -> Result<Self, err::ClauseDBError> {
        match self {
            Self::Unit(_) => {
                log::error!(target: targets::CLAUSE_DB, "Unit keys have a unique token");
                Err(err::ClauseDBError::InvalidKeyToken)
            }
            Self::Original(_) => {
                log::error!(target: targets::CLAUSE_DB, "Formula keys have a unique token");
                Err(err::ClauseDBError::InvalidKeyToken)
            }
            Self::Binary(_) => {
                log::error!(target: targets::CLAUSE_DB, "Binary keys have a unique token");
                Err(err::ClauseDBError::InvalidKeyToken)
            }
            Self::Addition(index, token) => {
                if *token == FormulaToken::MAX {
                    return Err(err::ClauseDBError::StorageExhausted);
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
            Self::Addition(key, token) => write!(f, "Addition({key}, {token})"),
            Self::Binary(key) => write!(f, "Binary({key})"),
        }
    }
}
