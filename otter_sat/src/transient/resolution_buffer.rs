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

use std::{borrow::Borrow, collections::HashSet, rc::Rc};

use crate::{
    config::{Config, StoppingCriteria},
    db::{atom::AtomDB, clause::ClauseDB, literal::LiteralDB, ClauseKey},
    dispatch::{
        library::delta::{
            self,
            Resolution::{self},
        },
        macros::{self},
        Dispatch,
    },
    misc::log::targets::{self},
    structures::{
        atom::Atom,
        clause::{CClause, Clause},
        consequence,
        literal::{CLiteral, Literal},
        valuation::Valuation,
    },
    types::err::{self, ResolutionBufferError},
};

use super::{cell::Cell, config::BufferConfig, ResolutionOk};

/// A buffer for use when applying resolution to a sequence of clauses.
pub struct ResolutionBuffer {
    /// A count of literals in the clause whose atoms do not have a value on the given interpretation.
    valueless_count: usize,

    /// The length of the clause.
    clause_length: usize,

    /// The literal asserted by the current resolution candidate, if it exists..
    asserts: Option<CLiteral>,

    /// The (direct) premises used top derive the clause.
    premises: HashSet<ClauseKey>,

    /// The buffer.
    buffer: Vec<Cell>,

    /// Where to send dispatches.
    dispatcher: Option<Rc<dyn Fn(Dispatch)>>,

    /// A (typically derived) configuration for the instance of resolution.
    config: BufferConfig,
}

impl ResolutionBuffer {
    pub fn new(config: &Config, dispatcher: Option<Rc<dyn Fn(Dispatch)>>) -> Self {
        Self {
            valueless_count: 0,
            clause_length: 0,
            asserts: None,
            premises: HashSet::default(),
            buffer: Vec::default(),
            dispatcher,
            config: BufferConfig::from(config),
        }
    }

