#![allow(dead_code)]

use clap::Parser;
use std::fs;
use structures::solve::config::{config_time_limit, ExplorationPriority, StoppingCriteria};
mod io;
mod procedures;
mod structures;

use crate::structures::formula::Formula;
use crate::structures::solve::{config::config_show_stats, Solve, SolveResult};

// Configuration variables
static mut CONFIG_GLUE_STRENGTH: usize = 2;
static mut CONFIG_SHOW_STATS: bool = false;
static mut CONFIG_SHOW_CORE: bool = false;
static mut CONFIG_SHOW_ASSIGNMENT: bool = false;
static mut CONFIG_EXPLORATION_PRIORITY: ExplorationPriority = ExplorationPriority::Default;
static mut CONFIG_STOPPING_CRITERIA: StoppingCriteria = StoppingCriteria::FirstAssertingUIP;
static mut RESTARTS_ALLOWED: bool = true;
static mut HOBSON_CHOICES: bool = false;
static mut TIME_LIMIT: Option<std::time::Duration> = None; // Some(std::time::Duration::new(10, 0));

static CONFIG_MULTI_JUMP_MAX: bool = false;

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

    /// Suggest priority exploring conflcits, implications, or take no interest
    #[arg(long, default_value_t = String::from("Default"))]
    exploration_priority: String,

    /// Allow for the decisions to be forgotten, on occassion
    #[arg(short, long, default_value_t = true)]
    restarts: bool,

    /// Initially settle all atoms which occur with a unique polarity
    #[arg(long, default_value_t = false)]
    hobson: bool,

    /// Time limit for the solve
    #[arg(short, long, value_parser = |seconds: &str| seconds.parse().map(std::time::Duration::from_secs))]
    time: Option<std::time::Duration>,
}

fn main() {
    match log4rs::init_file("config/log4rs.yaml", Default::default()) {
        Ok(_) => log::trace!("Log find loaded"),
        Err(e) => {
            log::error!("{e:?}")
        }
    }

    let args = Args::parse();

    // set up the configuration, unsafely as global variables are used
    // see the config file for access procedures
    unsafe {
        CONFIG_GLUE_STRENGTH = args.glue_strength;
        CONFIG_SHOW_STATS = args.stats;
        CONFIG_EXPLORATION_PRIORITY = match args.exploration_priority.as_str() {
            "Implication" | "implication" | "imp" => ExplorationPriority::Implication,
            "Conflict" | "conflict" | "conf" => ExplorationPriority::Conflict,
            "Default" | "default" => ExplorationPriority::Default,
            _ => panic!("Unknown conflict priority"),
        };
        CONFIG_STOPPING_CRITERIA = match args.stopping_criteria.as_str() {
            "FirstUIP" | "firstUIP" | "1UIP" | "1uip" => StoppingCriteria::FirstAssertingUIP,
            "None" | "none" => StoppingCriteria::None,
            _ => panic!("Unknown stopping critera"),
        };
        CONFIG_SHOW_CORE = args.core;
        CONFIG_SHOW_ASSIGNMENT = args.assignment;
        RESTARTS_ALLOWED = args.restarts;
        TIME_LIMIT = args.time;
    }

    if let Ok(contents) = fs::read_to_string(&args.formula_file) {
        let formula = Formula::from_dimacs(&contents);

        if config_show_stats() {
            println!("c 🦦");
            println!("c Parsing formula from file: {:?}", args.formula_file);
            println!(
                "c Parsed formula with {} variables and {} clauses",
                formula.variables.len(),
                formula.clause_count()
            );
            if let Some(limit) = config_time_limit() {
                println!("c TIME LIMIT: {:.2?}", limit);
            }
        }
        log::trace!("Formula processed");
        let mut the_solve = Solve::from_formula(formula);
        log::trace!("Solve initialised");

        let (result, stats) = the_solve.do_solve();
        if config_show_stats() {
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
