use std::borrow::Borrow;

use crossbeam::channel::Sender;

use crate::{
    config::{context::Config, StoppingCriteria},
    db::{clause::ClauseDB, keys::ClauseKey, literal::LiteralDB, variable::VariableDB},
    dispatch::{
        library::delta::{self},
        Dispatch,
    },
    misc::log::targets::{self},
    structures::{
        clause::Clause,
        literal::{Literal, LiteralT},
        variable::Variable,
    },
    types::{
        err::{self},
        gen::{self},
    },
};

#[derive(Clone, Copy)]
enum Cell {
    Value(Option<bool>),
    NoneLiteral(Literal),
    ConflictLiteral(Literal),
    Strengthened,
    Pivot,
}

pub struct ResolutionBuffer {
    valueless_count: usize,
    clause_length: usize,
    asserts: Option<Literal>,
    buffer: Vec<Cell>,
    used: Vec<bool>,
    tx: Option<Sender<Dispatch>>,
    config: BufferConfig,
}

struct BufferConfig {
    subsumption: bool,
    stopping: StoppingCriteria,
}

impl ResolutionBuffer {
    pub fn clause_legnth(&self) -> usize {
        self.clause_length
    }

    pub fn from_variable_store(
        variable_db: &VariableDB,
        tx: Option<Sender<Dispatch>>,
        config: &Config,
    ) -> Self {
        let valuation_copy = variable_db
            .valuation()
            .iter()
            .map(|v| Cell::Value(*v))
            .collect();

        ResolutionBuffer {
            valueless_count: 0,
            clause_length: 0,
            asserts: None,

            buffer: valuation_copy,

            used: vec![false; variable_db.count()],
            tx,

            config: BufferConfig {
                subsumption: config.enabled.subsumption,
                stopping: config.stopping_criteria,
            },
        }
    }

    #[allow(dead_code)]
    // May be helpful to debug issues
    pub fn partial_valuation_in_use(&self) -> Vec<Literal> {
        self.buffer
            .iter()
            .enumerate()
            .filter_map(|(i, v)| match v {
                Cell::Value(Some(value)) => Some(Literal::new(i as Variable, *value)),
                _ => None,
            })
            .collect::<Vec<_>>()
    }

