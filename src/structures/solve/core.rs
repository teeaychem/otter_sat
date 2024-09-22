use crate::structures::{
    Clause, ClauseId, ClauseSource, Formula, ImplicationGraph, Level, LevelIndex, Literal,
    LiteralError, LiteralSource, StoredClause, Valuation, ValuationVec, Variable, VariableId,
};

use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use std::collections::BTreeSet;

pub struct SolveStatus {
    pub implications: Vec<(ClauseId, Literal)>,
    pub unsat: Vec<ClauseId>,
}

impl SolveStatus {
    pub fn new() -> Self {
        SolveStatus {
            implications: vec![],
            unsat: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Solve<'formula> {
    _formula: &'formula Formula,
    pub conflicts: usize,
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
            conflicts: 0,
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

    pub fn select_unsat(&self, clauses: &[ClauseId]) -> Option<ClauseId> {
        clauses.first().copied()
    }
}

impl Solve<'_> {
    fn examine_clauses<'sc>(
        &self,
        val: &impl Valuation,
        clauses: impl Iterator<Item = &'sc StoredClause>,
    ) -> SolveStatus {
        let mut status = SolveStatus::new();

        for stored_clause in clauses {
            if let Some(the_unset) = stored_clause.watch_choices(val) {
                match the_unset.len() {
                    0 => {
                        status.unsat.push(stored_clause.id());
                    }
                    1 => {
                        let the_pair: (ClauseId, Literal) =
                            (stored_clause.id(), *the_unset.first().unwrap());
                        status.implications.push(the_pair);
                    }
                    _ => {}
                }
            }
        }
        status
    }

    /* ideally the check on an ignored unit is improved
     for example, with watched literals a clause can be ignored in advance if the ignored literal is watched and it's negation is not part of the given valuation.
    whether this makes sense to do…
    */
    pub fn examine_all_clauses_on(&self, valuation: &impl Valuation) -> SolveStatus {
        self.examine_clauses(valuation, &mut self.clauses.iter())
    }

    pub fn examine_level_clauses_on<T: Valuation>(&self, valuation: &T) -> SolveStatus {
        let literals = self.levels[self.current_level().index()].updated_watches();

        let clauses = literals
            .iter()
            .flat_map(|l| self.variables[l.v_id].occurrences())
            .map(|clause_id| {
                let x = self.get_stored_clause(clause_id);
                x
            })
            .collect::<BTreeSet<_>>()
            .into_iter();

        self.examine_clauses(valuation, clauses)
    }
}

impl Solve<'_> {
    pub fn process_unsat(&mut self, clause_ids: &[ClauseId]) {
        for &conflict in clause_ids {
            self.conflicts += 1;
            if self.conflicts % 256 == 0 {
                for variable in &mut self.variables {
                    variable.activity /= 2.0
                }
            }

            unsafe {
                let the_stored_conflict = self.get_stored_clause(conflict) as *const StoredClause;
                if let Some(cls) = the_stored_conflict.as_ref() {
                    for literal in cls.clause().variables() {
                        self.variables[literal].activity += 1.0;
                    }
                }
            }
        }
    }

    pub fn most_active_none(&self, val: &impl Valuation) -> Option<VariableId> {
        val.to_vec()
            .into_iter()
            .enumerate()
            .filter(|(_, v)| v.is_none())
            .map(|(i, _)| (i, self.variables[i].activity))
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(a, _)| a)
    }
}

impl std::fmt::Display for Solve<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = writeln!(f, "Valuation: {}", self.valuation.as_display_string(self));
        let _ = write!(f, "More to be added…");
        Ok(())
    }
}
