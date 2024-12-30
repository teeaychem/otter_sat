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
    db::{atom::AtomDB, clause::ClauseDB, keys::ClauseKey, literal::LiteralDB},
    dispatch::{
        library::delta::{
            self,
            Resolution::{self},
        },
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
#[derive(Debug)]
pub enum Ok {
    FirstUIP,
    Exhausted,
    Proof,
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
    valueless_count: usize,
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

macro_rules! send {
    ( $dispatcher:ident, $dispatch:expr ) => {{
        $dispatcher(Dispatch::Delta(delta::Delta::Resolution($dispatch)));
    }};
}

impl ResolutionBuffer {
    pub fn clause_legnth(&self) -> usize {
        self.clause_length
    }

    pub fn from_atom_db(
        atom_db: &AtomDB,
        dispatcher: Option<Rc<dyn Fn(Dispatch)>>,
        config: &Config,
    ) -> Self {
        let valuation_copy = atom_db.valuation().values().map(Cell::Value).collect();

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

    /// Returns the resolved clause and the index to the asserted literal, if one exists.
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

    pub fn clear_literal(&mut self, literal: impl Borrow<abLiteral>) {
        unsafe { self.set(literal.borrow().atom(), Cell::Value(None)) }
    }

    pub fn resolve_with(
        &mut self,
        conflict: ClauseKey,
        literal_db: &LiteralDB,
        clause_db: &mut ClauseDB,
        atom_db: &mut AtomDB,
    ) -> Result<Ok, err::ResolutionBuffer> {
        self.merge_clause(clause_db.get_db_clause(conflict).expect("missing clause"));

        // Maybe the conflit clause was already asserting after the previous choice…
        if let Some(asserted_literal) = self.asserted_literal() {
            return Ok(Ok::Missed(conflict, asserted_literal));
        };
        if let Some(dispatcher) = &self.dispatcher {
            send!(dispatcher, delta::Resolution::Begin);
            send!(dispatcher, delta::Resolution::Used(conflict));
        }

        // bump clause activity
        if let ClauseKey::Addition(index, _) = conflict {
            clause_db.bump_activity(index)
        };

        'resolution_loop: for (source, literal) in literal_db.last_consequences().iter().rev() {
            match source {
                literal::Source::BCP(the_key) => {
                    let source_clause = match clause_db.get_db_clause(*the_key) {
                        Err(_) => {
                            log::error!(target: targets::RESOLUTION, "Lost resolution clause {the_key:?}");
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
                                if let Some(dispatcher) = &self.dispatcher {
                                    send!(dispatcher, Resolution::Used(*the_key));
                                    send!(dispatcher, delta::Resolution::End);
                                }
                                return Ok(Ok::Proof);
                            }
                            _ => match the_key {
                                ClauseKey::Unit(_) => {
                                    panic!("a prior check on the clause length was removed")
                                }
                                ClauseKey::Binary(_) => {
                                    todo!("a formula is found which triggers this…");
                                }
                                ClauseKey::Original(_) | ClauseKey::Addition(_, _) => {
                                    let new_key = clause_db.subsume(*the_key, *literal, atom_db)?;

                                    if let Some(dispatcher) = &self.dispatcher {
                                        send!(dispatcher, delta::Resolution::End);
                                        send!(dispatcher, delta::Resolution::Begin);
                                        send!(dispatcher, delta::Resolution::Used(new_key));
                                    }
                                }
                            },
                        }
                    } else {
                        if let Some(dispatcher) = &self.dispatcher {
                            send!(dispatcher, delta::Resolution::Used(*the_key));
                        }
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
                        if let Some(dispatcher) = &self.dispatcher {
                            send!(dispatcher, delta::Resolution::End);
                        }
                        return Ok(Ok::FirstUIP);
                    }
                    StoppingCriteria::None => {}
                };
            }
        }
        if let Some(dispatcher) = &self.dispatcher {
            send!(dispatcher, delta::Resolution::End);
        }
        Ok(Ok::Exhausted)
    }

    /// Remove literals which conflict with those at level zero from the clause
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

impl ResolutionBuffer {
    /// Merge a clause into the buffer
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

    fn resolve_clause(
        &mut self,
        clause: &impl Clause,
        using: impl Borrow<abLiteral>,
    ) -> Result<(), err::ResolutionBuffer> {
        let using = using.borrow();
        let contents = unsafe { *self.buffer.get_unchecked(using.atom() as usize) };
        match contents {
            Cell::None(literal) if using == &literal.negate() => {
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                unsafe { self.set(using.atom(), Cell::Pivot) };
                self.valueless_count -= 1;

                Ok(())
            }
            Cell::Conflict(literal) if using == &literal.negate() => {
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                unsafe { self.set(using.atom(), Cell::Pivot) };

                Ok(())
            }
            _ => {
                // Skip over any clauses which are not involved in the current resolution trail
                Err(err::ResolutionBuffer::LostClause)
            }
        }
    }

    unsafe fn set(&mut self, index: Atom, to: Cell) {
        *self.buffer.get_unchecked_mut(index as usize) = to
    }

    fn asserted_literal(&self) -> Option<abLiteral> {
        if self.valueless_count == 1 {
            self.asserts
        } else {
            None
        }
    }
}
