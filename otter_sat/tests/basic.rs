use otter_sat::{builder::ClauseOk, config::Config, context::Context, reports::Report};

mod basic {

    use otter_sat::structures::{clause::Clause, literal::Literal, valuation::Valuation};

    use super::*;
    #[test]
    fn one_literal() {
        let mut ctx = Context::from_config(Config::default());
        let p = ctx.fresh_or_max_literal();

        assert_eq!(Ok(ClauseOk::Added), ctx.add_clause(p));

        assert!(ctx.solve().is_ok());

        assert_eq!(ctx.report(), Report::Satisfiable)
    }

    #[test]
    fn conflict() {
        let mut ctx = Context::from_config(Config::default());

        let [p, q] = *ctx.fresh_or_max_literals(2).as_slice() else {
            panic!("Insufficient literals");
        };

        let p_q_clause = vec![p, q];
        assert!(ctx.add_clause(p_q_clause).is_ok());

        let not_p_not_q_clause = vec![-p, -q];
        assert!(ctx.add_clause(not_p_not_q_clause).is_ok());

        let p_not_q_clause = vec![p, -q];
        assert!(ctx.add_clause(p_not_q_clause).is_ok());

        let not_p_q_clause = vec![-p, q];
        assert!(ctx.add_clause(not_p_q_clause).is_ok());

        assert!(ctx.solve().is_ok());
        assert!(matches!(ctx.report(), Report::Unsatisfiable))
    }

    #[test]
    fn unit_conjunct() {
        let mut ctx = Context::from_config(Config::default());

        let [p, q] = *ctx.fresh_or_max_literals(2).as_slice() else {
            panic!("Insufficient literals");
        };

        assert_eq!(Ok(ClauseOk::Added), ctx.add_clause(vec![p, q]));

        assert!(ctx.add_clause(-p).is_ok());

        assert_eq!(ctx.solve(), Ok(Report::Satisfiable));

        assert_eq!(ctx.value_of(p.atom()), Some(false));
        assert_eq!(ctx.value_of(q.atom()), Some(true));
    }

    #[test]
    fn duplicates() {
        let mut ctx = Context::from_config(Config::default());

        let [p, q] = *ctx.fresh_or_max_literals(2).as_slice() else {
            panic!("Insufficient literals");
        };

        assert!(ctx.add_clause(vec![p, p, q, q]).is_ok());

        // The atom db always contains top, and so the expected atom count is plus one.
        assert_eq!(3, ctx.valuation().atom_count());

        let database = ctx
            .clause_db
            .all_nonunit_clauses()
            .map(|(_key, clause)| clause)
            .collect::<Vec<_>>();

        let the_clause_dimacs = database[0].as_dimacs(true);

        assert_eq!(
            the_clause_dimacs.split_whitespace().count(),
            "1 2 0".split_whitespace().count()
        );
    }

    #[test]
    fn tautology_skip() {
        let mut ctx = Context::from_config(Config::default());

        let [p, q] = *ctx.fresh_or_max_literals(2).as_slice() else {
            panic!("Insufficient literals");
        };

        assert!(ctx.add_clause(vec![p, -q, -p]).is_ok());
        let mut clause_iter = ctx.clause_db.all_nonunit_clauses();
        assert!(clause_iter.next().is_none());
    }

    // TOOD: Incremental tests based on example
}
