use otter_lib::{
    config::Config,
    context::Context,
    dispatch::library::report::{self},
};

mod basic {

    use super::*;
    #[test]
    fn one_literal() {
        let mut the_context = Context::from_config(Config::default(), None);
        assert!(the_context.clause_from_string("p").is_ok());
        assert!(the_context.solve().is_ok());
        assert_eq!(the_context.report(), report::Solve::Satisfiable)
    }

    #[test]
    fn conflict() {
        let mut the_context = Context::from_config(Config::default(), None);
        assert!(the_context.clause_from_string("p q").is_ok());
        assert!(the_context.clause_from_string("-p -q").is_ok());
        assert!(the_context.clause_from_string("p -q").is_ok());
        assert!(the_context.clause_from_string("-p q").is_ok());
        assert!(the_context.solve().is_ok());
        assert!(matches!(the_context.report(), report::Solve::Unsatisfiable))
    }

    #[test]
    fn assumption() {
        let mut the_context = Context::from_config(Config::default(), None);

        assert!(the_context.clause_from_string("p q").is_ok());

        let not_p = the_context.literal_from_string("-p").expect("oh");

        assert!(the_context.assume(not_p).is_ok());
        assert!(the_context.solve().is_ok());
        assert_eq!(the_context.report(), report::Solve::Satisfiable);

        let the_valuation = the_context.valuation_string();
        assert!(the_valuation.contains("-p"));
        assert!(the_valuation.contains("q"));
    }

    #[test]
    fn duplicates() {
        let mut the_context = Context::from_config(Config::default(), None);
        assert!(the_context.clause_from_string("p q q").is_ok());
        let database = the_context.clause_database();
        assert_eq!(database.len(), 1);
        assert_eq!(database.first().unwrap(), "p q 0");
    }

    #[test]
    fn tautology_skip() {
        let mut the_context = Context::from_config(Config::default(), None);
        assert!(the_context.clause_from_string("p q -p").is_ok());
        let database = the_context.clause_database();
        assert_eq!(database.len(), 0);
    }

    // TOOD: Incremental tests based on example
}
