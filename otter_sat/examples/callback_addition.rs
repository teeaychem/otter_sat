/*
A toy example of interacting with the IPASIR API from Rust.

The example builds a context with the number of atoms given as an input, and exhausts all models by the negation of each valuation found as a clause.

The addition_hook prints an ascii character for each clause added to the formula by the context (this does not include original clauses) and updates a pointer to the longest clause found.

To run the example (e.g.): cargo run --profile release --example ipasir_conflict 10
 */

use otter_sat::{
    config::Config,
    context::Context,
    db::clause::db_clause::DBClause,
    reports::Report,
    structures::{
        clause::{ClauseSource, IntClause},
        literal::IntLiteral,
        valuation::Valuation,
    },
};

fn addition_hook(clause: &DBClause, _: &ClauseSource) {
    let length = clause.len();

    match length {
        1 => {
            print!("!")
        }

        2 => {
            print!("'")
        }

        l => {
            if l < 5 {
                print!("*")
            } else if l < 7 {
                print!(":")
            } else if l < 9 {
                print!("`")
            } else {
                print!(".")
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <atom_count>", args[0]);
        std::process::exit(-1);
    }
    let atom_count = args[1].parse::<usize>().expect("?");

    let config = Config::default();

    let mut ctx: Context = Context::from_config(config);
    ctx.set_callback_addition(Box::new(addition_hook));

    for _ in 0..atom_count {
        let _ = ctx.fresh_atom();
    }

    let mut models_found = 0;

    loop {
        assert!(ctx.solve().is_ok());

        match ctx.report() {
            Report::Satisfiable => {}

            Report::Unknown | Report::Unsatisfiable => break,
        };

        models_found += 1;

        let clause: IntClause = ctx
            .valuation()
            .atom_valued_pairs()
            .map(|(a, v)| match v {
                false => a as IntLiteral,
                true => -(a as IntLiteral),
            })
            .collect();

        ctx.clear_decisions();

        match ctx.add_clause(clause) {
            Ok(_) => {}
            Err(_) => break,
        };
    }

    println!();
    println!("Models found {models_found}");
}
