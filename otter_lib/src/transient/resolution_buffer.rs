//! Resolution buffer.
//!
//! A cell for each atom.
//! Valution.
//!

use std::{borrow::Borrow, rc::Rc};

use crate::{
    config::{Config, StoppingCriteria},
    db::{atom::AtomDB, clause::ClauseDB, keys::ClauseKey, literal::LiteralDB},
    dispatch::{
        library::delta::{self},
        Dispatch,
    },
    misc::log::targets::{self},
    structures::{
        atom::Atom,
        clause::{vClause, Clause},
        literal::{vbLiteral, Literal},
        valuation::Valuation,
    },
    types::{
        err::{self},
        gen::{self},
    },
};

/// Cells of a resolution buffer.
#[derive(Clone, Copy)]
pub enum Cell {
    /// Initial valuation
    Value(Option<bool>),
    /// The atom was not valued.
    None(vbLiteral),
    /// The atom had a conflicting value.
    Conflict(vbLiteral),
    /// The atom was part of resolution but was already proven.
    Strengthened,
    /// The atom was used as a pivot when reading a clause into the buffer.
    Pivot,
}

/// A buffer for use when applying resolution to a sequence of clauses.
pub struct ResolutionBuffer {
    valueless_count: usize,
    clause_length: usize,
    asserts: Option<vbLiteral>,
    buffer: Vec<Cell>,
    used: Vec<bool>,
    dispatcher: Option<Rc<dyn Fn(Dispatch)>>,
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

            used: vec![false; atom_db.count()],
            dispatcher,

            config: BufferConfig {
                subsumption: config.switch.subsumption,
                stopping: config.stopping_criteria,
            },
        }
    }

    #[allow(dead_code)]
    // May be helpful to debug issues
    pub fn partial_valuation_in_use(&self) -> Vec<vbLiteral> {
        self.buffer
            .iter()
            .enumerate()
            .filter_map(|(i, v)| match v {
                Cell::Value(Some(value)) => Some(vbLiteral::new(i as Atom, *value)),
                _ => None,
            })
            .collect::<Vec<_>>()
    }

    /// Returns the possible assertion and clause of the buffer as a pair
    pub fn to_assertion_clause(&self) -> (Option<vbLiteral>, vClause) {
        let mut the_clause = vec![];
        let mut conflict_literal = None;
        for item in &self.buffer {
            match item {
                Cell::Strengthened | Cell::Value(_) | Cell::Pivot => {}
                Cell::Conflict(literal) => the_clause.push(*literal),
                Cell::None(literal) => {
                    if self.valueless_count == 1 {
                        conflict_literal = Some(*literal)
                    } else {
                        the_clause.push(*literal)
                    }
                }
            }
        }

        // assert!(conflict_literal.is_some() && the_clause.len() == self.clause_legnth - 1 || the_clause.len() == self.clause_legnth);
        (conflict_literal, the_clause)
    }

    pub fn clear_literal(&mut self, literal: vbLiteral) {
        self.set(literal.var(), Cell::Value(None))
    }

    pub fn resolve_with(
        &mut self,
        conflict: ClauseKey,
        literal_db: &LiteralDB,
        clause_db: &mut ClauseDB,
        atom_db: &mut AtomDB,
    ) -> Result<gen::RBuf, err::ResolutionBuffer> {
        self.merge_clause(clause_db.get_db_clause(conflict).expect("missing clause"));

        // Maybe the conflit clause was already asserting after the previous choice…
        if let Some(asserted_literal) = self.asserts() {
            return Ok(gen::RBuf::Missed(conflict, asserted_literal));
        };
        if let Some(dispatcher) = &self.dispatcher {
            let delta = delta::Delta::Resolution(delta::Resolution::Begin);
            dispatcher(Dispatch::Delta(delta));
            let delta = delta::Resolution::Used(conflict);
            dispatcher(Dispatch::Delta(delta::Delta::Resolution(delta)));
        }

        // bump clause activity
        if let ClauseKey::Addition(index, _) = conflict {
            clause_db.bump_activity(index)
        };

        'resolution_loop: for (source, literal) in literal_db.last_consequences().iter().rev() {
            match source {
                gen::src::Literal::BCP(the_key) => {
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
                                    let delta = delta::Resolution::Used(*the_key);
                                    dispatcher(Dispatch::Delta(delta::Delta::Resolution(delta)));
                                    dispatcher(Dispatch::Delta(delta::Delta::Resolution(
                                        delta::Resolution::End,
                                    )));
                                }
                                return Ok(gen::RBuf::Proof);
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
                                        dispatcher(Dispatch::Delta(delta::Delta::Resolution(
                                            delta::Resolution::End,
                                        )));
                                        dispatcher(Dispatch::Delta(delta::Delta::Resolution(
                                            delta::Resolution::Begin,
                                        )));
                                        dispatcher(Dispatch::Delta(delta::Delta::Resolution(
                                            delta::Resolution::Used(new_key),
                                        )));
                                    }
                                }
                            },
                        }
                    } else {
                        if let Some(dispatcher) = &self.dispatcher {
                            let delta = delta::Resolution::Used(*the_key);
                            dispatcher(Dispatch::Delta(delta::Delta::Resolution(delta)));
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
                            dispatcher(Dispatch::Delta(delta::Delta::Resolution(
                                delta::Resolution::End,
                            )));
                        }
                        return Ok(gen::RBuf::FirstUIP);
                    }
                    StoppingCriteria::None => {}
                };
            }
        }
        if let Some(dispatcher) = &self.dispatcher {
            let delta = delta::Resolution::End;
            dispatcher(Dispatch::Delta(delta::Delta::Resolution(delta)));
        }
        Ok(gen::RBuf::Exhausted)
    }

    /// Remove literals which conflict with those at level zero from the clause
    pub fn strengthen_given<'l>(&mut self, literals: impl Iterator<Item = &'l vbLiteral>) {
        for literal in literals {
            match unsafe { *self.buffer.get_unchecked(literal.var() as usize) } {
                Cell::None(_) | Cell::Conflict(_) => {
                    if let Some(length_minus_one) = self.clause_length.checked_sub(1) {
                        self.clause_length = length_minus_one;
                    }
                    self.set(literal.var(), Cell::Strengthened)
                }
                _ => {}
            }
        }
    }

    pub fn atoms_used(&self) -> impl Iterator<Item = Atom> + '_ {
        self.used
            .iter()
            .enumerate()
            .filter_map(|(index, used)| match used {
                true => Some(index as Atom),
                false => None,
            })
    }
}

