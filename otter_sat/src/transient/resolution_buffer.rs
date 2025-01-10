//! A structure derive the resolution of some collection of clauses with stopping points.
//!
//! Resolution allows the derivation of a clause from a collection of clauses.
//!
//! - The *resolution* of two formulas φ ∨ *p* and ψ ∨ *-p* is the formula φ ∨ ψ.
//!   + Here:
//!     - φ and ψ stand for arbitrary disjunctions, such as *q ∨ r ∨ s* and *t*, etc.
//!     - *p* is called the 'pivot' for the instance resolution.
//!       More generally:
//!       * A *pivot* for a pair of clauses *c₁* and *c₂* is some literal *l* such that *l* is in *c₁* and -*l* is in *c₂*.
//!         - For example, *q* is a pivot for  *p ∨ -q* and *p ∨ q ∨ r*, as *-q* is in the first and *q* in the second.
//!           Similarly, there are two pivots in the pair of clauses *p ∨ -q* and *-p ∨ q*.
//!
//! Resolution is defined for a pair of formulas, but may be chained indefinetly so long as some pivot is present.
//! For example, given *p ∨ -q ∨ -r* and *-p*, resolution can be used to derive *-q ∨ -r* and in turn the clause *r ∨ s* can be used to derive *-q ∨ s*.
//!
//! Further, it is often useful to stop resolution when a clause becomes asserting on some valuation.
//! That is, when all but one literal conflicts with the valuation, as then the non-conflicting literal must hold on the valuation.
//!
//! The structure here allows for an arbitrary chain of resolution instances with stopping points by:
//! - Setting up a vector containing cells for all atoms that may be relevant to the resolution chain.
//! - Updating the contents of each cell to indicate whether that atom is part of the derived clause, or has been used as a pivot.
//! - While, keeping track of which cells used in resolution conflict with the valuation.
//!
//! In addition, the structure has been extended to support self-subsumption of clauses and clause strengthening.
//!
//!
//! Note, at present, the structure creates a cell for each atom in the context.
//! This allows for a simple implementation, but is likely inefficient for a large collection of atoms.
//! Improvement could be made by temporarily mapping relevant atoms to a temporary sub-language derived from the clauses which are candidates for resolution (so long as this is a finite collection…)

use std::{borrow::Borrow, rc::Rc};

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
        clause::{vClause, Clause},
        literal::{self, abLiteral, Literal},
        valuation::Valuation,
    },
    types::err::{self},
};

/// Possilbe 'Ok' results from resolution using a resolution buffer.
pub enum Ok {
    /// A unique implication point was identified.
    FirstUIP,

    /// Resolution was applied to every clause used at the given level.
    Exhausted,

    /// Resolution produced a unit clause.
    UnitClause,

    /// Resolution identified a clause already in the database.
    Missed(ClauseKey, abLiteral),
}

/// Cells of a resolution buffer.
#[derive(Clone, Copy)]
pub enum Cell {
    /// Initial valuation
    Value(Option<bool>),

    /// The atom was not valued.
    None(abLiteral),

    /// The atom had a conflicting value.
    Conflict(abLiteral),

    /// The atom was part of resolution but was already proven.
    Strengthened,

    /// The atom was used as a pivot when reading a clause into the buffer.
    Pivot,
}

/// A buffer for use when applying resolution to a sequence of clauses.
pub struct ResolutionBuffer {
    /// A count of literals in the clause whose atoms do not have a value on the given interpretation.
    valueless_count: usize,

    /// The length of the clause.
    clause_length: usize,

    /// The literal asserted by the current resolution candidate, if it exists
    asserts: Option<abLiteral>,

    /// The buffer
    buffer: Vec<Cell>,

    /// Where to send dispatches
    dispatcher: Option<Rc<dyn Fn(Dispatch)>>,

    /// A (typically derived) configuration for the instance of resolution
    config: BufferConfig,
}

/// Configuration for a resolution buffer.
pub struct BufferConfig {
    /// Whether check for and initiate subsumption.
    subsumption: bool,
    /// The stopping criteria to use during resolution.
    stopping: StoppingCriteria,
}

impl ResolutionBuffer {
    /// The length of the resolved clause.
    pub fn clause_legnth(&self) -> usize {
        self.clause_length
    }

    /// Creates a resolution buffer from a valuation, typically the current valuation.
    pub fn from_valuation(
        valuation: &impl Valuation,
        dispatcher: Option<Rc<dyn Fn(Dispatch)>>,
        config: &Config,
    ) -> Self {
        let valuation_copy = valuation.values().map(Cell::Value).collect();

        ResolutionBuffer {
            valueless_count: 0,
            clause_length: 0,
            asserts: None,

            buffer: valuation_copy,

            dispatcher,

            config: BufferConfig {
                subsumption: config.switch.subsumption,
                stopping: config.stopping_criteria,
            },
        }
    }

