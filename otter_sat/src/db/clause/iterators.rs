use crate::{
    db::keys::ClauseKey,
    structures::{
        clause::{CClause, Clause},
        literal::CLiteral,
    },
};

use super::ClauseDB;

impl ClauseDB {
    /// An iterator over all original unit clauses, given as [CLiteral]s.
    pub fn all_original_unit_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, CLiteral)> + use<'_> {
        self.unit_original.values().flat_map(|c| {
            c.clause()
                .literals()
                .last()
                .map(|literal| (ClauseKey::OriginalUnit(literal), literal))
        })
    }

    /// An iterator over all addition unit clauses, given as [CLiteral]s.
    pub fn all_addition_unit_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, CLiteral)> + use<'_> {
        self.unit_addition.values().flat_map(|c| {
            c.clause()
                .literals()
                .last()
                .map(|literal| (ClauseKey::AdditionUnit(literal), literal))
        })
    }

    /// An iterator over all unit clauses, given as [CLiteral]s.
    pub fn all_unit_clauses(&self) -> impl Iterator<Item = (ClauseKey, CLiteral)> + use<'_> {
        self.all_original_unit_clauses()
            .chain(self.all_addition_unit_clauses())
    }

    /// An iterator over all original binary clauses.
    pub fn all_original_binary_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.binary_original.iter().map(|c| (*c.key(), c.clause()))
    }

    /// An iterator over all addition binary clauses.
    pub fn all_addition_binary_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.binary_addition.iter().map(|c| (*c.key(), c.clause()))
    }

    /// An iterator over all addition binary clauses.
    pub fn all_binary_clauses(&self) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.all_original_binary_clauses()
            .chain(self.all_addition_binary_clauses())
    }

    /// An iterator over all original binary clauses.
    pub fn all_original_long_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.original.iter().map(|c| (*c.key(), c.clause()))
    }

    /// An iterator over all addition binary clauses.
    pub fn all_addition_long_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.addition
            .iter()
            .flat_map(|c| c.as_ref().map(|db_c| (*db_c.key(), db_c.clause())))
    }

    /// An iterator over all addition binary clauses.
    pub fn all_active_addition_long_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.addition.iter().flat_map(|c| match c {
            Some(db_c) => match db_c.is_active() {
                true => Some((*db_c.key(), db_c.clause())),
                false => None,
            },
            None => None,
        })
    }

    /// An iterator over all addition binary clauses.
    pub fn all_long_clauses(&self) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.all_original_long_clauses()
            .chain(self.all_addition_long_clauses())
    }

    /// An iterator over all non-unit clauses, given as [impl Clause]s.
    pub fn all_nonunit_clauses(&self) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.all_binary_clauses().chain(self.all_long_clauses())
    }

    /// An iterator over all active non-unit clauses, given as [impl Clause]s.
    pub fn all_active_nonunit_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.all_binary_clauses()
            .chain(self.all_original_long_clauses())
            .chain(self.all_active_addition_long_clauses())
    }
}
