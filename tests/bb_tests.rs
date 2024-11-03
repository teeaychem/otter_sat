#![allow(unused_imports)]

use otter_lib::{
    config::Config,
    context::{self, Context, Report},
    structures::{
        literal::{Literal, LiteralSource},
        variable::list::VariableList,
    },
};

#[test]
fn test_one_literal() {
    let mut the_context = Context::default_config(Config::default());
    assert!(the_context.clause_from_string("p").is_ok());
    assert!(the_context.solve().is_ok());
    assert_eq!(the_context.report(), Report::Satisfiable)
}

#[test]
fn test_two_conflict() {
    let mut the_context = Context::default_config(Config::default());
    let _ = the_context.clause_from_string("p q");
    let _ = the_context.clause_from_string("-p -q");
    let _ = the_context.clause_from_string("p -q");
    let _ = the_context.clause_from_string("-p q");
    assert!(the_context.solve().is_ok());
    assert!(matches!(the_context.report(), Report::Unsatisfiable))
}

#[test]
fn test_one_assumption() {
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
