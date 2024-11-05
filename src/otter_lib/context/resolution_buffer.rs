use crate::{
    config::{Config, StoppingCriteria},
    context::stores::{clause::ClauseStore, level::Level, variable::VariableStore, ClauseKey},
    structures::{
        clause::{stored::SubsumptionIssue, Clause},
        literal::{Literal, LiteralSource},
        variable::{list::VariableList, VariableId},
    },
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
    pub valueless_count: usize,
    pub clause_length: usize,
    asserts: Option<Literal>,
    buffer: Vec<ResolutionCell>,
    trail: Vec<ClauseKey>,
    used_variables: Vec<bool>,
}

#[derive(Debug)]
pub enum BufferStatus {
    FirstUIP,
    Exhausted,
}

#[derive(Debug)]
pub enum BufferIssue {
    MissingClause,
    Subsumption(SubsumptionIssue),
    SatisfiedResolution,
    Transfer,
}

impl ResolutionBuffer {
    #[allow(dead_code)]
    pub fn reset_with(&mut self, variables: &impl VariableList) {
        self.valueless_count = 0;
        self.asserts = None;
        for variable in variables.slice() {
            self.set(variable.index(), ResolutionCell::Value(variable.value()))
        }
        self.trail.clear();
        self.used_variables
            .iter_mut()
            .for_each(|index| *index = false);
    }

    pub fn from_variable_store(variables: &impl VariableList) -> Self {
        ResolutionBuffer {
            valueless_count: 0,
            clause_length: 0,
            asserts: None,
            buffer: variables
                .slice()
                .iter()
                .map(|variable| ResolutionCell::Value(variable.value()))
                .collect(),
            trail: vec![],
            used_variables: vec![false; variables.slice().len()],
        }
    }

    pub fn set_inital_clause(
        &mut self,
        clause: &impl Clause,
        key: ClauseKey,
    ) -> Result<(), BufferIssue> {
        self.trail.push(key);
        self.merge_clause(clause)
    }

