use crate::{
    context::{Context, GraphClause, ImplicationGraphNode},
    structures::{
        clause::stored::{Source, StoredClause},
        level::{Level, LevelIndex},
        literal::Literal,
        variable::variable_store::VariableStore,
    },
};

impl Context {
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

    pub fn variables(&self) -> &impl VariableStore {
        &self.variables
    }

    pub fn most_active_none(&self) -> Option<usize> {
        self.variables
            .iter()
            .enumerate()
            .filter(|(_, variable)| variable.polarity().is_none())
            .map(|(index, _)| (index, self.variables[index].activity()))
            .max_by(|(_, activity_a), (_, activity_b)| activity_a.total_cmp(activity_b))
            .map(|(index, _)| index)
    }

    /// Stores a clause with an automatically generated id.
    /// In order to use the clause the watch literals of the struct must be initialised.
    pub fn store_clause(&mut self, clause: Vec<Literal>, src: Source) -> &StoredClause {
        assert!(!clause.is_empty(), "Attempt to add an empty clause");

        let clause_key = self.stored_clauses.insert(src, clause, &self.variables);
        let the_clause = self.stored_clauses.retreive_mut(clause_key).expect("o");
        let node_index = self
            .implication_graph
            .add_node(ImplicationGraphNode::Clause(GraphClause {
                clause_id: the_clause.id(),
                key: the_clause.key(),
            }));
        the_clause.add_node_index(node_index);
        the_clause
    }

    pub fn backjump(&mut self, to: LevelIndex) {
        log::trace!("Backjump from {} to {}", self.level().index(), to);

        for _ in 0..(self.level().index() - to) {
            for literal in self.levels.pop().expect("Lost level").literals() {
                log::trace!("Noneset: {}", literal.index());
                self.variables.retract_valuation(literal.index());
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
