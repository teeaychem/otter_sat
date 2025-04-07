/*!
A structure derive the resolution of some collection of clauses with stopping points.

Resolution allows the derivation of a clause from a collection of clauses.

- The *resolution* of two formulas φ ∨ *p* and ψ ∨ *-p* is the formula φ ∨ ψ.
  + Here:
    - φ and ψ stand for arbitrary disjunctions, such as *q ∨ r ∨ s* and *t*, etc.
    - *p* is called the 'pivot' for the instance resolution.
      More generally:
      * A *pivot* for a pair of clauses *c₁* and *c₂* is some literal *l* such that *l* is in *c₁* and -*l* is in *c₂*.
        - For example, *q* is a pivot for  *p ∨ -q* and *p ∨ q ∨ r*, as *-q* is in the first and *q* in the second.
          Similarly, there are two pivots in the pair of clauses *p ∨ -q* and *-p ∨ q*.

Resolution is defined for a pair of formulas, but may be chained indefinetly so long as some pivot is present.
For example, given *p ∨ -q ∨ -r* and *-p*, resolution can be used to derive *-q ∨ -r* and in turn the clause *r ∨ s* can be used to derive *-q ∨ s*.

Further, it is often useful to stop resolution when a clause becomes asserting on some valuation.
That is, when all but one literal conflicts with the valuation, as then the non-conflicting literal must hold on the valuation.

The structure here allows for an arbitrary chain of resolution instances with stopping points by:
- Setting up a vector containing cells for all atoms that may be relevant to the resolution chain.
- Updating the contents of each cell to indicate whether that atom is part of the derived clause, or has been used as a pivot.
- While, keeping track of which cells used in resolution conflict with the valuation.

In addition, the structure has been extended to support self-subsumption of clauses and clause strengthening.


Note, at present, the structure creates a cell for each atom in the context.
This allows for a simple implementation, but is likely inefficient for a large collection of atoms.
Improvement could be made by temporarily mapping relevant atoms to a temporary sub-language derived from the clauses which are candidates for resolution (so long as this is a finite collection…)
*/

use std::{borrow::Borrow, collections::HashSet};

use crate::{
    config::{Config, StoppingCriteria},
    db::{ClauseKey, atom::AtomDB, clause::ClauseDB},
    misc::log::targets::{self},
    structures::{
        atom::Atom,
        clause::{CClause, Clause},
        consequence::{Assignment, AssignmentSource},
        literal::{CLiteral, Literal},
    },
    types::err::{self},
};

use super::{
    ResolutionBuffer, ResolutionOk,
    cell::{Cell, CellStatus},
    config::BufferConfig,
};

impl ResolutionBuffer {
    pub fn new(config: &Config) -> Self {
        Self {
            valueless_count: 0,
            clause_length: 0,
            premises: HashSet::default(),
            buffer: Vec::default(),
            merged_atoms: Vec::default(),
            config: BufferConfig::from(config),
            callback_premises: None,
        }
    }

    pub fn refresh(&mut self) {
        self.valueless_count = 0;
        self.clause_length = 0;
        self.premises.clear();
        self.merged_atoms.clear();
    }

    pub fn grow_to_include(&mut self, atom: Atom) {
        if self.buffer.len() <= atom as usize {
            self.buffer.resize(atom as usize + 1, Cell::default());
        }
    }

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
                CellStatus::Valuation | CellStatus::Backjump => {}

                CellStatus::Strengthened | CellStatus::Pivot => {}

                CellStatus::Asserting => {
                    clause.push(cell.clone().assignment.unwrap().literal().negate())
                }

