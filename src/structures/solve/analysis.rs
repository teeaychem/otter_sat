use crate::procedures::resolve_sorted_clauses;
use crate::structures::valuation::Valuation;
use crate::structures::{
    clause::{
        clause_vec::ClauseVec,
        stored_clause::{ClauseSource, StoredClause},
        Clause,
    },
    literal::{Literal, LiteralSource},
    solve::{config, retreive, the_solve::literal_update, ClauseKey, Solve, SolveStatus},
    variable::Variable,
};

use std::collections::VecDeque;

impl Solve {
    pub fn attempt_fix(&mut self, clause_key: ClauseKey) -> SolveStatus {
        let conflict_clause = retreive(&self.formula_clauses, &self.learnt_clauses, clause_key);

        log::trace!("Fix on clause {conflict_clause} @ {}", self.level().index());

        match self.level().index() {
            0 => SolveStatus::NoSolution,
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
                            _ => panic!("Analysis without resolution"),
                        }
                    }
                    _ => {
                        self.backjump(decision_level(&self.variables, asserting_clause.literals()));

                        LiteralSource::StoredClause(
                            self.store_clause(asserting_clause, clause_source),
                        )
                    }
                };
                literal_update(
                    assertion,
                    source,
                    &mut self.levels,
                    &self.variables,
                    &mut self.valuation,
                    &mut self.formula_clauses,
                    &mut self.learnt_clauses,
                );
                self.watch_q.push_back(assertion);
                SolveStatus::AssertingClause
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

        let previous_level_val = self.valuation_at(self.level().index() - 1);
        let mut asserted_literal = None;

        let mut used_variables = vec![false; self.variables.len()];

        for (src, literal) in self.level().observations.iter().rev() {
            match unsafe { config::STOPPING_CRITERIA } {
                config::StoppingCriteria::FirstAssertingUIP => {
                    if let Some(asserted) = resolved_clause.asserts(&previous_level_val) {
                        asserted_literal = Some(asserted);
                        break;
                    }
                }
                config::StoppingCriteria::None => (),
            }

            if let LiteralSource::StoredClause(clause_key) = src {
                let stored_source_clause =
                    retreive(&self.formula_clauses, &self.learnt_clauses, *clause_key);

                for involved_literal in stored_source_clause.literals() {
                    used_variables[involved_literal.index()] = true;
                }

                let for_the_borrow_checker = resolved_clause.clone();
                let resolution_result = resolve_sorted_clauses(
                    for_the_borrow_checker.literals(),
                    stored_source_clause.literals(),
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
                        let cls = retreive(&self.formula_clauses, &self.learnt_clauses, x);
                        println!("{}", cls.as_string());
                    }
                    for ob in self.level().observations() {
                        println!("OBS {:?}", ob);
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
        if !self.levels[0].observations().is_empty() {
            resolved_clause.retain(|l| {
                !self.levels[0]
                    .observations()
                    .iter()
                    .any(|(_, x)| l.negate() == *x)
            })
        }

        match unsafe { config::VSIDS_VARIANT } {
            config::VSIDS::C => {
                for variable_index in resolved_clause.literals().map(|l| l.index()) {
                    self.variables[variable_index].add_activity(config::ACTIVITY_CONFLICT);
                }
            }
            config::VSIDS::M => {
                for (index, used) in used_variables.into_iter().enumerate() {
                    if used {
                        self.variables[index].add_activity(config::ACTIVITY_CONFLICT);
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

    pub fn core(&self) {
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
        let mut origins = self.extant_origins(node_indicies.iter().cloned());
        origins.sort_unstable_by_key(|s| s.key());
        origins.dedup_by_key(|s| s.key());

        for clause in origins {
            println!("{}", clause.as_dimacs(&self.variables))
        }
        println!();
    }

    pub fn extant_origins(&self, clauses: impl Iterator<Item = ClauseKey>) -> Vec<&StoredClause> {
        let mut origin_nodes = vec![];
        let mut q = VecDeque::from_iter(clauses);

        while !q.is_empty() {
            let clause_key = q.pop_front().expect("Ah, the queue was empty…");
            let stored_clause = retreive(&self.formula_clauses, &self.learnt_clauses, clause_key);

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
fn decision_level(variables: &[Variable], literals: impl Iterator<Item = Literal>) -> usize {
    let mut top_two = (None, None);
    for lit in literals {
        if let Some(dl) = unsafe { (*variables.get_unchecked(lit.index())).decision_level() } {
            if top_two.1.is_none() {
                top_two.1 = Some(dl)
            } else if top_two.1.is_some_and(|t1| dl > t1) {
                top_two.0 = top_two.1;
                top_two.1 = Some(dl)
            } else if top_two.0.is_none() || top_two.0.is_some_and(|t2| dl > t2) {
                top_two.0 = Some(dl)
            };
        }
    }

    match top_two {
        (None, Some(_)) => 0,
        (Some(x), Some(_)) => x,
        _ => panic!("Decision level issue: {:?}", top_two),
    }
}
