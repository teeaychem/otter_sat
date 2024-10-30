#![allow(unused_imports)]

use otter_lib::{
    config::Config,
    context::{self, Context},
    structures::{
        literal::{Literal, Source},
        variable::list::VariableList,
    },
};

#[test]
fn test_one_literal() {
    let mut the_context = Context::default_config(Config::default());
    let _ = the_context.clause_from_string("p");
    let _the_result = the_context.solve();
    assert_eq!(the_context.status, context::Status::AllAssigned)
}

#[test]
fn test_two_conflict() {
    let mut the_context = Context::default_config(Config::default());
    let _ = the_context.clause_from_string("p q");
    let _ = the_context.clause_from_string("-p -q");
    let _ = the_context.clause_from_string("p -q");
    let _ = the_context.clause_from_string("-p q");
    let _the_result = the_context.solve();
    assert!(matches!(the_context.status, context::Status::NoSolution(_)))
}

#[test]
fn test_one_assumption() {
    let mut the_context = Context::default_config(Config::default());
    let _ = the_context.clause_from_string("p q");
    let not_p = the_context.literal_from_string("-p").expect("oh");
    let _ = the_context.assume_literal(not_p);
    let _the_result = the_context.solve();
    assert_eq!(the_context.status, context::Status::AllAssigned);
    let the_valuation = the_context.variables().as_internal_string();
    assert!(the_valuation.contains("-p"));
    assert!(the_valuation.contains("q"));
}
