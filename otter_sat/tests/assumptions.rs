use otter_sat::{config::Config, context::Context, reports::Report};

mod basic_assumptions {

    use super::*;

    #[test]
    fn direct() {
        let cfg = Config::default();
        let mut ctx = Context::from_config(cfg);

        let [p, q] = *ctx.fresh_or_max_literals(2).as_slice() else {
            panic!("Insufficient literals");
        };

        assert!(ctx.add_clause(vec![-p, q]).is_ok());

        assert!(ctx.add_clause(vec![-q]).is_ok());

        let result = ctx.solve_given(Some(vec![p]));

        assert!(result.is_ok());

        assert!(ctx.failed_assumpions().contains(&p));

        assert!(matches!(ctx.report(), Report::Unsatisfiable))
    }

    #[test]
    fn small_chain() {
        let mut ctx = Context::from_config(Config::default());

        let [p, q, r, s, t] = *ctx.fresh_or_max_literals(5).as_slice() else {
            panic!("Insufficient literals");
        };

        assert!(ctx.add_clause(vec![-p, q]).is_ok());
        assert!(ctx.add_clause(vec![-q, r]).is_ok());
        assert!(ctx.add_clause(vec![-r, s]).is_ok());
        assert!(ctx.add_clause(vec![-s, t]).is_ok());

        assert!(ctx.add_clause(vec![-t]).is_ok());

        let result = ctx.solve_given(Some(vec![p]));

        assert!(result.is_ok());

        assert!(ctx.failed_assumpions().contains(&p));

        assert!(matches!(ctx.report(), Report::Unsatisfiable))
    }
}
