use std::{borrow::Borrow, collections::HashSet};

use crate::{
    config::{Config, StoppingCriteria},
    db::{ClauseKey, clause::ClauseDB, trail::Trail, watches::Watches},
    misc::log::targets,
    structures::{
        atom::Atom,
        clause::{CClause, Clause},
        consequence::AssignmentSource,
        literal::{CLiteral, Literal},
    },
    types::err,
};

use super::{AtomCells, ResolutionOk, cell::ResolutionStatus};

impl AtomCells {
    /// The length of the resolved clause.
    pub fn clause_legnth(&self) -> usize {
        self.clause_length
    }

    /// Returns the resolved clause with the asserted literal as the first literal of the clause.
    pub fn to_assertion_clause(&mut self) -> CClause {
        let mut clause = Vec::with_capacity(self.clause_length);
        let mut asserted_index = 0;

        for atom in &self.merged_atoms {
            let cell = unsafe { self.buffer.get_unchecked_mut(*atom as usize) };
            match cell.status {
                ResolutionStatus::Valuation | ResolutionStatus::Backjump => {}

                ResolutionStatus::Proven
                | ResolutionStatus::Strengthened
                | ResolutionStatus::Pivot => {}

                ResolutionStatus::Asserting => {
                    let literal = CLiteral::new(*atom, !unsafe { cell.value.unwrap_unchecked() });
                    clause.push(literal);
                }

                ResolutionStatus::Asserted => {
                    asserted_index = clause.size();
                    let literal = CLiteral::new(*atom, !unsafe { cell.value.unwrap_unchecked() });
                    clause.push(literal);
                }
            }

            if !matches!(cell.status, ResolutionStatus::Proven) {
                cell.status = ResolutionStatus::Valuation;
            }
        }

        if !clause.is_empty() {
            clause.swap(0, asserted_index);
        }

        clause
    }

    /// Applies resolution with the clauses used to observe consequences at the current level.
    ///
    /// Clauses are examined in reverse order of use.
    pub fn resolve_through_current_level(
        &mut self,
        key: &ClauseKey,
        clause_db: &mut ClauseDB,
        watch_dbs: &mut Watches,
        trail: &mut Trail,
        config: &Config,
    ) -> Result<ResolutionOk, err::ResolutionBufferError> {
        // The key has already been used to access the conflicting clause.
        let base_clause = unsafe { clause_db.get_unchecked_mut(key) };

        self.merge_clause(base_clause);
        clause_db.note_use(*key);
        self.premises.insert(*key);

        // bump clause activity
        if let ClauseKey::Addition(index, _) = key {
            clause_db.bump_activity(*index)
        };

        // Resolution buffer is only used by analysis, which is only called after some decision has been made
        let the_trail = trail.take_assignments();
        'resolution_loop: for literal in the_trail.iter().rev() {
            if self.valueless_count <= 1 {
                match config.stopping_criteria.value {
                    StoppingCriteria::FirstUIP => {
                        break 'resolution_loop;
                    }
                    _ => {}
                }
            }

            log::info!(target: targets::ATOMCELLS, "Examining trail item {literal:?}");

            let source = *self.get_assignment_source(literal.atom());

            match source {
                AssignmentSource::None => panic!("! Missing source"),

                AssignmentSource::BCP(key) => {
                    let mut key = key;

                    let source_clause = unsafe { clause_db.get_unchecked_mut(&key) };

                    // Recorded here to avoid multiple mutable borrows of clause_db
                    let source_clause_size = source_clause.size();

                    let resolution_result = self.resolve_clause(source_clause, literal);

                    clause_db.note_use(key);
                    self.premises.insert(key);

                    if resolution_result.is_err() {
                        continue 'resolution_loop; // the clause wasn't relevant
                    }

                    key = match config.subsumption.value
                        && self.clause_length < source_clause_size
                        && self.clause_length > 2
                    {
                        false => key,
                        true => match key {
                            ClauseKey::OriginalUnit(_) | ClauseKey::AdditionUnit(_) => {
                                panic!("! Subsumption called on a unit clause")
                            }

                            ClauseKey::OriginalBinary(_) | ClauseKey::AdditionBinary(_) => {
                                panic!("! Subsumption called on a binary clause");
                            }

                            ClauseKey::Original(_) | ClauseKey::Addition(_, _) => {
                                let clause = unsafe { clause_db.get_unchecked_mut(&key) };
                                clause.subsume(literal, self, watch_dbs, true)?;

                                self.premises.insert(key);
                                clause_db.note_use(key);
                                key
                            }
                        },
                    };

                    if let ClauseKey::Addition(index, _) = key {
                        clause_db.bump_activity(index)
                    };
                }

                _ => {
                    log::error!(target: targets::ATOMCELLS, "Trail exhausted without assertion\nClause: {:?}\nValueless count: {}", self.to_assertion_clause(), self.valueless_count);

                    panic!("! Resolution hit a decision/assumption")
                }
            };
        }

