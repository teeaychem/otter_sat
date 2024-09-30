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

    /// Print stats
    #[arg(short, long, default_value_t = false)]
    stats: bool,

    /// Print an assignment if formula is satisfiable
    #[arg(short, long, default_value_t = false)]
    assignment: bool,

    /// Print core on unsat
    #[arg(short, long, default_value_t = false)]
    core: bool,

    /// Specify required glue strength
    #[arg(short, long, default_value_t = 2)]
    glue_strength: usize,

    /// Resolution stopping criteria
    #[arg(long, default_value_t = String::from("FirstUIP"))]
    stopping_criteria: String,
}

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    let args = Args::parse();

    let config = SolveConfig {
        stats: args.stats,
        show_assignment: args.assignment,
        glue_strength: args.glue_strength,
        core: args.core,
        analysis: 3,
        stopping_criteria: {
            let critera = args.stopping_criteria;
            if critera == "FirstUIP" {
                StoppingCriteria::FirstAssertingUIP
            } else if critera == "None" {
                StoppingCriteria::None
            } else {
                panic!("Unknown stopping critera")
            }
        },
        break_on_first: true,
        multi_jump_max: true,
    };

    if let Ok(contents) = fs::read_to_string(args.file) {
        if let Ok(formula) = Formula::from_dimacs(&contents) {
            let mut the_solve = Solve::from_formula(&formula, config);

            let (result, stats) = the_solve.implication_solve();
            if the_solve.config.stats {
                println!("{stats}");
            }
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
