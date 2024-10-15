use crate::structures::{
    clause::stored::Source,
    level::{Level, LevelIndex},
    literal::Literal,
    solve::{store::ClauseKey, Solve},
    valuation::Valuation,
    variable::Variable,
};

impl Solve {
    pub fn add_fresh_level(&mut self) -> LevelIndex {
        let index = self.levels.len();
        let the_level = Level::new(index);
        self.levels.push(the_level);
        index
    }

    pub fn level(&self) -> &Level {
        let index = self.levels.len() - 1;
        &self.levels[index]
    }

    pub fn variables(&self) -> &[Variable] {
        &self.variables
    }

    pub fn valuation(&self) -> &impl Valuation {
        &self.valuation
    }

    pub fn most_active_none(&self, val: &impl Valuation) -> Option<usize> {
        val.values()
            .enumerate()
            .filter(|(_, v)| v.is_none())
            .map(|(i, _)| (i, self.variables[i].activity()))
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(a, _)| a)
    }

    /// Stores a clause with an automatically generated id.
    /// In order to use the clause the watch literals of the struct must be initialised.
    pub fn store_clause(&mut self, clause: Vec<Literal>, src: Source) -> ClauseKey {
        assert!(!clause.is_empty(), "Attempt to add an empty clause");

        self.stored_clauses
            .insert(src, clause, &self.valuation, &mut self.variables)
    }

    pub fn backjump(&mut self, to: LevelIndex) {
        log::trace!("Backjump from {} to {}", self.level().index(), to);

        for _ in 0..(self.level().index() - to) {
            for literal in self.levels.pop().expect("Lost level").literals() {
                log::trace!("Noneset: {}", literal.index());

                unsafe {
                    *self.valuation.get_unchecked_mut(literal.index()) = None;
                    self.variables
                        .get_unchecked(literal.index())
                        .clear_decision_level();
                }
            }
        }
    }

    pub fn display_stats(&self) {
        println!("c STATS");
        println!("c   ITERATIONS      {}", self.iterations);
        println!("c   CONFLICTS       {}", self.conflicts);
        println!(
            "c   CONFLICT RATIO  {:.4?}",
            self.conflicts as f32 / self.iterations as f32
        );
        println!("c   TIME            {:.2?}", self.time);
    }
}
