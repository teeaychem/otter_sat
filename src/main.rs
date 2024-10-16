#![allow(dead_code)]

use clap::Parser;
use std::fs;
use structures::solve::config::Config;
use structures::valuation::Valuation;
mod io;
mod procedures;
mod structures;

use crate::structures::formula::Formula;
use crate::structures::solve::{config, Result, Solve};

// #[rustfmt::skip]
fn main() {
    match log4rs::init_file("config/log4rs.yaml", Default::default()) {
        Ok(()) => log::trace!("Log find loaded"),
        Err(e) => log::error!("{e:?}"),
    }

    let config = Config::from_args(config::Args::parse());

    match fs::read_to_string(&config.formula_file) {
        Ok(contents) => {
            let formula = Formula::from_dimacs(&contents);

            if config.show_stats {
                println!("c 🦦");
                println!("c Parsing formula from file: {:?}", config.formula_file);
                println!(
                    "c Parsed formula with {} variables and {} clauses",
                    formula.variable_count(),
                    formula.clause_count()
                );
                if let Some(limit) = config.time_limit {
                    println!("c TIME LIMIT: {limit:.2?}");
                }
                println!("c CHOICE POLARITY LEAN: {}", config.polarity_lean);
            }
            log::trace!("Formula processed");
            let mut the_solve = Solve::from_formula(formula, config.clone());
            log::trace!("Solve initialised");

            let result = the_solve.do_solve();

            if config.show_stats {
                the_solve.display_stats();
            }

            match result {
                Result::Unsatisfiable => {
                    println!("s UNSATISFIABLE");
                    if config.show_core {
                        the_solve.display_core();
                    }
                    std::process::exit(00);
                }
                Result::Satisfiable => {
                    println!("s SATISFIABLE");
                    if config.show_valuation {
                        println!("v {}", the_solve.valuation().as_display_string(&the_solve));
                    }
                    std::process::exit(10);
                }
                Result::Unknown => {
                    if let Some(limit) = config.time_limit {
                        if config.show_stats && the_solve.time > limit {
                            println!("c TIME LIMIT EXCEEDED");
                        }
                    }
                    println!("s UNKNOWN");
                    std::process::exit(20);
                }
            }
        }
        Err(e) => println!("Error reading file {e:?}"),
    }
}
