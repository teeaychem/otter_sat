#![allow(dead_code)]

use clap::Parser;
use std::fs;
use structures::valuation::Valuation;
mod io;
mod procedures;
mod structures;

use crate::structures::formula::Formula;
use crate::structures::solve::{config, Solve, SolveResult};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[allow(non_snake_case)]
struct Args {
    /// The DIMACS form CNF file to parse
    #[arg(short, long)]
    formula_file: std::path::PathBuf,

    /// Display stats on completion
    #[arg(short, long, default_value_t = false)]
    stats: bool,

    /// Display a satisfying valuation, if possible
    #[arg(short, long, default_value_t = false)]
    valuation: bool,

    /// Display an unsatisfiable core on UNSAT
    #[arg(short, long, default_value_t = false)]
    core: bool,

    /// Required glue strength
    #[arg(short, long, default_value_t = 2)]
    glue_strength: usize,

    /// Resolution stopping criteria
    #[arg(long, default_value_t, value_enum)]
    stopping_criteria: config::StoppingCriteria,

    /// The VSIDS variant to use
    #[arg(long = "VSIDS", default_value_t, value_enum)]
    vsids: config::VSIDS,

    /// Suggest priority exploring conflcits, implications, or take no interest (does nothing at the moment)
    #[arg(long, default_value_t, value_enum)]
    exploration_priority: config::ExplorationPriority,

    /// Reduce and restart, where:
    #[arg(short, long = "reduce-and-restart", default_value_t = false)]
    rr: bool,

    /// Allow for the clauses to be forgotten, on occassion
    #[arg(long, default_value_t = false)]
    reduce: bool,

    /// Allow for the decisions to be forgotten, on occassion
    #[arg(long, default_value_t = false)]
    restart: bool,

    /// Initially settle all atoms which occur with a unique polarity
    #[arg(long, default_value_t = false)]
    hobson: bool,

    #[arg(short, long, default_value_t = 0.0)]
    /// The chance of choosing assigning positive polarity to a variant when making a choice
    polarity_lean: f64,

    #[arg(short = 'u', long = "luby", default_value_t = 512)]
    /// The u value to use for the luby calculation when restarts are permitted
    luby_u: usize,

    /// Time limit for the solve
    #[arg(short, long, value_parser = |seconds: &str| seconds.parse().map(std::time::Duration::from_secs))]
    time: Option<std::time::Duration>,
}

// #[rustfmt::skip]
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
        config::EXPLORATION_PRIORITY = args.exploration_priority;
        config::STOPPING_CRITERIA = args.stopping_criteria;
        config::VSIDS_VARIANT = args.vsids;
        config::SHOW_CORE = args.core;
        config::SHOW_VALUATION = args.valuation;
        config::RESTARTS_ALLOWED = args.restart;
        config::REDUCTION_ALLOWED = if args.reduce && !args.restart {
            println!("c REDUCTION REQUIRES RESTARTS TO BE ENABLED");
            false
        } else {
            args.reduce
        };
        config::TIME_LIMIT = args.time;
        config::POLARITY_LEAN = args.polarity_lean;
        config::LUBY_CONSTANT = args.luby_u;
        if args.rr {
            config::RESTARTS_ALLOWED = true;
            config::REDUCTION_ALLOWED = true;
        }
    }

    match fs::read_to_string(&args.formula_file) {
        Ok(contents) => {
            let formula = Formula::from_dimacs(&contents);

            if unsafe { config::SHOW_STATS } {
                println!("c 🦦");
                println!("c Parsing formula from file: {:?}", args.formula_file);
                println!(
                    "c Parsed formula with {} variables and {} clauses",
                    formula.variable_count(),
                    formula.clause_count()
                );
                if let Some(limit) = unsafe { config::TIME_LIMIT } {
                    println!("c TIME LIMIT: {:.2?}", limit);
                }
                println!("c CHOICE POLARITY LEAN: {}", unsafe {
                    config::POLARITY_LEAN
                })
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
                    if unsafe { config::SHOW_CORE } {
                        the_solve.core()
                    }
                    std::process::exit(00);
                }
                SolveResult::Satisfiable => {
                    println!("s SATISFIABLE");
                    if unsafe { config::SHOW_VALUATION } {
                        println!("v {}", the_solve.valuation.as_display_string(&the_solve))
                    }
                    std::process::exit(10);
                }
                SolveResult::Unknown => {
                    println!("s UNKNOWN");
                    std::process::exit(20);
                }
            }
        }
        Err(_) => println!("Error reading file"),
    }
}
