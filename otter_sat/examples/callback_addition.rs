/*
A toy example of interacting with the IPASIR API from Rust.

The example builds a context with the number of atoms given as an input, and exhausts all models by the negation of each valuation found as a clause.

The addition_hook prints an ascii character for each clause added to the formula by the context (this does not include original clauses) and updates a pointer to the longest clause found.

To run the example (e.g.): cargo run --profile release --example ipasir_conflict 10
 */

use otter_sat::{
    config::Config,
    context::Context,
    db::clause::db_clause::dbClause,
    reports::Report,
    structures::{
        clause::{ClauseSource, IntClause},
        literal::IntLiteral,
    },
};

fn addition_hook(clause: &dbClause, _: &ClauseSource) {
    let length = clause.len();

    match length {
        1 => {
            print!("!")
        }
        2 => {
            print!("'")
        }
        l if l < 5 => {
            print!("*")
        }
        l if l < 7 => {
            print!(":")
        }
        l if l < 9 => {
            print!("`")
        }
        _ => {
            print!(".")
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

    let length: *mut i32 = Box::into_raw(Box::new(0_i32));

    let config = Config::default();

    let mut the_context: Context = Context::from_config(config);
    the_context.set_callback_addition(Box::new(addition_hook));

    for _ in 0..atom_count {
        let _ = the_context.fresh_atom();
    }

    let mut models_found = 0;

    loop {
        assert!(the_context.solve().is_ok());

        match the_context.report() {
            Report::Satisfiable => {}
            _ => break,
        };

        models_found += 1;

        let clause: IntClause = the_context
            .atom_db
            .valuation_canonical()
            .iter()
            .enumerate()
            .skip(1)
            .flat_map(|(a, v)| match v {
                Some(false) => Some(a as IntLiteral),
                Some(true) => Some(-(a as IntLiteral)),
                None => None,
            })
            .collect();

        the_context.clear_decisions();

        match the_context.add_clause(clause) {
            Ok(_) => {}
            Err(_) => break,
        };
    }

    println!();
    println!("Models found {models_found}");
    unsafe {
        println!("Longest clause learnt {}", *length);
    }
}
