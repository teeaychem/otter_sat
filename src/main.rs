// #![allow(unused_imports)]

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = Jemalloc;

use otter_lib::{
    cli::{
        config::ConfigIO,
        parse::{self},
    },
    config::Config,
    context::{builder::BuildErr, Context},
    dispatch::{
        report::{self},
        Dispatch,
    },
    types::errs::{self},
};

use std::fs;

use crossbeam::channel::unbounded;
use std::thread;

fn main() {
    #[cfg(feature = "log")]
    match log4rs::init_file("config/log4rs.yaml", Default::default()) {
        Ok(()) => log::trace!("log find loaded"),
        Err(e) => log::error!("{e:?}"),
    }

    let matches = parse::cli::cli().get_matches();

    let mut config = Config::from_args(&matches);
    let config_io = ConfigIO::from_args(&matches);

    if config_io.detail > 0 {
        println!("c Parsing {} files\n", config_io.files.len());
    }

    #[allow(clippy::collapsible_if)]
    if config_io.frat {
        if config.subsumption {
            if config_io.detail > 0 {
                println!("c Subsumption is disabled for FRAT proofs");
            }
            config.subsumption = false;
        }
    }

    let (tx, rx) = unbounded::<Dispatch>();

    let listener_handle = {
        let config = config.clone();
        let config_io = config_io.clone();
        thread::spawn(|| otter_lib::io::listener::general_receiver(rx, config, config_io))
    };

    /*
    The context is in a block as:
    - When the block closes the transmitter for the reciever is dropped
    - Unify different ways to get sat/unsat
    At least for now…
     */
    let report = 'report: {
        let mut the_context = Context::from_config(config, tx);

        for path in config_io.files {
            println!("{path:?}");
            match the_context.load_dimacs_file(path) {
                Ok(()) => {}
                Err(BuildErr::ClauseStore(errs::ClauseDB::EmptyClause)) => {
                    println!("s UNSATISFIABLE");
                    std::process::exit(20);
                }
                Err(e) => {
                    println!("c Error loading DIMACS: {e:?}")
                }
            };
        }

        if the_context.clause_count() == 0 {
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
                the_context.report_active();
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
                let _ = fs::remove_file(path);
            }
            std::process::exit(10)
        }
        report::Solve::Unsatisfiable => {
            if config_io.frat_path.is_some() {
                println!("c Finalising FRAT proof…");
                let _ = listener_handle.join();
            }
            std::process::exit(20)
        }
        report::Solve::Unknown => std::process::exit(30),
    };
}
