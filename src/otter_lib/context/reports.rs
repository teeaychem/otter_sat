use crate::{
    context::{stores::clause_key::ClauseKey, Context},
    structures::{clause::Clause, literal::LiteralSource},
};

impl Context {
    #[allow(clippy::single_match)]
    /// Display an unsatisfiable core given some conflict.
    pub fn get_unsat_core(&self, conflict_key: ClauseKey) -> Vec<ClauseKey> {
        println!("c An unsatisfiable core of the formula:\n",);

        /*
        Given the conflict clause, collect the following:

        - The formula clauses used to resolve the conflict clause
        - The formula clauses used to establish any literal whose negation appears in some considered clause

        The core_q queues clause keys for inspection
        The seen literal set helps to avoid checking the same literal twice
        Likewise, the key set helps to avoid checking the same key twice
         */

        let mut core_q = std::collections::VecDeque::<ClauseKey>::new();
        let mut seen_literal_set = std::collections::BTreeSet::new();
        let mut key_set = std::collections::BTreeSet::new();
        let mut core_keys = std::collections::BTreeSet::new();

        // for short arguments
        let observations = self.levels.get(0).observations();

        // start with the conflict, then loop
        core_q.push_back(conflict_key);

        /*
        key set ensures processing only happens on a fresh key

        if the key is for a formula, then clause is recorded and the literals of the clause are checked against the observed literals
        otherwise, the clauses used when resolving the learnt clause are added

         when checking literals, if the negation of the literal has been observed at level 0 then it was relevant to the conflict
         so, if the literal was obtained either by resolution or directly from some clause, then that clause or the clauses used for resolution are added to the q
         this skips assumed literals
         */

        while let Some(key) = core_q.pop_front() {
            if key_set.insert(key) {
                match key {
                    ClauseKey::Formula(_) => {
                        let clause = self.clause_store.get(key);

                        core_keys.insert(key);

                        for literal in clause.literal_slice() {
                            if seen_literal_set.insert(*literal) {
                                let found = observations.iter().find(|(_, observed_literal)| {
                                    *literal == observed_literal.negate()
                                });
                                if let Some((src, _)) = found {
                                    match src {
                                        LiteralSource::Resolution(_) => {
                                            let proof = &self
                                                .proofs
                                                .iter()
                                                .find(|(proven_literal, _)| {
                                                    *literal == proven_literal.negate()
                                                })
                                                .expect("no proof of resolved literal");
                                            for key in &proof.1 {
                                                core_q.push_back(*key);
                                            }
                                        }
                                        LiteralSource::Analysis(clause_key)
                                        | LiteralSource::BCP(clause_key)
                                        | LiteralSource::Missed(clause_key) => {
                                            core_q.push_back(*clause_key)
                                        }

                                        LiteralSource::Choice
                                        | LiteralSource::Pure
                                        | LiteralSource::Assumption => {}
                                    }
                                }
                            }
                        }
                    }
                    ClauseKey::Binary(index) => {
                        let source = &self.clause_store.binary_graph[index as usize];
                        for source_key in source {
                            core_q.push_back(*source_key);
                        }
                    }
                    ClauseKey::Learned(index, usage) => {
                        let source =
                            &self.clause_store.resolution_graph[index as usize][usage as usize];
                        for source_key in source {
                            core_q.push_back(*source_key);
                        }
                    }
                }
            }
        }

        core_keys.into_iter().collect()
    }

    pub fn display_core(&self, conflict_key: ClauseKey) {
        for key in self.get_unsat_core(conflict_key) {
            let clause = self.clause_store.get(key);
            println!("{}", clause.as_dimacs(&self.variables));
        }
    }
}
