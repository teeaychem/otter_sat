// #![allow(unused_imports)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = Jemalloc;

use otter_lib::{
    config::Config,
    context::Context,
    dispatch::{
        core::CoreDB,
        library::report::{self},
        Dispatch,
    },
    structures::clause::ClauseT,
    types::err::{self},
};

use crossbeam::channel::unbounded;
use std::{
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

fn main() {
    #[cfg(feature = "log")]
    env_logger::init();

    let matches = parse::cli::cli().get_matches();

    let mut config = parse::config::config_from_args(&matches);
    let mut config_io = ConfigIO::from_args(&matches);

    // dbg!(&config);

    config_preprocessing(&mut config, &mut config_io);

    let core_db_ptr = if config_io.show_core {
        Some(Arc::new(Mutex::new(CoreDB::default())))
    } else {
        None
    };

    let (transmitter, receiver) =
        match config_io.show_core || config_io.show_stats || config_io.frat {
            true => {
                let (tx, rx) = unbounded::<Dispatch>();
                let config = config.clone();
                let config_io = config_io.clone();
                let core_db_ptr_clone = core_db_ptr.clone();
                let rx = thread::spawn(|| {
                    records::general_recorder(rx, config, config_io, core_db_ptr_clone)
                });
                (Some(tx), Some(rx))
            }
            false => (None, None),
        };

    // As the context holds a transmitter it'll need to be dropped explicitly
    let mut the_context = Context::from_config(config, transmitter);
    let report = 'report: {
        for path in config_io.files {
            match load_dimacs(&mut the_context, path) {
                Ok(()) => {}
                Err(err::Build::ClauseDB(err::ClauseDB::EmptyClause)) => {
                    println!("s UNSATISFIABLE");
                    std::process::exit(20);
                }
                Err(e) => {
                    println!("c Error loading DIMACS: {e:?}")
                }
            };
        }

        if the_context.clause_db.clause_count() == 0 {
            break 'report report::Solve::Satisfiable;
        }

        let the_report = match the_context.solve() {
            Ok(r) => r,
            Err(e) => {
                println!("Context error: {e:?}");
                std::process::exit(1);
            }
        };

        match the_report {
            report::Solve::Unsatisfiable => {
                if config_io.show_core {
                    // let _ = self.display_core(clause_key);
                }
                the_context.dispatch_active();
            }
            report::Solve::Satisfiable => {
                if config_io.show_valuation {
                    println!("v {}", the_context.valuation_string());
                }
            }
            _ => {}
        }
        the_report
    };

    match report {
        report::Solve::Satisfiable => {
            if let Some(path) = config_io.frat_path {
                let _ = std::fs::remove_file(path);
            }
            if let Some(handle) = receiver {
                let _ = handle.join();
            }

            drop(the_context);
            println!("s SATISFIABLE");
            std::process::exit(10)
        }
        report::Solve::Unsatisfiable => {
            if config_io.frat_path.is_some() {
                println!("c Finalising FRAT proof…");
            }

            if let Some(handle) = receiver {
                let _ = handle.join();
            }

            if config_io.show_core {
                let the_core_db = core_db_ptr.expect("core_db should be present…");
                let the_core_db = the_core_db.lock().unwrap();
                let core_keys = the_core_db.core_clauses().unwrap();
                for core_clause in core_keys {
                    println!("{}", core_clause.as_dimacs(&the_context.variable_db, true));
                }
            }

            drop(the_context);
            println!("s UNSATISFIABLE");
            std::process::exit(20)
        }
        report::Solve::Unknown => {
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