impl ResolutionBuffer {
    /// Merge a clause into the buffer
    fn merge_clause(&mut self, clause: &impl Clause) -> Result<(), err::ResolutionBuffer> {
        for literal in clause.literals() {
            match unsafe { self.buffer.get_unchecked(literal.var() as usize) } {
                Cell::Conflict(_) | Cell::None(_) | Cell::Pivot => {}
                Cell::Value(maybe) => match maybe {
                    None => {
                        unsafe { *self.used.get_unchecked_mut(literal.var() as usize) = true };
                        self.clause_length += 1;
                        self.valueless_count += 1;
                        self.set(literal.var(), Cell::None(*literal));
                        if self.asserts.is_none() {
                            self.asserts = Some(*literal);
                        }
                    }
                    Some(value) if *value != literal.polarity() => {
                        unsafe { *self.used.get_unchecked_mut(literal.var() as usize) = true };
                        self.clause_length += 1;
                        self.set(literal.var(), Cell::Conflict(*literal));
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
        using: impl Borrow<vbLiteral>,
    ) -> Result<(), err::ResolutionBuffer> {
        let using = using.borrow();
        let contents = unsafe { *self.buffer.get_unchecked(using.var() as usize) };
        match contents {
            Cell::None(literal) if using == &literal.negate() => {
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                self.set(using.var(), Cell::Pivot);
                self.valueless_count -= 1;

                Ok(())
            }
            Cell::Conflict(literal) if using == &literal.negate() => {
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                self.set(using.var(), Cell::Pivot);

                Ok(())
            }
            _ => {
                // Skip over any clauses which are not involved in the current resolution trail
                Err(err::ResolutionBuffer::LostClause)
            }
        }
    }

    fn set(&mut self, index: Atom, to: Cell) {
        *unsafe { self.buffer.get_unchecked_mut(index as usize) } = to
    }

    fn asserts(&self) -> Option<vbLiteral> {
        if self.valueless_count == 1 {
            self.asserts
        } else {
            None
        }
    }
}