    #[allow(dead_code)]
    // May be helpful to debug issues
    pub fn partial_valuation_in_use(&self) -> Vec<Literal> {
        self.buffer
            .iter()
            .enumerate()
            .filter_map(|(i, v)| match v {
                ResolutionCell::Value(Some(value)) => Some(Literal::new(i as VariableId, *value)),
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

    pub fn clear_literals(&mut self, literals: impl Iterator<Item = Literal>) {
        for literal in literals {
            self.set(literal.index(), ResolutionCell::Value(None))
        }
    }

    pub fn resolve_with(
        &mut self,
        level: &Level,
        stored_clauses: &mut ClauseStore,
        variables: &mut VariableStore,
        config: &Config,
    ) -> Result<BufferStatus, BufferIssue> {
        for (source, literal) in level.observations().iter().rev() {
            if let LiteralSource::Analysis(the_key)
            | LiteralSource::BCP(the_key)
            | LiteralSource::Resolution(the_key)
            | LiteralSource::Missed(the_key) = source
            {
                let source_clause = match stored_clauses.get_carefully_mut(*the_key) {
                    None => {
                        log::error!(target: crate::log::targets::RESOLUTION, "Failed to find resolution clause {the_key:?}");
                        return Err(BufferIssue::MissingClause);
                    }
                    Some(clause) => clause,
                };

                if self.resolve_clause(source_clause, *literal).is_ok() {
                    for involved_literal in source_clause.literal_slice() {
                        self.used_variables[involved_literal.index()] = true;
                    }

                    // TODO: allow subsumption on binary clauses?
                    if config.subsumption
                        && self.clause_length < source_clause.length()
                        && source_clause.len() > 2
                    {
                        /*
                        If the resolved clause is binary then subsumption transfers the clause to the store for binary clauses
                        This is safe to do as:
                        - After backjumping all the observations at the current level will be forgotten
                        - The clause does not appear in the observations of any previous stage
                          + As, if the clause appeared in some previous stage then use of the clause would be a missed implication
                          + And, missed implications are checked prior to conflicts
                         */

                        match self.clause_length {
                            2 => match the_key {
                                ClauseKey::Binary(_) => {}
                                ClauseKey::Formula(_) | ClauseKey::Learned(_, _) => {
                                    match source_clause.subsume(*literal, variables, false) {
                                        Ok(_) => {
                                            let Ok(new_key) = stored_clauses
                                                .transfer_to_binary(*the_key, variables, *literal)
                                            else {
                                                return Err(BufferIssue::Transfer);
                                            };
                                            self.trail.push(new_key);
                                        }
                                        Err(e) => return Err(BufferIssue::Subsumption(e)),
                                    };
                                }
                            },
                            _ => {
                                match source_clause.subsume(*literal, variables, true) {
                                    Ok(_) => {
                                        self.trail.push(*the_key);
                                    }
                                    Err(e) => return Err(BufferIssue::Subsumption(e)),
                                };
                            }
                        }
                    }

                    if self.valueless_count == 1 {
                        match config.stopping_criteria {
                            StoppingCriteria::FirstUIP => return Ok(BufferStatus::FirstUIP),
                            StoppingCriteria::None => {}
                        }
                    };
                }
            }
        }
        Ok(BufferStatus::Exhausted)
    }

    /// Remove literals which conflict with those at level zero from the clause
    pub fn strengthen_given<'l>(&mut self, literals: impl Iterator<Item = &'l Literal>) {
        for literal in literals {
            match self.buffer[literal.index()] {
                ResolutionCell::NoneLiteral(_) | ResolutionCell::ConflictLiteral(_) => {
                    if let Some(length_minus_one) = self.clause_length.checked_sub(1) {
                        self.clause_length = length_minus_one;
                    }
                    self.set(literal.index(), ResolutionCell::Strengthened)
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

    pub fn variables_used(&self) -> impl Iterator<Item = usize> + '_ {
        self.used_variables
            .iter()
            .enumerate()
            .filter_map(|(index, used)| match used {
                true => Some(index),
                false => None,
            })
    }

    pub fn trail(&self) -> &[ClauseKey] {
        &self.trail
    }
}

impl ResolutionBuffer {
    /// Merge a clause into the buffer
    fn merge_clause(&mut self, clause: &impl Clause) -> Result<(), BufferIssue> {
        for literal in clause.literal_slice() {
            match self.buffer.get(literal.index()).expect("lost literal") {
                ResolutionCell::ConflictLiteral(_) | ResolutionCell::NoneLiteral(_) => {}
                ResolutionCell::Pivot => {}
                ResolutionCell::Value(maybe) => match maybe {
                    None => {
                        self.clause_length += 1;
                        self.valueless_count += 1;
                        self.set(literal.index(), ResolutionCell::NoneLiteral(*literal));
                        if self.asserts.is_none() {
                            self.asserts = Some(*literal);
                        }
                    }
                    Some(value) if *value != literal.polarity() => {
                        self.clause_length += 1;
                        self.set(literal.index(), ResolutionCell::ConflictLiteral(*literal))
                    }
                    Some(_) => {
                        log::error!(target: crate::log::targets::RESOLUTION, "Resolution to a satisfied clause");

                        return Err(BufferIssue::SatisfiedResolution);
                    }
                },
                ResolutionCell::Strengthened => {}
            }
        }
        Ok(())
    }

    fn resolve_clause(&mut self, clause: &impl Clause, using: Literal) -> Result<(), BufferIssue> {
        match unsafe { *self.buffer.get_unchecked(using.index()) } {
            ResolutionCell::NoneLiteral(literal) if using == !literal => {
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                self.set(using.index(), ResolutionCell::Pivot);
                self.valueless_count -= 1;

                Ok(())
            }
            ResolutionCell::ConflictLiteral(literal) if using == !literal => {
                self.merge_clause(clause)?;
                self.clause_length -= 1;
                self.set(using.index(), ResolutionCell::Pivot);

                Ok(())
            }
            _ => Err(BufferIssue::MissingClause),
        }
    }

    fn set(&mut self, index: usize, to: ResolutionCell) {
        *unsafe { self.buffer.get_unchecked_mut(index) } = to
    }
}
