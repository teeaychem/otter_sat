use std::{borrow::Borrow, collections::HashSet};

use crate::{
    config::{Config, MinimizationCriteria, StoppingCriteria},
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

use super::{
    AtomCells, ReMiTodo, ResolutionOk,
    cell::{AtomCell, ResolutionFlag as Flag},
};

impl AtomCells {
    /// Returns the resolved clause with the asserted literal as the first literal of the clause.
    pub fn to_assertion_clause(&mut self, clause_db: &mut ClauseDB, config: &Config) -> CClause {
        let mut clause = Vec::with_capacity(self.clause_length);

        let mut index = 0;
        let limit = self.merged_atoms.len();

        let mut atom;
        let mut atom_cell;

        while index < limit {
            // # Safety: index is bounded by merged_atoms.len()
            atom = unsafe { *self.merged_atoms.get_unchecked(index) };
            atom_cell = self.get_cell(atom);

            // # Safety: As the atom has been merged, it has some value.
            let cell_value = unsafe { atom_cell.value.unwrap_unchecked() };

            match atom_cell.resolution_flag {
                Flag::Valuation | Flag::Backjump | Flag::Proven | Flag::Pivot | Flag::Derivable => {
                }

                Flag::Asserting => {
                    match &config.minimization.value {
                        MinimizationCriteria::Proven // Proven literals are already flagged
                        | MinimizationCriteria::None
                        | MinimizationCriteria::Recursive if !self.derivable_value(atom, clause_db) => {
                             {
                                clause.push(CLiteral::new(atom, !cell_value));
                            }
                        }

                        _ => {}
                    }
                }

                Flag::Asserted => {
                    let asserted_index = clause.size();
                    clause.push(CLiteral::new(atom, !cell_value));
                    clause.swap(0, asserted_index);
                }

                Flag::Independent => panic!("! Non-valuation atom marked as required"),
            }

            index += 1;
        }

        self.restore_cached_removable_status();

        while let Some(atom) = self.merged_atoms.pop() {
            let cell = self.get_cell_mut(atom);
            if !matches!(cell.resolution_flag, Flag::Proven) {
                cell.resolution_flag = Flag::Valuation;
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
        let mut key = *key;

        self.merge_clause(base_clause);
        clause_db.note_use(key);
        self.premises.insert(key);

        /*
        bump clause activity
        */
        if let ClauseKey::Addition(index, _) = key {
            clause_db.bump_activity(index)
        };

        let mut source_clause;
        let mut source_clause_size;
        let mut resolution_result;

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

            match self.get_assignment_source(literal.atom()) {
                AssignmentSource::None => panic!("! Missing source"),

                AssignmentSource::BCP(bcp_key) => {
                    key = *bcp_key;

                    source_clause = unsafe { clause_db.get_unchecked_mut(&key) };
                    source_clause_size = source_clause.size(); // Recorded here to avoid multiple mutable borrows of clause_db
                    resolution_result = self.resolve_clause(source_clause, literal);

                    clause_db.note_use(key);
                    self.premises.insert(key);

                    if resolution_result.is_err() {
                        continue 'resolution_loop; // the clause wasn't relevant
                    }

                    if self.clause_length < source_clause_size && config.subsumption.value {
                        match key {
                            ClauseKey::OriginalUnit(_) | ClauseKey::AdditionUnit(_) => {}

                            ClauseKey::OriginalBinary(_) | ClauseKey::AdditionBinary(_) => {}

                            ClauseKey::Original(_) | ClauseKey::Addition(_, _) => {
                                let clause = unsafe { clause_db.get_unchecked_mut(&key) };
                                clause.subsume(literal, self, watch_dbs, true)?;

                                self.premises.insert(key);
                                clause_db.note_use(key);
                            }
                        }
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
            let cell = unsafe { self.cells.get_unchecked_mut(literal.atom() as usize) };

            match cell.resolution_flag {
                Flag::Asserting | Flag::Asserted | Flag::Pivot | Flag::Proven => {
                    // If present, cells of these kinds are from previously merged clauses.
                }

                Flag::Backjump => {
                    self.clause_length += 1;
                    self.merged_atoms.push(literal.atom());

                    self.valueless_count += 1;
                    cell.resolution_flag = Flag::Asserted;
                }

                Flag::Valuation => match cell.value {
                    None => {}

                    Some(value) if value != literal.polarity() => {
                        self.clause_length += 1;
                        self.merged_atoms.push(literal.atom());

                        cell.resolution_flag = Flag::Asserting;
                    }

                    Some(_) => {
                        log::error!(target: targets::ATOMCELLS, "Satisfied clause");
                        return Err(err::ResolutionBufferError::SatisfiedClause);
                    }
                },

                Flag::Independent | Flag::Derivable => {
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
        let cell = unsafe { self.cells.get_unchecked_mut(pivot.atom() as usize) };
        match cell.resolution_flag {
            Flag::Asserted if pivot.polarity() == unsafe { cell.value.unwrap_unchecked() } => {
                cell.resolution_flag = Flag::Pivot;
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                self.valueless_count -= 1;

                Ok(())
            }

            Flag::Asserting if pivot.polarity() == unsafe { cell.value.unwrap_unchecked() } => {
                cell.resolution_flag = Flag::Pivot;
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
    /// Returns the [AssignmentSource] of an [Atom].
    pub fn get_assignment_source(&self, atom: Atom) -> &AssignmentSource {
        // # Safety: Every atom has a cell.
        unsafe { &self.cells.get_unchecked(atom as usize).source }
    }

    /// A helper method to cache information during [derivable_value](AtomCells::derivable_value).
    fn set_status(&mut self, atom: Atom, status: Flag) {
        let cell = self.get_cell_mut(atom);
        if cell.resolution_flag == Flag::Valuation {
            cell.resolution_flag = status;
            self.cached_removable_status_atoms.push(atom);
        }
    }

    /// Helper method to mark the DFS stack as independent when [derivable_value](AtomCells::derivable_value) returns false.
    fn flag_stack_independent(&mut self, clause_db: &mut ClauseDB) {
        while let Some(ReMiTodo { key, index }) = self.recursive_minimization_todo.pop() {
            if let Some(atom) = unsafe { clause_db.get_unchecked(&key) }.atom_at(index) {
                self.set_status(atom, Flag::Independent);
            };
        }
    }

    /// Returns true when the current value of `atom` in the current (implicit) learnt clause is a consequence of the current value of the other atoms in the learnt clause.
    ///
    /// If the method returns true, then the atom may be omitted from the learnt clause.
    ///
    /// Note, the method depends on atom cells retaining all relevant information leading to the derivation of a conflict.
    /// And, is intended to be called on 'antecedent' literals when finalising a learnt clause.
    ///
    /// Given a clause of the form: -a_0, ..., -a_l, b, the method returns true when there is an observed entailment a_i, ..., a_j => a_k, for 0 ≤ i,j,k ≤ l.
    /// So, if the clause is used to obtain the entailment a_0, ..., a_l => b, there is no need to establish a_k, as it follows from some subset of the established premises.
    /// And, for any other entailment, it is not possible to establish -a_k without also establishing -a_k', for some i ≤ k' ≤ j.
    ///
    /// From a practical perspective, a_k is not required for any entailment using the learnt clause and cannot be set by the learnt clause.
    ///
    /// The method is reasonably efficient as the check for some entailment is limited to examining entailments in the current BCP history.
    /// And, in particular, does not return true when there is an unobserved entailment a_i, ..., a_j => a_k.
    /// To do so would be quire difficult.
    ///
    /// With respect to the details, the method performs depth first search on the BCP history of the atom, and terminates with false whenever a decision or assumption is found.
    /// Otherwise, the value of each atom used to propagate is proven or part of the clause and so the relevant entailment holds.
    /// Use is made of the invariant to keep the propagated atom/literal as the first element of any long clause to skip inspection of *propagated* atoms, though as this invariant is not upheld for binary clauses, the atom to examine is determined case-by-case.
    ///
    /// The status of literals are cached for repeat calls, and [restore_cached_removable_status](AtomCells::restore_cached_removable_status) must be called after the learnt clause is finalised to clear the cache.
    pub fn derivable_value(&mut self, atom: Atom, clause_db: &mut ClauseDB) -> bool {
        let mut key: ClauseKey = {
            match self.get_assignment_source(atom) {
                AssignmentSource::BCP(initial_key) => *initial_key,

                _ => return false,
            }
        };

        // The index of the atom being checking in the clause.
        // Set to 1 by default to avoid inspecting asserted atoms in long clauses (this rests on upholding maininting the invariant that assrted atoms are moved to the first position of an asserting clause).
        // # Safety: It is always the case that index  ≤ clause size, and that index < clause size when an possible to inspect an atom.
        let mut index: usize = 1;
        let mut check_atom: Atom; // The atom at the index to check.
        let mut check_cell: &AtomCell; // The cell of the atom to check.

        // # Safety: Each key is obtained from inspecting the most recent round of BCP, and so each clause is present in the clause database.
        let mut clause = unsafe { clause_db.get_unchecked_mut(&key) };

        // index is set to 1 by default, but as binary clauses do not uphold the relevant invariant, switch the index to the non-asserted clause if needed.
        if (clause.size() == 2) && atom != unsafe { clause.atom_at_unchecked(0) } {
            index = 0;
        }

        'dfs_loop: loop {
            // Exhaustion of the current clause is indicated by index holding the size of the clause.
            // And, if exhausted the atom asserted by the clause can be derived from the current valuation.
            if index == clause.size() {
                self.set_status(unsafe { clause.atom_at_unchecked(0) }, Flag::Derivable);

                // In the case of a binary clause it isn't known which atom is asserted, though both must be derivable and so the status of both is cached.
                if clause.size() == 2 {
                    self.set_status(unsafe { clause.atom_at_unchecked(1) }, Flag::Derivable);
                }

                // If the DFS stack has been emptied, the initial clause was derivable. Otherwise, backtrack to the previous clause/index.
                if let Some(next) = self.recursive_minimization_todo.pop() {
                    key = next.key;
                    index = next.index;
                    clause = unsafe { clause_db.get_unchecked_mut(&key) };

                    continue 'dfs_loop;
                } else {
                    return true;
                }
            }

            // Get the releant literal for inspection.
            // # Safety: It must be that index < clause size for the previous if to fall through.
            check_atom = unsafe { clause.atom_at_unchecked(index) };
            check_cell = self.get_cell(check_atom);
            // check the source of the source of the literal.

            // Literals in the learnt clause, so continue.
            // As, of interest is whether the given literal is derivable given those literals.
            match check_cell.resolution_flag {
                Flag::Proven | Flag::Derivable | Flag::Asserted | Flag::Asserting => {
                    index = if clause.size() == 2 { 2 } else { index + 1 };

                    continue 'dfs_loop;
                }

                Flag::Independent => {
                    self.flag_stack_independent(clause_db);

                    return false;
                }

                Flag::Valuation => {
                    // Clone to avoid double borrow
                    match *self.get_assignment_source(check_atom) {
                        AssignmentSource::Original
                        | AssignmentSource::Addition
                        | AssignmentSource::Pure => {
                            // Original or adddition units should be handled by the outer match on Proven. In any case, continue the search.
                            index = if clause.size() == 2 { 2 } else { index + 1 };

                            continue 'dfs_loop;
                        }

                        AssignmentSource::Decision | AssignmentSource::Assumption => {
                            self.set_status(check_atom, Flag::Independent);
                            self.flag_stack_independent(clause_db);

                            return false;
                        }

                        AssignmentSource::BCP(source_key) => {
                            // If BCP then immediately store the current clause/index on the stack to return to an move to explore the clause.
                            // Though, for effiency on the stack is the next index to explore, or the size of the clause to indicate exhaustion.
                            // Given this, what happens depends on whether the clause is binary or long.
                            index = 1;

                            if clause.size() == 2 {
                                self.recursive_minimization_todo
                                    .push(ReMiTodo { key, index: 2 });

                                clause = unsafe { clause_db.get_unchecked_mut(&source_key) };

                                // Fix the index, if required.
                                if unsafe { check_atom != clause.atom_at_unchecked(0) } {
                                    index = 0;
                                }
                            } else {
                                self.recursive_minimization_todo.push(ReMiTodo {
                                    key,
                                    index: index + 1,
                                });

                                clause = unsafe { clause_db.get_unchecked_mut(&source_key) };
                            }

                            key = source_key;
                        }

                        AssignmentSource::None => panic!("! Missing assignment in BCP DFS"),
                    }
                }

                Flag::Pivot | Flag::Backjump => {
                    // No pivot atom can appear as the learnt clause was obtained by resolution on the pivot.
                    // No backjump status can appear as it will have been replaced by asserted/asserting status.
                    panic!("! Invalid cell status within resolution")
                }
            }
        }
    }

    /// Clears the status values made during [derivable_value](AtomCells::derivable_value) for efficient search.
    /// Must be called immediately after a learnt clause has been finalised (and may be called before if some inefficiancy is to taste).
    fn restore_cached_removable_status(&mut self) {
        while let Some(atom) = self.cached_removable_status_atoms.pop() {
            self.get_cell_mut(atom).resolution_flag = Flag::Valuation;
        }
    }
}
