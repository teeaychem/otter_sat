/*
A toy example of interacting with the IPASIR API from Rust.

The example builds a context with the number of atoms given as an input, and exhausts all models by the negation of each valuation found as a clause.

The addition_hook prints an ascii character for each clause added to the formula by the context (this does not include original clauses) and updates a pointer to the longest clause found.

To run the example (e.g.): cargo run --profile release --example ipasir_conflict 10
 */

use std::ffi::c_void;

use otter_sat::{
    config::Config,
    context::Context,
    dispatch::library::report,
    ipasir::IpasirCallbacks,
    structures::{clause::IntClause, literal::IntLiteral},
};

extern "C" fn addition_hook(data: *mut c_void, clause: *mut i32) {
    let mut length = 0;
    loop {
        let literal = unsafe { clause.offset(length) };
        if unsafe { *literal } == 0 {
            break;
        } else {
            length += 1;
        }
    }

    unsafe {
        if length > (*(data as *mut i32)).try_into().unwrap() {
            std::ptr::write(data as *mut i32, length.try_into().unwrap());
        } else {
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

    let callbacks = IpasirCallbacks {
        learn_callback: Some(addition_hook),
        addition_data: length as *mut c_void,
        addition_length: atom_count,
        ..Default::default()
    };

    let mut the_context: Context = Context::from_config(config, None);
    the_context.ipasir_callbacks = Some(callbacks);

    for _ in 0..atom_count {
        let _ = the_context.fresh_atom();
    }

    let mut models_found = 0;

    loop {
        assert!(the_context.solve().is_ok());

        match the_context.report() {
            report::SolveReport::Satisfiable => {}
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
