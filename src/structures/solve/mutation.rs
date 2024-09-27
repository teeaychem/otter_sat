use crate::structures::{
    solve::{Solve, SolveError},
    stored_clause::update_watch,
    Clause, ClauseSource, ImplicationSource, LevelIndex, Literal, LiteralSource, StoredClause,
    Valuation, ValuationError,
};
use std::rc::Rc;

impl<'borrow, 'solve> Solve<'solve> {
    /// Stores a clause with an automatically generated id.
    /// Note: In order to use the clause the watch literals of the struct must be initialised.
    pub fn store_clause(
        &'borrow mut self,
        clause: impl Clause,
        src: ClauseSource,
    ) -> Rc<StoredClause> {
        match clause.len() {
            0 => panic!("Attempt to add an empty clause"),
            _ => match &src {
                ClauseSource::Formula => {
                    let stored_clause =
                        StoredClause::new_from(Solve::fresh_clause_id(), &clause, src);

                    for literal in stored_clause.clause().literals() {
                        self.variables[literal.v_id]
                            .note_occurence(stored_clause.clone(), literal.polarity);
                    }

                    self.formula_clauses.push(stored_clause.clone());
                    stored_clause
                }
                ClauseSource::Resolution(_) => {
                    log::warn!("Learning clause {}", clause.as_string());
                    let stored_clause =
                        StoredClause::new_from(Solve::fresh_clause_id(), &clause, src);

                    for literal in stored_clause.clause().literals() {
                        self.variables[literal.v_id].increase_activity(1.0);
                        self.variables[literal.v_id]
                            .note_occurence(stored_clause.clone(), literal.polarity);
                    }
                    self.learnt_clauses.push(stored_clause.clone());
                    stored_clause
                }
            },
        }
    }

    pub fn drop_clause(&mut self, stored_clause: &Rc<StoredClause>) {
        if let Some(p) = self
            .learnt_clauses
            .iter()
            .position(|sc| sc == stored_clause)
        {
            let removed = self.learnt_clauses.swap_remove(p);
            let a = removed.watched_a();
            let b = removed.watched_b();
            self.variables[a.v_id].watch_removed(&removed);
            self.variables[b.v_id].watch_removed(&removed);
            // println!("removed: {}", stored_clause);
        } else {
            panic!("Unable to remove: {}", stored_clause);
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
                    LiteralSource::Assumption | LiteralSource::HobsonChoice => 0,
                    LiteralSource::StoredClause(_) | LiteralSource::Conflict => {
                        self.current_level().index()
                    }
                };

                {
                    let occurrences = self.variables[lit.v_id].occurrences().collect::<Vec<_>>();
                    let valuation = self.valuation.clone();
                    for clause in occurrences {
                        match update_watch(&clause, &valuation, lit.v_id, &mut self.variables) {
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
                    LiteralSource::Assumption => {
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
                    LiteralSource::StoredClause(weak) => {
                        self.variables[lit.v_id].set_decision_level(level_index);
                        self.current_level_mut().record_literal(lit, src.clone());
                        if let Some(stored_clause) = weak.upgrade() {
                            let literals = stored_clause
                                .literals()
                                .map(|l| l.negate())
                                .collect::<Vec<_>>();

                            self.implication_graph.add_implication(
                                literals.into_iter(),
                                lit,
                                level_index,
                                ImplicationSource::StoredClause(weak.clone()),
                            );
                        } else {
                            panic!("Lost clause");
                        }

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
    pub fn backjump(&mut self, to: LevelIndex) {
        log::warn!("Backjump from {} to {}", self.current_level().index(), to);

        for _ in 0..(self.current_level().index() - to) {
            let the_level = self.levels.pop().unwrap();
            self.implication_graph.remove_level(&the_level);
            for literal in the_level.literals() {
                self.unset_literal(literal);
            }
        }
    }
}
