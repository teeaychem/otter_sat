mod decision_levels {
    use otter_sat::{config::Config, context::Context, structures::literal::Literal};

    #[test]
    fn two_stacked() {
        let mut cfg = Config::default();
        cfg.atom_db.stacked_assumptions.value = true;
        let mut ctx = Context::from_config(cfg);

        let [p, q, r, s] = *ctx.fresh_or_max_literals(4).as_slice() else {
            panic!("Insufficient literals");
        };

        let _ = ctx.add_clause(vec![-p, q]);
        let _ = ctx.add_clause(vec![-r, -s]);

        let _ = ctx.assert_assumptions(vec![p, r]);

        assert!(ctx.atom_db.trail.assumption_is_made());
        assert!(ctx.atom_db.trail.initial_decision_level == 2);

        assert!(ctx.atom_db.value_of(q.atom()) == Some(true));
        assert!(ctx.atom_db.value_of(s.atom()) == Some(false));
    }

    #[test]
    fn two_unstacked() {
        let mut cfg = Config::default();
        cfg.atom_db.stacked_assumptions.value = false;
        let mut ctx = Context::from_config(cfg);

        let [p, q, r, s] = *ctx.fresh_or_max_literals(4).as_slice() else {
            panic!("Insufficient literals");
        };

        let _ = ctx.add_clause(vec![-p, q]);
        let _ = ctx.add_clause(vec![-r, -s]);

        let _ = ctx.assert_assumptions(vec![p, r]);

        assert!(ctx.atom_db.trail.assumption_is_made());
        assert!(ctx.atom_db.trail.initial_decision_level == 1);

        assert!(ctx.atom_db.value_of(q.atom()).is_none());
        assert!(ctx.atom_db.value_of(s.atom()).is_none());
    }

    #[test]
    fn proven_backjump() {
        let cfg = Config::default();
        let mut ctx = Context::from_config(cfg);

        let [p, q, r, s] = *ctx.fresh_or_max_literals(4).as_slice() else {
            panic!("Insufficient literals");
        };

        let _ = ctx.add_clause(vec![-p, q]);
        let _ = ctx.add_clause(vec![p]);
        let _ = ctx.add_clause(vec![-r, -s]);

        let _ = ctx.propagate_unless_error();

        let _ = ctx.assert_assumptions(vec![r]);

        assert!(ctx.atom_db.trail.assumption_is_made());
        assert!(ctx.atom_db.trail.initial_decision_level == 1);

        assert!(ctx.atom_db.value_of(q.atom()) == Some(true));
        assert!(ctx.atom_db.value_of(s.atom()) == Some(false));

        ctx.backjump(0);

        assert!(ctx.atom_db.value_of(q.atom()) == Some(true));
        assert!(ctx.atom_db.value_of(s.atom()).is_none());
    }
}
