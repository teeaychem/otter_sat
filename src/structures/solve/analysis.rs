use crate::structures::{
    clause::{
        stored::{Source as ClauseSource, StoredClause},
        Clause, ClauseVec,
    },
    literal::{Literal, Source as LiteralSource},
    solve::{config, retreive_unsafe, ClauseKey, Solve, Status},
    valuation::Valuation,
    variable::Variable,
};

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
enum RSItem {
    Value(Option<bool>),
    NoneLiteral(Literal),
    ConflictLiteral(Literal),
}

#[derive(Debug)]
struct ResolutionBuffer {
    missing: usize,
    asserts: Option<Literal>,
    buffer: Vec<RSItem>,
}

impl ResolutionBuffer {
    fn from_valuation(valuation: &impl Valuation) -> Self {
        ResolutionBuffer {
            missing: 0,
            asserts: None,
            buffer: valuation
                .slice()
                .iter()
                .map(|value| RSItem::Value(*value))
                .collect(),
        }
    }

    fn eat_clause(&mut self, clause: &impl Clause) {
        for literal in clause.literal_slice() {
            match self.buffer.get(literal.index()).expect("wuh") {
                RSItem::ConflictLiteral(_) | RSItem::NoneLiteral(_) => {}
                RSItem::Value(maybe) => match maybe {
                    None => {
                        self.buffer[literal.index()] = {
                            self.missing += 1;
                            RSItem::NoneLiteral(*literal)
                        }
                    }
                    Some(value) if *value == literal.polarity() => {
                        panic!("huh")
                    }
                    Some(_) => self.buffer[literal.index()] = RSItem::ConflictLiteral(*literal),
                },
            }
        }
    }

    fn resolve_clause(&mut self, clause: &impl Clause, using: Literal) -> bool {
        if self.buffer[using.index()] == RSItem::NoneLiteral(using.negate()) {
            for literal in clause.literal_slice() {
                match self.buffer.get(literal.index()).expect("wuh") {
                    RSItem::ConflictLiteral(_) | RSItem::NoneLiteral(_) => {}
                    RSItem::Value(maybe) => match maybe {
                        None => {
                            self.missing += 1;
                            self.asserts = Some(*literal);
                            self.buffer[literal.index()] = RSItem::NoneLiteral(*literal)
                        }
                        Some(value) if *value == literal.polarity() => {
                            panic!("huh")
                        }
                        Some(_) => self.buffer[literal.index()] = RSItem::ConflictLiteral(*literal),
                    },
                }
            }
            self.missing -= 1;
            self.buffer[using.index()] = RSItem::Value(Some(false));
            true
        } else {
            false
        }
    }

    fn to_clause(&self) -> (Option<Literal>, Vec<Literal>) {
        let mut the_clause = Vec::with_capacity(self.buffer.len());
        let mut conflict_literal = None;
        for item in &self.buffer {
            match item {
                RSItem::Value(_) => {}
                RSItem::ConflictLiteral(literal) => the_clause.push(*literal),
                RSItem::NoneLiteral(literal) => {
                    conflict_literal = Some(*literal);
                    the_clause.push(*literal)
                }
            }
        }
        (conflict_literal, the_clause)
    }
}

impl Solve {
    pub fn attempt_fix(&mut self, clause_key: ClauseKey) -> Status {
        let conflict_clause =
            retreive_unsafe(&self.formula_clauses, &self.learnt_clauses, clause_key);

        log::trace!("Fix on clause {conflict_clause} @ {}", self.level().index());
        if self.level().index() == 0 {
            return Status::NoSolution;
        }

        let mut previous_level_val = self.valuation.clone();
        for literal in self.level().literals() {
            previous_level_val[literal.index()] = None;
        }
        if let Some(asserted) = conflict_clause.asserts(&previous_level_val) {
            let missed_level = backjump_level(&self.variables, conflict_clause.literal_slice());
            self.backjump(missed_level);
            self.literal_update(asserted, &LiteralSource::StoredClause(clause_key));
            self.consequence_q.push_back(asserted);
            Status::MissedImplication
        } else {
            let (asserting_clause, clause_source, assertion) =
                self.conflict_analysis(conflict_clause);

            let source = match asserting_clause.len() {
                1 => {
                    self.backjump(0);
                    match clause_source {
                        ClauseSource::Resolution(resolution_vector) => {
                            LiteralSource::Resolution(resolution_vector)
                        }
                        ClauseSource::Formula => panic!("Analysis without resolution"),
                    }
                }
                _ => {
                    self.backjump(backjump_level(
                        &self.variables,
                        asserting_clause.literal_slice(),
                    ));

                    LiteralSource::StoredClause(self.store_clause(asserting_clause, clause_source))
                }
            };
            self.literal_update(assertion, &source);
            self.consequence_q.push_back(assertion);
            Status::AssertingClause
        }
    }

