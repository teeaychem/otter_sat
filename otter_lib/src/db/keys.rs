use crate::types::errs;

pub type VariableIndex = u32;

pub type ChoiceIndex = u32;

pub type FormulaIndex = u32;
pub type FormulaToken = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClauseKey {
    Formula(FormulaIndex),
    Binary(FormulaIndex),
    Learned(FormulaIndex, FormulaToken),
}

impl ClauseKey {
    pub fn index(&self) -> usize {
        match self {
            Self::Formula(i) => *i as usize,
            Self::Binary(i) => *i as usize,
            Self::Learned(i, _) => *i as usize,
        }
    }

    pub fn retoken(&self) -> Result<Self, errs::ClauseDB> {
        match self {
            Self::Formula(_) => {
                log::error!(target: crate::log::targets::CLAUSE_DB, "Formula keys have a unique token");
                Err(errs::ClauseDB::InvalidKeyToken)
            }
            Self::Binary(_) => {
                log::error!(target: crate::log::targets::CLAUSE_DB, "Binary keys have a unique token");
                Err(errs::ClauseDB::InvalidKeyToken)
            }
            Self::Learned(index, token) => {
                if *token == FormulaToken::MAX {
                    return Err(errs::ClauseDB::StorageExhausted);
                }
                Ok(ClauseKey::Learned(*index, token + 1))
            }
        }
    }
}

impl std::fmt::Display for ClauseKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Formula(key) => write!(f, "Formula({key})"),
            Self::Learned(key, token) => write!(f, "Learned({key}, {token})"),
            Self::Binary(key) => write!(f, "Binary({key})"),
        }
    }
}
