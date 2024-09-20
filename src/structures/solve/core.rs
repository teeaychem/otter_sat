use crate::structures::{
    Clause, ClauseId, ClauseSource, Formula, ImplicationGraph, Level, LevelIndex, Literal,
    LiteralError, LiteralSource, StoredClause, Valuation, ValuationVec, Variable, VariableId,
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
    Deduction(Literal),
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
}

impl Solve<'_> {
    pub fn from_formula(formula: &Formula) -> Solve {
        let mut the_solve = Solve {
            _formula: formula,
            variables: formula.vars().clone(),
            valuation: Vec::<Option<bool>>::new_for_variables(formula.vars().len()),
            levels: vec![Level::new(0)],
            clauses: vec![],
            graph: ImplicationGraph::new_for(formula),
        };

        let empty_val = the_solve.valuation.clone();

        formula.clauses().for_each(|formula_clause| {
            let as_vec = formula_clause.as_vec();
            match as_vec.len() {
                0 => panic!("Zero length clause from formula"),
                1 => {
                    match the_solve.set_literal(*as_vec.first().unwrap(), LiteralSource::Deduced) {
                        Ok(_) => (),
                        Err(e) => panic!("{e:?}"),
                    }
                }
                _ => the_solve.add_clause(as_vec, ClauseSource::Formula, &empty_val),
            }
        });

        the_solve
    }

    pub fn valuation_at(&self, level_index: LevelIndex) -> ValuationVec {
        let mut valuation = ValuationVec::new_for_variables(self.valuation.len());
        (0..=level_index).for_each(|i| {
            self.levels[i].literals().for_each(|l| {
                let _ = valuation.set_literal(l);
            })
        });
        valuation
    }

    pub fn valuation_before_choice_at(&self, level_index: LevelIndex) -> ValuationVec {
        match level_index {
            0 => self.valuation_at(0),
            _ => self.valuation_at(level_index - 1),
        }
    }

    pub fn is_unsat_on(&self, valuation: &ValuationVec) -> bool {
        self.clauses().any(|clause| clause.is_unsat_on(valuation))
    }

    pub fn is_sat_on(&self, valuation: &ValuationVec) -> bool {
        self.clauses().all(|clause| clause.is_sat_on(valuation))
    }

    /* ideally the check on an ignored unit is improved
     for example, with watched literals a clause can be ignored in advance if the ignored literal is watched and it's negation is not part of the given valuation.
    whether this makes sense to do…
    */
    pub fn examine_all_clauses_on<T: Valuation>(&self, valuation: &T) -> SolveStatus {
        let mut status = SolveStatus::new();
        for stored_clause in &self.clauses {
            // let collected_choices = stored_clause.clause().collect_choices(valuation);
            let collected_choices = stored_clause.watch_choices(valuation);
            if let Some(the_unset) = collected_choices {
                match the_unset.len() {
                    0 => {
                        if self.current_level().index() > 0
                            && stored_clause.clause().literals().any(|lit| {
                                lit.v_id == self.current_level().get_choice().unwrap().v_id
                            })
                        {
                            status.choice_conflicts.push((
                                stored_clause.id(),
                                self.current_level().get_choice().unwrap(),
                            ));
                        } else {
                            status.unsat.push(stored_clause.id());
                        }
                    }
                    1 => {
                        let the_pair: (ClauseId, Literal) =
                            (stored_clause.id(), *the_unset.first().unwrap());
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
                    }
                    _ => {
                        for literal in the_unset {
                            status.choices.insert(literal);
                        }
                    }
                }
            }
        }
        status
    }

    pub fn examine_level_clauses_on<T: Valuation>(&self, valuation: &T) -> SolveStatus {
        let mut status = SolveStatus::new();


        let literals = self.levels[self.current_level().index()].updated_watches();

        let clauses = literals.iter()
            .flat_map(|l| self.variables[l.v_id].occurrences())
            .collect::<BTreeSet<_>>();

        for stored_clause_id in clauses {
            let stored_clause = self.get_stored_clause(stored_clause_id);
            // let collected_choices = stored_clause.clause().collect_choices(valuation);
            let collected_choices = stored_clause.watch_choices(valuation);
            if let Some(the_unset) = collected_choices {
                match the_unset.len() {
                    0 => {
                        if self.current_level().index() > 0
                            && stored_clause.clause().literals().any(|lit| {
                                lit.v_id == self.current_level().get_choice().unwrap().v_id
                            })
                        {
                            status.choice_conflicts.push((
                                stored_clause.id(),
                                self.current_level().get_choice().unwrap(),
                            ));
                        } else {
                            status.unsat.push(stored_clause.id());
                        }
                    }
                    1 => {
                        let the_pair: (ClauseId, Literal) =
                            (stored_clause.id(), *the_unset.first().unwrap());
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
                    }
                    _ => {
                        for literal in the_unset {
                            status.choices.insert(literal);
                        }
                    }
                }
            }
        }
        status
    }

    pub fn clauses(&self) -> impl Iterator<Item = &impl Clause> {
        self.clauses
            .iter()
            .map(|stored_clause| stored_clause.clause())
    }

    pub fn fresh_clause_id() -> ClauseId {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, AtomicOrdering::Relaxed) as ClauseId
    }

    pub fn get_unassigned_id(&self, solve: &Solve) -> Option<VariableId> {
        solve
            .variables
            .iter()
            .find(|&v| self.valuation.of_v_id(v.id()).is_ok_and(|p| p.is_none()))
            .map(|found| found.id())
    }

    pub fn get_stored_clause(&self, id: ClauseId) -> &StoredClause {
        self.clauses
            .iter()
            .find(|stored_clause| stored_clause.id() == id)
            .expect("Unable to find clause with {id}")
    }

    pub fn var_by_id(&self, id: VariableId) -> Option<&Variable> {
        self.variables.get(id)
    }

    pub fn decision_levels_of<'borrow, 'clause: 'borrow>(
        &'borrow self,
        clause: &'clause impl Clause,
    ) -> impl Iterator<Item = LevelIndex> + 'borrow {
        clause
            .literals()
            .filter_map(move |literal| self.variables[literal.v_id].decision_level())
    }

    pub fn level_choice(&self, index: LevelIndex) -> Literal {
        self.levels[index]
            .get_choice()
            .expect("No choice at level {index}")
    }

    pub fn set_from_lists(&mut self, the_choices: (Vec<VariableId>, Vec<VariableId>)) {
        the_choices.0.iter().for_each(|&v_id| {
            let _ = self.set_literal(Literal::new(v_id, false), LiteralSource::HobsonChoice);
        });
        the_choices.1.iter().for_each(|&v_id| {
            let _ = self.set_literal(Literal::new(v_id, true), LiteralSource::HobsonChoice);
        });
    }

    /*
    If a clause is unsatisfiable due to a valuation which conflicts with each literal of the clause, then at least one such conflicting literal was set at the current level.
    This function returns some clause and mentioned literal from a list of unsatisfiable clauses.
     */
    pub fn select_unsat(&self, clauses: &[ClauseId]) -> Option<(ClauseId, Literal)> {
        if !clauses.is_empty() {
            let the_clause_id = *clauses.first().unwrap();
            let the_stored_clause = self.get_stored_clause(the_clause_id);
            log::warn!("Chose: {:?}", the_stored_clause.clause().as_string());
            let current_variables = self.current_level().variables().collect::<BTreeSet<_>>();
            let mut overlap = the_stored_clause
                .literals()
                .filter(|l| current_variables.contains(&l.v_id));
            let the_literal = overlap.next().expect("No overlap");
            Some((the_clause_id, the_literal))
        } else {
            None
        }
    }

    pub fn select_conflict(&self, clauses: &[(ClauseId, Literal)]) -> Option<(ClauseId, Literal)> {
        if !clauses.is_empty() {
            Some(clauses.first().unwrap()).cloned()
        } else {
            None
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
