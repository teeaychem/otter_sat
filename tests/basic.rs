use otter_lib::{config::Config, context::Context, dispatch::SolveReport};

mod basic {

    use super::*;
    #[test]
    fn one_literal() {
        let (tx, _) = crossbeam::channel::bounded(0);
        let mut the_context = Context::from_config(Config::default(), tx);
        assert!(the_context.clause_from_string("p").is_ok());
        assert!(the_context.solve().is_ok());
        assert_eq!(the_context.report(), SolveReport::Satisfiable)
    }

    #[test]
    fn conflict() {
        let (tx, _) = crossbeam::channel::bounded(0);
        let mut the_context = Context::from_config(Config::default(), tx);
        assert!(the_context.clause_from_string("p q").is_ok());
        assert!(the_context.clause_from_string("-p -q").is_ok());
        assert!(the_context.clause_from_string("p -q").is_ok());
        assert!(the_context.clause_from_string("-p q").is_ok());
        assert!(the_context.solve().is_ok());
        assert!(matches!(the_context.report(), SolveReport::Unsatisfiable))
    }

    #[test]
    fn assumption() {
        let (tx, _) = crossbeam::channel::bounded(0);
        let mut the_context = Context::from_config(Config::default(), tx);

        assert!(the_context.clause_from_string("p q").is_ok());

        let not_p = the_context.literal_from_string("-p").expect("oh");

        assert!(the_context.assume(not_p).is_ok());
        assert!(the_context.solve().is_ok());
        assert_eq!(the_context.report(), SolveReport::Satisfiable);

        let the_valuation = the_context.valuation_string();
        assert!(the_valuation.contains("-p"));
        assert!(the_valuation.contains("q"));
    }

    #[test]
    fn duplicates() {
        let (tx, _) = crossbeam::channel::bounded(0);
        let mut the_context = Context::from_config(Config::default(), tx);
        assert!(the_context.clause_from_string("p q q").is_ok());
        let database = the_context.clause_database();
        assert_eq!(database.len(), 1);
        assert_eq!(database.first().unwrap(), "p q 0");
    }

    #[test]
    fn tautology_skip() {
        let (tx, _) = crossbeam::channel::bounded(0);
        let mut the_context = Context::from_config(Config::default(), tx);
        assert!(the_context.clause_from_string("p q -p").is_ok());
        let database = the_context.clause_database();
        assert_eq!(database.len(), 0);
    }

    // TOOD: Incremental tests based on example
}
