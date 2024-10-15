use crate::{
    config,
    structures::{
        clause::Clause,
        literal::{Literal, Source as LiteralSource},
        solve::{retreive_unsafe, ClauseKey, ClauseStore},
        valuation::Valuation,
    },
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ResolutionCell {
    Value(Option<bool>),
    NoneLiteral(Literal),
    ConflictLiteral(Literal),
}

#[derive(Debug)]
pub struct ResolutionBuffer {
    missing: usize,
    asserts: Option<Literal>,
    buffer: Vec<ResolutionCell>,
    trail: Vec<ClauseKey>,
    used_variables: Vec<bool>,
}

impl ResolutionBuffer {
    pub fn from_valuation(valuation: &impl Valuation) -> Self {
        ResolutionBuffer {
            missing: 0,
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

    fn set(&mut self, index: usize, to: ResolutionCell) {
        *unsafe { self.buffer.get_unchecked_mut(index) } = to
    }

    pub fn eat_clause(&mut self, clause: &impl Clause) {
        for literal in clause.literal_slice() {
            match self.buffer.get(literal.index()).expect("wuh") {
                ResolutionCell::ConflictLiteral(_) | ResolutionCell::NoneLiteral(_) => {}
                ResolutionCell::Value(maybe) => match maybe {
                    None => {
                        self.missing += 1;
                        self.asserts = Some(*literal);
                        self.set(literal.index(), ResolutionCell::NoneLiteral(*literal));
                    }
                    Some(value) if *value == literal.polarity() => {
                        panic!("huh")
                    }
                    Some(_) => self.set(literal.index(), ResolutionCell::ConflictLiteral(*literal)),
                },
            }
        }
    }

    fn resolve_clause(&mut self, clause: &impl Clause, using: Literal) -> bool {
        if self.buffer[using.index()] == ResolutionCell::NoneLiteral(using.negate()) {
            self.eat_clause(clause);
            self.missing -= 1;
            self.set(using.index(), ResolutionCell::Value(Some(false)));
            true
        } else {
            false
        }
    }

    pub fn to_assertion_clause(&self) -> (Option<Literal>, Vec<Literal>) {
        let mut the_clause = Vec::with_capacity(self.buffer.len());
        let mut conflict_literal = None;
        for item in &self.buffer {
            match item {
                ResolutionCell::Value(_) => {}
                ResolutionCell::ConflictLiteral(literal) => the_clause.push(*literal),
                ResolutionCell::NoneLiteral(literal) => conflict_literal = Some(*literal),
            }
        }
        (conflict_literal, the_clause)
    }

    fn to_clause(&self) -> Vec<Literal> {
        let (assertion, mut clause) = self.to_assertion_clause();
        if let Some(asserted) = assertion {
            clause.push(asserted)
        }
        clause
    }

    pub fn clear_literals(&mut self, literals: impl Iterator<Item = Literal>) {
        for literal in literals {
            self.set(literal.index(), ResolutionCell::Value(None))
        }
    }

    pub fn resolve_with<'a>(
        &mut self,
        observations: impl Iterator<Item = &'a (LiteralSource, Literal)>,
        formula_clauses: &ClauseStore,
        learnt_clauses: &ClauseStore,
    ) {
        'resolution_loop: for (src, literal) in observations {
            if let LiteralSource::StoredClause(clause_key) = src {
                let stored_source_clause =
                    retreive_unsafe(formula_clauses, learnt_clauses, *clause_key);

                if self.resolve_clause(stored_source_clause, *literal) {
                    self.trail.push(*clause_key);
                }

                for involved_literal in stored_source_clause.literal_slice() {
                    self.used_variables[involved_literal.index()] = true;
                }

                if self.missing == 1 {
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
    This is a temp implementation which sets the cell in the buffer to no value so the variable isn't collected into a clause using the given methods
     */
    pub fn strengthen_given(&mut self, literals: impl Iterator<Item = Literal>) {
        for literal in literals {
            self.set(literal.index(), ResolutionCell::Value(None))
        }
    }

    pub fn asserts(&self) -> Option<Literal> {
        if self.missing == 1 {
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
