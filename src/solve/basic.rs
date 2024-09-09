use crate::{
    structures::{Literal, LiteralSource, Solve, SolveError, ValuationVec, VariableId},
    Clause, ValuationError,
};
use std::collections::BTreeSet;

impl Solve<'_> {
    pub fn single_deduction_solve(&mut self) -> Result<(bool, ValuationVec), SolveError> {
        println!("~~~ a basic solve ~~~");
        let sat_valuation: Option<(bool, ValuationVec)>;
        self.settle_hobson_choices(); // settle any literals which do occur with some fixed polarity

        loop {
            // 1. (un)sat check
            if Some(false) == self.sat {
                if let Some(level) = self.pop_level() {
                    level.choices.into_iter().for_each(|choice| {
                        let _ = self.set_literal(&choice.negate(), LiteralSource::Conflict);
                    })
                } else {
                    sat_valuation = Some((false, self.valuation.clone()));
                    break;
                }
            }

            // 2. search
            let (the_units, the_choices) =
                self.find_all_unset_on(&self.valuation_at_level(self.current_level()));

            // 3. decide
            if !the_units.is_empty() {
                let current_level = self.current_level();
                for (clause_id, literal) in &the_units {
                    self.levels[current_level]
                        .add_literal(literal, LiteralSource::Clause(*clause_id));
                    let _ = self.set_literal(literal, LiteralSource::Clause(*clause_id));
                }
            } else if !the_choices.is_empty() {
                let _ = self.set_literal(the_choices.first().unwrap(), LiteralSource::Choice);
            } else {
                sat_valuation = Some((true, self.valuation.clone()));
                break;
            }
        }
        match sat_valuation {
            Some(pair) => Ok(pair),
            None => Err(SolveError::Hek),
        }
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

    pub fn settle_hobson_choices(&mut self) {
        let the_choices = self.hobson_choices();
        the_choices.0.iter().for_each(|&v_id| {
            let the_choice = Literal::new(v_id, false);
            let _ = self.set_literal(&the_choice, LiteralSource::HobsonChoice);
        });
        the_choices.1.iter().for_each(|&v_id| {
            let the_choice = Literal::new(v_id, true);
            let _ = self.set_literal(&the_choice, LiteralSource::HobsonChoice);
        });
    }

    pub fn implication_solve(&mut self) -> Result<(bool, ValuationVec), SolveError> {
        println!("~~~ an implication solve ~~~");
        let sat_valuation: Option<(bool, ValuationVec)>;
        // self.settle_hobson_choices(); // settle any literals which do occur with some fixed polarity

        loop {
            // 1. (un)sat check
            if Some(false) == self.sat {
                let val = self.valuation_at_level(self.current_level());
                let popped_level = self.pop_level();
                if let Some(mut level) = popped_level {
                    // println!(". {:?}", level.clauses_violated);
                    level.implications.generate_details();
                    let x = *level.clauses_violated.first().unwrap();
                    let v_c = self.formula.clauses.iter().find(|c|  c.id == x).unwrap();
                    let c_v = v_c.literals.iter().map(|l| l.v_id ).collect::<Vec<_>>();
                    level.implications.trace_implication_paths(level.choices.first().unwrap().v_id, c_v);
                    level.implications.unique_point(v_c, val, &level);
                    if level.choices.len() == 1 {
                        let the_literal = &level.choices.first().unwrap().negate();
                        let _ = self.set_literal(the_literal, LiteralSource::Conflict);
                    } else {
                        let the_clause = level.choices.into_iter().map(|l| l.negate()).collect();
                        self.learn_as_clause(the_clause);
                    }
                } else {
                    sat_valuation = Some((false, self.valuation.clone()));
                    break;
                }
            }

            // 2. search
            let (the_units, the_choices) =
                self.find_all_unset_on(&self.valuation_at_level(self.current_level()));

            // 3. decide, either
            if !the_units.is_empty() { // apply unit clauses
                for (clause_id, literal) in &the_units {
                    match self.set_literal(literal, LiteralSource::Clause(*clause_id)) {
                        Err(ValuationError::Inconsistent) => {
                            // println!("{:?}", self.formula.clauses.iter().find(|c| c.id == *clause_id).unwrap());
                            // break
                        },
                        _ => continue
                    }
                }

                self.extend_implication_graph(
                    self.current_level(),
                    &the_units.iter().cloned().collect()
                );
            } else if !the_choices.is_empty() { // make a choice
                let first_choice = the_choices.first().unwrap();
                let _ = self.set_literal(first_choice, LiteralSource::Choice);
            } else { // return sat
                sat_valuation = Some((true, self.valuation.clone()));
                break;
            }
        }
        match sat_valuation {

            Some(pair) => Ok(pair),
            None => Err(SolveError::Hek),
        }
    }

}