    pub fn refresh(&mut self, valuation: &impl Valuation) {
        self.valueless_count = 0;
        self.clause_length = 0;
        self.asserts = None;
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

    /// Returns the resolved clause and an index to where asserted literal is *within the clause*, if one exists.
    /// ```rust,ignore
    /// let (resolved_clause, assertion_index) = the_buffer.to_assertion_clause();
    /// ```
    pub fn to_assertion_clause(&self) -> (CClause, Option<usize>) {
        let mut the_clause = vec![];
        let mut conflict_index = None;

        for item in &self.buffer {
            match item {
                Cell::Strengthened | Cell::Value(_) | Cell::Pivot => {}
                Cell::Conflict(literal) => the_clause.push(*literal),
                Cell::None(literal) => {
                    if self.valueless_count == 1 {
                        conflict_index = Some(the_clause.size())
                    }
                    the_clause.push(*literal)
                }
            }
        }

        (the_clause, conflict_index)
    }

    /// Sets an atom to have no valuation in the resolution buffer.
    ///
    /// Useful to initialise the resolution buffer with the current valuation and then to 'roll it back' to the previous valuation.
    pub fn clear_atom_value(&mut self, atom: Atom) {
        unsafe { self.set(atom, Cell::Value(None)) }
    }

    /// Applies resolution with the clauses used to observe consequences at the current level.
    ///
    /// Clauses are examined in reverse order of use.
    pub fn resolve_through_current_level(
        &mut self,
        key: &ClauseKey,
        literal_db: &LiteralDB,
        clause_db: &mut ClauseDB,
        atom_db: &mut AtomDB,
    ) -> Result<ResolutionOk, err::ResolutionBufferError> {
        // The key has already been used to access the conflicting clause.
        let base_clause = match unsafe { clause_db.get_unchecked_mut(key) } {
            Ok(clause) => clause,
            Err(_) => return Err(err::ResolutionBufferError::MissingClause),
        };

        self.merge_clause(base_clause);
        base_clause.increment_proof_count();
        clause_db.note_use(*key);
        self.premises.insert(*key);

        // Maybe the conflit clause was already asserting after the previous decision…
        if let Some(literal) = self.asserted_literal() {
            return Ok(ResolutionOk::Repeat(*key, literal));
        };

        // bump clause activity
        if let ClauseKey::Addition(index, _) = key {
            clause_db.bump_activity(*index)
        };

        // Resolution buffer is only used by analysis, which is only called after some decision has been made
        let the_trail = unsafe { literal_db.top_consequences_unchecked().iter().rev() };
        'resolution_loop: for consequence in the_trail {
            match consequence.source() {
                consequence::Source::BCP(key) => {
                    let mut key = *key;

                    let source_clause = match unsafe { clause_db.get_unchecked_mut(&key) } {
                        Err(_) => {
                            log::error!(target: targets::RESOLUTION, "Lost resolution clause: {key}");
                            println!("Missing key");
                            return Err(err::ResolutionBufferError::LostClause);
                        }
                        Ok(clause) => clause,
                    };

                    // Recorded here to avoid multiple mutable borrows of clause_db
                    let source_clause_size = source_clause.size();

                    let resolution_result =
                        self.resolve_clause(source_clause, consequence.literal());

                    source_clause.increment_proof_count();
                    clause_db.note_use(key);
                    self.premises.insert(key);

                    if resolution_result.is_err() {
                        continue 'resolution_loop; // the clause wasn't relevant
                    }

                    key = match self.config.subsumption
                        && self.clause_length > 2
                        && self.clause_length < source_clause_size
                    {
                        false => key,
                        true => {
                            match key {
                                ClauseKey::OriginalUnit(_) | ClauseKey::AdditionUnit(_) => {
                                    panic!("! Subsumption called on a unit clause")
                                }

                                ClauseKey::OriginalBinary(_) | ClauseKey::AdditionBinary(_) => {
                                    panic!("! Subsumption called on a binary clause");
                                }

                                ClauseKey::Original(_) | ClauseKey::Addition(_, _) => unsafe {
                                    let premises = self.take_premises();

                                    // TODO: Subsumption should use the appropriate valuation
                                    let rekey = clause_db.subsume(
                                        key,
                                        consequence.literal(),
                                        atom_db,
                                        premises,
                                        true, // Increment the proof count as this is self-subsumption.
                                    )?;
                                    self.premises.insert(rekey);
                                    clause_db.note_use(rekey);
                                    rekey
                                },
                            }
                        }
                    };

                    if let ClauseKey::Addition(index, _) = key {
                        clause_db.bump_activity(index)
                    };
                }

                _ => panic!("! The resolution trail contains a literal whose source is not BCP"),
            };

            if self.valueless_count == 1 {
                match self.config.stopping {
                    StoppingCriteria::FirstUIP => {
                        break 'resolution_loop;
                    }
                    _ => {}
                }
            }
        }

        macros::dispatch_resolution_delta!(self, Resolution::Begin);
        for premise in &self.premises {
            macros::dispatch_resolution_delta!(self, Resolution::Used(*premise));
        }
        macros::dispatch_resolution_delta!(self, Resolution::End);

        match self.valueless_count {
            1 => Ok(ResolutionOk::UIP),
            _ => {
                println!("Exhausted");
                Err(ResolutionBufferError::Exhausted)
            }
        }
    }

    /// Remove literals which conflict with those at level zero from the clause.
    pub fn strengthen_given<'l>(&mut self, literals: impl Iterator<Item = &'l CLiteral>) {
        for literal in literals {
            match unsafe { *self.buffer.get_unchecked(literal.atom() as usize) } {
                Cell::None(_) | Cell::Conflict(_) => {
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
    /// ```rust,ignore
    /// self.atom_db.bump_relative(resolution_buffer.atoms_used());
    /// ```
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
        for literal in clause.literals() {
            match unsafe { self.buffer.get_unchecked(literal.atom() as usize) } {
                Cell::Conflict(_) | Cell::None(_) | Cell::Pivot => {}
                Cell::Value(maybe) => match maybe {
                    None => {
                        self.clause_length += 1;
                        self.valueless_count += 1;
                        unsafe { self.set(literal.atom(), Cell::None(literal)) };
                        if self.asserts.is_none() {
                            self.asserts = Some(literal);
                        }
                    }
                    Some(value) if *value != literal.polarity() => {
                        self.clause_length += 1;
                        unsafe { self.set(literal.atom(), Cell::Conflict(literal)) };
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
            Cell::None(literal) if pivot == &literal.negate() => {
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                unsafe { self.set(pivot.atom(), Cell::Pivot) };
                self.valueless_count -= 1;

                Ok(())
            }
            Cell::Conflict(literal) if pivot == &literal.negate() => {
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
        *self.buffer.get_unchecked_mut(atom as usize) = to
    }

    /// The literal asserted by the resolved clause, if it exists.
    fn asserted_literal(&self) -> Option<CLiteral> {
        if self.valueless_count == 1 {
            self.asserts
        } else {
            None
        }
    }
}
