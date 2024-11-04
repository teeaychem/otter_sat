use crate::context::stores::ClauseKey;
use crate::context::stores::FormulaToken;

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

    pub fn token(&self) -> FormulaToken {
        match self {
            Self::Formula(_) => panic!("Formula keys have a unique token"),
            Self::Binary(_) => panic!("Binary keys have a unique token"),
            Self::Learned(_, issue) => *issue,
        }
    }

    pub fn retoken(&self) -> Self {
        match self {
            Self::Formula(_) => panic!("Formula keys have a unique token"),
            Self::Binary(_) => panic!("Binary keys have a unique token"),
            Self::Learned(index, token) => {
                assert!(*token < FormulaToken::MAX);
                ClauseKey::Learned(*index, token + 1)
            }
        }
    }
}
