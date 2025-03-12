/*!
Determines the satisfiability of the formula in a context.

# Overview

[solve](crate::procedures::solve) casts the conflict-driven clause-learning algorithm through a valuation relative consequence operator over formulas.

On this operator a formula entails either itself, or a tautological consequence of itself with some additional clause.
And, if the operator cannot be applied, the formula is unsatisfiable.[^op-note]
[^op-note]: Alternatively, the operator may return some designated formula such as falsum.

- If the formula entails itself, then inspection of the valuation is required:
  + If the valuation is partial (and not complete) the valuation *may be* satisfiable, though the rules of inference associated with the operator do not support the derivation of a complete valuation (and so some decision must be made).
  + If the valuation is complete, the formula is satisfiable (on the given valuation).
- If the formula entails some formula with an additional clause, then:
  + The formula is unsatisfiable on the given valuation, but *may be* satisfiable on some other valuation.\
    Specifically, there is some sub-valuation of the current valuation on which the added clause asserts some literal, and a '[backjump](crate::procedures::backjump)' may be made to that valuation.

[solve](crate::procedures::solve), then, manages the detailed operator, whose implementation is given in [apply_consequences].
This amounts to applying an instance of the operator which:

- Returns *unsatisfiable*, if it is not possible to apply the consequence relation.
- Returns *satisfiable*, if the formula entails itself and the valuation is complete.
- Makes a decision, if the formula entails itself and the valuation is partial.
- Backjumps to a different valuation, if the formula entails some formula with an additional clause.

Though, at points this process may be interrupted for some other action.
In particular, [solve](crate::procedures::solve) may revise the valuation to some other valuation (e.g. by forgetting any decisions made) regardless of whether the formula entails some formula with an additional clause.

Roughly, the loop is as diagrammed:

```none
          +---------------+
  +-------| make_decision |
  |       +---------------+
  |               ⌃
  |               |
  |               | if there is no update to the formula, and the valuation is partial
  |               |
  |               |              +-----> satisfiable, if the valuation is full
  ⌄   +--------------------+     |
--+-->| apply_consequences |-----+
  ⌃   +--------------------+     |
  |               |              +-----> unsatisfiable, if apply_consequences fails
  |               |
  |               | if a clause is added to the formula
  |               |
  |               ⌄
  |          +----------+
  +----------| backjump |
             +----------+
```

And, abstracting from various other bookkeeping tasks and optional actions after a context, solve is:

```rust,ignore
loop {

    match self.apply_consequences()? {
        ApplyConsequencesOk::FundamentalConflict => break,

        ApplyConsequencesOk::Exhausted => {
            //
            match self.make_decision()? {
                decision::Ok::Made => continue,
                decision::Ok::Exhausted => break,
            }
        }

        ApplyConsequencesOk::UnitClause(literal) => {
            self.backjump(0);
            self.q_literal(literal)?;
        }

        apply_consequences::Ok::AssertingClause(key, literal) => {
            let the_clause = self.clause_db.get(&key)?;
            self.backjump(self.non_chronological_backjump_level(the_clause)?);
            self.q_literal(literal)?;
        }
    }
    // Additional actions after a conflict, before the next loop.
    ...
}
```

The distinction between a unit clause and clause being returned from [apply_consequence](crate::procedures::apply_consequences) is made only to avoid the overhead of accessing a clause and determing the relevant backjump level in the case of a unit clause.

# Example

```rust
# use otter_sat::config::Config;
# use otter_sat::context::Context;
# use otter_sat::reports::Report;
# use otter_sat::structures::literal::{CLiteral, Literal};
let config = Config::default();
let mut ctx: Context = Context::from_config(config);

let p = ctx.fresh_or_max_atom();
let q = ctx.fresh_or_max_atom();

let not_p_or_q = vec![CLiteral::new(p, false), CLiteral::new(q, true)];
let p_or_not_q = vec![CLiteral::new(p, true), CLiteral::new(q, false)];
assert!(ctx.add_clause(not_p_or_q).is_ok());
assert!(ctx.add_clause(p_or_not_q).is_ok());

assert!(ctx.solve().is_ok());

assert_eq!(ctx.atom_db.value_of(p), Some(false));
assert_eq!(ctx.atom_db.value_of(q), Some(false));

ctx.clear_decisions();

let p_clause = vec![CLiteral::new(p, true)];
assert!(ctx.add_clause(p_clause).is_ok());

assert_eq!(ctx.atom_db.value_of(p), Some(true));

assert!(ctx.solve().is_ok());

assert_eq!(ctx.report(), Report::Satisfiable);
```

# Literature

The core solve procedure was developed by reading [Decision Procedures](https://doi.org/10.1007/978-3-662-50497-0)[^a]
and the [Handbook of satisfiability](https://www.iospress.com/catalog/books/handbook-of-satisfiability-2).[^b]
Though, the presentation given is original.

[^a]: Specifically, Chapter 2 on decision procedures for propositional logic.
[^b]: Specifcally, chapters 3 and 4 on complete algorithms and CDCL techniques.
*/

