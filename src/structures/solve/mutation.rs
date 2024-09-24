use crate::structures::{
    solve::{Solve, SolveError, SolveOk},
    Clause, ClauseSource, ImplicationSource, LevelIndex, Literal, LiteralSource, StoredClause,
    Valuation, ValuationError,
};
use std::rc::Rc;

impl<'borrow, 'solve> Solve<'solve> {
    pub fn add_clause(
        &'borrow mut self,
        clause: impl Clause,
        src: ClauseSource,
        val: &impl Valuation,
    ) -> Rc<StoredClause> {
        let clause_as_vec = clause.as_vec();
        match clause_as_vec.len() {
            0 => panic!("Attempt to add an empty clause"),
            _ => {
                let clause = StoredClause::new_from(Solve::fresh_clause_id(), &clause, src, val);

                for literal in clause.clause().literals() {
                    self.variables[literal.v_id].note_occurence(
                        clause.clone(),
                        src,
                        literal.polarity,
                    );
                }
                self.clauses.insert(clause.clone());
                clause
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
        log::trace!("Set literal: {} | src: {:?}", lit, src);
        match self.valuation.set_literal(lit) {
            Ok(()) => {
                let level_index = match src {
                    LiteralSource::Choice => self.add_fresh_level(),
                    LiteralSource::Assumption
                    | LiteralSource::Deduced
                    | LiteralSource::HobsonChoice => 0,
                    LiteralSource::StoredClause(_) | LiteralSource::Conflict => {
                        self.current_level().index()
                    }
                };

                {
                    let occurrences = self.variables[lit.v_id].occurrences().collect::<Vec<_>>();
                    let valuation = self.valuation.clone();
                    for clause in occurrences {
                        match clause.update_watch(&valuation, lit.v_id) {
                            true => self.current_level_mut().note_watch(lit),
                            false => (),
                        };
                    }
                }
                match &src {
                    LiteralSource::Choice => {
                        self.current_level_mut().record_literal(lit, src);
                        self.implication_graph.add_literal(
                            lit,
                            self.current_level().index(),
                            false,
                        );
                        self.variables[lit.v_id].set_decision_level(level_index);
                        log::debug!("+Set choice: {lit}");
                    }
                    LiteralSource::Assumption | LiteralSource::Deduced => {
                        self.variables[lit.v_id].set_decision_level(level_index);
                        self.top_level_mut().record_literal(lit, src);
                        self.implication_graph.add_literal(lit, level_index, false);
                        log::debug!("+Set assumption/deduction: {lit}");
                    }
                    LiteralSource::HobsonChoice => {
                        self.variables[lit.v_id].set_decision_level(level_index);
                        self.top_level_mut().record_literal(lit, src);
                        self.implication_graph.add_literal(lit, level_index, false);
                        log::debug!("+Set hobson choice: {lit}");
                    }
                    LiteralSource::StoredClause(stored_clause) => {
                        self.variables[lit.v_id].set_decision_level(level_index);
                        self.current_level_mut().record_literal(lit, src.clone());

                        let literals = self
                            .clauses
                            .iter()
                            .find(|clause| clause.id() == stored_clause.id())
                            .unwrap()
                            .literals()
                            .map(|l| l.negate());

                        self.implication_graph.add_implication(
                            literals,
                            lit,
                            level_index,
                            ImplicationSource::StoredClause(stored_clause.clone()),
                        );

                        log::debug!("+Set deduction: {lit}");
                    }
                    LiteralSource::Conflict => {
                        self.variables[lit.v_id].set_decision_level(level_index);
                        self.current_level_mut().record_literal(lit, src);
                        if level_index != 0 {
                            self.implication_graph.add_contradiction(
                                self.current_level().get_choice().expect("No choice 0+"),
                                lit,
                                self.current_level().index(),
                            );
                        } else {
                            self.implication_graph.add_literal(lit, level_index, false);
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
        log::trace!("Unset: {}", literal);

        self.valuation[literal.v_id] = None;
        self.variables[literal.v_id].clear_decision_level();
    }
}

impl Solve<'_> {
    pub fn backtrack_once(&mut self) -> Result<SolveOk, SolveError> {
        if self.current_level().index() == 0 {
            Err(SolveError::NoSolution)
        } else {
            let the_level = self.levels.pop().unwrap();
            self.implication_graph.remove_level(&the_level);
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
