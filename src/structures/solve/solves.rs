use crate::procedures::hobson_choices;
use crate::structures::solve::{Solve, SolveError, SolveOk};
use crate::structures::{ClauseStatus, Literal, LiteralSource, StoredClause, Valuation};
use std::rc::Rc;

pub enum SolveResult {
    Satisfiable,
    Unsatisfiable,
    Unknown,
}

pub struct SolveStats {
    pub total_time: std::time::Duration,
    pub examination_time: std::time::Duration,
    pub implication_time: std::time::Duration,
    pub unsat_time: std::time::Duration,
    pub reduction_time: std::time::Duration,
    pub choice_time: std::time::Duration,
    pub iterations: usize,
    pub conflicts: usize,
}

impl SolveStats {
    pub fn new() -> Self {
        SolveStats {
            total_time: std::time::Duration::new(0, 0),
            examination_time: std::time::Duration::new(0, 0),
            implication_time: std::time::Duration::new(0, 0),
            unsat_time: std::time::Duration::new(0, 0),
            reduction_time: std::time::Duration::new(0, 0),
            choice_time: std::time::Duration::new(0, 0),
            iterations: 0,
            conflicts: 0,
        }
    }
}

impl std::fmt::Display for SolveStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "c STATS")?;
        writeln!(f, "c ITERATIONS: {}", self.iterations)?;
        writeln!(f, "c CONFLICTS: {}", self.conflicts)?;
        writeln!(f, "c TIME: {:.2?}", self.total_time)?;
        writeln!(f, "c \tEXAMINATION: {:.2?}", self.examination_time)?;
        writeln!(f, "c \tIMPLICATION: {:.2?}", self.implication_time)?;
        writeln!(f, "c \tUNSAT: {:.2?}", self.unsat_time)?;
        writeln!(f, "c \tREDUCTION: {:.2?}", self.reduction_time)?;
        writeln!(f, "c \tCHOICE: {:.2?}", self.choice_time)?;
        Ok(())
    }
}

macro_rules! propagation_macro {
    ($self:ident,
        $sc_vec:expr,
        $some_deduction:ident,
        $stats:ident,
        $conflict_vec:ident
    ) => {
        for i in 0..$sc_vec.len() {
            let stored_clause = $sc_vec[i].clone();

            match stored_clause.watch_choices(&$self.valuation) {
                ClauseStatus::Entails(consequent) => {
                    let this_implication_time = std::time::Instant::now();
                    match $self.set_literal(
                        consequent,
                        LiteralSource::StoredClause(stored_clause.clone()),
                    ) {
                        Err(SolveError::Conflict(_, _)) => {
                            panic!("Conflict when setting a variable")
                        }
                        Err(e) => {
                            panic!("Unexpected error {e:?} when setting literal {consequent}")
                        }
                        Ok(()) => {
                            $some_deduction = true;
                        }
                    }
                    $stats.implication_time += this_implication_time.elapsed();
                }
                ClauseStatus::Conflict => {
                    match $conflict_vec {
                        Conflicts::None => {
                            $conflict_vec = Conflicts::Single(stored_clause);
                        }
                        Conflicts::Multiple(ref mut vec) => vec.push(stored_clause),
                        Conflicts::Single(_) => panic!("Conflict already set"),
                    };
                    match $self.config.break_on_first {
                        true => break,
                        false => continue,
                    };
                }
                ClauseStatus::Unsatisfied => (),
                ClauseStatus::Satisfied => (),
            }
        }
    };
}

#[derive(PartialEq)]
enum Conflicts {
    None,
    Single(Rc<StoredClause>),
    Multiple(Vec<Rc<StoredClause>>),
}

