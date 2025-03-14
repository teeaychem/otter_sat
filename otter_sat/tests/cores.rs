use otter_sat::{config::Config, context::Context, reports::Report};

use std::io::Write;

use otter_sat::structures::clause::Clause;

fn core_as_dimacs(context: &mut Context) -> Vec<u8> {
    let mut dimacs = vec![];
    for clause in context.core_keys() {
        let _ = dimacs.write(clause.as_dimacs(true).as_bytes());
        let _ = dimacs.write("\n".as_bytes());
    }
    dimacs
}

fn unsat_u8(dimacs: Vec<u8>) -> Report {
    let mut ctx = Context::from_config(Config::default());

    let _ = ctx.read_dimacs(dimacs.as_slice());
    assert!(ctx.solve().is_ok());

    ctx.report()
}

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

mod cores {

    use super::*;

    #[test]
    fn tiny_core() {
        let cfg = Config::default();
        let mut ctx = Context::from_config(cfg);

        let [p, q] = *ctx.fresh_or_max_literals(2).as_slice() else {
            panic!("Insufficient literals");
        };

        assert!(ctx.add_clause(vec![-p, q]).is_ok());

        assert!(ctx.add_clause(vec![p]).is_ok());
        assert!(ctx.add_clause(vec![-q]).is_ok());

        let result = ctx.solve();

        assert!(result.is_ok());

        assert!(matches!(ctx.report(), Report::Unsatisfiable))
    }

    #[test]
    fn tiny_core_subset() {
        let cfg = Config::default();
        let mut ctx = Context::from_config(cfg);

        let [p, q, r, s] = *ctx.fresh_or_max_literals(4).as_slice() else {
            panic!("Insufficient literals");
        };

        assert!(ctx.add_clause(vec![-p, q]).is_ok());
        assert!(ctx.add_clause(vec![r, s]).is_ok());

        assert!(ctx.add_clause(vec![p]).is_ok());
        assert!(ctx.add_clause(vec![-q]).is_ok());

        let result = ctx.solve();

        assert!(result.is_ok());

        let dimacs = core_as_dimacs(&mut ctx);
        assert_eq!(unsat_u8(dimacs), Report::Unsatisfiable);

        assert!(matches!(ctx.report(), Report::Unsatisfiable))
    }

    #[test]
    fn tiny_indirect_core() {
        let cfg = Config::default();
        let mut ctx = Context::from_config(cfg);

        let [p, q, r] = *ctx.fresh_or_max_literals(3).as_slice() else {
            panic!("Insufficient literals");
        };

        assert!(ctx.add_clause(vec![p, -q, r]).is_ok());
        assert!(ctx.add_clause(vec![p, q, r]).is_ok());

        assert!(ctx.add_clause(vec![-p, -q, r]).is_ok());
        assert!(ctx.add_clause(vec![-p, q, r]).is_ok());

        assert!(ctx.add_clause(vec![-r]).is_ok());

        let result = ctx.solve();

        assert!(result.is_ok());

        let dimacs = core_as_dimacs(&mut ctx);
        assert_eq!(unsat_u8(dimacs), Report::Unsatisfiable);

        assert!(matches!(ctx.report(), Report::Unsatisfiable))
    }

    #[test]
    fn chain_core() {
        let mut ctx = Context::from_config(Config::default());

        let [p, q, r, s, t] = *ctx.fresh_or_max_literals(5).as_slice() else {
            panic!("Insufficient literals");
        };

        assert!(ctx.add_clause(vec![-p, q]).is_ok());
        assert!(ctx.add_clause(vec![-q, r]).is_ok());
        assert!(ctx.add_clause(vec![-r, s]).is_ok());
        assert!(ctx.add_clause(vec![-s, t]).is_ok());

        assert!(ctx.add_clause(vec![p]).is_ok());
        assert!(ctx.add_clause(vec![-t]).is_ok());

        let result = ctx.solve();

        assert!(result.is_ok());

        let dimacs = core_as_dimacs(&mut ctx);
        assert_eq!(unsat_u8(dimacs), Report::Unsatisfiable);

        assert!(matches!(ctx.report(), Report::Unsatisfiable))
    }
}
