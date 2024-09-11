use crate::structures::{
    Clause, ClauseId, Formula, ImplicationEdge, ImplicationGraph, ImplicationNode, Level, Literal,
    LiteralError, LiteralSource, Valuation, ValuationError, ValuationOk, ValuationVec, VariableId,
};
use std::result;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use std::collections::BTreeSet;

#[derive(Debug)]
pub struct Solve<'formula> {
    pub formula: &'formula Formula,
    pub valuation: Vec<Option<bool>>,
    pub levels: Vec<Level>,
    pub clauses: Vec<Clause>,
    pub graph: ImplicationGraph,
}

#[derive(Debug, PartialEq)]
pub enum SolveError {
    Literal(LiteralError),
    // Clause(ClauseError),
    ParseFailure,
    PrefaceLength,
    PrefaceFormat,
    Hek,
    OutOfBounds,
    SetIssue,
    UnsatClause(ClauseId),
    NoSolution,
}

impl Solve<'_> {
    pub fn from_formula(formula: &Formula) -> Solve {
        let valuation = Vec::<Option<bool>>::new_for_variables(formula.vars().len());
        let mut the_solve = Solve {
            formula,
            valuation,
            levels: vec![],
            clauses: vec![],
            graph: ImplicationGraph::new_for(formula),
        };
        let level_zero = Level::new(0, &the_solve);
        the_solve.levels.push(level_zero);
        the_solve
    }
}

// SAT related things
impl Solve<'_> {
    pub fn is_unsat_on(&self, valuation: &ValuationVec) -> bool {
        self.formula
            .clauses
            .iter()
            .any(|clause| clause.is_unsat_on(valuation))
    }

    pub fn is_sat_on(&self, valuation: &ValuationVec) -> bool {
        self.formula
            .clauses
            .iter()
            .all(|clause| clause.is_sat_on(valuation))
    }

    /* ideally the check on an ignored unit is improved
     for example, with watched literals a clause can be ignored in advance if the ignored literal is watched and it's negation is not part of the given valuation.
    whether this makes sense to do…
    */

    pub fn find_all_unset_on<T: Valuation>(
        &self,
        valuation: &T,
    ) -> Result<(BTreeSet<(ClauseId, Literal)>, BTreeSet<Literal>), SolveError> {
        let mut the_unit_set = BTreeSet::new();
        let mut the_choice_set = BTreeSet::new();
        for clause in &self.formula.clauses {
            if let Some(the_unset) = clause.collect_choices(valuation) {
                if the_unset.is_empty() {
                    return Err(SolveError::UnsatClause(clause.id));
                } else if the_unset.len() == 1 {
                    let the_pair: (ClauseId, Literal) = (clause.id, *the_unset.first().unwrap());
                    the_unit_set.insert(the_pair);
                    if the_choice_set.contains(&the_pair.1) {
                        the_choice_set.remove(&the_pair.1);
                    }
                } else {
                    for literal in the_unset {
                        the_choice_set.insert(literal);
                    }
                }
            }
        }
        Ok((the_unit_set, the_choice_set))
    }

    pub fn literals_of_polarity(&self, polarity: bool) -> impl Iterator<Item = Literal> {
        let mut literal_vec: Vec<Option<Literal>> = vec![None; self.formula.var_count()];
        self.formula.clauses.iter().for_each(|clause| {
            clause.literals.iter().for_each(|literal| {
                if literal.polarity == polarity {
                    literal_vec[literal.v_id] = Some(*literal)
                }
            })
        });

        literal_vec.into_iter().flatten()
    }

    pub fn fresh_clause_id() -> ClauseId {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, AtomicOrdering::Relaxed) as ClauseId
    }

    pub fn get_unassigned_id(&self, solve: &Solve) -> Option<VariableId> {
        solve
            .formula
            .vars()
            .iter()
            .find(|&v| self.valuation.of_v_id(v.id).is_ok_and(|p| p.is_none()))
            .map(|found| found.id)
    }
}