    /// Returns the resolved clause and an index to where asserted literal is *within the clause*, if one exists.
    /// ```rust,ignore
    /// let (resolved_clause, assertion_index) = the_buffer.to_assertion_clause();
    /// ```
    pub fn to_assertion_clause(self) -> (vClause, Option<usize>) {
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
    ) -> Result<Ok, err::ResolutionBuffer> {
        // The key has already been used to access the conflicting clause.
        let base_clause = match unsafe { clause_db.get_unchecked(key) } {
            Ok(c) => c,
            Err(_) => return Err(err::ResolutionBuffer::MissingClause),
        };

        self.merge_clause(base_clause);

        // Maybe the conflit clause was already asserting after the previous decision…
        if let Some(literal) = self.asserted_literal() {
            return Ok(Ok::Missed(*key, literal));
        };

        macros::send_resolution_delta!(self, delta::Resolution::Begin);
        macros::send_resolution_delta!(self, delta::Resolution::Used(*key));

        // bump clause activity
        if let ClauseKey::Addition(index, _) = key {
            clause_db.bump_activity(*index)
        };

        // Resolution buffer is only used by analysis, which is only called after some decision has been made
        let the_trail = unsafe { literal_db.last_consequences_unchecked().iter().rev() };
        'resolution_loop: for (source, literal) in the_trail {
            match source {
                literal::Source::BCP(the_key) => {
                    let source_clause = match unsafe { clause_db.get_unchecked(the_key) } {
                        Err(_) => {
                            log::error!(target: targets::RESOLUTION, "Lost resolution clause {the_key}");
                            return Err(err::ResolutionBuffer::LostClause);
                        }
                        Ok(clause) => clause,
                    };

                    let resolution_result = self.resolve_clause(source_clause, literal);

                    if resolution_result.is_err() {
                        // the clause wasn't relevant
                        continue 'resolution_loop;
                    }

                    if self.config.subsumption && self.clause_length < source_clause.size() {
                        match self.clause_length {
                            0 => {}
                            1 => {
                                macros::send_resolution_delta!(self, Resolution::Used(*the_key));
                                macros::send_resolution_delta!(self, delta::Resolution::End);

                                return Ok(Ok::UnitClause);
                            }
                            _ => match the_key {
                                ClauseKey::Unit(_) => {
                                    panic!("!")
                                }
                                ClauseKey::Binary(_) => {
                                    todo!("a formula is found which triggers this…");
                                }
                                ClauseKey::Original(_) | ClauseKey::Addition(_, _) => unsafe {
                                    let k = clause_db.subsume(*the_key, literal, atom_db)?;

                                    macros::send_resolution_delta!(self, delta::Resolution::End);
                                    macros::send_resolution_delta!(self, delta::Resolution::Begin);
                                    macros::send_resolution_delta!(
                                        self,
                                        delta::Resolution::Used(k)
                                    );
                                },
                            },
                        }
                    } else {
                        macros::send_resolution_delta!(self, delta::Resolution::Used(*the_key));
                    }

                    if let ClauseKey::Addition(index, _) = the_key {
                        clause_db.bump_activity(*index)
                    };
                }
                _ => panic!("unexpected"),
            };

            if self.valueless_count == 1 {
                match self.config.stopping {
                    StoppingCriteria::FirstUIP => {
                        macros::send_resolution_delta!(self, delta::Resolution::End);

                        return Ok(Ok::FirstUIP);
                    }
                    StoppingCriteria::None => {}
                };
            }
        }
        macros::send_resolution_delta!(self, delta::Resolution::End);

        Ok(Ok::Exhausted)
    }

    /// Remove literals which conflict with those at level zero from the clause.
    /// ```rust,ignore
    /// resolution_buffer.strengthen_given(self.clause_db.all_unit_clauses());
    /// ```
    pub fn strengthen_given<'l>(&mut self, literals: impl Iterator<Item = &'l abLiteral>) {
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
    fn merge_clause(&mut self, clause: &impl Clause) -> Result<(), err::ResolutionBuffer> {
        for literal in clause.literals() {
            match unsafe { self.buffer.get_unchecked(literal.atom() as usize) } {
                Cell::Conflict(_) | Cell::None(_) | Cell::Pivot => {}
                Cell::Value(maybe) => match maybe {
                    None => {
                        self.clause_length += 1;
                        self.valueless_count += 1;
                        unsafe { self.set(literal.atom(), Cell::None(*literal)) };
                        if self.asserts.is_none() {
                            self.asserts = Some(*literal);
                        }
                    }
                    Some(value) if *value != literal.polarity() => {
                        self.clause_length += 1;
                        unsafe { self.set(literal.atom(), Cell::Conflict(*literal)) };
                    }
                    Some(_) => {
                        log::error!(target: targets::RESOLUTION, "Satisfied clause");
                        return Err(err::ResolutionBuffer::SatisfiedClause);
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
        pivot: impl Borrow<abLiteral>,
    ) -> Result<(), err::ResolutionBuffer> {
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
                Err(err::ResolutionBuffer::LostClause)
            }
        }
    }

    /// Sets a cell corresponding to an atoms to the given enum case.
    unsafe fn set(&mut self, atom: Atom, to: Cell) {
        *self.buffer.get_unchecked_mut(atom as usize) = to
    }

    /// The literal asserted by the resolved clause, if it exists.
    fn asserted_literal(&self) -> Option<abLiteral> {
        if self.valueless_count == 1 {
            self.asserts
        } else {
            None
        }
    }
}