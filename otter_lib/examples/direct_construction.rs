use otter_lib::{
    config::Config, context::Context, dispatch::library::report, structures::clause::Clause,
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

    let mut the_context: Context = Context::from_config(config, None);

    let not_p_or_q = the_context.clause_from_string("-p q");
    assert!(not_p_or_q.is_ok());

    let not_p_or_q = not_p_or_q.expect("failed to build clause");
    let not_p_or_q_internal_string = not_p_or_q.as_string();
    let not_p_or_q_external_string = not_p_or_q.as_dimacs(&the_context.atom_db, false);
    println!(
        "
Representations of: ¬p ∨ q

- Debug           {not_p_or_q:?}
- Internal string {not_p_or_q_internal_string}
- External string {not_p_or_q_external_string}
"
    );

    let p_or_not_q = the_context.clause_from_string("p -q").expect("make failed");

    let p_variable = the_context.atom_db.atom_representation("p").unwrap();
    let q_variable = the_context.atom_db.atom_representation("q").unwrap();
    let p = the_context.literal_db.make_literal(p_variable, true);
    let not_q = the_context.literal_db.make_literal(q_variable, false);

    assert_eq!(p_or_not_q, vec![p, not_q]);

    // made clauses must be added to the context:
    for (i, clause) in the_context.clause_db.all_nonunit_clauses().enumerate() {
        println!("  ? {i}: {}", clause.as_dimacs(&the_context.atom_db, false))
    }

    let _ = the_context.add_clause(not_p_or_q);
    let _ = the_context.add_clause(p_or_not_q);

    println!("The clause database after adding ¬p ∨ q and ¬p ∨ q is:");
    for clause in the_context.clause_db.all_nonunit_clauses() {
        println!("  C {}", clause.as_dimacs(&the_context.atom_db, false))
    }
    println!();

    let status = the_context.report();
    println!("Prior to solving the status of 𝐅 is:  {status}");
    assert!(the_context.solve().is_ok());
    let status = the_context.report();
    let valuation = the_context.valuation_string();
    println!("After solving the status of 𝐅 is:     {status} (with valuation 𝐕: {valuation})");
    println!();

    assert_eq!(value_of("p", &the_context), Some(false));
    assert_eq!(value_of("q", &the_context), Some(false));

    let p_clause = the_context.clause_from_string("p").unwrap();
    let p_error = the_context.add_clause(p_clause);

    println!("p is incompatible with 𝐕 as so cannot be added to the context ({p_error:?}) without clearing choices made…
");

    the_context.clear_choices();

    let p_clause = the_context.clause_from_string("p").unwrap();
    let _p_ok = the_context.add_clause(p_clause);

    assert_eq!(value_of("p", &the_context), Some(true));

    assert!(the_context.solve().is_ok());

    println!(
        "After (re)solving the status of 𝐅 is: {status} (with valuation 𝐕: {valuation})
"
    );

    assert_eq!(the_context.report(), report::Solve::Satisfiable);

    // Likewise it is not possible to add ¬p ∨ ¬q to 𝐅
    let clause_np_nq = the_context.clause_from_string("-p -q").unwrap();
    assert!(the_context.add_clause(clause_np_nq).is_err());

    assert_eq!(the_context.report(), report::Solve::Satisfiable);

    // todo: update with unit clauses
    println!("The clause database is now:");
    for clause in the_context.clause_db.all_nonunit_clauses() {
        println!("  C {}", clause.as_dimacs(&the_context.atom_db, false))
    }

    // It is possible to add p ∨ q to 𝐅
    let clause_p_q = the_context.clause_from_string("p q").unwrap();
    assert!(the_context.add_clause(clause_p_q).is_ok());
}
