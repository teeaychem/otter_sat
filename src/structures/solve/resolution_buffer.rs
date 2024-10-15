use crate::{
    config,
    structures::{
        clause::Clause,
        literal::{Literal, Source as LiteralSource},
        solve::store::{ClauseKey, ClauseStore},
        valuation::Valuation,
    },
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ResolutionCell {
    Value(Option<bool>),
    NoneLiteral(Literal),
    ConflictLiteral(Literal),
    Strengthened,
}

#[derive(Debug)]
pub struct ResolutionBuffer {
    valuless_count: usize,
    asserts: Option<Literal>,
    buffer: Vec<ResolutionCell>,
    trail: Vec<ClauseKey>,
    used_variables: Vec<bool>,
}

impl ResolutionBuffer {
    pub fn new(size: usize) -> Self {
        ResolutionBuffer {
            valuless_count: 0,
            asserts: None,
            buffer: vec![ResolutionCell::Value(None); size],
            trail: vec![],
            used_variables: vec![false; size],
        }
    }

    pub fn reset_with(&mut self, valuation: &impl Valuation) {
        self.valuless_count = 0;
        self.asserts = None;
        for (index, value) in valuation.slice().iter().enumerate() {
            self.set(index, ResolutionCell::Value(*value))
        }
        self.trail.clear();
        self.used_variables
            .iter_mut()
            .for_each(|index| *index = false);
    }

    pub fn from_valuation(valuation: &impl Valuation) -> Self {
        ResolutionBuffer {
            valuless_count: 0,
            asserts: None,
            buffer: valuation
                .slice()
                .iter()
                .map(|value| ResolutionCell::Value(*value))
                .collect(),
            trail: vec![],
            used_variables: vec![false; valuation.slice().len()],
        }
    }

    pub fn merge_clause(&mut self, clause: &impl Clause) {
        for literal in clause.literal_slice() {
            match self.buffer.get(literal.index()).expect("wuh") {
                ResolutionCell::ConflictLiteral(_) | ResolutionCell::NoneLiteral(_) => {}
                ResolutionCell::Value(maybe) => match maybe {
                    None => {
                        self.valuless_count += 1;
                        self.asserts = Some(*literal);
                        self.set(literal.index(), ResolutionCell::NoneLiteral(*literal));
                    }
                    Some(value) if *value != literal.polarity() => {
                        self.set(literal.index(), ResolutionCell::ConflictLiteral(*literal))
                    }
                    Some(_) => {}
                },
                ResolutionCell::Strengthened => {}
            }
        }
    }

    pub fn to_assertion_clause(&self) -> (Option<Literal>, Vec<Literal>) {
        let mut the_clause = Vec::with_capacity(self.buffer.len());
        let mut conflict_literal = None;
        for item in &self.buffer {
            match item {
                ResolutionCell::Strengthened | ResolutionCell::Value(_) => {}
                ResolutionCell::ConflictLiteral(literal) => the_clause.push(*literal),
                ResolutionCell::NoneLiteral(literal) => conflict_literal = Some(*literal),
            }
        }
        (conflict_literal, the_clause)
    }

    pub fn clear_literals(&mut self, literals: impl Iterator<Item = Literal>) {
        for literal in literals {
            self.set(literal.index(), ResolutionCell::Value(None))
        }
    }

    pub fn resolve_with<'a>(
        &mut self,
        observations: impl Iterator<Item = &'a (LiteralSource, Literal)>,
        stored_clauses: &ClauseStore,
    ) {
        'resolution_loop: for (src, literal) in observations {
            if let LiteralSource::StoredClause(clause_key) = src {
                let stored_source_clause = stored_clauses.retreive_unsafe(*clause_key);

                if self.resolve_clause(stored_source_clause, *literal) {
                    self.trail.push(*clause_key);
                }

                for involved_literal in stored_source_clause.literal_slice() {
                    self.used_variables[involved_literal.index()] = true;
                }

                if self.valuless_count == 1 {
                    match unsafe { config::STOPPING_CRITERIA } {
                        config::StoppingCriteria::FirstUIP => break 'resolution_loop,
                        config::StoppingCriteria::None => {}
                    }
                };
            }
        }
    }

    /*
    If some literals are known then their negation can be safely removed from the learnt clause.
     */
    pub fn strengthen_given(&mut self, literals: impl Iterator<Item = Literal>) {
        for literal in literals {
            self.set(literal.index(), ResolutionCell::Strengthened)
        }
    }

    pub fn asserts(&self) -> Option<Literal> {
        if self.valuless_count == 1 {
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
    fn resolve_clause(&mut self, clause: &impl Clause, using: Literal) -> bool {
        if self.buffer[using.index()] == ResolutionCell::NoneLiteral(using.negate()) {
            self.merge_clause(clause);
            self.valuless_count -= 1;
            self.set(using.index(), ResolutionCell::Value(Some(false)));
            true
        } else {
            false
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
