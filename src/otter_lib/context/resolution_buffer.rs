use crate::{
    config::{Config, StoppingCriteria},
    context::{
        store::{ClauseKey, ClauseStore},
        ImplicationGraphNode, ResolutionGraph,
    },
    structures::{
        clause::Clause,
        literal::{Literal, Source as LiteralSource},
        variable::list::VariableList,
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
    valueless_count: usize,
    clause_legnth: usize,
    asserts: Option<Literal>,
    buffer: Vec<ResolutionCell>,
    trail: Vec<ClauseKey>,
    used_variables: Vec<bool>,
}

pub enum Status {
    FirstUIP,
    Exhausted,
}

impl ResolutionBuffer {
    pub fn new(size: usize) -> Self {
        ResolutionBuffer {
            valueless_count: 0,
            clause_legnth: 0,
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
            clause_legnth: 0,
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
        observations: &[(LiteralSource, Literal)],
        stored_clauses: &mut ClauseStore,
        graph: &ResolutionGraph,
        variables: &impl VariableList,
        config: &Config,
    ) -> Status {
        for (source, literal) in observations.iter().rev() {
            if let LiteralSource::Clause(node_index) = source {
                let the_node = graph.node_weight(*node_index).expect("missing node");
                let the_key = match the_node {
                    ImplicationGraphNode::Clause(graph_clause) => graph_clause.key,
                    ImplicationGraphNode::Literal(_) => panic!("literal panic"),
                };

                let source_clause = stored_clauses.retreive_mut(the_key);

                if self.resolve_clause(source_clause, *literal).is_ok() {
                    self.trail.push(the_key);

                    for involved_literal in source_clause.literal_slice() {
                        self.used_variables[involved_literal.index()] = true;
                    }

                    if config.subsumption && self.clause_legnth < source_clause.length() {
                        let _ = source_clause.subsume(*literal, variables);
                        // variables.get_unsafe(literal.index()).multiply_activity(1.1);
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
    pub fn strengthen_given(&mut self, literals: impl Iterator<Item = Literal>) {
        for literal in literals {
            match self.buffer[literal.index()] {
                ResolutionCell::NoneLiteral(_) | ResolutionCell::ConflictLiteral(_) => {
                    if let Some(length_minus_one) = self.clause_legnth.checked_sub(1) {
                        self.clause_legnth = length_minus_one;
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
                        self.clause_legnth += 1;
                        self.valueless_count += 1;
                        self.set(literal.index(), ResolutionCell::NoneLiteral(*literal));
                        if self.asserts.is_none() {
                            self.asserts = Some(*literal);
                        }
                    }
                    Some(value) if *value != literal.polarity() => {
                        self.clause_legnth += 1;
                        self.set(literal.index(), ResolutionCell::ConflictLiteral(*literal))
                    }
                    Some(_) => panic!("Resolution to a satisfied clause"),
                },
                ResolutionCell::Strengthened => {}
            }
        }
    }

    fn resolve_clause(&mut self, clause: &impl Clause, using: Literal) -> Result<(), ()> {
        match unsafe { *self.buffer.get_unchecked(using.index()) } {
            ResolutionCell::NoneLiteral(literal) if using == !literal => {
                self.merge_clause(clause);
                self.clause_legnth -= 1;
                self.set(using.index(), ResolutionCell::Pivot);
                self.valueless_count -= 1;

                Ok(())
            }
            ResolutionCell::ConflictLiteral(literal) if using == !literal => {
                self.merge_clause(clause);
                self.clause_legnth -= 1;
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
