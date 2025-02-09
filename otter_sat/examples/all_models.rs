use std::collections::HashMap;

use otter_sat::{
    config::Config,
    context::Context,
    dispatch::library::report::{self},
    structures::{
        atom::Atom,
        clause::Clause,
        literal::{CLiteral, Literal},
    },
};

/// A default context is created and some sequences of variables are added.
/// A loop then feeds back the negation of any satisfying assignment to the model.
/// This loop breaks as soon as either
///  - The cumulative formula is unsatisfiable
///  - It is not possible to add an additional clause as the formula would become unsatisfiable
///  - Or, there's some error in the solver.
fn main() {
    let config = Config::default();

    let mut the_context: Context = Context::from_config(config, None);

    let mut atom_map = HashMap::<char, Atom>::default();
    // Each character in some string as a literal.
    let characters = "model".chars().collect::<Vec<_>>();
    for character in characters {
        atom_map.insert(character, the_context.fresh_or_max_atom());
    }

    let mut count = 0;

    loop {
        assert!(the_context.solve().is_ok());

        match the_context.report() {
            report::SolveReport::Satisfiable => {}
            _ => break,
        };

        count += 1;

        let last_valuation = the_context.atom_db.valuation_string();
        println!("v {count}\t {last_valuation}");

        let mut clause = Vec::new();

        for (atom, value) in the_context
            .atom_db
            .valuation_canonical()
            .iter()
            .enumerate()
            .skip(1)
        {
            if let Some(v) = value {
                clause.push(CLiteral::new(atom as Atom, !v));
            }
        }

        println!("To add: {}", clause.as_dimacs(false));

        the_context.clear_decisions();
        // std::process::exit(1);

        match the_context.add_clause(clause) {
            Ok(_) => {}
            Err(_) => break,
        };
    }

    assert_eq!(count, 2_usize.pow(atom_map.len().try_into().unwrap()));
}