    /// Simple analysis performs resolution on any clause used to obtain a conflict literal at the current decision
    pub fn conflict_analysis(
        &self,
        conflict_clause: &StoredClause,
    ) -> (ClauseVec, ClauseSource, Literal) {
        let mut resolved_clause = conflict_clause.clause_clone();
        let mut resolution_trail = vec![];

        let mut previous_level_val = self.valuation.clone();
        for literal in self.level().literals() {
            previous_level_val[literal.index()] = None;
        }

        let mut resolution_buffer = ResolutionBuffer::from_valuation(&previous_level_val);
        resolution_buffer.eat_clause(&resolved_clause);

        let mut asserted_literal = None;

        let mut used_variables = vec![false; self.variables.len()];

        'resolution_loop: for (src, literal) in self.level().observations.iter().rev() {
            if let LiteralSource::StoredClause(clause_key) = src {
                let stored_source_clause =
                    retreive_unsafe(&self.formula_clauses, &self.learnt_clauses, *clause_key);

                if resolution_buffer.resolve_clause(stored_source_clause, *literal) {
                    resolution_trail.push(*clause_key);
                }

                for involved_literal in stored_source_clause.literal_slice() {
                    used_variables[involved_literal.index()] = true;
                }

                if resolution_buffer.missing == 1 {
                    (asserted_literal, resolved_clause) = resolution_buffer.to_clause();

                    match unsafe { config::STOPPING_CRITERIA } {
                        config::StoppingCriteria::FirstUIP => break 'resolution_loop,
                        config::StoppingCriteria::None => {}
                    }
                };
            }
        }

        /*
        If some literals are known then their negation can be safely removed from the learnt clause.
        Though, this isn't a particular effective method… really, iteration should be over the clause
         */
        for (_, other_literal) in &self.levels[0].observations {
            if let Some(x) = resolved_clause.literal_position(other_literal.negate()) {
                resolved_clause.remove(x);
            }
        }

        unsafe {
            match config::VSIDS_VARIANT {
                config::VSIDS::Chaff => {
                    for literal in resolved_clause.literal_slice() {
                        self.variables
                            .get_unchecked(literal.index())
                            .add_activity(config::ACTIVITY_CONFLICT);
                    }
                }
                config::VSIDS::MiniSAT => {
                    for (index, used) in used_variables.into_iter().enumerate() {
                        if used {
                            self.variables
                                .get_unchecked(index)
                                .add_activity(config::ACTIVITY_CONFLICT);
                        }
                    }
                }
            }
        }

        (
            resolved_clause,
            ClauseSource::Resolution(resolution_trail),
            asserted_literal.unwrap(),
        )
    }

    pub fn display_core(&self) {
        println!();
        println!("c An unsatisfiable core of the original formula:\n");
        let mut node_indicies = vec![];
        for (source, _) in &self.levels[0].observations {
            match source {
                LiteralSource::StoredClause(key) => node_indicies.push(*key),
                LiteralSource::Resolution(keys) => node_indicies.extend(keys),
                _ => {}
            }
        }
        let mut origins = self.extant_origins(node_indicies.iter().copied());
        origins.sort_unstable_by_key(|s| s.key());
        origins.dedup_by_key(|s| s.key());

        for clause in origins {
            println!("{}", clause.as_dimacs(&self.variables));
        }
        println!();
    }

    pub fn extant_origins(&self, clauses: impl Iterator<Item = ClauseKey>) -> Vec<&StoredClause> {
        let mut origin_nodes = vec![];
        let mut q = clauses.collect::<VecDeque<_>>();

        while !q.is_empty() {
            let clause_key = q.pop_front().expect("Ah, the queue was empty…");

            let stored_clause =
                retreive_unsafe(&self.formula_clauses, &self.learnt_clauses, clause_key);
            match stored_clause.source() {
                ClauseSource::Resolution(origins) => {
                    for antecedent in origins {
                        q.push_back(*antecedent);
                    }
                }
                ClauseSource::Formula => origin_nodes.push(stored_clause),
            }
        }
        origin_nodes
    }
}

/// Either the most recent decision level in the resolution clause prior to the current level or 0.
/*
The implementation works through the clause, keeping an ordered record of the top two decision levels: (second_to_top, top)
the top decision level will be for the literal to be asserted when clause is learnt
 */
fn backjump_level(variables: &[Variable], literals: &[Literal]) -> usize {
    let mut top_two = (None, None);
    for lit in literals {
        if let Some(dl) = unsafe { (*variables.get_unchecked(lit.index())).decision_level() } {
            match top_two {
                (_, None) => top_two.1 = Some(dl),
                (_, Some(t1)) if dl > t1 => {
                    top_two.0 = top_two.1;
                    top_two.1 = Some(dl);
                }
                (None, _) => top_two.0 = Some(dl),
                (Some(t2), _) if dl > t2 => top_two.0 = Some(dl),
                _ => {}
            }
        }
    }

    match top_two {
        (None, _) => 0,
        (Some(second_to_top), _) => second_to_top,
    }
}
