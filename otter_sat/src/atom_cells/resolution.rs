use std::{borrow::Borrow, collections::HashSet};

use crate::{
    config::{Config, StoppingCriteria, StrengtheningCriteria},
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

use super::{AtomCells, ResolutionOk, cell::CellStatus};

impl AtomCells {
    /// The length of the resolved clause.
    pub fn clause_legnth(&self) -> usize {
        self.clause_length
    }

    /// Returns the resolved clause with the asserted literal as the first literal of the clause.
    pub fn to_assertion_clause(&mut self, clause_db: &mut ClauseDB, config: &Config) -> CClause {
        let mut clause = Vec::with_capacity(self.clause_length);
        let mut asserted_index = 0;

        let mut index = 0;
        let limit = self.merged_atoms.len();

        while index < limit {
            let atom = unsafe { *self.merged_atoms.get_unchecked(index) };

            match unsafe { self.buffer.get_unchecked_mut(atom as usize) }.status {
                CellStatus::Valuation | CellStatus::Backjump => {}

                CellStatus::Proven | CellStatus::Pivot => {}

                CellStatus::Asserting => {
                    let cell = unsafe { self.buffer.get_unchecked_mut(atom as usize) };
                    let literal = CLiteral::new(atom, !unsafe { cell.value.unwrap_unchecked() });

                    match &config.strengthening.value {
                        StrengtheningCriteria::RecursiveBCP => {
                            match self.derivable_literal(atom, clause_db) {
                                true => {}
                                false => {
                                    clause.push(literal);
                                }
                            }
                        }

                        StrengtheningCriteria::None => clause.push(literal),
                    }
                }

                CellStatus::Asserted => {
                    asserted_index = clause.size();
                    let cell = unsafe { self.buffer.get_unchecked_mut(atom as usize) };
                    let literal = CLiteral::new(atom, !unsafe { cell.value.unwrap_unchecked() });
                    clause.push(literal);
                }

                CellStatus::Independent => panic!("! Non-valuation atom marked as required"),

                CellStatus::Removable => {}
            }

            index += 1;
        }

        if !clause.is_empty() {
            clause.swap(0, asserted_index);
        }

        self.restore_cached_removable_status();

        for atom in &self.merged_atoms {
            let cell = unsafe { self.buffer.get_unchecked_mut(*atom as usize) };
            if !matches!(cell.status, CellStatus::Proven) {
                cell.status = CellStatus::Valuation;
            }
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
                    log::error!(target: targets::ATOMCELLS, "Trail exhausted without assertion\nClause: {:?}\nValueless count: {}", self.to_assertion_clause(clause_db, config), self.valueless_count);

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
                log::error!(target: targets::ATOMCELLS, "Trail exhausted without assertion\nClause: {:?}\nValueless count: {}", self.to_assertion_clause(clause_db, config), self.valueless_count);
                panic!("! A clause which does not assert");
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
                CellStatus::Asserting | CellStatus::Asserted | CellStatus::Pivot => {
                    // If present, cells of these kinds are from previously merged clauses.
                }

                CellStatus::Backjump => {
                    self.clause_length += 1;
                    self.merged_atoms.push(literal.atom());

                    self.valueless_count += 1;
                    cell.status = CellStatus::Asserted;
                }

                CellStatus::Proven => {}

                CellStatus::Valuation => match cell.value {
                    None => {}

                    Some(value) if value != literal.polarity() => {
                        self.clause_length += 1;
                        self.merged_atoms.push(literal.atom());

                        cell.status = CellStatus::Asserting;
                    }

                    Some(_) => {
                        log::error!(target: targets::ATOMCELLS, "Satisfied clause");
                        return Err(err::ResolutionBufferError::SatisfiedClause);
                    }
                },

                CellStatus::Independent | CellStatus::Removable => {
                    panic!("! Uncleared removable tags")
                }
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
            CellStatus::Asserted
                if pivot.polarity() == unsafe { cell.value.unwrap_unchecked() } =>
            {
                cell.status = CellStatus::Pivot;
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                self.valueless_count -= 1;

                Ok(())
            }

            CellStatus::Asserting
                if pivot.polarity() == unsafe { cell.value.unwrap_unchecked() } =>
            {
                cell.status = CellStatus::Pivot;
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

    fn set_status(&mut self, atom: Atom, status: CellStatus) {
        let cell = self.get_cell_mut(atom);
        if cell.status == CellStatus::Valuation {
            cell.status = status;
            self.cached_removable_status_atoms.push(atom);
        }
    }

    /// Caches atoms_to_restore.
    pub fn derivable_literal(&mut self, atom: Atom, clause_db: &mut ClauseDB) -> bool {
        /*
        The core task is DFS through the derivation of the literal.

        This is done iteratively with a stack, and two variables tracking a key to clause and the index of some literal.
        So long as the index is to some literal in the clause, DFS may continue.
        Whenever the index is not to some literal in the clause, this property is restored by taking a clause and index from the stack, or else the literal can be derived.
         */
        let mut key: ClauseKey = {
            match self.get_assignment_source(atom) {
                AssignmentSource::BCP(source_key) => *source_key,

                _ => return false,
            }
        };

        self.removable_dfs_todo.clear();

        let mut index = 1;

        let mut clause = unsafe { clause_db.get_unchecked_mut(&key) };
        if (clause.size() == 2) && atom != unsafe { clause.atom_at_unchecked(0) } {
            index = 0;
        }

        'dfs_loop: loop {
            if clause.size() == index {
                let check_atom = unsafe { clause.atom_at_unchecked(0) };
                self.set_status(check_atom, CellStatus::Removable);

                if clause.size() == 2 {
                    let check_atom = unsafe { clause.atom_at_unchecked(1) };
                    self.set_status(check_atom, CellStatus::Removable);
                }

                if self.removable_dfs_todo.is_empty() {
                    return false;
                } else {
                    (key, index) = self.removable_dfs_todo.pop().unwrap();

                    clause = unsafe { clause_db.get_unchecked_mut(&key) };

                    continue 'dfs_loop;
                }
            }

            // Get the releant literal for inspection, may fail
            if let Some(check_atom) = clause.atom_at(index) {
                // check the source of the source of the literal.

                match self.get_cell(check_atom).status {
                    CellStatus::Proven | CellStatus::Removable => {
                        // The literal is proven.
                        // Or, the literal is provable given the other elements of the learnt caluse.
                        index = if clause.size() == 2 { 2 } else { index + 1 };

                        continue 'dfs_loop;
                    }

                    CellStatus::Asserted | CellStatus::Asserting => {
                        // Literals in the learnt clause, so continue.
                        // As, of interest is whether the given literal is derivable given those literals.
                        index = if clause.size() == 2 { 2 } else { index + 1 };

                        continue 'dfs_loop;
                    }

                    CellStatus::Independent => {
                        // Removing the given literal would require some other literal.
                        // So, tidy and return false.

                        while let Some((key, index)) = self.removable_dfs_todo.pop() {
                            if let Some(atom) =
                                unsafe { clause_db.get_unchecked(&key) }.atom_at(index)
                            {
                                self.set_status(atom, CellStatus::Independent);
                            };
                        }

                        return false;
                    }

                    CellStatus::Valuation => {
                        // Clone to avoid double borrow
                        match *self.get_assignment_source(check_atom) {
                            AssignmentSource::Original
                            | AssignmentSource::Addition
                            | AssignmentSource::Pure => {
                                // Original or adddition units should be handled by the outer match on CellStatus::Proven.
                                // In any case, continue the search.
                                index = if clause.size() == 2 { 2 } else { index + 1 };

                                continue 'dfs_loop;
                            }

                            AssignmentSource::Decision | AssignmentSource::Assumption => {
                                // Removing the given literal would require some other literal.
                                // (Specifically, the decision or assumption.)
                                // So, tidy and return false.

                                self.get_cell_mut(check_atom).status = CellStatus::Independent;
                                self.cached_removable_status_atoms.push(check_atom);

                                while let Some((key, index)) = self.removable_dfs_todo.pop() {
                                    if let Some(atom) =
                                        unsafe { clause_db.get_unchecked(&key) }.atom_at(index)
                                    {
                                        self.set_status(atom, CellStatus::Independent);
                                    };
                                }

                                return false;
                            }

                            AssignmentSource::BCP(source_key) => {
                                index = 1;

                                if clause.size() == 2 {
                                    self.removable_dfs_todo.push((key, 2));

                                    clause = unsafe { clause_db.get_unchecked_mut(&source_key) };
                                    if unsafe { check_atom != clause.atom_at_unchecked(0) } {
                                        index = 0;
                                    }
                                } else {
                                    self.removable_dfs_todo.push((key, index + 1));

                                    clause = unsafe { clause_db.get_unchecked_mut(&source_key) };
                                }

                                key = source_key;
                            }

                            AssignmentSource::None => panic!("! Missing assignment in BCP DFS"),
                        }
                    }

                    CellStatus::Pivot | CellStatus::Backjump => {
                        // No pivot atom can appear as the learnt clause was obtained by resolution on the pivot.
                        // No backjump status can appear as it will have been replaced by asserted/asserting status.
                        panic!("! Invalid cell status within resolution")
                    }
                }
            } else if let Some((next_key, next_index)) = self.removable_dfs_todo.pop() {
                key = next_key;
                index = next_index;

                clause = unsafe { clause_db.get_unchecked_mut(&key) };

                continue 'dfs_loop;
            }
        }
    }

    pub fn restore_cached_removable_status(&mut self) {
        while let Some(atom) = self.cached_removable_status_atoms.pop() {
            let cell = self.get_cell_mut(atom);
            cell.status = CellStatus::Valuation;
        }
    }
}
