use otter_lib::{
    config::Config,
    context::{Context, Report},
};

mod basic {
    use super::*;
    #[test]
    fn one_literal() {
        let mut the_context = Context::default_config(Config::default());
        assert!(the_context.clause_from_string("p").is_ok());
        assert!(the_context.solve().is_ok());
        assert_eq!(the_context.report(), Report::Satisfiable)
    }

    #[test]
    fn conflict() {
        let mut the_context = Context::default_config(Config::default());
        assert!(the_context.clause_from_string("p q").is_ok());
        assert!(the_context.clause_from_string("-p -q").is_ok());
        assert!(the_context.clause_from_string("p -q").is_ok());
        assert!(the_context.clause_from_string("-p q").is_ok());
        assert!(the_context.solve().is_ok());
        assert!(matches!(the_context.report(), Report::Unsatisfiable))
    }

    #[test]
    fn assumption() {
        let mut the_context = Context::default_config(Config::default());

        assert!(the_context.clause_from_string("p q").is_ok());

        let not_p = the_context.literal_from_string("-p").expect("oh");

        assert!(the_context.assume(not_p).is_ok());
        assert!(the_context.solve().is_ok());
        assert_eq!(the_context.report(), Report::Satisfiable);

        let the_valuation = the_context.valuation_string();
        assert!(the_valuation.contains("-p"));
        assert!(the_valuation.contains("q"));
    }

    #[test]
    fn duplicates() {
        let mut the_context = Context::default_config(Config::default());
        assert!(the_context.clause_from_string("p q q").is_ok());
        let database = the_context.clause_database();
        assert_eq!(database.len(), 1);
        assert_eq!(database.first().unwrap(), "p q 0");
    }

    #[test]
    fn tautology_skip() {
        let mut the_context = Context::default_config(Config::default());
        assert!(the_context.clause_from_string("p q -p").is_ok());
        let database = the_context.clause_database();
        assert_eq!(database.len(), 0);
    }

    #[test]
    fn incremental_basic() {
        let mut the_context = Context::default_config(Config::default());

        assert!(the_context.clause_from_string("p q r").is_ok());
        assert!(the_context.solve().is_ok());

        let the_valuation = the_context.valuation_string();
        let p_false = the_valuation.contains("-p");
        let q_false = the_valuation.contains("-q");

        assert!(p_false && q_false);

        the_context.clear_decisions();

        assert!(the_context.clause_from_string("p q").is_ok());
        assert!(the_context.solve().is_ok());

        let the_valuation = the_context.valuation_string();

        let p_true = the_valuation.contains("p") && !the_valuation.contains("-p");
        let q_true = the_valuation.contains("q") && !the_valuation.contains("-q");

        assert!(p_true || q_true);
    }

    #[test]
    fn incremental_basic_two() {
        let mut the_context = Context::default_config(Config::default());

        assert!(the_context.clause_from_string("p q").is_ok());
        assert!(the_context.solve().is_ok());

        let the_valuation = the_context.valuation_string();
        let p_false = the_valuation.contains("-p");

        assert!(p_false);

        the_context.clear_decisions();

        assert!(the_context.clause_from_string("p").is_ok());
        assert!(the_context.solve().is_ok());

        let the_valuation = the_context.valuation_string();

        let p_true = the_valuation.contains("p") && !the_valuation.contains("-p");

        assert!(p_true);
    }
}
