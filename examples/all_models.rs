use otter_lib::{config::Config, context::Context, types::gen::Report};

/*
A default context is created and some sequences of variables are added.
A loop then feeds back the negation of any satisfying assignment to the model.
This loop breaks as soon as either
  - The cumulative formula is unsatisfiable
  - It is not possible to add an additional clause as the formula would become unsatisfiable
  - Or, there's some error in the solver

This is not particularly efficient
 */

fn main() {
    let config = Config {
        ..Default::default()
    };

    let (tx, _) = crossbeam::channel::bounded(0);
    let mut the_context: Context = Context::from_config(config, tx);

    // Each character in some string as a literal
    let mut variables = "let's_finds_all_models".chars().collect::<Vec<_>>();
    for variable in &variables {
        assert!(the_context
            .variable_from_string(&variable.to_string())
            .is_ok())
    }

    let mut count = 0;

    loop {
        the_context.clear_decisions();
        match the_context.solve() {
            Ok(_) => {}
            Err(_) => break,
        };
        match the_context.report() {
            Report::Satisfiable => {}
            _ => break,
        };

        count += 1;

        let last_valuation = the_context.valuation_string();
        println!("v {count}\t {last_valuation}");
        let valuation_parts = last_valuation.split_whitespace();

        let mut new_valuation = String::new();
        for literal in valuation_parts {
            match literal.chars().next() {
                Some('-') => new_valuation.push_str(&literal[1..]),
                Some(_) => new_valuation.push_str(format!("-{literal}").as_str()),
                None => {}
            };
            new_valuation.push(' ');
        }

        match the_context.clause_from_string(&new_valuation) {
            Ok(()) => {}
            Err(_) => break,
        };
    }

    // Shake out any duplicate variables
    variables.sort_unstable();
    variables.dedup();

    assert_eq!(count, 2_usize.pow(variables.len().try_into().unwrap()));
}
