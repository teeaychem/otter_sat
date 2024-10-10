#![allow(dead_code)]

use clap::Parser;
use std::fs;
use structures::solve::config::{ExplorationPriority, StoppingCriteria};
mod io;
mod procedures;
mod structures;

use crate::structures::formula::Formula;
use crate::structures::solve::{config, Solve, SolveResult};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The DIMACS form CNF file to parse
    #[arg(short, long)]
    formula_file: std::path::PathBuf,

    /// Display stats on completion
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

    /// VSIDS variant
    #[arg(long, default_value_t = String::from("M"))]
    vsids_variant: String,

    /// Suggest priority exploring conflcits, implications, or take no interest
    #[arg(long, default_value_t = String::from("Default"))]
    exploration_priority: String,

    /// Allow for the clauses to be forgotten, on occassion
    #[arg(long, default_value_t = false)]
    reduction: bool,

    /// Allow for the decisions to be forgotten, on occassion
    #[arg(long, default_value_t = false)]
    restarts: bool,

    /// Initially settle all atoms which occur with a unique polarity
    #[arg(long, default_value_t = false)]
    hobson: bool,

    /// Time limit for the solve
    #[arg(short, long, value_parser = |seconds: &str| seconds.parse().map(std::time::Duration::from_secs))]
    time: Option<std::time::Duration>,
}

#[rustfmt::skip]
fn main() {
    match log4rs::init_file("config/log4rs.yaml", Default::default()) {
        Ok(_) => log::trace!("Log find loaded"),
        Err(e) => log::error!("{e:?}"),
    }

    let args = Args::parse();

    // set up the configuration, unsafely as global variables are used
    // see the config file for access procedures
    unsafe {
        config::GLUE_STRENGTH = args.glue_strength;
        config::SHOW_STATS = args.stats;
        config::EXPLORATION_PRIORITY = match args.exploration_priority.as_str() {
            "Implication" | "implication" | "imp" => ExplorationPriority::Implication,
            "Conflict" | "conflict" | "conf" => ExplorationPriority::Conflict,
            "Default" | "default" => ExplorationPriority::Default,
            _ => panic!("Unknown conflict priority"),
        };
        config::STOPPING_CRITERIA = match args.stopping_criteria.as_str() {
            "FirstUIP" | "firstUIP" | "1UIP" | "1uip" => StoppingCriteria::FirstAssertingUIP,
            "None" | "none" => StoppingCriteria::None,
            _ => panic!("Unknown stopping critera"),
        };
        config::VSIDS_VARIANT = match args.vsids_variant.as_str() {
            "M" | "m" => config::VSIDS::M,
            "C" | "c" => config::VSIDS::C,
            _ => panic!("Unknown VSIDS variant"),
        };
        config::SHOW_CORE = args.core;
        config::SHOW_ASSIGNMENT = args.assignment;
        config::RESTARTS_ALLOWED = args.restarts;
        config::REDUCTION_ALLOWED =
            if  args.reduction && !args.restarts {
                println!("c REDUCTION REQUIRES RESTARTS TO BE ENABLED");
                false
            } else {
                args.reduction
            };
        config::TIME_LIMIT = args.time;
    }

    if let Ok(contents) = fs::read_to_string(&args.formula_file) {
        let formula = Formula::from_dimacs(&contents);

        if unsafe { config::SHOW_STATS } {
            println!("c 🦦");
            println!("c Parsing formula from file: {:?}", args.formula_file);
            println!("c Parsed formula with {} variables and {} clauses", formula.variable_count(), formula.clause_count());
            if let Some(limit) = unsafe { config::TIME_LIMIT } {
                println!("c TIME LIMIT: {:.2?}", limit);
            }
        }
        log::trace!("Formula processed");
        let mut the_solve = Solve::from_formula(formula);
        log::trace!("Solve initialised");

        let (result, stats) = the_solve.do_solve();
        if unsafe { config::SHOW_STATS } {
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
                println!("s UNKNOWN");
                std::process::exit(20);
            }
        }
    } else {
        println!("Error reading file")
    }
}
