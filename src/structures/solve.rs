use crate::structures::{
    binary_resolution, Clause, ClauseId, ClauseSource, ClauseVec, Formula, ImplicationEdge,
    ImplicationGraph, ImplicationSource, Level, LevelIndex, Literal, LiteralError, LiteralSource,
    StoredClause, Valuation, ValuationError, ValuationOk, ValuationVec, Variable, VariableId,
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
    Hek,
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
                _ => the_solve.add_clause(as_vec, ClauseSource::Formula),
            }
        });

        the_solve
    }
}

// SAT related things
impl Solve<'_> {
    pub fn is_unsat_on(&self, valuation: &ValuationVec) -> bool {
        self.clauses
            .iter()
            .any(|stored_clause| stored_clause.clause().is_unsat_on(valuation))
    }

    pub fn is_sat_on(&self, valuation: &ValuationVec) -> bool {
        self.clauses
            .iter()
            .all(|stored_clause| stored_clause.clause().is_sat_on(valuation))
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
            if let Some(the_unset) = stored_clause.clause().collect_choices(valuation) {
                if the_unset.is_empty() {
                    if self.current_level().index() > 0
                        && stored_clause
                            .clause()
                            .literals()
                            .any(|lit| lit.v_id == self.current_level().get_choice().unwrap().v_id)
                    {
                        status.choice_conflicts.push((
                            stored_clause.id(),
                            self.current_level().get_choice().unwrap(),
                        ));
                    } else {
                        status.unsat.push(stored_clause.id());
                    }
                } else if the_unset.len() == 1 {
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
            clause.literals().for_each(|literal| {
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
            .find(|&v| self.valuation.of_v_id(v.id()).is_ok_and(|p| p.is_none()))
            .map(|found| found.id())
    }

    pub fn find_stored_clause(&self, id: ClauseId) -> Option<&StoredClause> {
        self.clauses
            .iter()
            .find(|stored_clause| stored_clause.id() == id)
    }
}

impl<'borrow, 'solve> Solve<'solve> {
    pub fn add_clause(&'borrow mut self, clause: impl Clause, source: ClauseSource) {
        let clause_as_vec = clause.as_vec();
        match clause_as_vec.len() {
            0 => panic!("Attempt to add an empty clause"),
            1 => panic!("Attempt to add an single literal clause"),
            _ => {
                let clause = StoredClause::new_from(Solve::fresh_clause_id(), &clause, source);
                for literal in clause.clause().literals() {
                    self.variables[literal.v_id].note_occurence(clause.id())
                }
                self.clauses.push(clause);
            }
        }
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
                        self.variables[literal.v_id].set_decision_level(new_level_index);
                        log::debug!("+Set choice: {literal}");
                    }
                    LiteralSource::Assumption | LiteralSource::Deduced => {
                        self.variables[literal.v_id].set_decision_level(0);
                        self.top_level_mut().record_literal(literal, source);
                        self.graph.add_literal(literal, 0, false);
                        log::debug!("+Set assumption/deduction: {literal}");
                    }
                    LiteralSource::HobsonChoice => {
                        self.variables[literal.v_id].set_decision_level(0);
                        self.top_level_mut().record_literal(literal, source);
                        self.graph.add_literal(literal, 0, false);
                        log::debug!("+Set hobson choice: {literal}");
                    }
                    LiteralSource::StoredClause(clause_id) => {
                        let current_level = self.current_level().index();
                        self.variables[literal.v_id].set_decision_level(current_level);
                        self.current_level_mut().record_literal(literal, source);

                        let literals = self
                            .clauses
                            .iter()
                            .find(|clause| clause.id() == clause_id)
                            .unwrap()
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
                        let current_level = self.current_level().index();
                        self.variables[literal.v_id].set_decision_level(current_level);
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
                    LiteralSource::Deduced => {
                        panic!("Attempt to deduce the flip of {}", literal.v_id);
                    }
                    _ => {
                        log::error!("Attempting to flip {} via {:?}", literal, source);
                        panic!("Attempting to flip the valuation")
                    }
                }
            }
        }
    }

    pub fn level_choice(&'borrow self, index: LevelIndex) -> Literal {
        self.levels[index].get_choice().expect("No choice at level")
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
    /// Simple analysis performs resolution on any clause used to obtain a conflict literal at the current decision level.
    fn simple_analysis_one(&mut self, conflict_clause_id: ClauseId) -> Option<ClauseVec> {
        let the_conflict_clause = self
            .find_stored_clause(conflict_clause_id)
            .expect("Hek")
            .clause()
            .as_vec();

        let mut the_resolved_clause = the_conflict_clause;

        'resolution_loop: loop {
            log::trace!("Analysis clause: {}", the_resolved_clause.as_string());
            // the current choice will never be a resolution literal, as these are those literals in the clause which are the result of propagation
            let resolution_literals = self
                .graph
                .resolution_candidates_at_level(&the_resolved_clause, self.current_level().index())
                .collect::<BTreeSet<_>>();

            match resolution_literals.is_empty() {
                true => {
                    return Some(the_resolved_clause);
                }
                false => {
                    let (clause_id, resolution_literal) =
                        resolution_literals.first().expect("No resolution literal");

                    let resolution_clause = self
                        .find_stored_clause(*clause_id)
                        .expect("Unable to find clause");

                    the_resolved_clause = binary_resolution(
                        &the_resolved_clause.as_vec(),
                        &resolution_clause.clause().as_vec(),
                        resolution_literal.v_id,
                    )
                    .expect("Resolution failed")
                    .as_vec();

                    continue 'resolution_loop;
                }
            }
        }
    }

    fn simple_analysis_two(&mut self, conflict_clause_id: ClauseId) -> Option<ClauseVec> {
        /*
        Unsafe for the moment.

        At issue is temporarily updating the implication graph to include the conflict clause implying falsum and then examining the conflcit clause.
        For, ideally a conflict clause is only borrowed from the store of clauses, and this means either retreiving for the stored twice, or dereferencing the borrow so it can be used while mutably borrowing the solve to update the graph.
        As retreiving the stored clause is a basic find operation, unsafely dereferencing the borrow is preferred.
         */
        unsafe {
            let the_conflict_clause =
                self.find_stored_clause(conflict_clause_id).expect("Hek") as *const StoredClause;

            let conflict_decision_level = self
                .decision_levels_of(the_conflict_clause.as_ref()?.clause())
                .max()
                .expect("No clause decision level");

            let mut the_resolved_clause = the_conflict_clause.as_ref()?.clause().as_vec();
            let the_conflict_level_choice = self.level_choice(conflict_decision_level);

            let the_immediate_domiator = self
                .graph
                .immediate_dominators(the_resolved_clause.literals(), the_conflict_level_choice)
                .expect("No immediate dominator");

            for literal in the_conflict_clause.as_ref()?.literals() {
                let mut paths = self
                    .graph
                    .paths_between(the_immediate_domiator, literal.negate());
                match paths.next() {
                    None => continue,
                    Some(path) => {
                        let mut path_clause_ids = self.graph.connecting_clauses(path.iter());
                        path_clause_ids.reverse();
                        for clause_id in path_clause_ids {
                            let path_clause = &self
                                .find_stored_clause(clause_id)
                                .expect("Failed to find clause")
                                .clause();
                            if let Some(shared_literal) =
                                path_clause.literals().find(|path_literal| {
                                    the_resolved_clause.contains(&path_literal.negate())
                                })
                            {
                                the_resolved_clause = binary_resolution(
                                    &the_resolved_clause,
                                    &path_clause.as_vec(),
                                    shared_literal.v_id,
                                )
                                .expect("Resolution failed")
                                .to_vec();
                            }
                        }
                    }
                }
            }

            return Some(the_resolved_clause);
        }
    }

    pub fn analyse_conflict(
        &mut self,
        clause_id: ClauseId,
        literal: Option<Literal>,
    ) -> Result<SolveOk, SolveError> {
        // match self.simple_analysis_one(clause_id) {
        match self.simple_analysis_two(clause_id) {
            Some(clause) => {
                match clause.len() {
                    0 => panic!("Empty clause from analysis"),
                    1 => {
                        let the_literal = *clause.first().unwrap();
                        return Ok(SolveOk::Deduction(the_literal));
                    }
                    _ => {
                        // the relevant backtrack level is either 0 is analysis is being performed at 0 or the first decision level in the resolution clause prior to the current level.
                        // For, if a prior level does *not* appear in the resolution clause then the level provided no relevant information.
                        let backtrack_level = self
                            .decision_levels_of(&clause)
                            .filter(|level| *level != self.current_level().index())
                            .max()
                            .unwrap_or(0);

                        self.add_clause(clause, ClauseSource::Resolution);
                        return Ok(SolveOk::AssertingClause(backtrack_level));
                    }
                }
            }
            None => panic!("Unexpected result from basic analysis"),
        }
    }

    pub fn backtrack_once(&mut self) -> Result<SolveOk, SolveError> {
        if self.current_level().index() == 0 {
            Err(SolveError::NoSolution)
        } else {
            let the_level = self.levels.pop().unwrap();
            log::warn!("Backtracking from {}", the_level.index());
            self.graph.remove_level(&the_level);
            for literal in the_level.literals() {
                self.refresh_literal(literal)
            }
            log::warn!("Backtracked from {}", the_level.index());
            Ok(SolveOk::Backtracked)
        }
    }
}

impl Solve<'_> {
    pub fn refresh_literal(&mut self, literal: Literal) {
        self.valuation[literal.v_id] = None;
        self.variables[literal.v_id].clear_decision_level();
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
}
