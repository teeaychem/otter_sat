use otter_sat::{
    config::Config,
    context::Context,
    dispatch::library::report::{self},
};

/// A default context is created and some sequences of variables are added.
/// A loop then feeds back the negation of any satisfying assignment to the model.
/// This loop breaks as soon as either
///  - The cumulative formula is unsatisfiable
///  - It is not possible to add an additional clause as the formula would become unsatisfiable
///  - Or, there's some error in the solver.
///
/// This is not particularly efficient.
// TODO: make this efficient…
fn main() {
    let config = Config::default();

    let mut the_context: Context = Context::from_config(config, None);

    // Each character in some string as a literal.
    let mut atoms = "model".chars().collect::<Vec<_>>();
    for atom in &atoms {
        assert!(the_context.atom_from_string(&atom.to_string()).is_ok())
    }

    let mut count = 0;

    loop {
        the_context.clear_choices();
        assert!(the_context.solve().is_ok());

        match the_context.report() {
            report::Solve::Satisfiable => {}
            _ => break,
        };

        count += 1;

        let last_valuation = the_context.atom_db.valuation_string();
        println!("v {count}\t {last_valuation}");
        let valuation_parts = last_valuation.split_whitespace();

        let mut new_valuation = String::new();
        for literal in valuation_parts {
            match literal.chars().next() {
                Some('-') => new_valuation.push_str(&literal[1..]),
                Some(_) => new_valuation.push_str(format!("-{literal}").as_str()),
                None => break,
            };
            new_valuation.push(' ');
        }

        let the_clause = the_context.clause_from_string(&new_valuation).unwrap();

        match the_context.add_clause(the_clause) {
            Ok(()) => {}
            Err(_) => break,
        };
    }

    // Shake out any duplicate variables as these are ignored by the context.
    atoms.sort_unstable();
    atoms.dedup();

    assert_eq!(count, 2_usize.pow(atoms.len().try_into().unwrap()));
}
