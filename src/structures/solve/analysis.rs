use crate::procedures::resolve_sorted_clauses;
use crate::structures::valuation::Valuation;
use crate::structures::{
    clause::{
        stored::{Source as ClauseSource, StoredClause},
        Clause, ClauseVec,
    },
    literal::{Literal, Source as LiteralSource},
    solve::{config, retreive, ClauseKey, Solve, Status},
    variable::Variable,
};

use std::collections::VecDeque;

use super::retreive_unsafe;

impl Solve {
    pub fn attempt_fix(&mut self, clause_key: ClauseKey) -> Status {
        let conflict_clause =
            retreive_unsafe(&self.formula_clauses, &self.learnt_clauses, clause_key);

        log::trace!("Fix on clause {conflict_clause} @ {}", self.level().index());

        match self.level().index() {
            0 => Status::NoSolution,
            _ => {
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

                        LiteralSource::StoredClause(
                            self.store_clause(asserting_clause, clause_source),
                        )
                    }
                };
                self.literal_update(assertion, &source);
                self.consequence_q.push_back(assertion);
                Status::AssertingClause
            }
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

        let mut asserted_literal = None;

        let mut used_variables = vec![false; self.variables.len()];

        for (src, literal) in self.level().observations.iter().rev() {
            match unsafe { config::STOPPING_CRITERIA } {
                config::StoppingCriteria::FirstUIP => {
                    if let Some(asserted) = resolved_clause.asserts(&previous_level_val) {
                        asserted_literal = Some(asserted);
                        break;
                    }
                }
                config::StoppingCriteria::None => (),
            }

            if let LiteralSource::StoredClause(clause_key) = src {
                let stored_source_clause =
                    retreive_unsafe(&self.formula_clauses, &self.learnt_clauses, *clause_key);

                for involved_literal in stored_source_clause.literal_slice() {
                    used_variables[involved_literal.index()] = true;
                }

                let for_the_borrow_checker = resolved_clause.clone();
                let resolution_result = resolve_sorted_clauses(
                    for_the_borrow_checker.literal_slice(),
                    stored_source_clause.literal_slice(),
                    literal.v_id(),
                );
                if let Some(resolution) = resolution_result {
                    resolution_trail.push(*clause_key);
                    resolved_clause = resolution.to_clause_vec();
                };
            }
        }

        if asserted_literal.is_none() {
            match resolved_clause.asserts(&previous_level_val) {
                Some(asserted) => asserted_literal = Some(asserted),
                None => {
                    for x in resolution_trail {
                        let cls = retreive(&self.formula_clauses, &self.learnt_clauses, x).unwrap();
                        println!("{}", cls.as_string());
                    }
                    for ob in self.level().observations() {
                        println!("OBS {ob:?}");
                    }

                    println!("CC {}", conflict_clause.as_string());
                    println!("RC {}", resolved_clause.as_string());
                    println!("PV {}", previous_level_val.as_internal_string());
                    println!("CV {}", self.valuation.as_internal_string());
                    panic!("No assertion…")
                }
            }
        }

        /*
        If some literals are known then their negation can be safely removed from the learnt clause.
        Though, this isn't a particular effective method…
         */
        resolved_clause.retain(|l| {
            !self.levels[0]
                .observations()
                .iter()
                .any(|(_, other_literal)| l.negate() == *other_literal)
        });

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