use crate::{
    context::{ContextState, GenericContext},
    db::{ClauseKey, atom::AtomValue},
    procedures::{apply_consequences::ApplyConsequencesOk, decision::DecisionOk},
    reports::Report,
    structures::{
        consequence::{Assignment, AssignmentSource},
        literal::CLiteral,
    },
    types::err::{self, ErrorKind},
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn solve(&mut self) -> Result<Report, err::ErrorKind> {
        self.solve_given(None)
    }

    /// Determines the satisfiability of the context, unless interrupted.
    pub fn solve_given(
        &mut self,
        assumptions: Option<Vec<CLiteral>>,
    ) -> Result<Report, err::ErrorKind> {
        use crate::db::consequence_q::QPosition::{self};

        match self.state {
            ContextState::Solving => {}

            ContextState::Satisfiable | ContextState::Unsatisfiable(_) => {
                return Ok(self.report());
            }

            ContextState::Configuration | ContextState::Input => {
                self.preprocess()?;

                // Initial BCP, this:
                // - Verifies clauses are satisfiable.
                // - Proves any available unit clauses prior to asserting assumptions.
                match self.propagate_queue() {
                    Ok(_) => {}

                    Err(ErrorKind::FundamentalConflict) => {
                        return Ok(self.report());
                    }

                    Err(e) => return Err(e),
                };

                if let Some(assumptions) = assumptions {
                    let assumption_result = self.assert_assumptions(assumptions);

                    match assumption_result {
                        Ok(_) => {}

                        // Each error lead to a return of some form…
                        Err(err::ErrorKind::SpecificValuationConflict(assumption)) => {
                            let assignment = self
                                .atom_db
                                .assignments
                                .iter()
                                .find(|a| a.literal == assumption);

                            let source = assignment.expect("! Conflict failure").source;

                            match source {
                                AssignmentSource::PureLiteral => todo!(),

                                AssignmentSource::Decision => {
                                    panic!("! Decision prior to main solve loop")
                                }

                                AssignmentSource::BCP(key) => {
                                    self.note_conflict(key);
                                    return Ok(self.report());
                                }

                                AssignmentSource::Assumption => {
                                    self.note_conflict(ClauseKey::OriginalUnit(assumption));
                                    return Ok(self.report());
                                }
                            }
                        }

                        Err(err::ErrorKind::FundamentalConflict) => {
                            return Ok(self.report());
                        }

                        Err(err::ErrorKind::AssumptionConflict(literal)) => {
                            self.state =
                                ContextState::Unsatisfiable(ClauseKey::AdditionUnit(literal));
                            return Ok(self.report());
                        }

                        Err(e) => {
                            log::info!("Failed to assert assumption: {e:?}");
                            panic!("! Unexpected error when asserting assumptions");
                        }
                    };
                }

                self.state = ContextState::Solving;
            }
        }

        let timer = std::time::Instant::now();

        'solve_loop: loop {
            self.counters.total_iterations += 1;
            log::trace!("Iteration {}", self.counters.total_iterations);

            self.counters.time = timer.elapsed();
            let time_limit = self.config.time_limit.value;
            if !time_limit.is_zero() && self.counters.time > time_limit {
                return Ok(self.report());
            }

            if self.check_callback_terminate_solve() {
                break 'solve_loop;
            }

            match self.apply_consequences()? {
                // Non-conflict variants. These variants break or continue the solve loop.
                ApplyConsequencesOk::FundamentalConflict => break 'solve_loop,

                ApplyConsequencesOk::Exhausted => {
                    //
                    match self.make_decision() {
                        DecisionOk::Literal(decision) => {
                            self.atom_db.push_fresh_decision(decision);
                            let level = self.atom_db.current_level();
                            log::info!("Decided on {decision} at level {level}");

                            match self.value_and_queue(decision, QPosition::Back, level) {
                                AtomValue::Different => return Err(ErrorKind::ValuationConflict),
                                _ => {}
                            }

                            continue 'solve_loop;
                        }
                        DecisionOk::Exhausted => break 'solve_loop,
                    }
                }

                // Conflict variants. These continue to the remaining contents of a loop.
                ApplyConsequencesOk::UnitClause { literal } => {
                    let q_result = self.value_and_queue(
                        literal,
                        QPosition::Front,
                        self.atom_db.current_level(),
                    );

                    match q_result {
                        AtomValue::NotSet | AtomValue::Same => {}

                        AtomValue::Different => {
                            self.note_conflict(ClauseKey::AdditionUnit(literal));

                            break 'solve_loop;
                        }
                    };
                }

                ApplyConsequencesOk::AssertingClause { key, literal } => {
                    self.clause_db.note_use(key);

                    let consequence = Assignment::from(literal, AssignmentSource::BCP(key));
                    unsafe { self.record_consequence(consequence) };
                    let level = self.atom_db.current_level();

                    match self.value_and_queue(literal, QPosition::Front, level) {
                        AtomValue::NotSet | AtomValue::Same => {}

                        AtomValue::Different => {
                            self.note_conflict(key);

                            break 'solve_loop;
                        }
                    };
                }
            }

            self.counters.total_conflicts += 1;
            self.counters.fresh_conflicts += 1;

            if self.luby_fresh_conflict_interrupt() {
                self.counters.luby.next();

                // TODO: Dispatch stats?

                if self.config.restarts.value {
                    self.backjump(self.atom_db.lowest_decision_level());
                    self.clause_db.refresh_heap();
                    self.counters.fresh_conflicts = 0;
                    self.counters.restarts += 1;
                };

                if self.restart_interrupt() {
                    self.clause_db
                        .reduce_by(self.clause_db.current_addition_count() / 2);
                }
            } else if self.conflict_total_interrupt() {
                self.clause_db
                    .reduce_by(self.clause_db.current_addition_count() / 2)?;
            }
        }

        Ok(self.report())
    }
}
