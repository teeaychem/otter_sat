use otter_sat::structures::literal::{CLiteral, Literal};
use otter_sat::{config::Config, context::Context, reports::Report};

mod basic_assumptions {

    use super::*;

    #[test]
    fn direct() {
        let mut ctx = Context::from_config(Config::default());

        let p = ctx.fresh_or_max_atom();
        let q = ctx.fresh_or_max_atom();

        let not_p_q_clause = vec![CLiteral::new(p, false), CLiteral::new(q, true)];
        assert!(ctx.add_clause(not_p_q_clause).is_ok());

        let not_q_clause = vec![CLiteral::new(q, false)];
        assert!(ctx.add_clause(not_q_clause).is_ok());

        let p_assumption = CLiteral::new(p, true);

        assert!(ctx.add_assumption(p_assumption).is_ok());

        assert!(ctx.solve().is_ok());

        assert!(ctx.failed_assumpions().contains(&p_assumption));

        assert!(matches!(ctx.report(), Report::Unsatisfiable))
    }

    #[test]
    fn small_chain() {
        let mut ctx = Context::from_config(Config::default());

        let p = ctx.fresh_or_max_atom();
        let q = ctx.fresh_or_max_atom();
        let r = ctx.fresh_or_max_atom();
        let s = ctx.fresh_or_max_atom();
        let t = ctx.fresh_or_max_atom();

        assert!(ctx
            .add_clause(vec![CLiteral::new(p, false), CLiteral::new(q, true)])
            .is_ok());
        assert!(ctx
            .add_clause(vec![CLiteral::new(q, false), CLiteral::new(r, true)])
            .is_ok());
        assert!(ctx
            .add_clause(vec![CLiteral::new(r, false), CLiteral::new(s, true)])
            .is_ok());
        assert!(ctx
            .add_clause(vec![CLiteral::new(s, false), CLiteral::new(t, true)])
            .is_ok());

        assert!(ctx.add_clause(vec![CLiteral::new(t, false)]).is_ok());

        let p_assumption = CLiteral::new(p, true);

        assert!(ctx.add_assumption(p_assumption).is_ok());

        assert!(ctx.solve().is_ok());

        println!("{:?}", ctx.failed_assumpions());

        assert!(ctx.failed_assumpions().contains(&p_assumption));

        assert!(matches!(ctx.report(), Report::Unsatisfiable))
    }
}
