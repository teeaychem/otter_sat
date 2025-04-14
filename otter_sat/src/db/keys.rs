use crate::{
    misc::log::targets::{self},
    structures::literal::{CLiteral, Literal},
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
    OriginalUnit(CLiteral),

    /// The key to a unit clause contains the (unit) clause.
    AdditionUnit(CLiteral),

    /// The key to a binary clause.
    OriginalBinary(FormulaIndex),

    /// The key to a binary clause.
    AdditionBinary(FormulaIndex),

    /// The key to an original clause.
    Original(FormulaIndex),

    /// The key to an addition.
    Addition(FormulaIndex, FormulaToken),
}

impl ClauseKey {
    /// Extracts the index from a key.
    pub fn index(&self) -> usize {
        match self {
            Self::OriginalUnit(l) | Self::AdditionUnit(l) => l.atom() as usize,
            Self::OriginalBinary(i) | Self::AdditionBinary(i) => *i as usize,
            Self::Original(i) => *i as usize,
            Self::Addition(i, _) => *i as usize,
        }
    }

    /// Retokens an addition key to distnguish multiple uses of the same index.
    ///
    /// Returns an error if used on any other key, or if the token limit has been reached.
    pub fn retoken(&self) -> Result<Self, err::ClauseDBError> {
        match self {
            Self::OriginalUnit(_) | Self::AdditionUnit(_) => {
                log::error!(target: targets::CLAUSE_DB, "Unit keys have a unique token");
                Err(err::ClauseDBError::InvalidKeyToken)
            }

            Self::Original(_) => {
                log::error!(target: targets::CLAUSE_DB, "Formula keys have a unique token");
                Err(err::ClauseDBError::InvalidKeyToken)
            }

            Self::OriginalBinary(_) | Self::AdditionBinary(_) => {
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
            Self::OriginalUnit(key) => write!(f, "OriginalUnit({key})"),
            Self::AdditionUnit(key) => write!(f, "AdditionUnit({key})"),
            Self::OriginalBinary(key) => write!(f, "OriginalBinary({key})"),
            Self::AdditionBinary(key) => write!(f, "AdditionBinary({key})"),
            Self::Original(key) => write!(f, "Original({key})"),
            Self::Addition(key, token) => write!(f, "Addition({key}, {token})"),
        }
    }
}
