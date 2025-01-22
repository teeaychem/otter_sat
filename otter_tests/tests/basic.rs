use otter_sat::{
    config::Config,
    context::Context,
    dispatch::library::report::{self},
};

mod basic {

    use otter_sat::structures::clause::Clause;

    use super::*;
    #[test]
    fn one_literal() {
        let mut the_context = Context::from_config(Config::default(), None);
        let p_clause = the_context.clause_from_string("p").unwrap();
        assert!(the_context.add_clause(p_clause).is_ok());
        assert!(the_context.solve().is_ok());
        assert_eq!(the_context.report(), report::SolveReport::Satisfiable)
    }

    #[test]
    fn conflict() {
        let mut the_context = Context::from_config(Config::default(), None);
        let p_q_clause = the_context.clause_from_string("p q").unwrap();
        assert!(the_context.add_clause(p_q_clause).is_ok());

        let np_nq_clause = the_context.clause_from_string("-p -q").unwrap();
        assert!(the_context.add_clause(np_nq_clause).is_ok());

        let p_nq_clause = the_context.clause_from_string("p -q").unwrap();
        assert!(the_context.add_clause(p_nq_clause).is_ok());

        let np_q_clause = the_context.clause_from_string("-p q").unwrap();
        assert!(the_context.add_clause(np_q_clause).is_ok());

        assert!(the_context.solve().is_ok());
        assert!(matches!(
            the_context.report(),
            report::SolveReport::Unsatisfiable
        ))
    }

    #[test]
    fn assumption() {
        let mut the_context = Context::from_config(Config::default(), None);

        let p_q_clause = the_context.clause_from_string("p q").unwrap();
        assert!(the_context.add_clause(p_q_clause).is_ok());

        let not_p = the_context.literal_from_string("-p").expect("oh");

        assert!(the_context.add_clause(not_p).is_ok());
        assert!(the_context.solve().is_ok());
        assert_eq!(the_context.report(), report::SolveReport::Satisfiable);

        let the_valuation = the_context.atom_db.valuation_string();
        assert!(the_valuation.contains("-p"));
        assert!(the_valuation.contains("q"));
    }

    #[test]
    fn duplicates() {
        let mut the_context = Context::from_config(Config::default(), None);
        let p_q_q_clause = the_context.clause_from_string("p q q").unwrap();
        assert!(the_context.add_clause(p_q_q_clause).is_ok());
        let database = the_context
            .clause_db
            .all_nonunit_clauses()
            .collect::<Vec<_>>();
        assert_eq!(database.len(), 1);
        let the_clause_dimacs = database[0].as_dimacs(&the_context.atom_db, true);
        assert_eq!(
            the_clause_dimacs.split_whitespace().count(),
            "p q 0".split_whitespace().count()
        );
    }

    #[test]
    fn tautology_skip() {
        let mut the_context = Context::from_config(Config::default(), None);
        let p_q_np_clause = the_context.clause_from_string("p q -p").unwrap();
        assert!(the_context.add_clause(p_q_np_clause).is_ok());
        let mut clause_iter = the_context.clause_db.all_nonunit_clauses();
        assert!(clause_iter.next().is_none());
    }

    // TOOD: Incremental tests based on example
}