                CellStatus::Asserted => {
                    asserted_index = clause.size();
                    clause.push(cell.clone().assignment.unwrap().literal().negate());
                }
            }

            cell.status = CellStatus::Valuation;
        }

        if !clause.is_empty() {
            clause.swap(0, asserted_index);
        }

        clause
    }

    pub fn set_valuation(
        &mut self,
        atom: Atom,
        value: Option<bool>,
        assignment: Option<Assignment>,
    ) {
        let cell = unsafe { self.buffer.get_unchecked_mut(atom as usize) };
        cell.value = value;
        cell.assignment = assignment;
        cell.status = CellStatus::Valuation;
    }

    pub fn mark_backjump(&mut self, atom: Atom) {
        let cell = unsafe { self.buffer.get_unchecked_mut(atom as usize) };
        cell.status = CellStatus::Backjump;
    }

    /// Sets an atom to have no valuation in the resolution buffer.
    ///
    /// Useful to initialise the resolution buffer with the current valuation and then to 'roll it back' to the previous valuation.
    pub fn clear_value(&mut self, atom: Atom) {
        let cell = unsafe { self.buffer.get_unchecked_mut(atom as usize) };
        cell.value = None;
        cell.status = CellStatus::Valuation;
    }

    /// Applies resolution with the clauses used to observe consequences at the current level.
    ///
    /// Clauses are examined in reverse order of use.
    pub fn resolve_through_current_level(
        &mut self,
        key: &ClauseKey,
        clause_db: &mut ClauseDB,
        atom_db: &mut AtomDB,
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
        let the_trail = atom_db.take_assignments();
        'resolution_loop: for assignment in the_trail.iter().rev() {
            if self.valueless_count <= 1 {
                match self.config.stopping {
                    StoppingCriteria::FirstUIP => {
                        break 'resolution_loop;
                    }
                    _ => {}
                }
            }

            log::info!(target: targets::RESOLUTION, "Examining trail item {assignment:?}");

            // TODO: Fix up
            let literal = assignment;
            let source = *self
                .buffer
                .get(literal.atom() as usize)
                .unwrap()
                .get_assignment()
                .clone()
                .unwrap()
                .source();

            match source {
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

                    key = match self.config.subsumption
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
                                clause.subsume(literal, atom_db, true)?;

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
                    log::error!(target: targets::RESOLUTION, "Trail exhausted without assertion\nClause: {:?}\nValueless count: {}", self.to_assertion_clause(), self.valueless_count);

                    panic!("! Resolution hit a decision/assumption")
                }
            };
        }

        atom_db.restore_assignments(the_trail);

        match self.valueless_count {
            0 | 1 => {
                let premises_switch = std::mem::take(&mut self.premises);
                self.make_callback_resolution_premises(&premises_switch);
                self.premises = premises_switch;

                Ok(ResolutionOk::UIP)
            }

            _ => {
                log::error!(target: targets::RESOLUTION, "Trail exhausted without assertion\nClause: {:?}\nValueless count: {}", self.to_assertion_clause(), self.valueless_count);
                panic!("! A clause which does not assert");
            }
        }
    }

    /// Remove literals which conflict with those at level zero from the clause.
    pub fn strengthen_given<'l>(&mut self, literals: impl Iterator<Item = &'l CLiteral>) {
        for literal in literals {
            let cell = unsafe { self.buffer.get_unchecked_mut(literal.atom() as usize) };

            match cell.status {
                CellStatus::Asserted | CellStatus::Asserting => {
                    if let Some(length_minus_one) = self.clause_length.checked_sub(1) {
                        self.clause_length = length_minus_one;
                    }

                    cell.status = CellStatus::Strengthened;
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

// Private methods

impl ResolutionBuffer {
    /// Merge a clause into the resolution buffer, used to set up the resolution buffer and to merge additional clauses.
    ///
    /// Updates relevant 'value' cells in the resolution buffer to reflect their relation to the given clause along with connected metadata.
    ///
    /// Cells which have already been merged with some other clause are skipped.
    ///
    /// If the clause is satisfied and error is returned.
    fn merge_clause(&mut self, clause: &impl Clause) -> Result<(), err::ResolutionBufferError> {
        log::info!(target: targets::RESOLUTION, "Merging clause: {:?}", clause.as_dimacs(false));
        for literal in clause.literals() {
            let cell = unsafe { self.buffer.get_unchecked_mut(literal.atom() as usize) };

            match cell.status {
                CellStatus::Asserting
                | CellStatus::Asserted
                | CellStatus::Pivot
                | CellStatus::Strengthened => {
                    // If present, cells of these kinds are from previously merged clauses.
                }

                CellStatus::Backjump => {
                    self.clause_length += 1;
                    self.merged_atoms.push(literal.atom());

                    self.valueless_count += 1;
                    cell.status = CellStatus::Asserted;
                }

                CellStatus::Valuation => match cell.value {
                    None => {}

                    Some(value) if value != literal.polarity() => {
                        self.clause_length += 1;
                        self.merged_atoms.push(literal.atom());

                        cell.status = CellStatus::Asserting;
                    }

                    Some(_) => {
                        log::error!(target: targets::RESOLUTION, "Satisfied clause");
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
    fn resolve_clause(
        &mut self,
        clause: &impl Clause,
        pivot: impl Borrow<CLiteral>,
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

impl ResolutionBuffer {
    pub fn get_assignment(&self, atom: Atom) -> &Option<Assignment> {
        match self.buffer.get(atom as usize) {
            None => &None,
            Some(cell) => &cell.assignment,
        }
    }
}
