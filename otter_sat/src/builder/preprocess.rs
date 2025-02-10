use crate::{
    structures::{clause::CClause, literal::Literal},
    types::err::{self, PreprocessingError},
};

/// Primarily to distinguish the case where preprocessing results in an empty clause.
#[derive(PartialEq, Eq)]
pub enum PreprocessingOk {
    /// A tautology.
    Tautology,

    /// Any clause.
    Clause,
}

/// Preprocess a clause to remove duplicate literals.
pub(super) fn preprocess_clause(
    clause: &mut CClause,
) -> Result<PreprocessingOk, err::PreprocessingError> {
    let mut index = 0;
    let mut max = clause.len();
    'clause_loop: loop {
        if index == max {
            break;
        }
        let literal = clause[index];

        for other_index in 0..index {
            let other_literal = clause[other_index];
            if other_literal.atom() == literal.atom() {
                if other_literal.polarity() == literal.polarity() {
                    clause.swap_remove(index);
                    max -= 1;
                    continue 'clause_loop;
                } else {
                    return Ok(PreprocessingOk::Tautology);
                }
            }
        }
        index += 1
    }

    match clause.is_empty() {
        false => Ok(PreprocessingOk::Clause),
        true => Err(PreprocessingError::Unsatisfiable),
    }
}

#[cfg(test)]
mod preprocessing_tests {
    use crate::structures::literal::CLiteral;

    use super::*;

    #[test]
    fn pass() {
        let p = CLiteral::new(1, true);
        let not_q = CLiteral::new(2, false);
        let r = CLiteral::new(3, true);

        let clause = vec![p, not_q, r];
        let mut processed_clause = clause.clone();
        let _ = preprocess_clause(&mut processed_clause);

        assert!(clause.eq(&processed_clause));
    }

    #[test]
    fn duplicate_removal() {
        let p = CLiteral::new(1, true);
        let not_q = CLiteral::new(2, false);
        let r = CLiteral::new(3, true);

        let clause = vec![p, not_q, r];
        let mut processed_clause = vec![p, not_q, r, r, not_q, p];
        let _ = preprocess_clause(&mut processed_clause);

        assert!(clause.eq(&processed_clause));
    }

    #[test]
    fn contradiction_error() {
        let p = CLiteral::new(1, true);
        let not_p = CLiteral::new(1, false);

        let mut clause = vec![p, not_p];
        let preprocessing_result = preprocess_clause(&mut clause);

        assert!(preprocessing_result.is_ok_and(|k| k == PreprocessingOk::Tautology));
    }
}
