#![allow(dead_code)]

use clap::Parser;
use std::fs;
use structures::solve::StoppingCriteria;
mod io;
mod procedures;
mod structures;

use crate::structures::solve::{Solve, SolveConfig, SolveResult};
use crate::structures::Formula;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// file to parse
    #[arg(short, long)]
    file: String,

    /// Print core on unsat
    #[arg(short, long, default_value_t = false)]
    core: bool,
}

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    let args = Args::parse();

    let config = SolveConfig {
        core: args.core,
        analysis: 3,
        min_glue_strength: 2,
        stopping_criteria: StoppingCriteria::FirstAssertingUIP,
    };

    if let Ok(contents) = fs::read_to_string(args.file) {
        if let Ok(formula) = Formula::from_dimacs(&contents) {
            let mut the_solve = Solve::from_formula(&formula, config);

            let (result, stats) = the_solve.implication_solve();
            println!("{stats}");
            match result {
                SolveResult::Unsatisfiable => {
                    println!("s UNSATISFIABLE");
                    std::process::exit(00);
                }
                SolveResult::Satisfiable => {
                    println!("s SATISFIABLE");
                    std::process::exit(10);
                }
                SolveResult::Unknown => {
                    println!("s Unkown");
                    std::process::exit(20);
                }
            }
        }
    }
}
