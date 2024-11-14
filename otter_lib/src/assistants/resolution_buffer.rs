use std::borrow::Borrow;

use crossbeam::channel::Sender;

use crate::{
    config::{Config, StoppingCriteria},
    db::{clause::ClauseDB, keys::ClauseKey, literal::LiteralDB, variable::VariableDB},
    dispatch::{
        delta::{self},
        Dispatch,
    },
    structures::{
        clause::Clause,
        literal::{Literal, LiteralT},
        variable::Variable,
    },
    types::gen,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ResolutionCell {
    Value(Option<bool>),
    NoneLiteral(Literal),
    ConflictLiteral(Literal),
    Strengthened,
    Pivot,
}

#[derive(Debug)]
pub struct ResolutionBuffer {
    valueless_count: usize,
    clause_length: usize,
    asserts: Option<Literal>,
    buffer: Vec<ResolutionCell>,
    used: Vec<bool>,
    tx: Sender<Dispatch>,
    config: BufferConfig,
}

#[derive(Debug)]
struct BufferConfig {
    subsumption: bool,
    stopping: StoppingCriteria,
}

#[derive(Debug)]
pub enum BufOk {
    FirstUIP,
    Exhausted,
    Proof,
    Missed(ClauseKey, Literal),
}

#[derive(Debug)]
pub enum BufErr {
    MissingClause,
    Subsumption,
    SatisfiedResolution,
    Transfer,
}

impl ResolutionBuffer {
    pub fn clause_legnth(&self) -> usize {
        self.clause_length
    }

    pub fn from_variable_store(
        variable_db: &VariableDB,
        tx: Sender<Dispatch>,
        config: &Config,
    ) -> Self {
        ResolutionBuffer {
            valueless_count: 0,
            clause_length: 0,
            asserts: None,
            buffer: variable_db
                .valuation()
                .iter()
                .map(|v| ResolutionCell::Value(*v))
                .collect(),
            used: vec![false; variable_db.count()],
            tx,
            config: BufferConfig {
                subsumption: config.subsumption,
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
                ResolutionCell::Value(Some(value)) => Some(Literal::new(i as Variable, *value)),
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
                ResolutionCell::Strengthened | ResolutionCell::Value(_) | ResolutionCell::Pivot => {
                }
                ResolutionCell::ConflictLiteral(literal) => the_clause.push(*literal),
                ResolutionCell::NoneLiteral(literal) => {
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
        self.set(literal.var(), ResolutionCell::Value(None))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn resolve_with(
        &mut self,
        conflict: ClauseKey,
        levels: &LiteralDB,
        clause_db: &mut ClauseDB,
        variables: &mut VariableDB,
    ) -> Result<BufOk, BufErr> {
        self.merge_clause(clause_db.get(conflict).expect("missing clause"));

        // Maybe the conflit clause was already asserting after the previous choice…
        if let Some(asserted_literal) = self.asserts() {
            return Ok(BufOk::Missed(conflict, asserted_literal));
        };
        self.tx.send(Dispatch::Resolution(delta::Resolution::Start));

        let delta = delta::Resolution::Used(conflict);
        self.tx.send(Dispatch::Resolution(delta));

        // bump clause activity
        if let ClauseKey::Learned(index, _) = conflict {
            clause_db.bump_activity(index)
        };

        'resolution_loop: for (source, literal) in levels.last_consequences().iter().rev() {
            if let gen::LiteralSource::Analysis(the_key)
            | gen::LiteralSource::BCP(the_key)
            | gen::LiteralSource::Resolution(the_key)
            | gen::LiteralSource::Missed(the_key) = source
            {
                let source_clause = match clause_db.get(*the_key) {
                    Err(_) => {
                        log::error!(target: crate::log::targets::RESOLUTION, "Failed to find resolution clause {the_key:?}");
                        return Err(BufErr::MissingClause);
                    }
                    Ok(clause) => clause,
                };

                let resolution_result = self.resolve_clause(source_clause, literal);
                if resolution_result.is_err() {
                    // the clause wasn't relevant
                    continue 'resolution_loop;
                }

                if self.config.subsumption && self.clause_length < source_clause.literals().len() {
                    /*
                    TODO: Move
                    If the resolved clause is binary then subsumption transfers the clause to the store for binary clauses
                    This is safe to do as:
                    - After backjumping all the observations at the current level will be forgotten
                    - The clause does not appear in the observations of any previous stage
                      + As, if the clause appeared in some previous stage then use of the clause would be a missed implication
                      + And, missed implications are checked prior to conflicts
                     */
                    match self.clause_length {
                        0 => {}
                        1 => {
                            let delta = delta::Resolution::Used(*the_key);
                            self.tx.send(Dispatch::Resolution(delta));
                            self.tx
                                .send(Dispatch::Resolution(delta::Resolution::Finish));
                            return Ok(BufOk::Proof);
                        }
                        _ => match the_key {
                            ClauseKey::Binary(_) => {
                                todo!("a formula is found which triggers this…");
                            }
                            ClauseKey::Formula(_) | ClauseKey::Learned(_, _) => {
                                self.tx
                                    .send(Dispatch::Resolution(delta::Resolution::Finish));

                                let new_key = clause_db.subsume(*the_key, *literal, variables)?;

                                self.tx.send(Dispatch::Resolution(delta::Resolution::Start));
                                self.tx
                                    .send(Dispatch::Resolution(delta::Resolution::Used(new_key)));
                            }
                        },
                    }
                } else {
                    let delta = delta::Resolution::Used(*the_key);
                    self.tx.send(Dispatch::Resolution(delta));
                }

                // bump clause activity
                if let ClauseKey::Learned(index, _) = the_key {
                    clause_db.bump_activity(*index)
                };

                if self.valueless_count == 1 {
                    match self.config.stopping {
                        StoppingCriteria::FirstUIP => {
                            self.tx
                                .send(Dispatch::Resolution(delta::Resolution::Finish));
                            return Ok(BufOk::FirstUIP);
                        }
                        StoppingCriteria::None => {}
                    };
                }
            }
        }
        let delta = delta::Resolution::Finish;
        self.tx.send(Dispatch::Resolution(delta));
        Ok(BufOk::Exhausted)
    }

    /// Remove literals which conflict with those at level zero from the clause
    pub fn strengthen_given<'l>(&mut self, literals: impl Iterator<Item = &'l Literal>) {
        for literal in literals {
            match unsafe { *self.buffer.get_unchecked(literal.var() as usize) } {
                ResolutionCell::NoneLiteral(_) | ResolutionCell::ConflictLiteral(_) => {
                    if let Some(length_minus_one) = self.clause_length.checked_sub(1) {
                        self.clause_length = length_minus_one;
                    }
                    self.set(literal.var(), ResolutionCell::Strengthened)
                }
                _ => {}
            }
        }
    }

    pub fn asserts(&self) -> Option<Literal> {
        if self.valueless_count == 1 {
            self.asserts
        } else {
            None
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
    fn merge_clause(&mut self, clause: &impl Clause) -> Result<(), BufErr> {
        for literal in clause.literals() {
            match unsafe { self.buffer.get_unchecked(literal.var() as usize) } {
                ResolutionCell::ConflictLiteral(_)
                | ResolutionCell::NoneLiteral(_)
                | ResolutionCell::Pivot => {}
                ResolutionCell::Value(maybe) => match maybe {
                    None => {
                        unsafe { *self.used.get_unchecked_mut(literal.var() as usize) = true };
                        self.clause_length += 1;
                        self.valueless_count += 1;
                        self.set(literal.var(), ResolutionCell::NoneLiteral(*literal));
                        if self.asserts.is_none() {
                            self.asserts = Some(*literal);
                        }
                    }
                    Some(value) if *value != literal.polarity() => {
                        unsafe { *self.used.get_unchecked_mut(literal.var() as usize) = true };
                        self.clause_length += 1;
                        self.set(literal.var(), ResolutionCell::ConflictLiteral(*literal));
                    }
                    Some(_) => {
                        log::error!(target: crate::log::targets::RESOLUTION, "resolution to a satisfied clause");
                        return Err(BufErr::SatisfiedResolution);
                    }
                },
                ResolutionCell::Strengthened => {}
            }
        }
        Ok(())
    }

    fn resolve_clause<L: Borrow<Literal>>(
        &mut self,
        clause: &impl Clause,
        using: L,
    ) -> Result<(), BufErr> {
        let using = using.borrow();
        let contents = unsafe { *self.buffer.get_unchecked(using.var() as usize) };
        match contents {
            ResolutionCell::NoneLiteral(literal) if using == &literal.negate() => {
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                self.set(using.var(), ResolutionCell::Pivot);
                self.valueless_count -= 1;

                Ok(())
            }
            ResolutionCell::ConflictLiteral(literal) if using == &literal.negate() => {
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                self.set(using.var(), ResolutionCell::Pivot);

                Ok(())
            }
            _ => {
                // Skip over any clauses which are not involved in the current resolution trail
                Err(BufErr::MissingClause)
            }
        }
    }

    fn set(&mut self, index: Variable, to: ResolutionCell) {
        *unsafe { self.buffer.get_unchecked_mut(index as usize) } = to
    }
}
