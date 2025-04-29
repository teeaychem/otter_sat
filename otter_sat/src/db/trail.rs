//! The trail of assignments.
//!
//! All assignments made with each assignment distinguished by the level at which it was made.
//!
//! The first level (level zero) contains proven literals, and each leel greater than zero begins with either an assumption or decision.
//! Every following assignment is on the same level unless the assignment is a decision or, if stacked assumptions is enabled, an assumption.

use crate::structures::literal::CLiteral;

use super::LevelIndex;

/// A structure to hold the trail.
#[derive(Default)]
pub struct Trail {
    /// Each assignment made, recorded as a literal, in order from first to last.
    pub assignments: Vec<CLiteral>,

    /// Indices to the initial assignment of each level.
    pub level_indices: Vec<usize>,

    /// The index of the first assignment whose consequences have not been examined.
    pub q_head: usize,

    /// The first level at which a decision is made.
    /// Zero, if no assumptions has been made.
    pub initial_decision_level: LevelIndex,
}
impl Trail {
    /// Writes a consequence to the top decision level.
    pub fn write_literal(&mut self, literal: CLiteral) {
        self.assignments.push(literal);
    }

    /// True if some assumption has been made, false otherwise.
    pub fn assumption_is_made(&self) -> bool {
        self.initial_decision_level > 0
    }

    /// Returns the lowest decision level.
    /// As a decision level always has some index greater than zero, a return value of zero indicates no decision has been made.
    pub fn lowest_decision_level(&self) -> LevelIndex {
        self.initial_decision_level
    }

    /// The assignments made at the (current) top level, in order of assignment.
    pub fn top_level_assignments(&self) -> &[CLiteral] {
        if let Some(&level_start) = self.level_indices.last() {
            &self.assignments[level_start..]
        } else {
            &[]
        }
    }

    /// A count of how many decisions have been made.
    /// That is, the count of only those levels containing decisions (as opposed to assumptions).
    ///
    /// In other words, a count of how many decisions have been made.
    pub fn decision_count(&self) -> LevelIndex {
        (self.level_indices.len() as LevelIndex) - self.initial_decision_level
    }

    /// Returns true if some decision is active, false otherwise (regardless of whether an assumption has been made).
    pub fn decision_is_made(&self) -> bool {
        self.decision_count() > 0
    }

    /// The current level.
    pub fn level(&self) -> LevelIndex {
        self.level_indices.len() as LevelIndex
    }

    /// Removes the top level, if it exists.
    ///
    /// # Soundness
    /// Does not clear the *valuation* of the decision.
    pub fn forget_top_level(&mut self) -> Vec<CLiteral> {
        if let Some(top_start) = self.level_indices.pop() {
            self.assignments.split_off(top_start)
        } else {
            Vec::default()
        }
    }

    /// Takes the current list of assignments, leaving the default assignment container, until the list is restored.
    /// To be used in conjunction with [Trail::restore_assignments].
    pub fn take_assignments(&mut self) -> Vec<CLiteral> {
        std::mem::take(&mut self.assignments)
    }

    /// Sets the current lists of assignments to `assignments`.
    /// To be used in conjunction with [Trail::take_assignments].
    pub fn restore_assignments(&mut self, assignents: Vec<CLiteral>) {
        self.assignments = assignents;
    }

    /// Removes levels above the given level index, if they exist.
    ///
    /// # Soundness
    /// Does not clear the *valuation* of the decision.
    pub fn clear_assignments_above(&mut self, level: LevelIndex) -> Vec<CLiteral> {
        // level_indices stores with zero-indexing.
        // So, for example, the first assignment is accessed by assignments[level_indices[0]].
        // This means, in particular, that all assignments made after level i can be cleared by clearing any assignment at and after assignments[level_indices[0]].
        // And, as a corollary, that this method can not be used to clear any assignments at level zero.
        if let Some(&level_start) = self.level_indices.get(level as usize) {
            self.level_indices.split_off(level as usize);
            self.assignments.split_off(level_start)
        } else {
            Vec::default()
        }
    }
}