        trail.restore_assignments(the_trail);

        match self.valueless_count {
            0 | 1 => {
                let premises_switch = std::mem::take(&mut self.premises);
                self.make_callback_resolution_premises(&premises_switch);
                self.premises = premises_switch;

                Ok(ResolutionOk::UIP)
            }

            _ => {
                log::error!(target: targets::ATOMCELLS, "Trail exhausted without assertion\nClause: {:?}\nValueless count: {}", self.to_assertion_clause(), self.valueless_count);
                panic!("! A clause which does not assert");
            }
        }
    }

    /// Remove literals which conflict with those at level zero from the clause.
    pub fn strengthen_given<'l>(&mut self, literals: impl Iterator<Item = &'l CLiteral>) {
        for literal in literals {
            let cell = unsafe { self.buffer.get_unchecked_mut(literal.atom() as usize) };

            match cell.status {
                ResolutionStatus::Asserted | ResolutionStatus::Asserting => {
                    if let Some(length_minus_one) = self.clause_length.checked_sub(1) {
                        self.clause_length = length_minus_one;
                    }

                    cell.status = ResolutionStatus::Strengthened;
                }
                _ => {}
            }
        }
    }

    /// The atoms used during resolution.
    pub fn atoms_used(&mut self) -> impl Iterator<Item = Atom> + '_ {
        self.merged_atoms.sort_unstable();
        self.merged_atoms.iter().cloned()
    }

    pub fn take_premises(&mut self) -> HashSet<ClauseKey> {
        std::mem::take(&mut self.premises)
    }
}

impl AtomCells {
    /// Merge a clause into the resolution buffer, used to set up the resolution buffer and to merge additional clauses.
    ///
    /// Updates relevant 'value' cells in the resolution buffer to reflect their relation to the given clause along with connected metadata.
    ///
    /// Cells which have already been merged with some other clause are skipped.
    ///
    /// If the clause is satisfied and error is returned.
    fn merge_clause<C: Clause>(&mut self, clause: &C) -> Result<(), err::ResolutionBufferError> {
        log::info!(target: targets::ATOMCELLS, "Merging clause: {:?}", clause.as_dimacs(false));
        for literal in clause.literals() {
            let cell = unsafe { self.buffer.get_unchecked_mut(literal.atom() as usize) };

            match cell.status {
                ResolutionStatus::Proven
                | ResolutionStatus::Asserting
                | ResolutionStatus::Asserted
                | ResolutionStatus::Pivot
                | ResolutionStatus::Strengthened => {
                    // If present, cells of these kinds are from previously merged clauses.
                }

                ResolutionStatus::Backjump => {
                    self.clause_length += 1;
                    self.merged_atoms.push(literal.atom());

                    self.valueless_count += 1;
                    cell.status = ResolutionStatus::Asserted;
                }

                ResolutionStatus::Valuation => match cell.value {
                    None => {}

                    Some(value) if value != literal.polarity() => {
                        self.clause_length += 1;
                        self.merged_atoms.push(literal.atom());

                        cell.status = ResolutionStatus::Asserting;
                    }

                    Some(_) => {
                        log::error!(target: targets::ATOMCELLS, "Satisfied clause");
                        return Err(err::ResolutionBufferError::SatisfiedClause);
                    }
                },
            }
        }

        Ok(())
    }

    /// Resolves an additional clause into the buffer.
    ///
    /// Ensures the given pivot can be used to apply resolution with the given clause and the clause in the resolution buffer and applies resolution.
    // # Safety
    // The use of unwrap_unchecked in the conditional matches is safe as a cell must have already been verified to have some value in order to be marked as asserted or asserting.
    fn resolve_clause<C: Clause, L: Borrow<CLiteral>>(
        &mut self,
        clause: &C,
        pivot: L,
    ) -> Result<(), err::ResolutionBufferError> {
        let pivot = pivot.borrow();
        let cell = unsafe { self.buffer.get_unchecked_mut(pivot.atom() as usize) };
        match cell.status {
            ResolutionStatus::Asserted
                if pivot.polarity() == unsafe { cell.value.unwrap_unchecked() } =>
            {
                cell.status = ResolutionStatus::Pivot;
                self.merge_clause(clause)?;
                self.clause_length -= 1;

                self.valueless_count -= 1;

                Ok(())
            }

            ResolutionStatus::Asserting
                if pivot.polarity() == unsafe { cell.value.unwrap_unchecked() } =>
            {
                cell.status = ResolutionStatus::Pivot;
                self.merge_clause(clause)?;
                self.clause_length -= 1;

                Ok(())
            }

            _ => {
                // Skip over any clauses which are not involved in the current resolution trail
                Err(err::ResolutionBufferError::LostClause)
            }
        }
    }
}

impl AtomCells {
    pub fn get_assignment_source(&self, atom: Atom) -> &AssignmentSource {
        // # Safety: Every atom has a cell.
        unsafe { &self.buffer.get_unchecked(atom as usize).source }
    }
}
