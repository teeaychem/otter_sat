#![allow(unused_imports, unused_variables, unused_features, dead_code)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = Jemalloc;

use otter_sat::{
    config::Config,
    context::Context,
    dispatch::{
        library::report::{self},
        Dispatch,
    },
    structures::clause::Clause,
    types::err::{self},
};

use crossbeam::channel::unbounded;
use std::{
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};

mod config_io;
mod misc;
mod parse;
mod records;
mod window;

use config_io::ConfigIO;

use crate::misc::load_dimacs;

fn hand(_: Dispatch) {}

fn main() {
    #[cfg(feature = "log")]
    env_logger::init();

    let matches = parse::cli::cli().get_matches();

    let mut config = parse::config::config_from_args(&matches);
    let mut config_io = ConfigIO::from_args(&matches);

    // dbg!(&config);

    config_preprocessing(&mut config, &mut config_io);

    let (transmitter, receiver) =
        match config_io.show_core || config_io.show_stats || config_io.frat {
            true => {
                let (tx, rx) = unbounded::<Dispatch>();
                let config = config.clone();
                let config_io = config_io.clone();
                (Some(tx), Some(rx))
            }
            false => (None, None),
        };

    // As the context holds a transmitter it'll need to be dropped explicitly
    let mut the_context = match transmitter {
        Some(tx) => {
            let tx = tx;
            Context::from_config(
                config,
                Some(Rc::new(move |d: Dispatch| {
                    let _ = tx.send(d);
                })),
            )
        }
        None => Context::from_config(config, Some(Rc::new(hand))),
    };

    let report = 'report: {
        for path in config_io.files {
            match load_dimacs(&mut the_context, path) {
                Ok(()) => {}
                Err(err::ErrorKind::ClauseDB(err::ClauseDBError::EmptyClause)) => {
                    println!("s UNSATISFIABLE");
                    std::process::exit(20);
                }
                Err(e) => {
                    println!("c Error loading DIMACS: {e:?}")
                }
            };
        }

        if the_context.clause_db.total_clause_count() == 0 {
            break 'report report::SolveReport::Satisfiable;
        }

        let the_report = match the_context.solve() {
            Ok(r) => r,
            Err(e) => {
                println!("Context error: {e:?}");
                std::process::exit(1);
            }
        };

        match the_report {
            report::SolveReport::Unsatisfiable => {
                if config_io.show_core {
                    // let _ = self.display_core(clause_key);
                }
                the_context.dispatch_active();
            }
            report::SolveReport::Satisfiable => {
                if config_io.show_valuation {
                    println!("v {}", the_context.atom_db.valuation_string());
                }
            }
            _ => {}
        }
        the_report
    };

    match report {
        report::SolveReport::Satisfiable => {
            if let Some(path) = config_io.frat_path {
                let _ = std::fs::remove_file(path);
            }

            drop(the_context);
            println!("s SATISFIABLE");
            std::process::exit(10)
        }
        report::SolveReport::Unsatisfiable => {
            if config_io.frat_path.is_some() {
                println!("c Finalising FRAT proof…");
            }

            if config_io.show_core {
                println!("Core: Not yet reimplemented for CLI.")
            }

            drop(the_context);
            println!("s UNSATISFIABLE");
            std::process::exit(20)
        }
        report::SolveReport::TimeUp => {
            drop(the_context);
            if config_io.detail > 0 {
                println!("c Time up");
            }
            println!("s UNKNOWN");
            std::process::exit(30)
        }
        report::SolveReport::Unknown => {
            drop(the_context);
            println!("s UNKNOWN");
            std::process::exit(30)
        }
    };
}

fn config_preprocessing(config: &mut Config, config_io: &mut ConfigIO) {
    if config_io.detail > 0 {
        println!("c Parsing {} files\n", config_io.files.len());
    }

    if config_io.frat {
        if config.switch.subsumption {
            if config_io.detail > 1 {
                println!("c Subsumption is disabled for FRAT proofs");
            }
            config.switch.subsumption = false;
        }
    }
}
