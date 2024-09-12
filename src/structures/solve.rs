use petgraph::matrix_graph::Zero;
use petgraph::visit::EdgeRef;

use crate::clause::ClauseVec;
use crate::structures::{
    binary_resolution, Clause, ClauseId, ClauseSource, Formula, ImplicationEdge, ImplicationGraph,
    ImplicationSource, Level, LevelIndex, Literal, LiteralError, LiteralSource, StoredClause,
    Valuation, ValuationError, ValuationOk, ValuationVec, Variable, VariableId,
};

use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use std::collections::BTreeSet;

pub struct SolveStatus {
    pub choice_conflicts: Vec<(ClauseId, Literal)>,
    pub implications: Vec<(ClauseId, Literal)>,
    pub choices: BTreeSet<Literal>,
    pub unsat: Vec<ClauseId>,
}

impl SolveStatus {
    pub fn new() -> Self {
        SolveStatus {
            choice_conflicts: vec![],
            implications: vec![],
            choices: BTreeSet::new(),
            unsat: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Solve<'formula> {
    _formula: &'formula Formula,
    pub variables: Vec<Variable>,
    pub valuation: Vec<Option<bool>>,
    pub levels: Vec<Level>,
    pub clauses: Vec<StoredClause>,
    pub graph: ImplicationGraph,
}

#[derive(Debug, PartialEq)]
pub enum SolveOk {
    AssertingClause(LevelIndex),
    Backtracked,
}

#[derive(Debug, PartialEq)]
pub enum SolveError {
    Literal(LiteralError),
    // Clause(ClauseError),
    OutOfBounds,
    UnsatClause(ClauseId),
    Conflict(ClauseId, Literal),
    NoSolution,
    Hek,
}

impl Solve<'_> {
    pub fn from_formula(formula: &Formula) -> Solve {
        let valuation = Vec::<Option<bool>>::new_for_variables(formula.vars().len());
        let mut the_solve = Solve {
            _formula: formula,
            variables: formula.vars().clone(),
            valuation,
            levels: vec![],
            clauses: formula
                .clauses()
                .map(|formula_clause| {
                    StoredClause::new_from(
                        Solve::fresh_clause_id(),
                        formula_clause,
                        ClauseSource::Formula,
                    )
                })
                .collect(),
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
        self.clauses
            .iter()
            .any(|stored_clause| stored_clause.clause.is_unsat_on(valuation))
    }

    pub fn is_sat_on(&self, valuation: &ValuationVec) -> bool {
        self.clauses
            .iter()
            .all(|stored_clause| stored_clause.clause.is_sat_on(valuation))
        // self.formula
        //     .clauses()
        //     .all(|clause| clause.is_sat_on(valuation))
    }

    /* ideally the check on an ignored unit is improved
     for example, with watched literals a clause can be ignored in advance if the ignored literal is watched and it's negation is not part of the given valuation.
    whether this makes sense to do…
    */

    pub fn examine_clauses_on<T: Valuation>(&self, valuation: &T) -> SolveStatus {
        let mut status = SolveStatus::new();
        for stored_clause in &self.clauses {
            if let Some(the_unset) = stored_clause.clause.collect_choices(valuation) {
                if the_unset.is_empty() {
                    if self.current_level().index() > 0
                        && stored_clause
                            .clause
                            .iter()
                            .any(|lit| lit.v_id == self.current_level().get_choice().unwrap().v_id)
                    {
                        status
                            .choice_conflicts
                            .push((stored_clause.id, self.current_level().get_choice().unwrap()));
                    } else {
                        status.unsat.push(stored_clause.id);
                    }
                } else if the_unset.len() == 1 {
                    let the_pair: (ClauseId, Literal) =
                        (stored_clause.id, *the_unset.first().unwrap());
                    if self.current_level().index() > 0
                        && the_pair.1.v_id == self.current_level().get_choice().unwrap().v_id
                    {
                        status.choice_conflicts.push(the_pair)
                    } else {
                        status.implications.push(the_pair);
                    }
                    if status.choices.contains(&the_pair.1) {
                        status.choices.remove(&the_pair.1);
                    }
                } else {
                    for literal in the_unset {
                        status.choices.insert(literal);
                    }
                }
            }
        }
        status
    }

    pub fn literals_of_polarity(&self, polarity: bool) -> impl Iterator<Item = Literal> {
        let mut literal_vec: Vec<Option<Literal>> = vec![None; self.variables.len()];
        self.clauses.iter().for_each(|clause| {
            clause.clause.literals().for_each(|literal| {
                if literal.polarity == polarity {
                    literal_vec[literal.v_id] = Some(literal)
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
            .variables
            .iter()
            .find(|&v| self.valuation.of_v_id(v.id).is_ok_and(|p| p.is_none()))
            .map(|found| found.id)
    }
}

impl Solve<'_> {
    pub fn find_stored_clause(&self, id: ClauseId) -> Option<&StoredClause> {
        self.clauses
            .iter()
            .find(|stored_clause| stored_clause.id == id)
    }
}

impl<'borrow, 'solve> Solve<'solve> {
    pub fn learn_clause(&'borrow mut self, clause: impl Clause) {
        let clause = StoredClause {
            id: Solve::fresh_clause_id(),
            source: ClauseSource::Temp,
            clause: clause.to_vec(),
        };
        log::warn!(
            "Learnt clause: {} @ level {:?}",
            clause.to_string(),
            self.decision_level_of(&clause.clause)
        );
        self.clauses.push(clause);
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
        literal: Literal,
        source: LiteralSource,
    ) -> Result<(), SolveError> {
        match self.valuation.set_literal(literal) {
            Ok(()) => {
                match source {
                    LiteralSource::Choice => {
                        let new_level_index = self.add_fresh_level();
                        self.current_level_mut().record_literal(literal, source);
                        self.graph
                            .add_literal(literal, self.current_level().index(), false);
                        self.variables[literal.v_id].decision_level = Some(new_level_index);
                        log::debug!("+Set choice: {literal}");
                    }
                    LiteralSource::Assumption => {
                        self.variables[literal.v_id].decision_level = Some(0);
                        self.top_level_mut().record_literal(literal, source);
                        self.graph.add_literal(literal, 0, false);
                        log::debug!("+Set assumption: {literal}");
                    }
                    LiteralSource::HobsonChoice => {
                        self.variables[literal.v_id].decision_level = Some(0);
                        self.top_level_mut().record_literal(literal, source);
                        self.graph.add_literal(literal, 0, false);
                        log::debug!("+Set hobson choice: {literal}");
                    }
                    LiteralSource::StoredClause(clause_id) => {
                        self.variables[literal.v_id].decision_level =
                            Some(self.current_level().index());
                        self.current_level_mut().record_literal(literal, source);

                        let literals = self
                            .clauses
                            .iter()
                            .find(|clause| clause.id == clause_id)
                            .unwrap()
                            .clause
                            .literals()
                            .map(|l| l.negate());

                        self.graph.add_implication(
                            literals,
                            literal,
                            self.current_level().index(),
                            ImplicationSource::StoredClause(clause_id),
                        );

                        log::debug!("+Set deduction: {literal}");
                    }
                    LiteralSource::Conflict => {
                        self.variables[literal.v_id].decision_level =
                            Some(self.current_level().index());
                        self.current_level_mut().record_literal(literal, source);
                        if self.current_level().index() != 0 {
                            self.graph.add_contradiction(
                                self.current_level().get_choice().expect("No choice 0+"),
                                literal,
                                self.current_level().index(),
                            );
                        } else {
                            self.graph
                                .add_literal(literal, self.current_level().index(), false);
                        }
                        log::debug!("+Set conflict: {literal}");
                    }
                };
                Ok(())
            }
            Err(ValuationError::Match) => match source {
                LiteralSource::StoredClause(_) => {
                    // A literal may be implied by multiple clauses
                    Ok(())
                }
                _ => {
                    log::error!("Attempting to restate {} via {:?}", literal, source);
                    panic!("Attempting to restate the valuation")
                }
            },
            Err(ValuationError::Conflict) => {
                match source {
                    LiteralSource::StoredClause(id) => {
                        // A literal may be implied by multiple clauses
                        Err(SolveError::Conflict(id, literal))
                    }
                    _ => {
                        log::error!("Attempting to flip {} via {:?}", literal, source);
                        panic!("Attempting to flip the valuation")
                    }
                }
            }
        }
    }
}

impl std::fmt::Display for Solve<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = writeln!(f, "Valuation: {}", self.valuation.as_display_string(self));
        let _ = write!(f, "More to be added…");
        Ok(())
    }
}

impl Solve<'_> {
    fn simple_analysis(&mut self, conflict_clause_id: ClauseId) -> Result<SolveOk, SolveError> {
        let the_conflict_clause = &self
            .find_stored_clause(conflict_clause_id)
            .expect("Hek")
            .clause;
        let conflict_decision_level = self
            .decision_level_of(the_conflict_clause)
            .expect("No clause decision level");

        let mut the_resolved_clause = the_conflict_clause.as_vec();

        'resolution_loop: loop {
            log::trace!("Analysis clause: {}", the_resolved_clause.as_string());
            // the current choice will never be a resolution literal, as these are those literals in the clause which are the result of propagation
            let resolution_literals = self
                .graph
                .naive_resolution_candidates(&the_resolved_clause, conflict_decision_level)
                .collect::<BTreeSet<_>>();

            match resolution_literals.is_empty() {
                true => {
                    let decision_level = self
                        .decision_level_of(&the_resolved_clause)
                        .expect("Learnt clause without decision level");
                    self.learn_clause(the_resolved_clause);

                    return Ok(SolveOk::AssertingClause(decision_level));
                }
                false => {
                    let (clause_id, resolution_literal) =
                        resolution_literals.first().expect("No resolution literal");

                    let resolution_clause = self
                        .find_stored_clause(*clause_id)
                        .expect("Unable to find clause");

                    the_resolved_clause = binary_resolution(
                        &the_resolved_clause.as_vec(),
                        &resolution_clause.clause,
                        resolution_literal.v_id,
                    )
                    .expect("Resolution failed")
                    .as_vec();

                    continue 'resolution_loop;
                }
            }
        }
    }

    pub fn analyse_conflict(
        &mut self,
        clause_id: ClauseId,
        literal: Option<Literal>,
    ) -> Result<SolveOk, SolveError> {
        self.simple_analysis(clause_id)
    }

    pub fn backtrack(&mut self) -> Result<SolveOk, SolveError> {
        if self.current_level().index() == 0 {
            Err(SolveError::NoSolution)
        } else {
            let the_level = self.levels.pop().unwrap();
            log::warn!("Backtracking from {}", the_level.index());
            self.graph.remove_level(&the_level);
            for literal in the_level.literals() {
                self.refresh_literal(literal)
            }
            Ok(SolveOk::Backtracked)
        }
    }
}

impl Solve<'_> {
    pub fn refresh_literal(&mut self, literal: Literal) {
        self.valuation[literal.v_id] = None;
        self.variables[literal.v_id].decision_level = None;
    }

    pub fn var_by_id(&self, id: VariableId) -> Option<&Variable> {
        self.variables.get(id as usize)
    }

    pub fn decision_level_of(&self, clause: &impl Clause) -> Option<usize> {
        clause
            .literals()
            .filter_map(|literal| self.variables[literal.v_id].decision_level)
            .max()
    }
}
