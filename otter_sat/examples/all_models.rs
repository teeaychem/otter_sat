use otter_sat::{
    config::Config,
    context::Context,
    dispatch::library::report::{self},
    structures::{
        atom::Atom,
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
    let mut context: Context = Context::from_config(Config::default(), None);

    let characters = "model".chars().collect::<Vec<_>>();
    let mut atom_count: u32 = 0;
    for _character in &characters {
        match context.fresh_atom() {
            Ok(_) => atom_count += 1,
            Err(_) => {
                panic!("Atom limit exhausted.")
            }
        }
    }

    let mut count = 0;

    loop {
        assert!(context.solve().is_ok());

        match context.report() {
            report::SolveReport::Satisfiable => {}
            _ => break,
        };

        count += 1;

        let last_valuation = context
            .atom_db
            .valuation_isize()
            .iter()
            .map(|a| {
                let c = &characters[a.unsigned_abs() - 1];
                match a.is_positive() {
                    true => format!(" {c}"),
                    false => format!("-{c}"),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        println!("v {count}\t {last_valuation}");

        let mut clause = Vec::new();

        for (atom, value) in context
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

        context.clear_decisions();

        match context.add_clause(clause) {
            Ok(_) => {}
            Err(_) => break,
        };
    }

    assert_eq!(count, 2_usize.pow(atom_count));
}
