/*!
Applies queued consequences.

For an overview of [apply_consequences](GenericContext::apply_consequences) within a solve, see the documentation of the [solve procedure](crate::procedures::solve).

Roughly, apply_consequences implements an instance of the operator which:

- Returns *unsatisfiable*, if it is not possible to apply the consequence relation.
- Returns *satisfiable*, if the formula entails itself and the valuation is complete.
- Makes a decision, if the formula entails itself and the valuation is partial.
- Backjumps to a different valuation, if the formula entails some formula with an additional clause.

- A return of *unsatisfiable* is represented as a [FundamentalConflict](ApplyConsequencesOk::FundamentalConflict).
- A return of a new clause is represented with a [key](crate::db::ClauseKey) to the clause, and an asserted literal.
- No change is represented by a return of [Exhausted](ApplyConsequencesOk::Exhausted).
  + It is up to a caller of apply_consequences to note whether the background valuation is complete.

The following invariant is upheld:
<div class="warning">
apply_consequences returns the same formula only if there are no further consequences to apply.
</div>

# Overview

apply_consequences

At a high level [apply_consequences](GenericContext::apply_consequences) sequences a handful of more basic procedures in a loop:
- Take a queued consequence.
- Apply boolean constraint propagation with respect to the consequence.
- If no conflict is found, continue.
- Otherwise, perform conflict analysis and break.

These procedures are sequenced as a single procedure as the procedure may loop until inconsistency of the formula is established, a consistent valuation is found, or some decision needs to be made in order to progress.
Though, in practice [apply_consequences](GenericContext::apply_consequences) returns at the first conflict found.
This is to allow for further actions to be taken due to a conflict having been found.

```rust,ignore
while let Some((literal, _)) = self.consequence_q.front() {
    match self.bcp(literal) {
        Ok(()) => self.consequence_q.pop_front(), // continue applying consequences
        Err(err::BCP::Conflict(key)) => {
            if !self.literal_db.decision_made() {
                return Ok(FundamentalConflict);
            }

            match self.conflict_analysis(&key)? {
                // Analysis is only called when some decision has been made.
                analysis::Ok::FundamentalConflict => !,

                analysis::Ok::MissedPropagation { key, literal } => {
                    ... // return and complete the missed propagation
                    continue 'application;
                }

                analysis::Ok::UnitClause { key } => {
                    return Ok(UnitClause(key));
                }

                analysis::Ok::AssertingClause { key, literal } => {
                    return Ok(AssertingClause { key, literal });
                }
            }
        }
    }
}
Ok(Exhausted)
```

# Missed propagations

In some situations the opportunity to propagate a consequence may be 'missed' --- conflict analysis may return a clause already present in the clause database.
In this case, the clause returned was asserting at some prior decision level, and in turn the clause could have been used for propagation.

Missed propagations are supported as these do not *necessarily* entail an unsound solve procedure.
For:

- If the solve returns the formula is unsatisfiable then the propagations *observed* are sufficient to force some atom to be valued both true and false, and any missed propagations are not required.
- If the solve returns the formula is satisfiable, things are a little more difficult.
  Still, a satisfiable formula is satisfiable so long all original clause propagations are made.
  Or, so long as all propagations with respect to each value set in the final valuation have been made.

In order to maintain the invariant that [apply_consequences](GenericContext::apply_consequences) returns the same formula only if there are no further consequences to apply, missed propagations are returned to and their consequences applied *within* an instance.

Still, missed conflicts may conflict with other invariants.
For example, if all propagations via watched literals occurr prior to making a new decision and a watch is always given to a satisfied literal or a literal whose atom has no valuation, no propagation will be missed (as every asserting clause will be identified).
So, caution should be taken to avoid overlooking a failed invariant.
*/

use crate::{
    context::{ContextState, GenericContext},
    db::ClauseKey,
    procedures::analysis,
    structures::{
        consequence::{self, Consequence},
        literal::CLiteral,
    },
    types::err::{self, ErrorKind},
};

/// Ok results of [apply_consequences](GenericContext::apply_consequences).
pub enum ApplyConsequencesOk {
    /// A conflict was found, and so the formula is unsatisfiable.
    FundamentalConflict,

    /// A unit clause was derived from some conflict.
    UnitClause {
        /// The literal of the clause.
        literal: CLiteral,
    },

    /// A non-unit asserting clause was derived from some conflict.
    AssertingClause {
        /// The key to the clause.
        key: ClauseKey,

        /// The literal asserted by the clause.
        literal: CLiteral,
    },

    /// There were no (further) consequences to apply.
    Exhausted,
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Applies queued consequences.
    /// See [procedures::apply_consequences](crate::procedures::apply_consequences) for details.
    ///
    /// apply_consequences applies BCP to the consequence queue until either a conflict is found or the queue is exhausted.
    ///
    /// Queued consequences are removed from the queue only if BCP was successful.
    /// For, in the case of a conflict the consequence may remain, and otherwise will be removed from the queue during a backjump.
    pub fn apply_consequences(&mut self) -> Result<ApplyConsequencesOk, err::ErrorKind> {
        use crate::db::consequence_q::QPosition::{self};

        'application: loop {
            let Some((literal, _)) = self.consequence_q.front().cloned() else {
                return Ok(ApplyConsequencesOk::Exhausted);
            };

            match unsafe { self.bcp(literal) } {
                Ok(()) => {
                    self.consequence_q.pop_front();
                }
                Err(err::BCPError::Conflict(key)) => {
                    //
                    if !self.literal_db.decision_is_made() {
                        self.state = ContextState::Unsatisfiable(key);

                        let clause = unsafe { self.clause_db.get_unchecked(&key).unwrap().clone() };
                        self.clause_db.make_callback_unsatisfiable(&clause);

                        return Ok(ApplyConsequencesOk::FundamentalConflict);
                    }

                    match self.conflict_analysis(&key)? {
                        // Analysis is only called when some decision has been made.
                        analysis::ConflictAnalysisOk::FundamentalConflict => panic!("!"),

                        analysis::ConflictAnalysisOk::MissedPropagation {
                            key,
                            literal: asserted_literal,
                        } => {
                            let the_clause = unsafe { self.clause_db.get_unchecked(&key)? };

                            let index = self.non_chronological_backjump_level(the_clause)?;
                            self.backjump(index);

                            self.value_and_queue(
                                asserted_literal,
                                QPosition::Front,
                                self.literal_db.current_level(),
                            )?;

                            let consequence = Consequence::from(
                                asserted_literal,
                                consequence::ConsequenceSource::BCP(key),
                            );
                            self.record_consequence(consequence);

                            continue 'application;
                        }

                        analysis::ConflictAnalysisOk::UnitClause { literal: key } => {
                            return Ok(ApplyConsequencesOk::UnitClause { literal: key });
                        }

                        analysis::ConflictAnalysisOk::AssertingClause { key, literal } => {
                            return Ok(ApplyConsequencesOk::AssertingClause { key, literal });
                        }
                    }
                }
                Err(non_conflict_bcp_error) => return Err(ErrorKind::BCP(non_conflict_bcp_error)),
            }
        }
    }
}
