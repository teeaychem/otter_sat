use otter_sat::{
    builder::ClauseOk,
    config::Config,
    context::Context,
    dispatch::library::report::{self},
};

mod basic {

    use otter_sat::structures::{
        clause::Clause,
        literal::{CLiteral, Literal},
    };

    use super::*;
    #[test]
    fn one_literal() {
        let mut the_context = Context::from_config(Config::default(), None);
        let p = the_context.fresh_atom().unwrap();
        let p_clause = CLiteral::new(p, true);

        assert_eq!(Ok(ClauseOk::Added), the_context.add_clause(p_clause));

        assert!(the_context.solve().is_ok());

        assert_eq!(the_context.report(), report::SolveReport::Satisfiable)
    }

    #[test]
    fn conflict() {
        let mut the_context = Context::from_config(Config::default(), None);

        let p = the_context.fresh_atom().unwrap();
        let q = the_context.fresh_atom().unwrap();

        let p_q_clause = vec![CLiteral::new(p, true), CLiteral::new(q, true)];
        assert!(the_context.add_clause(p_q_clause).is_ok());

        let not_p_not_q_clause = vec![CLiteral::new(p, false), CLiteral::new(q, false)];
        assert!(the_context.add_clause(not_p_not_q_clause).is_ok());

        let p_not_q_clause = vec![CLiteral::new(p, true), CLiteral::new(q, false)];
        assert!(the_context.add_clause(p_not_q_clause).is_ok());

        let not_p_q_clause = vec![CLiteral::new(p, false), CLiteral::new(q, true)];
        assert!(the_context.add_clause(not_p_q_clause).is_ok());

        assert!(the_context.solve().is_ok());
        assert!(matches!(
            the_context.report(),
            report::SolveReport::Unsatisfiable
        ))
    }

    #[test]
    fn assumption() {
        let mut the_context = Context::from_config(Config::default(), None);

        let p = the_context.fresh_atom().unwrap();
        let q = the_context.fresh_atom().unwrap();

        let p_q_clause = vec![CLiteral::new(p, true), CLiteral::new(q, true)];
        assert_eq!(Ok(ClauseOk::Added), the_context.add_clause(p_q_clause));

        let not_p = CLiteral::new(p, false);

        assert!(the_context.add_clause(not_p).is_ok());
        assert!(the_context.solve().is_ok());
        assert_eq!(the_context.report(), report::SolveReport::Satisfiable);

        assert_eq!(the_context.atom_db.value_of(p), Some(false));
        assert_eq!(the_context.atom_db.value_of(q), Some(true));
    }

    #[test]
    fn duplicates() {
        let mut the_context = Context::from_config(Config::default(), None);

        let p = the_context.fresh_atom().unwrap();
        let q = the_context.fresh_atom().unwrap();

        let p_p_q_q_clause = vec![
            CLiteral::new(p, true),
            CLiteral::new(p, true),
            CLiteral::new(q, true),
            CLiteral::new(q, true),
        ];
        assert!(the_context.add_clause(p_p_q_q_clause).is_ok());

        // The atom db always contains top, and so the expected atom count is plus one.
        assert_eq!(3, the_context.atom_db.count());

        let database = the_context
            .clause_db
            .all_nonunit_clauses()
            .collect::<Vec<_>>();
        assert_eq!(database.len(), 1);
        let the_clause_dimacs = database[0].as_dimacs(true);
        println!("{the_clause_dimacs}");
        assert_eq!(
            the_clause_dimacs.split_whitespace().count(),
            "1 2 0".split_whitespace().count()
        );
    }

    #[test]
    fn tautology_skip() {
        let mut the_context = Context::from_config(Config::default(), None);

        let p = the_context.fresh_atom().unwrap();
        let q = the_context.fresh_atom().unwrap();

        let p_q_not_p_clause = vec![
            CLiteral::new(p, true),
            CLiteral::new(q, false),
            CLiteral::new(p, false),
        ];
        assert!(the_context.add_clause(p_q_not_p_clause).is_ok());
        let mut clause_iter = the_context.clause_db.all_nonunit_clauses();
        assert!(clause_iter.next().is_none());
    }

    // TOOD: Incremental tests based on example
}
