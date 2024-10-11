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

        {
            let level = self.current_level().index();
            log::trace!("Fix on clause {conflict_clause} @ {level}");
        }

        match self.current_level().index() {
            0 => SolveStatus::NoSolution,
            _ => {
                let (asserting_clause, clause_source, assertion) =
                    self.conflict_analysis(conflict_clause);

                if asserting_clause.len() == 1 {
                    self.backjump(0);

                    let literal_source = match clause_source {
                        ClauseSource::Resolution(resolution_vector) => {
                            LiteralSource::Resolution(resolution_vector)
                        }
                        _ => panic!("Analysis without resolution"),
                    };
                    literal_update(
                        assertion,
                        literal_source.clone(),
                        &mut self.levels,
                        &self.variables,
                        &mut self.valuation,
                        &mut self.formula_clauses,
                        &mut self.learnt_clauses,
                    );

                } else {
                    self.backjump(decision_level(&self.variables, asserting_clause.literals()));

                    let clause_key = self.store_clause(asserting_clause, clause_source);
                    let stored_clause =
                        retreive(&self.formula_clauses, &self.learnt_clauses, clause_key);

                    let anticipated_literal_source = LiteralSource::StoredClause(clause_key);

                    stored_clause.set_lbd(&self.variables);

                    literal_update(
                        assertion,
                        anticipated_literal_source.clone(),
                        &mut self.levels,
                        &self.variables,
                        &mut self.valuation,
                        &mut self.formula_clauses,
                        &mut self.learnt_clauses,
                    );

                }
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

        let previous_level_val = self.valuation_at(self.current_level().index() - 1);
        let mut asserted_literal = None;

        let stopping_criteria = unsafe { config::STOPPING_CRITERIA };

        let variable_count = self.variables.len();
        let mut used_variables = vec![false; variable_count];

        let mut x = self.current_level().observations.clone();
        x.reverse();

        for (src, literal) in &x {
            match stopping_criteria {
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
                    resolved_clause = resolution.to_vec();
                };
            }
        }

        if asserted_literal.is_none() {
            if let Some(asserted) = resolved_clause.asserts(&previous_level_val) {
                asserted_literal = Some(asserted);
            } else {
                for x in resolution_trail {
                    let cls = retreive(&self.formula_clauses, &self.learnt_clauses, x);
                    println!("{}", cls.as_string());
                }
                for ob in self.current_level().observations() {
                    println!("OBS {:?}", ob);
                }

                println!("CC {}", conflict_clause.as_string());
                println!("RC {}", resolved_clause.as_string());
                println!("PV {}", previous_level_val.as_internal_string());
                println!("CV {}", self.valuation.as_internal_string());
                panic!("No assertion…")
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
                for variable in resolved_clause.variables() {
                    self.variables[variable as usize].add_activity(config::ACTIVITY_CONFLICT);
                }
            }
            config::VSIDS::M =>
            {
                #[allow(clippy::needless_range_loop)]
                for index in 0..variable_count {
                    if used_variables[index] {
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
        let node_indicies = self.levels[0]
            .observations()
            .iter()
            .filter_map(|(source, _)| match source {
                LiteralSource::StoredClause(weak) => Some(*weak),
                _ => None,
            });
        let node_indicies_vec = node_indicies.collect::<Vec<_>>();
        let simple_core = self.extant_origins(node_indicies_vec);
        for clause in simple_core {
            println!("{}", clause.as_dimacs(&self.variables))
        }
        println!();
    }

    pub fn extant_origins(&self, clauses: Vec<ClauseKey>) -> Vec<&StoredClause> {
        #[allow(clippy::mutable_key_type)]
        let mut origin_nodes = vec![];

        let mut q = VecDeque::new();
        for clause in clauses {
            q.push_back(clause);
        }
        loop {
            if q.is_empty() {
                break;
            }

            let clause_key = q.pop_front().expect("Ah, the queue was empty…");
            let stored_clause = retreive(&self.formula_clauses, &self.learnt_clauses, clause_key);

            match stored_clause.source() {
                ClauseSource::Resolution(origins) => {
                    for antecedent in origins {
                        q.push_back(*antecedent);
                    }
                }
                ClauseSource::Formula => {
                    origin_nodes.push(stored_clause);
                }
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