impl Solve<'_> {
    pub fn implication_solve(&mut self) -> (SolveResult, SolveStats) {
        let this_total_time = std::time::Instant::now();

        let mut stats = SolveStats::new();

        self.set_from_lists(hobson_choices(self.clauses())); // settle any literals which occur only as true or only as false

        let result: SolveResult;

        'main_loop: loop {
            stats.iterations += 1;

            log::trace!("Loop on valuation: {}", self.valuation.as_internal_string());

            let this_examination_time = std::time::Instant::now();

            stats.examination_time += this_examination_time.elapsed();

            let mut some_deduction = false;
            let mut conflicts = match self.config.break_on_first {
                true => Conflicts::None,
                false => Conflicts::Multiple(vec![]),
            };

            if self.current_level().get_choice().is_some() {
                let current_level = self.current_level().index();
                let literals = self.levels[current_level].updated_watches().clone();

                for literal in literals {
                    let v_id = literal.v_id;
                    propagation_macro!(
                        self,
                        self.variables[v_id].watch_occurrences(),
                        some_deduction,
                        stats,
                        conflicts
                    );

                    if self.config.break_on_first && conflicts != Conflicts::None {
                        break;
                    }
                }
            } else {
                propagation_macro!(self, self.formula_clauses, some_deduction, stats, conflicts);

                propagation_macro!(self, self.learnt_clauses, some_deduction, stats, conflicts);
            }

            if conflicts != Conflicts::None {
                match conflicts {
                    Conflicts::None => (),
                    Conflicts::Single(stored_conflict) => {
                        match process_conflict(self, &stored_conflict, &mut stats) {
                            ProcessOption::ContinueMain => {
                                if true {
                                    continue 'main_loop;
                                }
                            }
                            ProcessOption::Unsatisfiable => {
                                result = SolveResult::Unsatisfiable;
                                break 'main_loop;
                            }
                            _ => panic!("Unepected process option given a conflict"),
                        }
                    }
                    Conflicts::Multiple(conflict_vec) => {
                        for stored_conflict in conflict_vec {
                            match process_conflict(self, &stored_conflict, &mut stats) {
                                ProcessOption::ContinueMain => {
                                    if true {
                                        continue 'main_loop;
                                    }
                                }
                                ProcessOption::Unsatisfiable => {
                                    result = SolveResult::Unsatisfiable;
                                    break 'main_loop;
                                }
                                _ => panic!("Unepected process option given a conflict"),
                            }
                        }
                    }
                }
            }

            if !some_deduction {
                if let Some(available_v_id) = self.most_active_none(&self.valuation) {
                    if self.time_to_reduce() {
                        reduce(self, &mut stats)
                    }

                    let this_choice_time = std::time::Instant::now();
                    log::trace!(
                        "Choice: {available_v_id} @ {} with activity {}",
                        self.current_level().index(),
                        self.variables[available_v_id].activity()
                    );

                    let _ = self
                        .set_literal(Literal::new(available_v_id, false), LiteralSource::Choice);
                    stats.choice_time += this_choice_time.elapsed();

                    continue 'main_loop;
                } else {
                    result = SolveResult::Satisfiable;
                    break 'main_loop;
                }
            }
        }

        stats.total_time = this_total_time.elapsed();
        match result {
            SolveResult::Satisfiable => {
                println!(
                    "c ASSIGNMENT: {}",
                    self.valuation.to_vec().as_display_string(self)
                );
            }
            SolveResult::Unsatisfiable => {}
            SolveResult::Unknown => {}
        }
        (result, stats)
    }
}

// #[inline(always)]
fn reduce(solve: &mut Solve, stats: &mut SolveStats) {
    let this_reduction_time = std::time::Instant::now();
    println!("time to reduce");
    solve.learnt_clauses.sort_unstable_by_key(|a| a.lbd());

    let learnt_count = solve.learnt_clauses.len();
    println!("Learnt count: {}", learnt_count);
    for _ in 0..learnt_count {
        if solve
            .learnt_clauses
            .last()
            .is_some_and(|lc| lc.lbd() > solve.config.min_glue_strength)
        {
            let goodbye = solve.learnt_clauses.last().unwrap().clone();
            solve.drop_clause(&goodbye);
        } else {
            break;
        }
    }
    solve.forgets += 1;
    solve.conflcits_since_last_forget = 0;
    stats.reduction_time += this_reduction_time.elapsed();
    println!("Reduced to: {}", solve.learnt_clauses.len());
}

enum ProcessOption {
    Unsatisfiable,
    ContinueMain,
    Implicationed,
    Conflict,
}

fn process_clause(
    solve: &mut Solve,
    stored_clause: &Rc<StoredClause>,
    clause_status: &ClauseStatus,
    stats: &mut SolveStats,
    some_conflict: &mut bool,
    some_deduction: &mut bool,
) -> ProcessOption {
    match clause_status {
        ClauseStatus::Entails(consequent) => {
            let this_implication_time = std::time::Instant::now();
            match solve.set_literal(
                *consequent,
                LiteralSource::StoredClause(stored_clause.clone()),
            ) {
                Err(SolveError::Conflict(_, _)) => {
                    *some_conflict = true;
                }
                Err(e) => panic!("Unexpected error {e:?} when setting literal {consequent}"),
                Ok(()) => {
                    *some_deduction = true;
                }
            }
            stats.implication_time += this_implication_time.elapsed();
            ProcessOption::Implicationed
        }
        ClauseStatus::Conflict => ProcessOption::Conflict,
        _ => panic!("Something unexpected"),
    }
}

fn process_conflict(
    solve: &mut Solve,
    stored_conflict: &Rc<StoredClause>,
    stats: &mut SolveStats,
) -> ProcessOption {
    let this_unsat_time = std::time::Instant::now();
    solve.notice_conflict(stored_conflict);
    stats.conflicts += 1;
    match solve.attempt_fix(stored_conflict.clone()) {
        Err(SolveError::NoSolution) => {
            if solve.config.core {
                solve.core();
            }
            ProcessOption::Unsatisfiable
        }
        Ok(SolveOk::AssertingClause) | Ok(SolveOk::Deduction(_)) => {
            stats.unsat_time += this_unsat_time.elapsed();
            ProcessOption::ContinueMain
        }
        Ok(ok) => panic!("Unexpected ok {ok:?} when attempting a fix"),
        Err(err) => {
            panic!("Unexpected error {err:?} when attempting a fix")
        }
    }
}
