use otter_sat::{config::Config, context::Context, reports::Report};

mod failed_literals {

    use super::*;

    #[test]
    fn direct_failure() {
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
    fn multiple_failures() {
        let mut ctx = Context::from_config(Config::default());

        let [p, q, r, s, t, u] = *ctx.fresh_or_max_literals(6).as_slice() else {
            panic!("Insufficient literals");
        };

        assert!(ctx.add_clause(vec![-p, q]).is_ok());
        assert!(ctx.add_clause(vec![-r, s]).is_ok());
        assert!(ctx.add_clause(vec![-s, t]).is_ok());
        assert!(ctx.add_clause(vec![-q, -t]).is_ok());

        let result = ctx.solve_given(Some(vec![p, r, u]));

        assert!(result.is_ok());

        assert!(ctx.failed_assumpions().contains(&p));
        assert!(ctx.failed_assumpions().contains(&r));

        assert!(!ctx.failed_assumpions().contains(&u));

        assert!(matches!(ctx.report(), Report::Unsatisfiable))
    }
}
