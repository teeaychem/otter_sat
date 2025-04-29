use otter_sat::{
    config::Config,
    context::Context,
    reports::Report,
    structures::{
        atom::Atom,
        literal::{CLiteral, Literal},
        valuation::Valuation,
    },
};

/// A default context is created and some sequences of variables are added.
/// A loop then feeds back the negation of any satisfying assignment to the model.
/// This loop breaks as soon as either
///  - The cumulative formula is unsatisfiable
///  - It is not possible to add an additional clause as the formula would become unsatisfiable
///  - Or, there's some error in the solver.
fn main() {
    // The context in which a solve takes place.
    let mut context: Context = Context::from_config(Config::default());

    // Atoms will be represented by characters of some string.
    let characters = "model".chars().collect::<Vec<_>>();
    let mut atom_count: u32 = 0;

    // Each call to fresh_atom expands the context to include a fresh (new) atom.
    // Atoms form a contiguous range from 1 to some limit.
    for _character in &characters {
        match context.fresh_atom() {
            Ok(_) => atom_count += 1,
            Err(_) => {
                panic!("Atom limit exhausted.")
            }
        }
    }

    let mut model_count = 0;

    while let Ok(Report::Satisfiable) = context.solve() {
        model_count += 1;

        let mut valuation_representation = String::new();

        // To exclude the current valuation, the negation of the current valuation is added as a clause.
        // As valuations are conjunctions and clauses disjunctions, this may be done by negating each literal.
        let mut exclusion_clause = Vec::new();

        // The context provides an iterator over (atom, value) pairs.
        // Though every non-constant atom has a value in this model, this avoids handling the no value option.
        for (atom, value) in context.assignment().atom_valued_pairs() {
            // As atoms begin at 1, a step back is required to find the appropriate character.
            match value {
                true => valuation_representation.push(' '),
                false => valuation_representation.push('-'),
            }
            valuation_representation.push(characters[(atom as usize) - 1]);
            valuation_representation.push(' ');

            exclusion_clause.push(CLiteral::new(atom as Atom, !value));
        }

        valuation_representation.pop();
        println!("{model_count}\t {}", valuation_representation);

        // After a solve, the context is refreshed to clear any decisions made.
        // Learnt clauses remain, though any assumptions made are also removed.
        context.refresh();

        match context.add_clause(exclusion_clause) {
            Ok(_) => {}
            Err(_) => break,
        };
    }

    assert_eq!(model_count, 2_usize.pow(atom_count));
}
