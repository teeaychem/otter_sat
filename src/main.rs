#![allow(dead_code)]

use clap::Parser;
use std::fs;
use structures::solve::{ExplorationPriority, StoppingCriteria};
mod io;
mod procedures;
mod structures;

use crate::structures::solve::{Solve, SolveConfig, SolveResult};
use crate::structures::Formula;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The DIMACS form CNF file to parse
    #[arg(short, long)]
    file: String,

    /// Display stats
    #[arg(short, long, default_value_t = false)]
    stats: bool,

    /// Display an assignment on SAT
    #[arg(short, long, default_value_t = false)]
    assignment: bool,

    /// Display an unsatisfiable core on UNSAT
    #[arg(short, long, default_value_t = false)]
    core: bool,

    /// Required glue strength
    #[arg(short, long, default_value_t = 2)]
    glue_strength: usize,

    /// Resolution stopping criteria
    #[arg(long, default_value_t = String::from("FirstUIP"))]
    stopping_criteria: String,

    /// Conflict priority
    #[arg(long, default_value_t = String::from("Default"))]
    exploration_priority: String,
}

fn main() {
    match log4rs::init_file("config/log4rs.yaml", Default::default()) {
        Ok(_) => log::trace!("Log find loaded"),
        Err(e) => {
            log::error!("{e:?}")
        }
    }

    let args = Args::parse();

    if let Ok(contents) = fs::read_to_string(&args.file) {
        match Formula::from_dimacs(&contents) {
            Ok(formula) => {
                let config = config_builder(&args);
                if config.stats {
                    println!("c Parsing formula from file: {}", args.file);
                    println!(
                        "c Parsed formula with {} variables and {} clauses",
                        formula.vars().len(),
                        formula.clauses().count()
                    );
                }
                log::trace!("Formula processed");
                let mut the_solve = Solve::from_formula(&formula, config);
                log::trace!("Solve initialised");

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
            Err(e) => panic!("{e:?}"),
        }
    } else {
        println!("Error reading file")
    }
}

fn config_builder(clap_args: &Args) -> SolveConfig {
    SolveConfig {
        stats: clap_args.stats,
        show_assignment: clap_args.assignment,
        glue_strength: clap_args.glue_strength,
        core: clap_args.core,
        analysis: 3,
        stopping_criteria: {
            match clap_args.stopping_criteria.as_str() {
                "FirstUIP" | "firstUIP" | "1UIP" | "1uip" => StoppingCriteria::FirstAssertingUIP,
                "None" | "none" => StoppingCriteria::None,
                _ => panic!("Unknown stopping critera"),
            }
        },
        break_on_first: true,
        multi_jump_max: true,
        conflict_priority: {
            match clap_args.exploration_priority.as_str() {
                "Implication" | "implication" | "imp" => ExplorationPriority::Implication,
                "Conflict" | "conflict" | "conf" => ExplorationPriority::Conflict,
                "Default" | "default" => ExplorationPriority::Default,
                _ => panic!("Unknown conflict priority"),
            }
        },
    }
}
