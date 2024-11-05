use crate::context::stores::ClauseKey;
use crate::context::stores::FormulaToken;

use super::clause::ClauseStoreError;

impl std::fmt::Display for ClauseKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Formula(key) => write!(f, "Formula({key})"),
            Self::Learned(key, token) => write!(f, "Learned({key}, {token})"),
            Self::Binary(key) => write!(f, "Learned({key})"),
        }
    }
}

impl ClauseKey {
    pub fn index(&self) -> usize {
        match self {
            Self::Formula(i) => *i as usize,
            Self::Binary(i) => *i as usize,
            Self::Learned(i, _) => *i as usize,
        }
    }

    pub fn retoken(&self) -> Result<Self, ClauseStoreError> {
        match self {
            Self::Formula(_) => {
                log::error!(target: crate::log::targets::CLAUSE_STORE, "Formula keys have a unique token");
                Err(ClauseStoreError::InvalidKeyToken)
            }
            Self::Binary(_) => {
                log::error!(target: crate::log::targets::CLAUSE_STORE, "Binary keys have a unique token");
                Err(ClauseStoreError::InvalidKeyToken)
            }
            Self::Learned(index, token) => {
                if *token == FormulaToken::MAX {
                    return Err(ClauseStoreError::StorageExhausted);
                }
                Ok(ClauseKey::Learned(*index, token + 1))
            }
        }
    }
}
