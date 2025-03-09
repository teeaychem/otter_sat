/*!
A database of literal indexed things.

For the moment, this amounts to a stack of assignments.
These include decisions, assumptions, consequences of BCP, and so on.

The assignments are split into 'levels', with a decision or set of assumption marking the start of a level and the remaining assignments of the level are (observed) consequences of (perhaps all) previous assignments.

The 'top' level is the level of the most recent decision made.
- For example, after deciding 'p' is true and 'q' is false, the top decision level stores the decision to bind 'q' to false.

A bottom (or first) level exists only after some assumption or decision has been made.
And, so, in particular, observed consequences which do --- or are known to --- *not* rest on some decision are stored as unit clauses in the [clause database](crate::db::clause::ClauseDB).


# Implementation

The split of assignemnts into levels through marks is inspired by MiniSAT.

The primary motivation for using a single vector of assignments over, say, a vector of levels (as in earlier implementations), is (far) fewer allocations.

In addition, a single collection of assignments significantly simplies traversing the assignments.
*/

use crate::{
    config::Config,
    db::LevelIndex,
    structures::{
        consequence::{Assignment, AssignmentSource},
        literal::CLiteral,
    },
};

#[doc(hidden)]
pub mod config;
pub use config::LiteralDBConfig;

#[allow(dead_code)]
/// A struct abstracting over assumption/decision levels.
pub struct LiteralDB {
    /// Configuration of the literal database.
    pub config: LiteralDBConfig,

    /// The level of the initial decision during a solve.
    /// In other words, any level present *below* the limit contains assumptions.
    pub initial_decision_level: LevelIndex,

    /// A stack of levels.
    pub assignments: Vec<Assignment>,

    /// Indicies at which a new level begins.
    pub level_indicies: Vec<usize>,
}

impl LiteralDB {
    /// Pushes a fresh level to the top of the level stack with the given decision.
    /// ```rust,ignore
    /// self.literal_db.push_fresh_decision(chosen_literal);
    /// ```
    pub fn push_fresh_decision(&mut self, decision: CLiteral) {
        self.level_indicies.push(self.assignments.len());

        self.store_assignment(Assignment::from(decision, AssignmentSource::Decision));
    }

    /// Pushes a fresh level to the top of the level stack with the given assumption.
    pub fn push_fresh_assumption(&mut self, assumption: CLiteral) {
        self.initial_decision_level += 1;
        self.level_indicies.push(self.assignments.len());

        self.store_assignment(Assignment::from(assumption, AssignmentSource::Assumption));
    }
}

impl LiteralDB {
    /// True if some assumption has been made, false otherwise.
    pub fn assumption_is_made(&self) -> bool {
        self.initial_decision_level > 0
    }

    /// A new [LiteralDB] with local configuration options derived from `config`.
    pub fn new(config: &Config) -> Self {
        LiteralDB {
            config: config.literal_db.clone(),
            initial_decision_level: 0,
            assignments: Vec::default(),
            level_indicies: Vec::default(),
        }
    }

    /// Returns the lowest decision level.
    /// Zero, if no assumptions has been made, otherwise some higher level.
    pub fn lowest_decision_level(&self) -> LevelIndex {
        self.initial_decision_level
    }

    /// The assignments made at `level`, in order of assignment.
    ///
    /// # Safety
    /// No check is made to ensure the relevant number of assignments have been made.
    pub unsafe fn assignments_at_unchecked(&self, level: LevelIndex) -> &[Assignment] {
        let level_start = *self.level_indicies.get_unchecked(level as usize);

        let level_end = if ((level + 1) as usize) < self.level_indicies.len() {
            *self.level_indicies.get_unchecked((level + 1) as usize)
        } else {
            self.assignments.len()
        };

        &self.assignments[level_start..level_end]
    }

    /// The assignments made at `level`, in order of assignment.
    pub fn assignments_at_and_after(&self, level: LevelIndex) -> &[Assignment] {
        if let Some(&level_start) = self.level_indicies.get(level as usize) {
            &self.assignments[level_start..]
        } else {
            &[]
        }
    }

    /// The assignments made at the (current) top level, in order of assignment.
    pub fn top_assignments_unchecked(&self) -> &[Assignment] {
        if let Some(&level_start) = self.level_indicies.last() {
            &self.assignments[level_start..]
        } else {
            &[]
        }
    }

    /// Removes the top decision level.
    ///
    /// # Soundness
    /// Does not clear the *valuation* of the decision.
    pub fn forget_top_level(&mut self) -> Vec<Assignment> {
        if let Some(top_start) = self.level_indicies.pop() {
            self.assignments.split_off(top_start)
        } else {
            Vec::default()
        }
    }

    /// Removes the top decision level.
    ///
    /// # Soundness
    /// Does not clear the *valuation* of the decision.
    pub fn forget_at_and_after(&mut self, level: LevelIndex) -> Vec<Assignment> {
        if let Some(&level_start) = self.level_indicies.get(level as usize) {
            self.level_indicies.split_off(level as usize);
            self.assignments.split_off(level_start)
        } else {
            Vec::default()
        }
    }

    /// A count of how many decisions have been made.
    /// That is, the count of only those levels containing decisions (as opposed to assumptions).
    ///
    /// In other words, a count of how many decisions have been made.
    pub fn decision_count(&self) -> LevelIndex {
        (self.level_indicies.len() as LevelIndex) - self.initial_decision_level
    }

    /// Returns true if some decision is active, false otherwise (regardless of whether an assumption has been made).
    pub fn decision_is_made(&self) -> bool {
        self.decision_count() > 0
    }

    /// The current level.
    pub fn current_level(&self) -> LevelIndex {
        self.level_indicies.len() as LevelIndex
    }
}

impl LiteralDB {
    /// Stores a consequence of the top decision level.
    pub fn store_assignment(&mut self, assignment: Assignment) {
        self.assignments.push(assignment);
    }
}
