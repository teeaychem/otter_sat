use otter_sat::{
    config::Config,
    context::Context,
    dispatch::SolveReport,
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
///
/// This is not particularly efficient.
fn main() {
    let config = Config::default();

    let mut the_context: Context = Context::from_config(config);

    // The representation of an atom will be given by the corresponding index in this map…
    let mut atom_map = Vec::<char>::default();
    // … though as atoms are positive integers, an initial element is added as an offset.
    atom_map.push('䷼');

    let character_string = "model";
    let last_character_atom = character_string.len().try_into().unwrap();

    // The atoms in a context are a contiguous slice of the natural numbers, starting at 1.
    // So, ensuring an atom for the last character in the map entails there will be atoms for present for all other characters.
    assert!(the_context.ensure_atom(last_character_atom).is_ok());

    // Each character in some string as a literal.
    for character in character_string.chars() {
        atom_map.push(character);
    }

    let plural_atom = the_context.fresh_atom().unwrap();
    let _ = the_context.add_assumption(CLiteral::new(plural_atom, true));
    atom_map.push('s');

    let mut count = 0;

    loop {
        assert!(the_context.solve().is_ok());

        match the_context.report() {
            SolveReport::Satisfiable => {}
            _ => break,
        };

        count += 1;

        let last_valuation = the_context.atom_db.valuation();
        let mut valuation_as_chars = Vec::default();
        for (atom, value) in last_valuation.atom_value_pairs() {
            let character = atom_map[atom as usize];
            match value {
                Some(true) => valuation_as_chars.push(format!(" {character}")),
                Some(false) => valuation_as_chars.push(format!("-{character}")),
                None => {}
            }
        }

        println!("v {count}\t {}", valuation_as_chars.join(" "));

        let mut clause = Vec::new();

        for (atom, value) in the_context
            .atom_db
            .valuation_canonical()
            .iter()
            .enumerate()
            .skip(1)
        {
            match value {
                Some(v) => {
                    clause.push(CLiteral::new(atom as Atom, !v));
                }
                None => {}
            }
        }

        the_context.clear_decisions();
        // std::process::exit(1);

        match the_context.add_clause(clause) {
            Ok(_) => {}
            Err(_) => break,
        };
    }

    assert_eq!(
        count,
        2_usize.pow(atom_map.len().saturating_sub(2).try_into().unwrap())
    );
}
