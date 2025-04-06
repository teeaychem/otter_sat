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
        consequence::AssignmentSource,
        literal::{CLiteral, Literal},
        valuation::Valuation,
    },
    types::err::{self},
};

use super::{ResolutionBuffer, ResolutionOk, cell::Cell, config::BufferConfig};

impl ResolutionBuffer {
    pub fn new(config: &Config) -> Self {
        Self {
            valueless_count: 0,
            clause_length: 0,
            premises: HashSet::default(),
            buffer: Vec::default(),
            config: BufferConfig::from(config),
            callback_premises: None,
        }
    }

    pub fn refresh(&mut self, valuation: &impl Valuation) {
        self.valueless_count = 0;
        self.clause_length = 0;
        self.premises.clear();

        match self.buffer.len().cmp(&valuation.atom_count()) {
            std::cmp::Ordering::Less => self.buffer = valuation.values().map(Cell::Value).collect(),

            std::cmp::Ordering::Equal => unsafe {
                for index in 0..self.buffer.len() {
                    *self.buffer.get_unchecked_mut(index) =
                        Cell::Value(valuation.value_of_unchecked(index as Atom))
                }
            },

            std::cmp::Ordering::Greater => todo!(),
        }
    }

    /// The length of the resolved clause.
    pub fn clause_legnth(&self) -> usize {
        self.clause_length
    }

    /// Returns the resolved clause with the asserted literal as the first literal of the clause.
    pub fn to_assertion_clause(&self) -> CClause {
        let mut clause = Vec::with_capacity(self.clause_length);
        let mut conflict_index = 0;

        for (atom, cell) in self.buffer.iter().enumerate() {
            match cell {
                Cell::Strengthened | Cell::Value(_) | Cell::Pivot => {}

                Cell::Conflict(value) => clause.push(CLiteral::new(atom as Atom, *value)),

                Cell::Cleared(value) => {
                    conflict_index = clause.size();
                    clause.push(CLiteral::new(atom as Atom, *value));
                }
            }
        }

        if !clause.is_empty() {
            clause.swap(0, conflict_index);
        }

        clause
    }

    /// Sets an atom to have no valuation in the resolution buffer.
    ///
    /// Useful to initialise the resolution buffer with the current valuation and then to 'roll it back' to the previous valuation.
    pub fn clear_value(&mut self, atom: Atom) {
        unsafe { self.set(atom, Cell::Value(None)) }
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
        'resolution_loop: for consequence in the_trail.iter().rev() {
            if self.valueless_count <= 1 {
                match self.config.stopping {
                    StoppingCriteria::FirstUIP => {
                        break 'resolution_loop;
                    }
                    _ => {}
                }
            }

            log::info!(target: targets::RESOLUTION, "Examining trail item {consequence:?}");

            match consequence.source() {
                AssignmentSource::BCP(key) => {
                    let mut key = *key;

                    let source_clause = unsafe { clause_db.get_unchecked_mut(&key) };

                    // Recorded here to avoid multiple mutable borrows of clause_db
                    let source_clause_size = source_clause.size();

                    let resolution_result =
                        self.resolve_clause(source_clause, consequence.literal());

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
                                clause.subsume(consequence.literal(), atom_db, true)?;

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
                    log::error!(target: targets::RESOLUTION, "Trail exhausted without assertion");
                    log::error!(target: targets::RESOLUTION, "Clause: {:?}", self.to_assertion_clause());
                    log::error!(target: targets::RESOLUTION, "Valueless count: {}", self.valueless_count);

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
                log::error!(target: targets::RESOLUTION, "Trail exhausted without assertion");
                log::error!(target: targets::RESOLUTION, "Clause: {:?}", self.to_assertion_clause());
                log::error!(target: targets::RESOLUTION, "Valueless count: {}", self.valueless_count);
                panic!("! A clause which does not assert");
            }
        }
    }

    /// Remove literals which conflict with those at level zero from the clause.
    pub fn strengthen_given<'l>(&mut self, literals: impl Iterator<Item = &'l CLiteral>) {
        for literal in literals {
            match unsafe { *self.buffer.get_unchecked(literal.atom() as usize) } {
                Cell::Cleared(_) | Cell::Conflict(_) => {
                    if let Some(length_minus_one) = self.clause_length.checked_sub(1) {
                        self.clause_length = length_minus_one;
                    }
                    unsafe { self.set(literal.atom(), Cell::Strengthened) }
                }
                _ => {}
            }
        }
    }

    /// The atoms used during resolution.
    pub fn atoms_used(&self) -> impl Iterator<Item = Atom> + '_ {
        self.buffer
            .iter()
            .enumerate()
            .filter_map(|(index, cell)| match cell {
                Cell::Value(_) => None,
                _ => Some(index as Atom),
            })
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
            match unsafe { self.buffer.get_unchecked(literal.atom() as usize) } {
                Cell::Conflict(_) | Cell::Cleared(_) | Cell::Pivot => {}

                Cell::Value(maybe) => match maybe {
                    None => {
                        self.clause_length += 1;
                        self.valueless_count += 1;
                        unsafe { self.set(literal.atom(), Cell::Cleared(literal.polarity())) };
                    }

                    Some(value) if *value != literal.polarity() => {
                        self.clause_length += 1;
                        unsafe { self.set(literal.atom(), Cell::Conflict(literal.polarity())) };
                    }

                    Some(_) => {
                        log::error!(target: targets::RESOLUTION, "Satisfied clause");
                        return Err(err::ResolutionBufferError::SatisfiedClause);
                    }
                },
                Cell::Strengthened => {}
            }
        }
        Ok(())
    }

    /// Resolves an additional clause into the buffer.
    ///
    /// Ensures the given pivot can be used to apply resolution with the given clause and the clause in the resolution buffer and applies resolution.
    fn resolve_clause(
        &mut self,
        clause: &impl Clause,
        pivot: impl Borrow<CLiteral>,
    ) -> Result<(), err::ResolutionBufferError> {
        let pivot = pivot.borrow();
        let contents = unsafe { *self.buffer.get_unchecked(pivot.atom() as usize) };
        match contents {
            Cell::Cleared(value) if pivot.polarity() != value => {
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                unsafe { self.set(pivot.atom(), Cell::Pivot) };
                self.valueless_count -= 1;

                Ok(())
            }

            Cell::Conflict(value) if pivot.polarity() != value => {
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                unsafe { self.set(pivot.atom(), Cell::Pivot) };

                Ok(())
            }

            _ => {
                // Skip over any clauses which are not involved in the current resolution trail
                Err(err::ResolutionBufferError::LostClause)
            }
        }
    }

    /// Sets a cell corresponding to an atoms to the given enum case.
    unsafe fn set(&mut self, atom: Atom, to: Cell) {
        *unsafe { self.buffer.get_unchecked_mut(atom as usize) } = to
    }
}
