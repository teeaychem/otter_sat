use crate::structures::solve::{Solve, SolveError, SolveOk};
use crate::structures::{
    Clause, ClauseId, ClauseSource, ImplicationSource, LevelIndex, Literal, LiteralSource,
    StoredClause, Valuation, ValuationError,
};

impl<'borrow, 'solve> Solve<'solve> {
    pub fn add_clause(
        &'borrow mut self,
        clause: impl Clause,
        src: ClauseSource,
        val: &impl Valuation,
    ) {
        let clause_as_vec = clause.as_vec();
        match clause_as_vec.len() {
            0 => panic!("Attempt to add an empty clause"),
            1 => panic!("Attempt to add an single literal clause"),
            _ => {
                let clause = StoredClause::new_from(Solve::fresh_clause_id(), &clause, src, val);
                for literal in clause.clause().literals() {
                    self.variables[literal.v_id].note_occurence(clause.id());
                }
                self.clauses.push(clause);
            }
        }
    }

    /*
    Primary function for setting a literal during a solve
    Control branches on the current valuation and then the source of the literal.
    A few things may be updated:
    - The current valuation.
    - Records at the current level.
    - The implication graph.
     */
    pub fn set_literal(
        &'borrow mut self,
        lit: Literal,
        src: LiteralSource,
    ) -> Result<(), SolveError> {
        log::warn!("Set literal: {} | src: {:?}", lit, src);
        match self.valuation.set_literal(lit) {
            Ok(()) => {
                {
                    let occurrences = self.variables[lit.v_id].occurrences().collect::<Vec<_>>();
                    let valuation = self.valuation.clone();
                    for clause_id in occurrences {
                        let clause = self.stored_clause_mut(clause_id);
                        clause.update_watch(&valuation, lit.v_id);
                    }
                }
                match src {
                    LiteralSource::Choice => {
                        let new_level_index = self.add_fresh_level();
                        self.current_level_mut().record_literal(lit, src);
                        self.graph
                            .add_literal(lit, self.current_level().index(), false);
                        self.variables[lit.v_id].set_decision_level(new_level_index);
                        log::debug!("+Set choice: {lit}");
                    }
                    LiteralSource::Assumption | LiteralSource::Deduced => {
                        self.variables[lit.v_id].set_decision_level(0);
                        self.top_level_mut().record_literal(lit, src);
                        self.graph.add_literal(lit, 0, false);
                        log::debug!("+Set assumption/deduction: {lit}");
                    }
                    LiteralSource::HobsonChoice => {
                        self.variables[lit.v_id].set_decision_level(0);
                        self.top_level_mut().record_literal(lit, src);
                        self.graph.add_literal(lit, 0, false);
                        log::debug!("+Set hobson choice: {lit}");
                    }
                    LiteralSource::StoredClause(clause_id) => {
                        let current_level = self.current_level().index();
                        self.variables[lit.v_id].set_decision_level(current_level);
                        self.current_level_mut().record_literal(lit, src);

                        let literals = self
                            .clauses
                            .iter()
                            .find(|clause| clause.id() == clause_id)
                            .unwrap()
                            .literals()
                            .map(|l| l.negate());

                        self.graph.add_implication(
                            literals,
                            lit,
                            self.current_level().index(),
                            ImplicationSource::StoredClause(clause_id),
                        );

                        log::debug!("+Set deduction: {lit}");
                    }
                    LiteralSource::Conflict => {
                        let current_level = self.current_level().index();
                        self.variables[lit.v_id].set_decision_level(current_level);
                        self.current_level_mut().record_literal(lit, src);
                        if self.current_level().index() != 0 {
                            self.graph.add_contradiction(
                                self.current_level().get_choice().expect("No choice 0+"),
                                lit,
                                self.current_level().index(),
                            );
                        } else {
                            self.graph
                                .add_literal(lit, self.current_level().index(), false);
                        }
                        log::debug!("+Set conflict: {lit}");
                    }
                };
                Ok(())
            }
            Err(ValuationError::Match) => match src {
                LiteralSource::StoredClause(_) => {
                    // A literal may be implied by multiple clauses
                    Ok(())
                }
                _ => {
                    log::error!("Attempting to restate {} via {:?}", lit, src);
                    panic!("Attempting to restate the valuation")
                }
            },
            Err(ValuationError::Conflict) => {
                match src {
                    LiteralSource::StoredClause(id) => {
                        // A literal may be implied by multiple clauses
                        Err(SolveError::Conflict(id, lit))
                    }
                    LiteralSource::Deduced => {
                        panic!("Attempt to deduce the flip of {}", lit.v_id);
                    }
                    _ => {
                        log::error!("Attempting to flip {} via {:?}", lit, src);
                        panic!("Attempting to flip the valuation")
                    }
                }
            }
        }
    }

    pub fn unset_literal(&mut self, literal: Literal) {
        log::warn!("Unset: {}", literal);

        self.valuation[literal.v_id] = None;
        self.variables[literal.v_id].clear_decision_level();
    }
}

impl Solve<'_> {
    pub fn stored_clause_mut(&mut self, id: ClauseId) -> &mut StoredClause {
        self.clauses
            .iter_mut()
            .find(|stored_clause| stored_clause.id() == id)
            .unwrap()
    }

    pub fn backtrack_once(&mut self) -> Result<SolveOk, SolveError> {
        if self.current_level().index() == 0 {
            Err(SolveError::NoSolution)
        } else {
            let the_level = self.levels.pop().unwrap();
            log::warn!("Backtracking from {}", the_level.index());
            self.graph.remove_level(&the_level);
            for literal in the_level.literals() {
                self.unset_literal(literal)
            }
            log::warn!("Backtracked from {}", the_level.index());
            Ok(SolveOk::Backtracked)
        }
    }

    pub fn backjump(&mut self, to: LevelIndex) {
        while self.current_level().index() != 0 && self.current_level().index() >= to {
            let _ = self.backtrack_once();
        }
    }
}
