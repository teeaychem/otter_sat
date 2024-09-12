use crate::{
    structures::{Literal, LiteralSource, Solve, SolveError, ValuationVec, VariableId},
    ClauseId, StoredClause, Valuation, ValuationError,
};
use std::collections::BTreeSet;

impl Solve<'_> {
    /// general order for pairs related to booleans is 0 is false, 1 is true
    pub fn hobson_choices(&self) -> (Vec<VariableId>, Vec<VariableId>) {
        // let all_v_ids: BTreeSet<VariableId> = self.vars().iter().map(|v| v.id).collect();
        let the_true: BTreeSet<VariableId> =
            self.literals_of_polarity(true).map(|l| l.v_id).collect();
        let the_false: BTreeSet<VariableId> =
            self.literals_of_polarity(false).map(|l| l.v_id).collect();
        let hobson_false: Vec<_> = the_false.difference(&the_true).cloned().collect();
        let hobson_true: Vec<_> = the_true.difference(&the_false).cloned().collect();
        (hobson_false, hobson_true)
    }

    pub fn settle_hobson_choices(&mut self) {
        let the_choices = self.hobson_choices();
        the_choices.0.iter().for_each(|&v_id| {
            let the_choice = Literal::new(v_id, false);
            let _ = self.set_literal(the_choice, LiteralSource::HobsonChoice);
        });
        the_choices.1.iter().for_each(|&v_id| {
            let the_choice = Literal::new(v_id, true);
            let _ = self.set_literal(the_choice, LiteralSource::HobsonChoice);
        });
    }

    pub fn attempt_fix(&mut self, clause: ClauseId, literal: Literal) -> Result<(), SolveError> {
        let dead_end = self.pop_level();
        if let Some(level) = dead_end {
            self.analyse_conflict(&level, clause, literal);

            self.graph.remove_level(&level);
            println!(
                "Conflict implies {} @ {}",
                &level.get_choice().negate(),
                self.current_level().index()
            );
            let _ = self.set_literal(level.get_choice().negate(), LiteralSource::Conflict);
            Ok(())
        } else {
            Err(SolveError::NoSolution)
            // sat_valuation = Some((false, self.valuation.clone()));
        }
    }

    pub fn select_unsat_clause(
        &self,
        clauses: Vec<(ClauseId, Literal)>,
    ) -> Option<(ClauseId, Literal)> {
        if !clauses.is_empty() {
            Some(clauses.first().unwrap()).cloned()
        } else {
            None
        }
    }

    pub fn implication_solve(&mut self) -> Result<Option<ValuationVec>, SolveError> {
        println!("~~~ an implication solve ~~~");
        // self.settle_hobson_choices(); // settle any literals which do occur with some fixed polarity

        loop {
            match self.find_all_unset_on(&self.valuation_at_level(self.current_level().index())) {
                Err(SolveError::UnsatClause(clause_id)) => {
                    if self.current_level().index() != 0 {
                        match self.attempt_fix(clause_id, self.current_level().get_choice()) {
                            Ok(()) => {}
                            Err(SolveError::NoSolution) => {
                                return Ok(None);
                            }
                            _ => todo!(),
                        }
                    } else {
                        return Ok(None);
                    }
                }
                Ok((the_units, the_choices)) => {
                    if !the_units.is_empty() {
                        let mut unsat_clauses = vec![];

                        for (clause_id, consequent) in the_units {
                            match self
                                .set_literal(consequent, LiteralSource::StoredClause(clause_id))
                            {
                                Err(SolveError::UnsatClause(clause_id)) => {
                                    unsat_clauses.push((clause_id, consequent));
                                }
                                Ok(()) => {}
                                _ => todo!(),
                            }
                        }
                        if let Some(clause) = self.select_unsat_clause(unsat_clauses) {
                            match self.attempt_fix(clause.0, clause.1) {
                                Ok(()) => {}
                                Err(SolveError::NoSolution) => {
                                    return Ok(None);
                                }
                                _ => todo!(),
                            }
                        }
                    } else if !the_choices.is_empty() {
                        // make a choice
                        let a_choice = the_choices.first().unwrap();
                        let _ = self.set_literal(*a_choice, LiteralSource::Choice);
                    } else {
                        return Ok(Some(self.valuation.clone()));
                    }
                }
                _ => panic!("Unexpected"),
            }
        }
    }
}