impl<'borrow, 'solve> Solve<'solve> {
    pub fn find_clause(&'borrow self, id: ClauseId) -> Option<&'solve Clause> {
        self.formula.clauses.iter().find(|c| c.id == id)
    }

    pub fn learn_as_clause(&'borrow mut self, literals: Vec<Literal>) {
        panic!("learn as clause");
        let clause = Clause {
            id: Solve::fresh_clause_id(),
            position: self.clauses.len(),
            literals,
        };
        self.clauses.push(clause)
    }

    /*
    Primary function for setting a literal during a solve
    Control branches on the current valuation and then the source of the literal.
    A few things may be updated:
    - The current valuation.
    - Records at the current level.
    - The implication graph.
     */
    pub fn set_literal(
        &'borrow mut self,
        literal: &Literal,
        source: LiteralSource,
    ) -> Result<(), SolveError> {
        match self.valuation.check_literal(*literal) {
            Ok(ValuationOk::Match) => {
                match source {
                    LiteralSource::Choice | LiteralSource::HobsonChoice => {
                        panic!("Attempting to set a made choice")
                    }
                    LiteralSource::Assumption => {
                        panic!("Attempting to set a made assumption")
                    }
                    LiteralSource::Clause(_) | LiteralSource::Conflict => {
                        self.current_level_mut().record_literal(literal, source);
                    }
                };
                Ok(())
            }
            Ok(ValuationOk::NotSet) => {
                let _ = self.valuation.set_literal(*literal);
                match source {
                    LiteralSource::Choice => {
                        self.add_fresh_level();
                        self.current_level_mut().record_literal(literal, source);
                        self.graph
                            .add_literal(*literal, self.current_level_index(), false);
                        log::debug!("+Set choice: {literal}");
                    }
                    LiteralSource::Assumption => {
                        self.top_level_mut().record_literal(literal, source);
                        self.graph.add_literal(*literal, 0, false);
                        log::debug!("+Set assumption: {literal}");
                    }
                    LiteralSource::HobsonChoice => {
                        self.top_level_mut().record_literal(literal, source);
                        self.graph.add_literal(*literal, 0, false);
                        log::debug!("+Set hobson choice: {literal}");
                    }
                    LiteralSource::Clause(clause_id) => {
                        self.current_level_mut().record_literal(literal, source);
                        self.graph.add_implication(
                            self.find_clause(clause_id).unwrap(),
                            *literal,
                            self.current_level_index(),
                            false,
                        );
                        log::debug!("+Set deduction: {literal}");
                    }
                    LiteralSource::Conflict => {
                        self.current_level_mut().record_literal(literal, source);
                        if self.current_level_index() != 0 {
                            self.graph.add_contradiction(
                                self.current_level().get_choice(),
                                *literal,
                                self.current_level_index(),
                            );
                        } else {
                            self.graph
                                .add_literal(*literal, self.current_level_index(), false);
                        }
                        log::debug!("+Set conflict: {literal}");
                    }
                };
                Ok(())
            }
            Err(ValuationError::Inconsistent) => match source {
                LiteralSource::Clause(c) => Err(SolveError::UnsatClause(c)),
                _ => panic!("unsat without a clause"),
            },
            Err(ValuationError::AlreadySet) => Err(SolveError::SetIssue),
        }
    }
}

impl std::fmt::Display for Solve<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = writeln!(f, "Valuation: {}", self.valuation.as_display_string(self));
        let _ = write!(f, "{}", self.formula);

        Ok(())
    }
}

impl Solve<'_> {
    pub fn analyse_conflict(&mut self, level: &Level, clause: ClauseId, literal: Literal) {
        let level_choice = level.get_choice();
        let the_clause = self
            .formula
            .clauses
            .iter()
            .find(|c| c.id == clause)
            .expect("Missing clause");

        let the_choice_index = self.graph.get_literal(level_choice);
        let conflict_index = self
            .graph
            .add_implication(the_clause, literal, level.index(), true);

        self.graph.dominators(the_choice_index, conflict_index);

        self.graph.remove_node(conflict_index);
        println!("Analysis complete");
    }
}
