use crate::{
    config::{Config, StoppingCriteria},
    context::{
        level::Level,
        store::{ClauseKey, ClauseStore},
    },
    structures::{
        clause::Clause,
        literal::{Literal, LiteralSource},
        variable::{delegate::VariableStore, list::VariableList},
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
pub enum Status {
    FirstUIP,
    Exhausted,
    BinarySubsumption,
}

impl ResolutionBuffer {
    pub fn new(size: usize) -> Self {
        ResolutionBuffer {
            valueless_count: 0,
            clause_length: 0,
            asserts: None,
            buffer: vec![ResolutionCell::Value(None); size],
            trail: vec![],
            used_variables: vec![false; size],
        }
    }

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

    pub fn set_inital_clause(&mut self, clause: &impl Clause, key: ClauseKey) {
        self.trail.push(key);
        self.merge_clause(clause);
    }

    pub fn valuation_in_use(&self) {
        println!(
            "buffer val
{:?}",
            self.buffer
                .iter()
                .enumerate()
                .filter_map(|(i, v)| match v {
                    ResolutionCell::Value(Some(true)) => Some(i as isize),
                    ResolutionCell::Value(Some(false)) => Some(-(i as isize)),
                    _ => None,
                })
                .collect::<Vec<_>>()
        );
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

    #[allow(clippy::too_many_arguments)]
    pub fn resolve_with(
        &mut self,
        level: &Level,
        stored_clauses: &mut ClauseStore,
        variables: &VariableStore,
        config: &Config,
    ) -> Status {
        for (source, literal) in level.observations().iter().rev() {
            if let LiteralSource::Clause(the_key)
            | LiteralSource::BinaryPropagation(the_key)
            | LiteralSource::Propagation(the_key)
            | LiteralSource::Resolution(the_key)
            | LiteralSource::Missed(the_key, _) = source
            {
                let source_clause = match stored_clauses.get_carefully_mut(*the_key) {
                    None => {
                        println!("missing key {the_key:?} @ {}", level.index());
                        panic!("failed to find resolution clause")
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
                                            let new_key = stored_clauses
                                                .transfer_to_binary(*the_key, variables, *literal);
                                            self.trail.push(new_key);
                                        }
                                        Err(e) => panic!("{e:?}"),
                                    };
                                }
                            },
                            _ => {
                                match source_clause.subsume(*literal, variables, true) {
                                    Ok(_) => {
                                        self.trail.push(*the_key);
                                    }
                                    Err(e) => panic!("{e:?}"),
                                };
                            }
                        }
                    }

                    if self.valueless_count == 1 {
                        match config.stopping_criteria {
                            StoppingCriteria::FirstUIP => return Status::FirstUIP,
                            StoppingCriteria::None => {}
                        }
                    };
                }
            }
        }
        Status::Exhausted
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
    fn merge_clause(&mut self, clause: &impl Clause) {
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
                    Some(_) => panic!("resolution to a satisfied clause"),
                },
                ResolutionCell::Strengthened => {}
            }
        }
    }

    fn resolve_clause(&mut self, clause: &impl Clause, using: Literal) -> Result<(), ()> {
        match unsafe { *self.buffer.get_unchecked(using.index()) } {
            ResolutionCell::NoneLiteral(literal) if using == !literal => {
                self.merge_clause(clause);
                self.clause_length -= 1;
                self.set(using.index(), ResolutionCell::Pivot);
                self.valueless_count -= 1;

                Ok(())
            }
            ResolutionCell::ConflictLiteral(literal) if using == !literal => {
                self.merge_clause(clause);
                self.clause_length -= 1;
                self.set(using.index(), ResolutionCell::Pivot);

                Ok(())
            }
            _ => Err(()),
        }
    }

    fn to_clause(&self) -> Vec<Literal> {
        let (assertion, mut clause) = self.to_assertion_clause();
        if let Some(asserted) = assertion {
            clause.push(asserted)
        }
        clause
    }

    fn set(&mut self, index: usize, to: ResolutionCell) {
        *unsafe { self.buffer.get_unchecked_mut(index) } = to
    }
}
