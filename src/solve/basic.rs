use crate::structures::{
    Assignment, ClauseId, Literal, LiteralSource, Solve, SolveError, Valuation, VariableId,
};
use std::collections::BTreeSet;

impl Solve {
    // pub fn single_deduction_solve(&mut self) -> Result<(bool, Valuation), SolveError> {
    //     let mut the_search = Assignment::for_solve(self);
    //     let sat_assignment: Option<(bool, Valuation)>;

    //     loop {
    //         // 1. (un)sat check
    //         if self.is_sat_on(&the_search.valuation) {
    //             sat_assignment = Some((true, the_search.valuation.clone()));
    //             break;
    //         } else if self.is_unsat_on(&the_search.valuation) {
    //             if let Some(literal) = the_search.pop_last_choice() {
    //                 the_search.set(&literal.negate(), LiteralSource::Conflict)
    //             } else {
    //                 sat_assignment = Some((false, the_search.valuation.clone()));
    //                 break;
    //             }
    //         }
    //         // 2. search
    //         if let Some(_units_found) = self.propagate_unit(&mut the_search) {
    //             continue;
    //         }

    //         if let Some(v_id) = the_search.get_unassigned_id(self) {
    //             the_search.set(&Literal::new(v_id, true), LiteralSource::Choice);
    //             continue;
    //         }
    //     }
    //     match sat_assignment {
    //         Some(pair) => Ok(pair),
    //         None => Err(SolveError::Hek),
    //     }
    // }

    pub fn literals_of_polarity(&self, polarity: bool) -> BTreeSet<Literal> {
        self.clauses
            .iter()
            .fold(BTreeSet::new(), |mut acc: BTreeSet<Literal>, this| {
                acc.extend(
                    this.literals
                        .iter()
                        .filter(|&l| l.polarity == polarity)
                        .cloned()
                        .collect::<BTreeSet<Literal>>(),
                );
                acc
            })
    }

    /// general order for pairs related to booleans is 0 is false, 1 is true
    pub fn hobson_choices(&self) -> (Vec<VariableId>, Vec<VariableId>) {
        // let all_v_ids: BTreeSet<VariableId> = self.vars().iter().map(|v| v.id).collect();
        let the_true: BTreeSet<VariableId> = self
            .literals_of_polarity(true)
            .iter()
            .map(|l| l.v_id)
            .collect();
        let the_false: BTreeSet<VariableId> = self
            .literals_of_polarity(false)
            .iter()
            .map(|l| l.v_id)
            .collect();
        let hobson_false: Vec<_> = the_false.difference(&the_true).cloned().collect();
        let hobson_true: Vec<_> = the_true.difference(&the_false).cloned().collect();
        (hobson_false, hobson_true)
    }

    pub fn settle_hobson_choices(&self, search: &mut Assignment) {
        let the_choices = self.hobson_choices();
        the_choices.0.iter().for_each(|&v_id| {
            let the_choice = Literal::new(v_id, false);
            search.set(&the_choice, LiteralSource::HobsonChoice);
        });
        the_choices.1.iter().for_each(|&v_id| {
            let the_choice = Literal::new(v_id, true);
            search.set(&the_choice, LiteralSource::HobsonChoice);
        });
    }

    pub fn propagate_all_units(
        &self,
        assignment: &mut Assignment,
    ) -> Option<Vec<(usize, Literal)>> {
        let mut units_found = vec![];
        while let Some((clause_id, lit)) = self.find_unit_on(&assignment.valuation) {
            assignment.set(&lit, LiteralSource::Clause(clause_id));
            units_found.push((clause_id, lit));
        }
        match units_found.is_empty() {
            true => None,
            false => Some(units_found),
        }
    }

    pub fn propagate_by_implication_graph(
        &self,
        assignment: &mut Assignment,
    ) -> Option<Vec<(usize, Literal)>> {
        let mut units_found = vec![];
        while let Some((clause_id, lit)) = self.find_unit_on(&assignment.valuation) {
            assignment.set(&lit, LiteralSource::Clause(clause_id));
            units_found.push((clause_id, lit));
        }
        match units_found.is_empty() {
            true => None,
            false => Some(units_found),
        }
    }

    pub fn alt_deduction_solve(&mut self) -> Result<(bool, Assignment), SolveError> {
        let mut the_search = Assignment::for_solve(self);
        let sat_assignment: Option<(bool, Assignment)>;
        // settle any forced choices
        self.settle_hobson_choices(&mut the_search);
        self.propagate_all_units(&mut the_search);

        loop {
            // 1. (un)sat check
            if self.is_sat_on(&the_search.valuation) {
                sat_assignment = Some((true, the_search));
                break;
            } else if self.is_unsat_on(&the_search.valuation) {
                if let Some(level) = the_search.pop_last_level() {
                    level.choices.into_iter().for_each(|choice| {
                        the_search.set(&choice.negate(), LiteralSource::Conflict)
                    })
                } else {
                    sat_assignment = Some((false, the_search));
                    break;
                }
            }
            // 2. search
            if let Some(_the_units_found) = self.propagate_all_units(&mut the_search) {
                continue;
            }

            if let Some(v_id) = the_search.get_unassigned_id(self) {
                the_search.set(&Literal::new(v_id, true), LiteralSource::Choice);
                continue;
            }
        }
        match sat_assignment {
            Some(pair) => Ok(pair),
            None => Err(SolveError::Hek),
        }
    }

    pub fn implication_solve(&mut self) -> Result<(bool, Assignment), SolveError> {
        println!("~~~ an implication solve ~~~");
        let mut the_search = Assignment::for_solve(self);
        let sat_assignment: Option<(bool, Assignment)>;
        // settle any forced choices
        self.settle_hobson_choices(&mut the_search);
        // self.propagate_all_units(&mut the_search);

        loop {
            // 1. (un)sat check
            if self.is_sat_on(&the_search.valuation) {
                sat_assignment = Some((true, the_search));
                break;
            } else if self.is_unsat_on(&the_search.valuation) {
                if let Some(level) = the_search.pop_last_level() {
                    level.choices.into_iter().for_each(|choice| {
                        the_search.set(&choice.negate(), LiteralSource::Conflict)
                    })
                } else {
                    sat_assignment = Some((false, the_search));
                    break;
                }
            }
            // 2. search
            the_search.add_implication_graph_for_level(the_search.current_level(), self);
            if !the_search.graph_at_level(the_search.current_level()).units.is_empty() {
                the_search.add_literals_from_graph(the_search.current_level());
                continue;
            }

            if let Some(v_id) = the_search.get_unassigned_id(self) {
                the_search.set(&Literal::new(v_id, true), LiteralSource::Choice);
                continue;
            }
        }
        match sat_assignment {
            Some(pair) => Ok(pair),
            None => Err(SolveError::Hek),
        }
    }
}
