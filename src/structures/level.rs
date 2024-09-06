use std::fmt::Debug;

use crate::structures::{
    ClauseId, ImpGraph, ImpGraphEdge, Literal, LiteralSource, Solve, Valuation, ValuationError,
    ValuationVec, VariableId,
};

#[derive(Clone, Debug)]
pub struct Level<'formula> {
    index: usize,
    pub choices: Vec<Literal>,
    pub observations: Vec<Literal>,
    pub implications: ImpGraph<'formula>,
    pub clauses_unit: Vec<(ClauseId, Literal)>,
    // pub clauses_open: Vec<ClauseId>,
}

impl<'borrow, 'solve> Level<'solve> {
    pub fn new(index: usize, solve: &'borrow Solve<'solve>) -> Self {
        Level {
            index,
            choices: vec![],
            observations: vec![],
            implications: ImpGraph::for_formula(solve.formula),
            clauses_unit: vec![],
        }
    }
}

impl Level<'_> {
    pub fn add_literal(&mut self, literal: &Literal, source: LiteralSource) {
        match source {
            LiteralSource::Choice => self.choices.push(*literal),
            LiteralSource::Clause(_) => self.observations.push(*literal),
            _ => todo!(),
        }
    }

    pub fn literals(&self) -> Vec<Literal> {
        self.choices
            .iter()
            .chain(self.observations.iter())
            .cloned()
            .collect()
    }
}

impl<'borrow, 'formula> Solve<'formula> {
    pub fn add_fresh_level(&'borrow mut self) {
        let index = self.levels.len();
        let the_level = Level::new(index, self);
        self.levels.push(the_level);
    }
}

impl Solve<'_> {
    pub fn current_level(&self) -> usize {
        self.levels.len() - 1
    }

    pub fn pop_level(&mut self) -> Option<Level> {
        if self.levels.len() <= 1 {
            return None;
        }
        let the_level = self.levels.pop();
        self.valuation.clear_if_level(&the_level);
        self.sat = None;

        the_level
    }

    pub fn valuation_at_level(&self, index: usize) -> ValuationVec {
        let mut valuation = ValuationVec::new_for_variables(self.valuation.len());
        (0..=index).for_each(|i| {
            self.levels[i].literals().iter().for_each(|l| {
                let _ = valuation.set_literal(l);
            })
        });
        valuation
    }

    pub fn extend_implication_graph(
        &mut self,
        level: usize,
        the_units: Vec<(ClauseId, Literal)>,
        from_literals: Vec<Literal>,
    ) {
        // let valuation = self.valuation_at_level(index);
        // let the_graph = ImpGraph::for_level(self, index, &self.formula);
        // self.levels[index].implications = the_graph;

        for (clause_id, to_literal) in &the_units {
            for clause_literal in &self.formula.borrow_clause_by_id(*clause_id).literals {
                if from_literals.contains(&clause_literal.negate()) && clause_literal != to_literal
                {
                    self.levels[level].implications.add_edge(ImpGraphEdge::new(
                        clause_literal.v_id,
                        *clause_id,
                        to_literal.v_id,
                    ))
                }
            }
        }

        for (clause_id, literal) in &the_units {
            let _ = self.set_literal(literal, LiteralSource::Clause(*clause_id));
        }

        self.levels[level].clauses_unit.extend(the_units);
    }
}
