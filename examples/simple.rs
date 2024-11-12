use otter_lib::{
    config::Config,
    context::Context,
    dispatch::report::{self},
};

fn value_of(variable: &str, context: &Context) -> Option<bool> {
    let mut the_value = None;
    if context.valuation_string().contains(variable) {
        the_value = Some(true)
    }
    if context
        .valuation_string()
        .contains(format!("-{variable}").as_str())
    {
        the_value = Some(false)
    }
    the_value
}

fn main() {
    let config = Config {
        polarity_lean: 0.0, // Always choose to value a variable false
        ..Default::default()
    };

    let (tx, _rx) = crossbeam::channel::unbounded();
    let mut the_context: Context = Context::from_config(config, tx);

    let not_p_or_q = "-p q";
    let p_or_not_q = "p -q";

    assert!(the_context.clause_from_string(not_p_or_q).is_ok());
    assert!(the_context.clause_from_string(p_or_not_q).is_ok());

    println!(
        "
To begin, {not_p_or_q} and {p_or_not_q} have been added to a context.
We can check the database to verify this:\n
{}",
        the_context.clause_database().join("\n")
    );

    println!("
Still, without doing anything else the context has not established whether the formula consisting of these clauses is satisfiable.

Specifically, the context reports the status of the database is: {}",
        the_context.report(),
    );

    assert!(the_context.solve().is_ok());

    println!(
        "
After calling `solve` satisfiability is known!
The report is: {}

We can inspect the valuation found to check this…
{}",
        the_context.report(),
        the_context.valuation_string()
    );

    assert_eq!(value_of("p", &the_context), Some(false));
    assert_eq!(value_of("q", &the_context), Some(false));

    let p_clause = "p";

    println!("
The context can be extended, but first we need to clear any decisions.
For example, `{p_clause}` is compatible with the clause database, but not with the current valuation of `-{p_clause}`.
So, some error is returned when attempting to add {p_clause}.");

    let p_error = the_context.clause_from_string(p_clause);
    assert!(p_error.is_err());

    the_context.clear_decisions();

    println!(
        "
After clearing the decisions there is no valuation:
{}
So, it's safe to add the clause {p_clause}",
        the_context.valuation_string()
    );

    assert!(the_context.clause_from_string(p_clause).is_ok());

    assert_eq!(value_of("p", &the_context), Some(true));

    println!(
        "
If a literal must be true, it is considered proven.
For example, {p_clause} is proven after adding {p_clause}

This can be seen in the database of proven literals:
{}",
        the_context.proven_literal_database().join("\n")
    );

    assert!(the_context.solve().is_ok());

    println!(
        "
The valuation is now:
{}",
        the_context.valuation_string()
    );

    assert_eq!(value_of("q", &the_context), Some(true));
    assert_eq!(the_context.report(), report::Solve::Satisfiable);

    let an_error = the_context.clause_from_string("-p -q");
    assert!(an_error.is_err());

    the_context.clear_decisions();

    assert!(the_context.solve().is_ok());

    assert_eq!(the_context.report(), report::Solve::Satisfiable);
}
