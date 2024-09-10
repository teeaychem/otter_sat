use std::fmt::Debug;

use crate::structures::{
    ClauseId, Literal, LiteralSource, Solve, Valuation, ValuationError, ValuationVec, VariableId,
};

use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub struct Level {
    index: usize,
    choice: Option<Literal>,
    observations: Vec<Literal>,
    clauses_unit: Vec<(ClauseId, Literal)>,
    clauses_violated: Vec<ClauseId>, // pub clauses_open: Vec<ClauseId>,
}

impl<'borrow, 'solve> Level {
    pub fn new(index: usize, _solve: &'borrow Solve<'solve>) -> Self {
        Level {
            index,
            choice: None,
            observations: vec![],
            clauses_unit: vec![],
            clauses_violated: vec![],
        }
    }
}

impl Level {
    pub fn get_choice(&self) -> Literal {
        if self.choice.is_none() {
            panic!("Level {} has no choice", self.index)
        }
        self.choice.unwrap()
    }

    pub fn add_literal(&mut self, literal: &Literal, source: LiteralSource) {
        match source {
            LiteralSource::Choice => {
                if self.choice.is_some() {
                    panic!("Attempting to make multiple choices on a single level")
                }
                self.choice = Some(*literal);
            }
            LiteralSource::HobsonChoice
            | LiteralSource::Assumption
            | LiteralSource::Clause(_)
            | LiteralSource::Conflict => self.observations.push(*literal),
            _ => todo!(),
        }
    }

    pub fn add_violated_clause(&mut self, clause: ClauseId) {
        self.clauses_violated.push(clause)
}

    pub fn literals(&self) -> impl Iterator<Item = Literal> + '_ {
        self.choice
            .into_iter()
            .chain(self.observations.iter().cloned())
    }
}

impl<'borrow, 'solve> Solve<'solve> {
    pub fn add_fresh_level(&'borrow mut self) {
        let index = self.levels.len();
        let the_level = Level::new(index, self);
        self.levels.push(the_level);
    }
}

impl<'borrow, 'level, 'solve: 'level> Solve<'solve> {
    pub fn pop_level(&'borrow mut self) -> Option<Level> {
        if self.levels.len() <= 1 {
            return None;
        }
        let the_level: Option<Level> = self.levels.pop();
        self.valuation.clear_if_level(&the_level);
        self.sat = None;

        the_level
    }

    pub fn current_level_index(&'borrow self) -> usize {
        self.levels.len() - 1
    }

    pub fn top_level(&'borrow self) -> &Level {
        &self.levels[0]
    }

    pub fn top_level_mut(&'borrow mut self) -> &mut Level {
        &mut self.levels[0]
    }

    pub fn current_level(&'borrow self) -> &Level {
        let index = self.levels.len() - 1;
        &self.levels[index]
    }

    pub fn current_level_mut(&'borrow mut self) -> &mut Level {
        let index = self.levels.len() - 1;
        &mut self.levels[index]
    }
}

impl Solve<'_> {
    pub fn valuation_at_level(&self, index: usize) -> ValuationVec {
        let mut valuation = ValuationVec::new_for_variables(self.valuation.len());
        (0..=index).for_each(|i| {
            self.levels[i].literals().for_each(|l| {
                let _ = valuation.set_literal(l);
            })
        });
        valuation
    }

    // pub fn extend_implication_graph(
    //     &mut self,
    //     level: usize,
    //     the_units: &Vec<(ClauseId, Literal)>,
    // ) {
    //     // let valuation = self.valuation_at_level(index);
    //     // let the_graph = ImpGraph::for_level(self, index, &self.formula);
    //     // self.levels[index].implications = the_graph;

    //     for (clause_id, to_literal) in the_units {
    //         let the_clause = self.formula.clauses.iter().find(|c| c.id == *clause_id).unwrap();
    //         let the_level = &self.levels[level];
    //         let from = the_level.literals().iter().map(|l| l.v_id).collect::<BTreeSet<_>>();

    //         self.levels[level].implications.extend(from, the_clause, to_literal.v_id);
    //     }

    //     for (clause_id, literal) in the_units {
    //         let _ = self.set_literal(literal, LiteralSource::Clause(*clause_id));
    //     }

    //     self.levels[level].clauses_unit.extend(the_units);
    // }
}