    /// Returns the possible assertion and clause of the buffer as a pair
    pub fn to_assertion_clause(&self) -> (Option<Literal>, Vec<Literal>) {
        let mut the_clause = vec![];
        let mut conflict_literal = None;
        for item in &self.buffer {
            match item {
                Cell::Strengthened | Cell::Value(_) | Cell::Pivot => {}
                Cell::ConflictLiteral(literal) => the_clause.push(*literal),
                Cell::NoneLiteral(literal) => {
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

    pub fn clear_literal(&mut self, literal: Literal) {
        self.set(literal.var(), Cell::Value(None))
    }

    pub fn resolve_with(
        &mut self,
        conflict: ClauseKey,
        levels: &LiteralDB,
        clause_db: &mut ClauseDB,
        variables: &mut VariableDB,
    ) -> Result<gen::RBuf, err::RBuf> {
        self.merge_clause(clause_db.get(conflict).expect("missing clause"));

        // Maybe the conflit clause was already asserting after the previous choice…
        if let Some(asserted_literal) = self.asserts() {
            return Ok(gen::RBuf::Missed(conflict, asserted_literal));
        };
        if let Some(tx) = &self.tx {
            let delta = delta::Delta::Resolution(delta::Resolution::Begin);
            tx.send(Dispatch::Delta(delta));
            let delta = delta::Resolution::Used(conflict);
            tx.send(Dispatch::Delta(delta::Delta::Resolution(delta)));
        }

        // bump clause activity
        if let ClauseKey::Learned(index, _) = conflict {
            clause_db.bump_activity(index)
        };

        'resolution_loop: for (source, literal) in levels.last_consequences().iter().rev() {
            if let gen::src::Literal::Forced(the_key)
            | gen::src::Literal::BCP(the_key)
            | gen::src::Literal::Resolution(the_key)
            | gen::src::Literal::Missed(the_key) = source
            {
                let source_clause = match clause_db.get(*the_key) {
                    Err(_) => {
                        log::error!(target: targets::RESOLUTION, "Lost resolution clause {the_key:?}");
                        return Err(err::RBuf::LostClause);
                    }
                    Ok(clause) => clause,
                };

                let resolution_result = self.resolve_clause(source_clause, literal);
                if resolution_result.is_err() {
                    // the clause wasn't relevant
                    continue 'resolution_loop;
                }

                if self.config.subsumption && self.clause_length < source_clause.literals().len() {
                    match self.clause_length {
                        0 => {}
                        1 => {
                            if let Some(tx) = &self.tx {
                                let delta = delta::Resolution::Used(*the_key);
                                tx.send(Dispatch::Delta(delta::Delta::Resolution(delta)));
                                tx.send(Dispatch::Delta(delta::Delta::Resolution(
                                    delta::Resolution::End,
                                )));
                            }
                            return Ok(gen::RBuf::Proof);
                        }
                        _ => match the_key {
                            ClauseKey::Binary(_) => {
                                todo!("a formula is found which triggers this…");
                            }
                            ClauseKey::Formula(_) | ClauseKey::Learned(_, _) => {
                                let new_key = clause_db.subsume(*the_key, *literal, variables)?;

                                if let Some(tx) = &self.tx {
                                    tx.send(Dispatch::Delta(delta::Delta::Resolution(
                                        delta::Resolution::End,
                                    )));
                                    tx.send(Dispatch::Delta(delta::Delta::Resolution(
                                        delta::Resolution::Begin,
                                    )));
                                    tx.send(Dispatch::Delta(delta::Delta::Resolution(
                                        delta::Resolution::Used(new_key),
                                    )));
                                }
                            }
                        },
                    }
                } else {
                    if let Some(tx) = &self.tx {
                        let delta = delta::Resolution::Used(*the_key);
                        tx.send(Dispatch::Delta(delta::Delta::Resolution(delta)));
                    }
                }

                if let ClauseKey::Learned(index, _) = the_key {
                    clause_db.bump_activity(*index)
                };

                if self.valueless_count == 1 {
                    match self.config.stopping {
                        StoppingCriteria::FirstUIP => {
                            if let Some(tx) = &self.tx {
                                tx.send(Dispatch::Delta(delta::Delta::Resolution(
                                    delta::Resolution::End,
                                )));
                            }
                            return Ok(gen::RBuf::FirstUIP);
                        }
                        StoppingCriteria::None => {}
                    };
                }
            }
        }
        if let Some(tx) = &self.tx {
            let delta = delta::Resolution::End;
            tx.send(Dispatch::Delta(delta::Delta::Resolution(delta)));
        }
        Ok(gen::RBuf::Exhausted)
    }

    /// Remove literals which conflict with those at level zero from the clause
    pub fn strengthen_given<'l>(&mut self, literals: impl Iterator<Item = &'l Literal>) {
        for literal in literals {
            match unsafe { *self.buffer.get_unchecked(literal.var() as usize) } {
                Cell::NoneLiteral(_) | Cell::ConflictLiteral(_) => {
                    if let Some(length_minus_one) = self.clause_length.checked_sub(1) {
                        self.clause_length = length_minus_one;
                    }
                    self.set(literal.var(), Cell::Strengthened)
                }
                _ => {}
            }
        }
    }

    pub fn variables_used(&self) -> impl Iterator<Item = Variable> + '_ {
        self.used
            .iter()
            .enumerate()
            .filter_map(|(index, used)| match used {
                true => Some(index as Variable),
                false => None,
            })
    }
}

impl ResolutionBuffer {
    /// Merge a clause into the buffer
    fn merge_clause(&mut self, clause: &impl Clause) -> Result<(), err::RBuf> {
        for literal in clause.literals() {
            match unsafe { self.buffer.get_unchecked(literal.var() as usize) } {
                Cell::ConflictLiteral(_) | Cell::NoneLiteral(_) | Cell::Pivot => {}
                Cell::Value(maybe) => match maybe {
                    None => {
                        unsafe { *self.used.get_unchecked_mut(literal.var() as usize) = true };
                        self.clause_length += 1;
                        self.valueless_count += 1;
                        self.set(literal.var(), Cell::NoneLiteral(*literal));
                        if self.asserts.is_none() {
                            self.asserts = Some(*literal);
                        }
                    }
                    Some(value) if *value != literal.polarity() => {
                        unsafe { *self.used.get_unchecked_mut(literal.var() as usize) = true };
                        self.clause_length += 1;
                        self.set(literal.var(), Cell::ConflictLiteral(*literal));
                    }
                    Some(_) => {
                        log::error!(target: targets::RESOLUTION, "Satisfied clause");
                        return Err(err::RBuf::SatisfiedResolution);
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
        using: impl Borrow<Literal>,
    ) -> Result<(), err::RBuf> {
        let using = using.borrow();
        let contents = unsafe { *self.buffer.get_unchecked(using.var() as usize) };
        match contents {
            Cell::NoneLiteral(literal) if using == &literal.negate() => {
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                self.set(using.var(), Cell::Pivot);
                self.valueless_count -= 1;

                Ok(())
            }
            Cell::ConflictLiteral(literal) if using == &literal.negate() => {
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                self.set(using.var(), Cell::Pivot);

                Ok(())
            }
            _ => {
                // Skip over any clauses which are not involved in the current resolution trail
                Err(err::RBuf::LostClause)
            }
        }
    }

    fn set(&mut self, index: Variable, to: Cell) {
        *unsafe { self.buffer.get_unchecked_mut(index as usize) } = to
    }

    fn asserts(&self) -> Option<Literal> {
        if self.valueless_count == 1 {
            self.asserts
        } else {
            None
        }
    }
}
