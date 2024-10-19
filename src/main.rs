#![allow(dead_code)]
// #![allow(unused_imports)]
// #![allow(unused_variables)]

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use clap::Parser;
use std::fs;

mod context;
mod io;
mod procedures;
mod structures;

use crate::{io::ContextWindow, structures::formula::Formula};
use context::{
    config::{Args, Config},
    Context, Result,
};
use structures::variable::variable_store::VariableStore;

use crossterm::cursor;

// #[rustfmt::skip]
fn main() {
    match log4rs::init_file("config/log4rs.yaml", Default::default()) {
        Ok(()) => log::trace!("Log find loaded"),
        Err(e) => log::error!("{e:?}"),
    }

    let config = Config::from_args(Args::parse());

    let show_valuation = config.show_valuation;
    let show_stats = config.show_stats;
    let time_limit = config.time_limit;
    let show_core = config.show_core;

    match fs::read_to_string(&config.formula_file) {
        Ok(contents) => {
            let formula = Formula::from_dimacs(&contents);

            let the_window = if config.show_stats {
                Some(ContextWindow::new(
                    cursor::position().expect("Unable to display stats"),
                    &config,
                    &formula,
                ))
            } else {
                None
            };
            let mut the_context = Context::from_formula(formula, config, the_window);
            log::trace!("Context made");

            let result = the_context.solve();

            if show_stats {
                the_context.update_stats(the_context.window.as_ref().unwrap());
                the_context.window.as_ref().unwrap().flush();
            }

            match result {
                Result::Unsatisfiable(clause_key) => {
                    println!("s UNSATISFIABLE");
                    if show_core {
                        the_context.display_core(clause_key);
                    }
                    std::process::exit(00);
                }
                Result::Satisfiable => {
                    println!("s SATISFIABLE");
                    if show_valuation {
                        println!("v {}", the_context.variables().as_display_string());
                    }
                    std::process::exit(10);
                }
                Result::Unknown => {
                    if let Some(limit) = time_limit {
                        if show_stats && the_context.time > limit {
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
