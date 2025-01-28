use otter_sat::{
    config::Config,
    context::Context,
    dispatch::library::report,
    structures::{
        clause::Clause,
        literal::{cLiteral, Literal},
    },
};

fn main() {
    let config = Config {
        polarity_lean: 0.0, // Always choose to value a variable false
        ..Default::default()
    };

    let mut the_context: Context = Context::from_config(config, None);
    let p = the_context.fresh_atom().unwrap();
    let q = the_context.fresh_atom().unwrap();

    let not_p_or_q = vec![cLiteral::new(p, false), cLiteral::new(q, true)];
    let p_or_not_q = vec![cLiteral::new(p, true), cLiteral::new(q, false)];

    // made clauses must be added to the context:
    for (i, clause) in the_context.clause_db.all_nonunit_clauses().enumerate() {
        println!("  ? {i}: {}", clause.as_dimacs(false))
    }

    let _ = the_context.add_clause(not_p_or_q);
    let _ = the_context.add_clause(p_or_not_q);

    println!("The clause database after adding ¬p ∨ q and ¬p ∨ q is:");
    for clause in the_context.clause_db.all_nonunit_clauses() {
        println!("  C {}", clause.as_dimacs(false))
    }
    println!();

    let status = the_context.report();
    println!("Prior to solving the status of the formula is:  {status}");
    assert!(the_context.solve().is_ok());
    let status = the_context.report();
    let valuation = the_context.atom_db.valuation_string();
    println!(
        "After solving the status of the formula is:     {status} (with valuation: {valuation})"
    );
    println!();

    assert_eq!(the_context.atom_db.value_of(p), Some(false));
    assert_eq!(the_context.atom_db.value_of(q), Some(false));

    let p_error = the_context.add_clause(cLiteral::new(p, true));

    println!("p is incompatible with the valuation as so cannot be added to the context ({p_error:?}) without clearing decisions made…
");

    the_context.clear_decisions();

    let _p_ok = the_context.add_clause(cLiteral::new(p, true));

    assert_eq!(the_context.atom_db.value_of(p), Some(true));

    assert!(the_context.solve().is_ok());

    println!(
        "After (re)solving the status of the formula is: {status} (with valuation the valuation: {valuation})
"
    );

    assert_eq!(the_context.report(), report::SolveReport::Satisfiable);

    // Likewise it is not possible to add ¬p ∨ ¬q to the formula
    let not_p_or_not_q = vec![cLiteral::new(p, false), cLiteral::new(q, false)];
    assert!(the_context.add_clause(not_p_or_not_q).is_err());

    assert_eq!(the_context.report(), report::SolveReport::Satisfiable);

    // todo: update with unit clauses
    println!("The clause database is now:");
    for clause in the_context.clause_db.all_nonunit_clauses() {
        println!("  C {}", clause.as_dimacs(false))
    }

    // It is possible to add p ∨ q to the formula
    let p_or_q = vec![cLiteral::new(p, true), cLiteral::new(q, true)];
    assert!(the_context.add_clause(p_or_q).is_ok());
}
