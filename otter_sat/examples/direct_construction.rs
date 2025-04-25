use otter_sat::{
    config::Config,
    context::Context,
    reports::Report,
    structures::{
        clause::Clause,
        literal::{CLiteral, Literal},
    },
};

fn main() {
    let mut config = Config::default();
    config.polarity_lean.value = 0.0;

    let mut ctx: Context = Context::from_config(config);
    let p = ctx.fresh_or_max_atom();
    let q = ctx.fresh_or_max_atom();

    let not_p_or_q = vec![CLiteral::new(p, false), CLiteral::new(q, true)];
    let p_or_not_q = vec![CLiteral::new(p, true), CLiteral::new(q, false)];

    let _ = ctx.add_clause(not_p_or_q);
    let _ = ctx.add_clause(p_or_not_q);

    println!("The clause database after adding ¬p ∨ q and ¬p ∨ q is:");
    for (key, clause) in ctx.clause_db.all_nonunit_clauses() {
        println!("  {} {}", key, clause.as_dimacs(false))
    }
    println!();

    println!(
        "Prior to solving the status of the formula is:  {}",
        ctx.report()
    );
    assert!(ctx.solve().is_ok());
    println!(
        "After solving the status of the formula is:     {} (with valuation: {})
",
        ctx.report(),
        ctx.valuation_strings().collect::<Vec<_>>().join(" ")
    );

    assert_eq!(ctx.value_of(p), Some(false));
    assert_eq!(ctx.value_of(q), Some(false));

    let p_error = ctx.add_clause(CLiteral::new(p, true));

    println!("p is consistent with the formula.
However, p is inconsistent with the valuation as so cannot be added to the context in its current state:
\t({p_error:?})
Though, as the formula was satisfiable, the decisions made can be cleared, allowing p to be added.
");

    ctx.clear_decisions();

    let p_ok = ctx.add_clause(CLiteral::new(p, true));

    assert!(p_ok.is_ok());
    assert_eq!(ctx.value_of(p), Some(true));

    assert!(ctx.solve().is_ok());

    println!(
        "After (re)solving the status of the formula is: {} (with valuation the valuation: {})
",
        ctx.report(),
        ctx.valuation_strings().collect::<Vec<_>>().join(" ")
    );

    assert_eq!(ctx.report(), Report::Satisfiable);

    // Likewise it is not possible to add ¬p ∨ ¬q to the formula
    let not_p_or_not_q = vec![CLiteral::new(p, false), CLiteral::new(q, false)];
    assert!(ctx.add_clause(not_p_or_not_q).is_err());

    assert_eq!(ctx.report(), Report::Satisfiable);

    println!("The clause database is now:");
    for (key, clause) in ctx.clause_db.all_unit_clauses() {
        println!("  {key} {}", clause.as_dimacs(false))
    }
    for (key, clause) in ctx.clause_db.all_nonunit_clauses() {
        println!("  {key} {}", clause.as_dimacs(false))
    }

    // It is possible to add p ∨ q to the formula
    let p_or_q = vec![CLiteral::new(p, true), CLiteral::new(q, true)];
    assert!(ctx.add_clause(p_or_q).is_ok());
}
